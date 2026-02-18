const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { pathToFileURL } = require('node:url');

const CHART_LITE_PATH = 'dashboard/static/assets/vendor/chart-lite-1.0.0.min.js';

function loadClassicBrowserScript(relativePath, overrides = {}) {
  const absolutePath = path.resolve(__dirname, '..', relativePath);
  const source = fs.readFileSync(absolutePath, 'utf8');
  const sandbox = {
    window: {
      ...overrides
    },
    document: overrides.document,
    location: overrides.location,
    navigator: overrides.navigator,
    fetch: overrides.fetch || (typeof fetch === 'undefined' ? undefined : fetch),
    console,
    URL,
    Headers: typeof Headers === 'undefined' ? function HeadersShim() {} : Headers,
    Request: typeof Request === 'undefined' ? function RequestShim() {} : Request,
    Response: typeof Response === 'undefined' ? function ResponseShim() {} : Response
  };
  if (sandbox.document && !sandbox.window.document) {
    sandbox.window.document = sandbox.document;
  }
  if (sandbox.location && !sandbox.window.location) {
    sandbox.window.location = sandbox.location;
  }
  if (sandbox.navigator && !sandbox.window.navigator) {
    sandbox.window.navigator = sandbox.navigator;
  }
  sandbox.globalThis = sandbox.window;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: absolutePath });
  return sandbox.window;
}

function setGlobalValue(key, value) {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, key);
  Object.defineProperty(globalThis, key, {
    configurable: true,
    writable: true,
    value
  });
  return () => {
    if (descriptor) {
      Object.defineProperty(globalThis, key, descriptor);
    } else {
      delete globalThis[key];
    }
  };
}

async function withBrowserGlobals(overrides = {}, fn) {
  const defaultLocation = {
    origin: 'http://127.0.0.1:3000',
    pathname: '/dashboard/index.html',
    search: '',
    hash: ''
  };
  const defaultHistory = {
    replaceState: () => {}
  };
  const defaultDocument = {
    getElementById: () => null,
    querySelector: () => null,
    querySelectorAll: () => [],
    createElement: () => ({ innerHTML: '', classList: { add() {}, remove() {}, toggle() {}, contains() { return false; } } })
  };

  const windowValue = {
    ...(overrides.window || {}),
    location: overrides.location || (overrides.window && overrides.window.location) || defaultLocation,
    history: overrides.history || (overrides.window && overrides.window.history) || defaultHistory,
    document: overrides.document || (overrides.window && overrides.window.document) || defaultDocument,
    navigator: overrides.navigator || (overrides.window && overrides.window.navigator) || {},
    fetch: overrides.fetch || (overrides.window && overrides.window.fetch) || globalThis.fetch,
    setTimeout,
    clearTimeout,
    requestAnimationFrame:
      overrides.requestAnimationFrame ||
      (overrides.window && overrides.window.requestAnimationFrame) ||
      ((cb) => setTimeout(cb, 0))
  };

  const restoreFns = [];
  restoreFns.push(setGlobalValue('window', windowValue));
  restoreFns.push(setGlobalValue('document', windowValue.document));
  restoreFns.push(setGlobalValue('location', windowValue.location));
  restoreFns.push(setGlobalValue('history', windowValue.history));
  restoreFns.push(setGlobalValue('navigator', windowValue.navigator));
  if (windowValue.fetch) {
    restoreFns.push(setGlobalValue('fetch', windowValue.fetch));
  }
  restoreFns.push(setGlobalValue('URL', URL));
  if (typeof Headers !== 'undefined') restoreFns.push(setGlobalValue('Headers', Headers));
  if (typeof Request !== 'undefined') restoreFns.push(setGlobalValue('Request', Request));
  if (typeof Response !== 'undefined') restoreFns.push(setGlobalValue('Response', Response));

  try {
    return await fn();
  } finally {
    restoreFns.reverse().forEach((restore) => restore());
  }
}

async function importBrowserModule(relativePath) {
  const absolutePath = path.resolve(__dirname, '..', relativePath);
  const url = pathToFileURL(absolutePath).href;
  const cacheBust = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return import(`${url}?test=${cacheBust}`);
}

function toPlain(value) {
  return JSON.parse(JSON.stringify(value));
}

function waitForAsyncWork() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function createMockCanvasContext() {
  const calls = {
    fillText: [],
    moveTo: [],
    lineTo: [],
    fillStyle: []
  };
  const gradient = { addColorStop: () => {} };
  const canvas = { clientWidth: 320, clientHeight: 180, width: 320, height: 180 };
  const ctx = {
    canvas,
    save: () => {},
    restore: () => {},
    clearRect: () => {},
    setTransform: () => {},
    beginPath: () => {},
    moveTo: (x, y) => calls.moveTo.push([x, y]),
    lineTo: (x, y) => calls.lineTo.push([x, y]),
    arc: () => {},
    closePath: () => {},
    fill: () => {},
    stroke: () => {},
    fillRect: () => {},
    createLinearGradient: () => gradient,
    fillText: (text) => calls.fillText.push(String(text))
  };
  let fillStyleValue = '';
  Object.defineProperty(ctx, 'fillStyle', {
    get() {
      return fillStyleValue;
    },
    set(value) {
      fillStyleValue = String(value);
      calls.fillStyle.push(fillStyleValue);
    }
  });
  return { ctx, calls };
}

function createMockElement(initial = {}) {
  return {
    textContent: '',
    innerHTML: '',
    href: '',
    dataset: {},
    ...initial
  };
}

function listJsFilesRecursively(rootDir) {
  const entries = fs.readdirSync(rootDir, { withFileTypes: true });
  const files = [];
  entries.forEach((entry) => {
    const absolute = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listJsFilesRecursively(absolute));
      return;
    }
    if (entry.isFile() && entry.name.endsWith('.js')) {
      files.push(absolute);
    }
  });
  return files;
}

function stripCommentsAndStrings(source) {
  return source
    .replace(/\/\/.*$/gm, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/`(?:\\.|[^`\\])*`/g, '')
    .replace(/"(?:\\.|[^"\\])*"/g, '')
    .replace(/'(?:\\.|[^'\\])*'/g, '');
}

function parseRelativeImports(source) {
  const imports = [];
  const pattern = /^\s*import\s+[^'"]*['"](.+?)['"]\s*;?\s*$/gm;
  let match = pattern.exec(source);
  while (match) {
    const specifier = String(match[1] || '').trim();
    if (specifier.startsWith('.')) {
      imports.push(specifier);
    }
    match = pattern.exec(source);
  }
  return imports;
}

function detectCycles(adjacency) {
  const visiting = new Set();
  const visited = new Set();
  const stack = [];
  const cycles = [];

  const visit = (node) => {
    if (visited.has(node)) return;
    if (visiting.has(node)) {
      const cycleStart = stack.indexOf(node);
      if (cycleStart >= 0) {
        cycles.push([...stack.slice(cycleStart), node]);
      }
      return;
    }
    visiting.add(node);
    stack.push(node);
    const edges = adjacency.get(node) || [];
    edges.forEach((edge) => visit(edge));
    stack.pop();
    visiting.delete(node);
    visited.add(node);
  };

  Array.from(adjacency.keys()).forEach((node) => visit(node));
  return cycles;
}

test('dashboard API adapters normalize sparse payloads safely', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const api = await importBrowserModule('dashboard/modules/api-client.js');
    assert.ok(api);

    const events = api.adaptEvents({ recent_events: null, top_ips: [['198.51.100.8', '3']] });
    assert.deepEqual(toPlain(events.recent_events), []);
    assert.deepEqual(toPlain(events.top_ips), [['198.51.100.8', 3]]);

    const maze = api.adaptMaze({ total_hits: '9', unique_crawlers: '2', top_crawlers: [] });
    assert.equal(maze.total_hits, 9);
    assert.equal(maze.unique_crawlers, 2);
    assert.deepEqual(toPlain(maze.top_crawlers), []);

    const monitoring = api.adaptMonitoring({
      summary: { honeypot: { total_hits: 1 } },
      prometheus: { endpoint: '/metrics' }
    });
    assert.equal(monitoring.summary.honeypot.total_hits, 1);
    assert.equal(monitoring.prometheus.endpoint, '/metrics');
  });
});

test('dashboard API client parses JSON payloads when content-type is missing', { concurrency: false }, async () => {
  const payload = {
    recent_events: [{ event: 'AdminAction', ts: 1700000000 }],
    event_counts: { AdminAction: 1 },
    top_ips: [['198.51.100.8', 1]],
    unique_ips: 1
  };
  let requestUrl = '';

  await withBrowserGlobals({
    fetch: async (url) => {
      requestUrl = String(url);
      return {
        ok: true,
        status: 200,
        headers: { get: () => '' },
        text: async () => JSON.stringify(payload),
        json: async () => payload
      };
    }
  }, async () => {
    const apiModule = await importBrowserModule('dashboard/modules/api-client.js');
    const api = apiModule.create({
      getAdminContext: () => ({ endpoint: 'http://example.test', apikey: '' })
    });

    const events = await api.getEvents(24);
    assert.equal(requestUrl, 'http://example.test/admin/events?hours=24');
    assert.equal(events.recent_events.length, 1);
    assert.equal(events.unique_ips, 1);
    assert.deepEqual(toPlain(events.top_ips), [['198.51.100.8', 1]]);
  });
});

test('dashboard API client adds CSRF + same-origin for session-auth writes and strips empty bearer', { concurrency: false }, async () => {
  /** @type {{url?: string, init?: RequestInit}} */
  let captured = {};

  await withBrowserGlobals({
    fetch: async (url, init = {}) => {
      captured = { url: String(url), init };
      return {
        ok: true,
        status: 200,
        headers: { get: () => 'application/json' },
        json: async () => ({ config: { maze_enabled: true } }),
        text: async () => JSON.stringify({ config: { maze_enabled: true } })
      };
    }
  }, async () => {
    const apiModule = await importBrowserModule('dashboard/modules/api-client.js');
    const api = apiModule.create({
      getAdminContext: () => ({
        endpoint: 'http://example.test',
        apikey: '',
        sessionAuth: true,
        csrfToken: 'csrf-123'
      })
    });

    await api.request('/admin/config', {
      method: 'POST',
      headers: { Authorization: 'Bearer   ' },
      json: { maze_enabled: true }
    });
  });

  assert.equal(captured.url, 'http://example.test/admin/config');
  const headers = new Headers(captured.init && captured.init.headers ? captured.init.headers : undefined);
  assert.equal(headers.get('X-Shuma-CSRF'), 'csrf-123');
  assert.equal(headers.has('Authorization'), false);
  assert.equal(captured.init && captured.init.credentials, 'same-origin');
});

test('admin session leaves global fetch unpatched and sends CSRF header on logout', { concurrency: false }, async () => {
  const calls = [];
  const logoutButton = {
    disabled: false,
    textContent: 'Logout',
    onclick: null
  };
  const messageNode = {
    textContent: '',
    className: ''
  };

  await withBrowserGlobals({
    fetch: async (url, init = {}) => {
      calls.push({ url: String(url), init });
      if (String(url).endsWith('/admin/session')) {
        return {
          ok: true,
          status: 200,
          json: async () => ({ authenticated: true, csrf_token: 'csrf-logout' })
        };
      }
      return {
        ok: true,
        status: 200,
        json: async () => ({})
      };
    },
    document: {
      getElementById: (id) => {
        if (id === 'logout-btn') return logoutButton;
        if (id === 'admin-msg') return messageNode;
        return null;
      },
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const adminSessionModule = await importBrowserModule('dashboard/modules/admin-session.js');
    const originalFetch = window.fetch;
    const controller = adminSessionModule.create({
      resolveAdminApiEndpoint: () => ({ endpoint: 'http://example.test' }),
      redirectToLogin: () => {}
    });

    assert.equal(window.fetch, originalFetch);
    await controller.restoreAdminSession();
    controller.bindLogoutButton('logout-btn', 'admin-msg');
    assert.equal(typeof logoutButton.onclick, 'function');
    await logoutButton.onclick();
  });

  const logoutCall = calls.find((entry) => entry.url.endsWith('/admin/logout'));
  assert.ok(logoutCall, 'expected logout call');
  const logoutHeaders = new Headers(logoutCall.init && logoutCall.init.headers
    ? logoutCall.init.headers
    : undefined);
  assert.equal(logoutHeaders.get('X-Shuma-CSRF'), 'csrf-logout');
  assert.equal(logoutCall.init && logoutCall.init.credentials, 'same-origin');
});

test('admin session bootstrap handles authenticated then expired transitions', { concurrency: false }, async () => {
  let sessionCalls = 0;
  const messageNode = {
    textContent: '',
    className: ''
  };

  await withBrowserGlobals({
    fetch: async (url) => {
      if (String(url).endsWith('/admin/session')) {
        sessionCalls += 1;
        if (sessionCalls === 1) {
          return {
            ok: true,
            status: 200,
            json: async () => ({ authenticated: true, csrf_token: 'csrf-live' })
          };
        }
        return {
          ok: true,
          status: 200,
          json: async () => ({ authenticated: false, csrf_token: '' })
        };
      }
      return {
        ok: true,
        status: 200,
        json: async () => ({})
      };
    }
  }, async () => {
    const adminSessionModule = await importBrowserModule('dashboard/modules/admin-session.js');
    const controller = adminSessionModule.create({
      resolveAdminApiEndpoint: () => ({ endpoint: 'http://example.test' }),
      redirectToLogin: () => {}
    });

    const firstRestore = await controller.restoreAdminSession();
    assert.equal(firstRestore, true);
    assert.equal(controller.hasValidApiContext(), true);
    const activeContext = controller.getAdminContext(messageNode);
    assert.equal(Boolean(activeContext), true);
    assert.equal(activeContext.csrfToken, 'csrf-live');

    const secondRestore = await controller.restoreAdminSession();
    assert.equal(secondRestore, false);
    assert.equal(controller.hasValidApiContext(), false);
    const expiredContext = controller.getAdminContext(messageNode);
    assert.equal(expiredContext, null);
    assert.equal(messageNode.className, 'message warning');
    assert.equal(messageNode.textContent.includes('Login required'), true);
  });
});

test('chart-lite renders doughnut legend labels', () => {
  const browser = loadClassicBrowserScript(CHART_LITE_PATH, {
    matchMedia: () => ({ matches: false })
  });
  const { ctx, calls } = createMockCanvasContext();
  new browser.Chart(ctx, {
    type: 'doughnut',
    data: {
      labels: ['Ban', 'Challenge'],
      datasets: [{ data: [3, 2], backgroundColor: ['#111', '#222'] }]
    }
  });

  assert.ok(calls.fillText.some((text) => text.includes('Ban')));
  assert.ok(calls.fillText.some((text) => text.includes('Challenge')));
});

test('chart-lite uses non-white center fill in dark mode doughnut charts', () => {
  const browser = loadClassicBrowserScript(CHART_LITE_PATH, {
    matchMedia: () => ({ matches: true })
  });
  const { ctx, calls } = createMockCanvasContext();
  new browser.Chart(ctx, {
    type: 'doughnut',
    data: {
      labels: ['Ban', 'Challenge'],
      datasets: [{ data: [3, 2], backgroundColor: ['#111', '#222'] }]
    }
  });

  assert.ok(calls.fillStyle.includes('rgba(44, 36, 48, 1)'));
  assert.equal(calls.fillStyle.includes('#ffffff'), false);
});

test('chart-lite renders axis ticks and labels for bar and line charts', () => {
  const browser = loadClassicBrowserScript(CHART_LITE_PATH, {
    matchMedia: () => ({ matches: false })
  });

  const bar = createMockCanvasContext();
  new browser.Chart(bar.ctx, {
    type: 'bar',
    data: {
      labels: ['IP-A', 'IP-B', 'IP-C'],
      datasets: [{ data: [1, 3, 2] }]
    }
  });
  assert.ok(bar.calls.fillText.some((text) => text === '0'));
  assert.ok(bar.calls.fillText.some((text) => text.includes('IP-A')));

  const line = createMockCanvasContext();
  new browser.Chart(line.ctx, {
    type: 'line',
    data: {
      labels: ['09:00', '10:00', '11:00'],
      datasets: [{ data: [1, 2, 3], backgroundColor: ['#abc'] }]
    }
  });
  assert.ok(line.calls.fillText.some((text) => text === '0'));
  assert.ok(line.calls.fillText.some((text) => text.includes('09:00')));
});

test('dashboard state invalidation scopes are explicit and bounded', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const stateApi = await importBrowserModule('dashboard/modules/dashboard-state.js');
    assert.ok(stateApi);

    const state = stateApi.create();
    ['monitoring', 'ip-bans', 'status', 'config', 'tuning'].forEach((tab) => {
      state.markTabUpdated(tab);
    });
    state.invalidate('securityConfig');
    assert.equal(state.isTabStale('status'), true);
    assert.equal(state.isTabStale('config'), true);
    assert.equal(state.isTabStale('tuning'), true);
    assert.equal(state.isTabStale('monitoring'), false);
    assert.equal(state.isTabStale('ip-bans'), false);

    state.markTabUpdated('status');
    assert.equal(state.isTabStale('status'), false);
    assert.equal(state.isTabStale('config'), true);
  });
});

test('dashboard state exports action creators and selectors', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const stateApi = await importBrowserModule('dashboard/modules/dashboard-state.js');
    const initial = stateApi.createInitialState('monitoring');
    const next = stateApi.reduceState(initial, stateApi.actions.setActiveTab('status'));
    assert.equal(stateApi.selectors.activeTab(initial), 'monitoring');
    assert.equal(stateApi.selectors.activeTab(next), 'status');
    const withSession = stateApi.reduceState(next, stateApi.actions.setSession({
      authenticated: true,
      csrfToken: 'csrf-123'
    }));
    assert.equal(stateApi.selectors.session(withSession).authenticated, true);
    assert.equal(stateApi.selectors.session(withSession).csrfToken, 'csrf-123');
  });
});

test('tab lifecycle normalizes unknown tabs to monitoring default', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const lifecycle = await importBrowserModule('dashboard/modules/tab-lifecycle.js');
    assert.ok(lifecycle);

    assert.equal(lifecycle.normalizeTab('ip-bans'), 'ip-bans');
    assert.equal(lifecycle.normalizeTab('IP-BANS'), 'ip-bans');
    assert.equal(lifecycle.normalizeTab('unknown-tab'), 'monitoring');
    assert.equal(lifecycle.normalizeTab(''), 'monitoring');
  });
});

test('tab lifecycle init is single-mount and destroy cleans listeners/timers', { concurrency: false }, async () => {
  const createMockLink = (tab) => {
    const attributes = new Map();
    const listeners = new Map();
    const addCounts = new Map();
    const removeCounts = new Map();
    const classes = new Set();
    return {
      dataset: { dashboardTabLink: tab },
      tabIndex: -1,
      classList: {
        toggle: (className, enabled) => {
          if (enabled) {
            classes.add(className);
          } else {
            classes.delete(className);
          }
        }
      },
      setAttribute: (name, value) => {
        attributes.set(name, String(value));
      },
      getAttribute: (name) => attributes.get(name) || null,
      addEventListener: (name, handler) => {
        listeners.set(name, handler);
        addCounts.set(name, (addCounts.get(name) || 0) + 1);
      },
      removeEventListener: (name, handler) => {
        if (listeners.get(name) === handler) {
          listeners.delete(name);
        }
        removeCounts.set(name, (removeCounts.get(name) || 0) + 1);
      },
      focus: () => {},
      emit: (name, event) => {
        const handler = listeners.get(name);
        if (handler) handler(event);
      },
      getAddCount: (name) => addCounts.get(name) || 0,
      getRemoveCount: (name) => removeCounts.get(name) || 0
    };
  };

  const createMockPanel = (tab) => ({
    dataset: { dashboardTabPanel: tab },
    hidden: true,
    tabIndex: -1,
    setAttribute: () => {},
    focus: () => {}
  });

  const links = ['monitoring', 'ip-bans', 'status', 'config', 'tuning'].map(createMockLink);
  const adminPanels = ['ip-bans', 'status', 'config', 'tuning'].map(createMockPanel);
  const monitoringPanel = {
    hidden: false,
    tabIndex: 0,
    setAttribute: () => {},
    focus: () => {}
  };
  const adminSection = {
    hidden: true,
    setAttribute: () => {}
  };

  const windowListeners = { hashchange: [] };
  let hashValue = '';
  let historyReplaceCalls = 0;
  const location = {
    pathname: '/dashboard/index.html',
    search: '',
    get hash() {
      return hashValue;
    },
    set hash(value) {
      const text = String(value || '');
      hashValue = text.startsWith('#') ? text : `#${text}`;
    }
  };
  const history = {
    replaceState: (_state, _title, url) => {
      const hashIndex = String(url || '').indexOf('#');
      hashValue = hashIndex >= 0 ? String(url).slice(hashIndex) : '';
      historyReplaceCalls += 1;
    }
  };

  await withBrowserGlobals({
    window: {
      location,
      history,
      requestAnimationFrame: (task) => task(),
      addEventListener: (name, handler) => {
        if (name === 'hashchange') {
          windowListeners.hashchange.push(handler);
        }
      },
      removeEventListener: (name, handler) => {
        if (name !== 'hashchange') return;
        windowListeners.hashchange = windowListeners.hashchange.filter((entry) => entry !== handler);
      }
    },
    location,
    history,
    document: {
      getElementById: (id) => {
        if (id === 'dashboard-panel-monitoring') return monitoringPanel;
        if (id === 'dashboard-admin-section') return adminSection;
        return null;
      },
      querySelector: (selector) => {
        if (selector === '#dashboard-panel-monitoring') return monitoringPanel;
        const match = String(selector).match(/data-dashboard-tab-panel=\"([^\"]+)\"/);
        if (!match) return null;
        return adminPanels.find((panel) => panel.dataset.dashboardTabPanel === match[1]) || null;
      },
      querySelectorAll: (selector) => {
        if (selector === '[data-dashboard-tab-link]') return links;
        if (selector === '#dashboard-admin-section [data-dashboard-tab-panel]') return adminPanels;
        return [];
      }
    }
  }, async () => {
    const lifecycle = await importBrowserModule('dashboard/modules/tab-lifecycle.js');
    const mounts = [];
    const unmounts = [];

    const coordinator = lifecycle.createTabLifecycleCoordinator({
      controllers: {
        monitoring: { mount: () => mounts.push('monitoring'), unmount: () => unmounts.push('monitoring') },
        'ip-bans': { mount: () => mounts.push('ip-bans'), unmount: () => unmounts.push('ip-bans') },
        status: { mount: () => mounts.push('status'), unmount: () => unmounts.push('status') },
        config: { mount: () => mounts.push('config'), unmount: () => unmounts.push('config') },
        tuning: { mount: () => mounts.push('tuning'), unmount: () => unmounts.push('tuning') }
      }
    });

    coordinator.init();
    coordinator.init();
    assert.equal(windowListeners.hashchange.length, 1);
    links.forEach((link) => {
      assert.equal(link.getAddCount('click'), 1);
      assert.equal(link.getAddCount('keydown'), 1);
    });
    assert.equal(mounts.length <= 1, true);
    assert.equal(historyReplaceCalls >= 1, true);

    links[0].emit('keydown', { key: 'ArrowRight', preventDefault: () => {} });
    assert.equal(location.hash, '#ip-bans');
    windowListeners.hashchange.forEach((handler) => handler());
    assert.equal(links[1].getAttribute('aria-selected'), 'true');

    coordinator.destroy();
    assert.equal(windowListeners.hashchange.length, 0);
    assert.equal(unmounts.length >= 1, true);
    links.forEach((link) => {
      assert.equal(link.getRemoveCount('click') >= 1, true);
      assert.equal(link.getRemoveCount('keydown') >= 1, true);
    });
  });
});

test('svelte dashboard store exposes centralized state/actions/selectors', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    assert.ok(storeModule);

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    assert.equal(store.getState().activeTab, 'monitoring');

    store.setActiveTab('status');
    assert.equal(store.getState().activeTab, 'status');

    store.setSession({ authenticated: true, csrfToken: 'csrf-native' });
    assert.equal(store.getState().session.authenticated, true);
    assert.equal(store.getState().session.csrfToken, 'csrf-native');

    store.setTabLoading('status', true);
    const statusStore = store.tabStatus('status');
    let latestStatus = null;
    const unsubStatus = statusStore.subscribe((value) => {
      latestStatus = value;
    });
    assert.equal(latestStatus.loading, true);
    unsubStatus();

    store.setDraftBaseline('maze', { enabled: true, threshold: 50 });
    store.setDraft('maze', { enabled: true, threshold: 60 });
    assert.equal(store.isDraftDirty('maze'), true);

    store.recordRefreshMetrics({
      tab: 'monitoring',
      reason: 'manual',
      fetchLatencyMs: 123.45,
      renderTimingMs: 12.34
    });
    store.recordPollingSkip('page-hidden', 'monitoring', 30000);
    store.recordPollingResume('visibility-resume', 'monitoring', 30000);

    const telemetry = store.getRuntimeTelemetry();
    assert.equal(telemetry.refresh.lastTab, 'monitoring');
    assert.equal(telemetry.refresh.lastReason, 'manual');
    assert.equal(telemetry.refresh.fetchLatencyMs.last, 123.45);
    assert.equal(telemetry.refresh.fetchLatencyMs.p95, 123.45);
    assert.equal(telemetry.refresh.fetchLatencyMs.samples, 1);
    assert.equal(telemetry.refresh.fetchLatencyMs.totalSamples, 1);
    assert.equal(telemetry.refresh.renderTimingMs.last, 12.34);
    assert.equal(telemetry.refresh.renderTimingMs.p95, 12.34);
    assert.equal(telemetry.refresh.renderTimingMs.samples, 1);
    assert.equal(telemetry.refresh.renderTimingMs.totalSamples, 1);
    assert.equal(telemetry.polling.skips, 1);
    assert.equal(telemetry.polling.resumes, 1);
  });
});

test('svelte dashboard store uses bounded rolling telemetry windows with deterministic reset', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    const windowSize = storeModule.RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE;
    const totalSamples = windowSize + 10;

    for (let sample = 1; sample <= totalSamples; sample += 1) {
      store.recordRefreshMetrics({
        tab: 'monitoring',
        reason: sample % 2 === 0 ? 'auto-refresh' : 'manual',
        fetchLatencyMs: sample,
        renderTimingMs: sample * 2
      });
    }

    const telemetry = store.getRuntimeTelemetry();
    const fetchLatency = telemetry.refresh.fetchLatencyMs;
    const renderTiming = telemetry.refresh.renderTimingMs;
    const firstWindowSample = totalSamples - windowSize + 1;
    const expectedAvg = Number((((firstWindowSample + totalSamples) / 2).toFixed(2)));
    const expectedP95Index = Math.ceil(windowSize * 0.95) - 1;
    const expectedP95 = firstWindowSample + expectedP95Index;

    assert.equal(fetchLatency.windowSize, windowSize);
    assert.equal(fetchLatency.samples, windowSize);
    assert.equal(fetchLatency.totalSamples, totalSamples);
    assert.equal(fetchLatency.last, totalSamples);
    assert.equal(fetchLatency.avg, expectedAvg);
    assert.equal(fetchLatency.p95, expectedP95);
    assert.equal(fetchLatency.max, totalSamples);

    assert.equal(renderTiming.windowSize, windowSize);
    assert.equal(renderTiming.samples, windowSize);
    assert.equal(renderTiming.totalSamples, totalSamples);
    assert.equal(renderTiming.last, totalSamples * 2);
    assert.equal(renderTiming.avg, expectedAvg * 2);
    assert.equal(renderTiming.p95, expectedP95 * 2);
    assert.equal(renderTiming.max, totalSamples * 2);

    store.setActiveTab('status');
    store.resetRuntimeTelemetry();
    const telemetryAfterResetOnly = store.getRuntimeTelemetry();
    assert.equal(store.getState().activeTab, 'status');
    assert.equal(telemetryAfterResetOnly.refresh.lastReason, 'init');
    assert.equal(telemetryAfterResetOnly.refresh.fetchLatencyMs.samples, 0);
    assert.equal(telemetryAfterResetOnly.refresh.fetchLatencyMs.totalSamples, 0);
    assert.equal(telemetryAfterResetOnly.refresh.fetchLatencyMs.p95, 0);
    assert.equal(Array.isArray(telemetryAfterResetOnly.refresh.fetchLatencyMs.window), true);
    assert.equal(telemetryAfterResetOnly.refresh.fetchLatencyMs.window.length, 0);

    store.recordRefreshMetrics({
      tab: 'status',
      reason: 'manual',
      fetchLatencyMs: 42,
      renderTimingMs: 8
    });
    const telemetryAfterRestart = store.getRuntimeTelemetry();
    assert.equal(telemetryAfterRestart.refresh.fetchLatencyMs.samples, 1);
    assert.equal(telemetryAfterRestart.refresh.fetchLatencyMs.totalSamples, 1);
    assert.equal(telemetryAfterRestart.refresh.fetchLatencyMs.p95, 42);

    store.reset('monitoring');
    const telemetryAfterFullReset = store.getRuntimeTelemetry();
    assert.equal(store.getState().activeTab, 'monitoring');
    assert.equal(telemetryAfterFullReset.refresh.fetchLatencyMs.samples, 0);
    assert.equal(telemetryAfterFullReset.refresh.fetchLatencyMs.totalSamples, 0);
    assert.equal(telemetryAfterFullReset.refresh.fetchLatencyMs.p95, 0);
    assert.equal(telemetryAfterFullReset.refresh.fetchLatencyMs.window.length, 0);
  });
});

test('svelte dashboard actions orchestrate session bootstrap and keyboard/hash tab pipeline', { concurrency: false }, async () => {
  const listeners = {
    hashchange: [],
    visibilitychange: []
  };
  const hashWrites = [];
  const redirectCalls = [];
  const clearCalls = [];
  const setActiveCalls = [];
  const focusCalls = [];
  const refreshCalls = [];
  let sessionState = { authenticated: true, csrfToken: 'csrf-native' };
  const timerIds = [];
  const expectedRedirectPath = '/dashboard/login.html?next=%2Fdashboard%2Findex.html%23monitoring';

  await withBrowserGlobals({
    window: {
      location: {
        pathname: '/dashboard/index.html',
        search: '',
        hash: '#monitoring'
      }
    },
    document: {
      getElementById: (id) => {
        if (!String(id).startsWith('dashboard-tab-')) return null;
        return { focus: () => {} };
      },
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const actionsModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-actions.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });

    const effects = {
      setTimer: (_task, _ms) => {
        const id = Symbol('timer');
        timerIds.push(id);
        return id;
      },
      clearTimer: (id) => clearCalls.push(id),
      requestFrame: (task) => task(),
      readHashTab: () => String(window.location.hash || '').replace(/^#/, ''),
      writeHashTab: (tab) => {
        const normalized = String(tab || '').replace(/^#/, '');
        hashWrites.push(normalized);
        window.location.hash = `#${normalized}`;
      },
      buildLoginRedirectPath: () => expectedRedirectPath,
      onHashChange: (handler) => {
        listeners.hashchange.push(handler);
        return () => {
          listeners.hashchange = listeners.hashchange.filter((entry) => entry !== handler);
        };
      },
      onVisibilityChange: (handler) => {
        listeners.visibilitychange.push(handler);
        return () => {
          listeners.visibilitychange = listeners.visibilitychange.filter((entry) => entry !== handler);
        };
      },
      isPageVisible: () => true,
      focusTab: (tab) => {
        focusCalls.push(String(tab || ''));
        return true;
      },
      redirect: (path) => redirectCalls.push(String(path))
    };

    let restoreCount = 0;
    const runtime = {
      refreshTab: async (tab, reason) => {
        refreshCalls.push({ tab, reason });
      },
      setActiveTab: (tab) => {
        setActiveCalls.push(tab);
      },
      restoreSession: async () => {
        restoreCount += 1;
        if (restoreCount === 1) {
          sessionState = { authenticated: true, csrfToken: 'csrf-native' };
          return true;
        }
        sessionState = { authenticated: false, csrfToken: '' };
        return false;
      },
      getSessionState: () => ({ ...sessionState }),
      logout: async () => {}
    };

    const actions = actionsModule.createDashboardActions({
      store,
      effects,
      runtime
    });

    actions.init();
    assert.equal(listeners.hashchange.length, 1);
    assert.equal(listeners.visibilitychange.length, 1);

    const firstBootstrap = await actions.bootstrapSession();
    assert.equal(firstBootstrap, true);
    assert.equal(store.getState().session.authenticated, true);
    assert.equal(refreshCalls.length >= 1, true);

    actions.onTabKeydown({ key: 'ArrowRight', preventDefault: () => {} }, 'monitoring');
    await waitForAsyncWork();
    assert.equal(hashWrites.includes('ip-bans'), true);
    assert.equal(setActiveCalls.includes('ip-bans'), true);
    assert.equal(focusCalls.includes('ip-bans'), true);

    const secondBootstrap = await actions.bootstrapSession();
    assert.equal(secondBootstrap, false);
    assert.equal(store.getState().session.authenticated, false);
    assert.equal(redirectCalls.length >= 1, true);
    assert.equal(redirectCalls[0], expectedRedirectPath);

    actions.destroy();
    assert.equal(listeners.hashchange.length, 0);
    assert.equal(listeners.visibilitychange.length, 0);
    assert.equal(clearCalls.length >= 1, true);
  });
});

test('svelte dashboard actions remount loops do not accumulate listeners or polling timers', { concurrency: false }, async () => {
  const listeners = {
    hashchange: new Set(),
    visibilitychange: new Set()
  };
  const activeTimers = new Set();
  let refreshCount = 0;
  let timerSeed = 0;

  await withBrowserGlobals({
    window: {
      location: {
        pathname: '/dashboard/index.html',
        search: '',
        hash: '#monitoring'
      }
    },
    document: {
      getElementById: () => null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const actionsModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-actions.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });

    const effects = {
      setTimer: () => {
        const id = `timer-${++timerSeed}`;
        activeTimers.add(id);
        return id;
      },
      clearTimer: (id) => {
        activeTimers.delete(id);
      },
      requestFrame: (task) => task(),
      readHashTab: () => String(window.location.hash || '').replace(/^#/, ''),
      writeHashTab: (tab) => {
        window.location.hash = `#${String(tab || '').replace(/^#/, '')}`;
      },
      onHashChange: (handler) => {
        listeners.hashchange.add(handler);
        return () => listeners.hashchange.delete(handler);
      },
      onVisibilityChange: (handler) => {
        listeners.visibilitychange.add(handler);
        return () => listeners.visibilitychange.delete(handler);
      },
      isPageVisible: () => true,
      redirect: () => {}
    };

    const runtime = {
      refreshTab: async () => {
        refreshCount += 1;
      },
      setActiveTab: () => {},
      restoreSession: async () => true,
      getSessionState: () => ({ authenticated: true, csrfToken: 'csrf-native' }),
      logout: async () => {}
    };

    const actions = actionsModule.createDashboardActions({
      store,
      effects,
      runtime
    });

    for (let cycle = 0; cycle < 5; cycle += 1) {
      actions.init();
      assert.equal(listeners.hashchange.size, 1);
      assert.equal(listeners.visibilitychange.size, 1);

      const bootstrapped = await actions.bootstrapSession();
      assert.equal(bootstrapped, true);
      assert.equal(activeTimers.size <= 1, true);

      actions.destroy();
      assert.equal(listeners.hashchange.size, 0);
      assert.equal(listeners.visibilitychange.size, 0);
      assert.equal(activeTimers.size, 0);
    }

    assert.equal(refreshCount >= 5, true);
  });
});

test('svelte dashboard actions collect refresh and polling performance telemetry', { concurrency: false }, async () => {
  const listeners = {
    hashchange: [],
    visibilitychange: []
  };
  let visible = true;
  let nowTick = 0;

  await withBrowserGlobals({
    window: {
      location: {
        pathname: '/dashboard/index.html',
        search: '',
        hash: '#monitoring'
      }
    },
    document: {
      getElementById: () => null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const actionsModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-actions.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });

    const effects = {
      setTimer: () => Symbol('timer'),
      clearTimer: () => {},
      now: () => {
        nowTick += 5;
        return nowTick;
      },
      requestFrame: (task) => task(),
      readHashTab: () => String(window.location.hash || '').replace(/^#/, ''),
      writeHashTab: (tab) => {
        window.location.hash = `#${String(tab || '').replace(/^#/, '')}`;
      },
      onHashChange: (handler) => {
        listeners.hashchange.push(handler);
        return () => {
          listeners.hashchange = listeners.hashchange.filter((entry) => entry !== handler);
        };
      },
      onVisibilityChange: (handler) => {
        listeners.visibilitychange.push(handler);
        return () => {
          listeners.visibilitychange = listeners.visibilitychange.filter((entry) => entry !== handler);
        };
      },
      isPageVisible: () => visible,
      redirect: () => {}
    };

    const runtime = {
      refreshTab: async () => {},
      setActiveTab: () => {},
      restoreSession: async () => true,
      getSessionState: () => ({ authenticated: true, csrfToken: 'csrf-native' }),
      logout: async () => {}
    };

    const actions = actionsModule.createDashboardActions({
      store,
      effects,
      runtime
    });

    actions.init();
    await actions.bootstrapSession();

    const telemetryAfterBootstrap = store.getRuntimeTelemetry();
    assert.equal(telemetryAfterBootstrap.refresh.fetchLatencyMs.samples >= 1, true);
    assert.equal(telemetryAfterBootstrap.refresh.renderTimingMs.samples >= 1, true);
    assert.equal(telemetryAfterBootstrap.polling.skips >= 1, true);
    assert.equal(telemetryAfterBootstrap.polling.resumes >= 1, true);

    visible = false;
    listeners.visibilitychange.forEach((handler) => handler());
    const afterHidden = store.getRuntimeTelemetry();
    assert.equal(afterHidden.polling.lastSkipReason, 'page-hidden');

    const resumesBeforeVisible = afterHidden.polling.resumes;
    visible = true;
    listeners.visibilitychange.forEach((handler) => handler());
    const afterVisible = store.getRuntimeTelemetry();
    assert.equal(afterVisible.polling.resumes > resumesBeforeVisible, true);
    assert.equal(afterVisible.polling.lastResumeReason, 'visibility-resume');

    actions.destroy();
  });
});

test('svelte dashboard actions abort in-flight refresh when tab switches', { concurrency: false }, async () => {
  await withBrowserGlobals({
    window: {
      location: {
        pathname: '/dashboard/index.html',
        search: '',
        hash: '#monitoring'
      }
    },
    document: {
      getElementById: () => null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');
    const actionsModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-actions.js');
    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    store.setSession({ authenticated: true, csrfToken: 'csrf-live' });

    const hashListeners = [];
    const visibilityListeners = [];
    let firstRefreshSignal = null;
    let firstRefreshAborted = false;
    let refreshCalls = 0;

    const effects = {
      setTimer: () => null,
      clearTimer: () => {},
      requestFrame: (task) => task(),
      readHashTab: () => String(window.location.hash || '').replace(/^#/, ''),
      writeHashTab: (tab) => {
        window.location.hash = `#${String(tab || '').replace(/^#/, '')}`;
      },
      onHashChange: (handler) => {
        hashListeners.push(handler);
        return () => {
          const idx = hashListeners.indexOf(handler);
          if (idx >= 0) hashListeners.splice(idx, 1);
        };
      },
      onVisibilityChange: (handler) => {
        visibilityListeners.push(handler);
        return () => {
          const idx = visibilityListeners.indexOf(handler);
          if (idx >= 0) visibilityListeners.splice(idx, 1);
        };
      },
      isPageVisible: () => true,
      redirect: () => {}
    };

    const runtime = {
      refreshTab: (_tab, _reason, opts = {}) => {
        refreshCalls += 1;
        const signal = opts.signal || null;
        if (refreshCalls === 1) {
          firstRefreshSignal = signal;
          return new Promise((resolve, reject) => {
            if (!signal) {
              resolve();
              return;
            }
            signal.addEventListener('abort', () => {
              firstRefreshAborted = true;
              reject(new DOMException('Aborted', 'AbortError'));
            }, { once: true });
          });
        }
        return Promise.resolve();
      },
      setActiveTab: () => {},
      restoreSession: async () => true,
      getSessionState: () => ({ authenticated: true, csrfToken: 'csrf-live' }),
      logout: async () => {}
    };

    const actions = actionsModule.createDashboardActions({
      store,
      effects,
      runtime
    });

    actions.init();
    const firstRefresh = actions.applyActiveTab('monitoring', 'manual', { force: true });
    await waitForAsyncWork();
    await actions.applyActiveTab('status', 'click', { syncHash: true });
    await firstRefresh;

    assert.equal(Boolean(firstRefreshSignal), true);
    assert.equal(firstRefreshSignal.aborted, true);
    assert.equal(firstRefreshAborted, true);
    actions.destroy();
    assert.equal(hashListeners.length, 0);
    assert.equal(visibilityListeners.length, 0);
  });
});

test('svelte runtime effects provide login redirect and tab focus adapters', { concurrency: false }, async () => {
  let focusedId = '';
  const focusable = {
    focus: () => {
      focusedId = 'dashboard-tab-config';
    }
  };

  await withBrowserGlobals({
    window: {
      location: {
        pathname: '/dashboard/index.html',
        search: '?mode=native',
        hash: '#config',
        replace: () => {}
      },
      history: { replaceState: () => {} },
      fetch: async () => ({ ok: true }),
      setTimeout,
      clearTimeout,
      requestAnimationFrame: (task) => task(),
      addEventListener: () => {},
      removeEventListener: () => {}
    },
    document: {
      visibilityState: 'visible',
      getElementById: (id) => (id === 'dashboard-tab-config' ? focusable : null),
      querySelector: () => null,
      querySelectorAll: () => [],
      addEventListener: () => {},
      removeEventListener: () => {}
    }
  }, async () => {
    const effectsModule = await importBrowserModule('dashboard/src/lib/runtime/dashboard-effects.js');
    const effects = effectsModule.createDashboardEffects();
    const expectedPath = '/dashboard/login.html?next=%2Fdashboard%2Findex.html%3Fmode%3Dnative%23config';

    assert.equal(effects.buildLoginRedirectPath(), expectedPath);
    assert.equal(effects.focusTab('config'), true);
    assert.equal(focusedId, 'dashboard-tab-config');
    assert.equal(effects.focusTab('status'), false);
  });
});

test('monitoring view consumes prometheus helper payload as single-source contract', { concurrency: false }, async () => {
  const elements = {
    'monitoring-prometheus-example': createMockElement(),
    'monitoring-prometheus-copy-curl': createMockElement(),
    'monitoring-prometheus-facts': createMockElement(),
    'monitoring-prometheus-output': createMockElement(),
    'monitoring-prometheus-stats': createMockElement(),
    'monitoring-prometheus-windowed': createMockElement(),
    'monitoring-prometheus-summary-stats': createMockElement(),
    'monitoring-prometheus-observability-link': createMockElement(),
    'monitoring-prometheus-api-link': createMockElement()
  };

  await withBrowserGlobals({
    location: { origin: 'https://example.test' },
    document: {
      getElementById: (id) => elements[id] || null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const monitoringViewModule = await importBrowserModule('dashboard/modules/monitoring-view.js');
    const monitoringView = monitoringViewModule.create();
    monitoringView.updatePrometheusHelper({
      endpoint: '/metrics',
      notes: [
        '/metrics returns one full payload.',
        'Use /admin/monitoring for bounded summary.'
      ],
      example_js: "const metricsText = await fetch('/metrics').then(r => r.text());",
      example_output: '# TYPE bot_defence_requests_total counter',
      example_stats: 'const stats = { requestsMain: 1 };',
      example_windowed: 'const monitoring = await fetch(`/admin/monitoring?hours=24&limit=10`).then(r => r.json());',
      example_summary_stats: 'const stats = { honeypotHits: monitoring.summary.honeypot.total_hits };',
      docs: {
        observability: 'https://example.test/observability',
        api: 'https://example.test/api'
      }
    });
  });

  assert.equal(elements['monitoring-prometheus-example'].textContent.includes("fetch('/metrics')"), true);
  assert.equal(
    elements['monitoring-prometheus-copy-curl'].dataset.copyText,
    "curl -sS 'https://example.test/metrics'"
  );
  assert.equal(
    elements['monitoring-prometheus-facts'].innerHTML.includes('/admin/monitoring for bounded summary.'),
    true
  );
  assert.equal(elements['monitoring-prometheus-observability-link'].href, 'https://example.test/observability');
  assert.equal(elements['monitoring-prometheus-api-link'].href, 'https://example.test/api');
});

test('monitoring view teardown removes copy listeners safely', { concurrency: false }, async () => {
  const makeButton = () => {
    const listeners = new Map();
    let addCount = 0;
    let removeCount = 0;
    return {
      textContent: '',
      dataset: {},
      get addCount() {
        return addCount;
      },
      get removeCount() {
        return removeCount;
      },
      addEventListener: (name, handler) => {
        listeners.set(name, handler);
        addCount += 1;
      },
      removeEventListener: (name, handler) => {
        if (listeners.get(name) === handler) {
          listeners.delete(name);
        }
        removeCount += 1;
      }
    };
  };
  const copyButton = makeButton();
  const copyCurlButton = makeButton();
  const elements = {
    'monitoring-prometheus-example': { textContent: 'const metricsText = ...' },
    'monitoring-prometheus-copy': copyButton,
    'monitoring-prometheus-copy-curl': copyCurlButton,
    'monitoring-prometheus-facts': createMockElement(),
    'monitoring-prometheus-output': createMockElement(),
    'monitoring-prometheus-stats': createMockElement(),
    'monitoring-prometheus-windowed': createMockElement(),
    'monitoring-prometheus-summary-stats': createMockElement(),
    'monitoring-prometheus-observability-link': createMockElement(),
    'monitoring-prometheus-api-link': createMockElement()
  };

  await withBrowserGlobals({
    document: {
      getElementById: (id) => elements[id] || null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const monitoringViewModule = await importBrowserModule('dashboard/modules/monitoring-view.js');
    const monitoringView = monitoringViewModule.create({
      effects: {
        copyText: async () => {},
        setTimer: (task) => task()
      }
    });
    monitoringView.bindPrometheusCopyButtons();
    monitoringView.destroy();
  });

  assert.equal(copyButton.removeCount >= 1, true);
  assert.equal(copyCurlButton.removeCount >= 1, true);
});

test('monitoring view normalizes hashed offender labels in top offender cards', { concurrency: false }, async () => {
  const elements = {
    'honeypot-total-hits': createMockElement(),
    'honeypot-unique-crawlers': createMockElement(),
    'honeypot-top-offender': createMockElement(),
    'honeypot-top-offender-label': createMockElement(),
    'honeypot-top-paths': createMockElement(),
    'challenge-failures-total': createMockElement(),
    'challenge-failures-unique': createMockElement(),
    'challenge-top-offender': createMockElement(),
    'challenge-top-offender-label': createMockElement(),
    'challenge-failure-reasons': createMockElement(),
    'pow-failures-total': createMockElement(),
    'pow-failures-unique': createMockElement(),
    'pow-top-offender': createMockElement(),
    'pow-top-offender-label': createMockElement(),
    'pow-failure-reasons': createMockElement(),
    'rate-violations-total': createMockElement(),
    'rate-violations-unique': createMockElement(),
    'rate-top-offender': createMockElement(),
    'rate-top-offender-label': createMockElement(),
    'rate-outcomes-list': createMockElement(),
    'geo-violations-total': createMockElement(),
    'geo-action-mix': createMockElement(),
    'geo-top-countries': createMockElement()
  };

  await withBrowserGlobals({
    document: {
      getElementById: (id) => elements[id] || null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const monitoringViewModule = await importBrowserModule('dashboard/modules/monitoring-view.js');
    const monitoringView = monitoringViewModule.create();
    monitoringView.updateMonitoringSummary({
      honeypot: {
        total_hits: 43,
        unique_crawlers: 1,
        top_crawlers: [{ label: 'h382', count: 43 }],
        top_paths: []
      },
      challenge: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
      pow: { total_failures: 0, unique_offenders: 0, top_offenders: [], reasons: {}, trend: [] },
      rate: { total_violations: 0, unique_offenders: 0, top_offenders: [], outcomes: {} },
      geo: { total_violations: 0, actions: {}, top_countries: [] }
    });
  });

  assert.equal(elements['honeypot-top-offender'].textContent, 'untrusted/unknown');
  assert.equal(elements['honeypot-top-offender-label'].textContent, 'Top Offender (43 hits)');
});

test('monitoring view loading state hydrates placeholders consistently', { concurrency: false }, async () => {
  const ids = [
    'maze-total-hits',
    'maze-unique-crawlers',
    'maze-auto-bans',
    'maze-top-offender',
    'maze-top-offender-label',
    'honeypot-total-hits',
    'honeypot-unique-crawlers',
    'honeypot-top-offender',
    'honeypot-top-offender-label',
    'challenge-failures-total',
    'challenge-failures-unique',
    'challenge-top-offender',
    'challenge-top-offender-label',
    'pow-failures-total',
    'pow-failures-unique',
    'pow-top-offender',
    'pow-top-offender-label',
    'rate-violations-total',
    'rate-violations-unique',
    'rate-top-offender',
    'rate-top-offender-label',
    'geo-violations-total'
  ];
  const elements = Object.fromEntries(ids.map((id) => [id, createMockElement()]));

  await withBrowserGlobals({
    document: {
      getElementById: (id) => elements[id] || null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const monitoringViewModule = await importBrowserModule('dashboard/modules/monitoring-view.js');
    const monitoringView = monitoringViewModule.create();
    monitoringView.showLoadingState();
  });

  assert.equal(elements['maze-total-hits'].textContent, '...');
  assert.equal(elements['maze-top-offender'].textContent, '...');
  assert.equal(elements['maze-top-offender-label'].textContent, 'Top Offender');
  assert.equal(elements['rate-top-offender'].textContent, '...');
  assert.equal(elements['rate-top-offender-label'].textContent, 'Top Offender');
});

test('config schema centralizes advanced and status-writable path inventories', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const schema = await importBrowserModule('dashboard/modules/config-schema.js');
    assert.ok(schema);

    assert.equal(Array.isArray(schema.advancedConfigTemplatePaths), true);
    assert.equal(Array.isArray(schema.writableStatusVarPaths), true);
    assert.equal(schema.advancedConfigTemplatePaths.includes('edge_integration_mode'), true);
    assert.equal(schema.writableStatusVarPaths.includes('robots_block_ai_training'), true);
    assert.equal(schema.writableStatusVarPaths.includes('ai_policy_block_training'), true);
    assert.equal(schema.writableStatusVarPaths.includes('edge_integration_mode'), true);

    const uniqueWritable = new Set(schema.writableStatusVarPaths);
    assert.equal(uniqueWritable.size, schema.writableStatusVarPaths.length);
  });
});

test('config draft store tracks section snapshots and dirty checks', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const storeModule = await importBrowserModule('dashboard/modules/config-draft-store.js');
    assert.ok(storeModule);

    const store = storeModule.create({
      maze: { enabled: false, threshold: 50 }
    });
    assert.equal(store.isDirty('maze', { enabled: false, threshold: 50 }), false);
    assert.equal(store.isDirty('maze', { enabled: true, threshold: 50 }), true);
    store.set('maze', { enabled: true, threshold: 60 });
    assert.deepEqual(toPlain(store.get('maze', {})), { enabled: true, threshold: 60 });
    assert.equal(store.isDirty('maze', { enabled: true, threshold: 60 }), false);
  });
});

test('config ui state preserves in-progress maze edits during background config refresh', { concurrency: false }, async () => {
  const elements = {
    'maze-enabled-toggle': createMockElement({ checked: true }),
    'maze-auto-ban-toggle': createMockElement({ checked: false }),
    'maze-threshold': createMockElement({ value: '51' }),
    'save-maze-config': createMockElement({ disabled: false, dataset: { saving: 'false' } })
  };
  const drafts = {
    maze: { enabled: true, autoBan: false, threshold: 51 }
  };
  const statusPatches = [];

  const mockDocument = {
    activeElement: elements['maze-threshold'],
    getElementById: (id) => elements[id] || null,
    querySelector: () => null,
    querySelectorAll: () => [],
    createElement: () => ({ innerHTML: '', classList: { add() {}, remove() {}, toggle() {}, contains() { return false; } } })
  };

  await withBrowserGlobals({ document: mockDocument }, async () => {
    const configUiStateModule = await importBrowserModule('dashboard/modules/config-ui-state.js');
    const api = configUiStateModule.create({
      getById: (id) => elements[id] || null,
      setDraft: (section, value) => {
        drafts[section] = JSON.parse(JSON.stringify(value));
      },
      getDraft: (section) => drafts[section] || {},
      statusPanel: {
        applyPatch: (patch) => statusPatches.push(patch)
      }
    });

    api.updateMazeConfig({
      maze_enabled: false,
      maze_auto_ban: true,
      maze_auto_ban_threshold: 25
    });

    assert.equal(elements['maze-threshold'].value, '51');
    assert.equal(elements['maze-enabled-toggle'].checked, true);
    assert.equal(elements['maze-auto-ban-toggle'].checked, false);
    assert.equal(elements['save-maze-config'].disabled, false);
    assert.deepEqual(drafts.maze, { enabled: true, autoBan: false, threshold: 51 });
    assert.equal(statusPatches.length, 0);

    elements['save-maze-config'].disabled = true;
    mockDocument.activeElement = null;

    api.updateMazeConfig({
      maze_enabled: false,
      maze_auto_ban: true,
      maze_auto_ban_threshold: 25
    });

    assert.equal(elements['maze-threshold'].value, 25);
    assert.equal(elements['maze-enabled-toggle'].checked, false);
    assert.equal(elements['maze-auto-ban-toggle'].checked, true);
    assert.equal(elements['save-maze-config'].disabled, true);
    assert.deepEqual(drafts.maze, { enabled: false, autoBan: true, threshold: 25 });
    assert.deepEqual(statusPatches.pop(), { mazeEnabled: false, mazeAutoBan: true });
  });
});

test('core dom cache re-resolves disconnected and previously-missing nodes', { concurrency: false }, async () => {
  let byIdLookupCount = 0;
  const firstNode = { id: 'node', isConnected: true };
  const secondNode = { id: 'node', isConnected: true, marker: 'fresh' };

  await withBrowserGlobals({}, async () => {
    const domApi = await importBrowserModule('dashboard/modules/core/dom.js');
    const cache = domApi.createCache({
      document: {
        getElementById: () => {
          byIdLookupCount += 1;
          if (byIdLookupCount === 1) return firstNode;
          if (byIdLookupCount === 2) return null;
          return secondNode;
        },
        querySelector: () => null,
        querySelectorAll: () => []
      }
    });

    assert.equal(cache.byId('node'), firstNode);
    firstNode.isConnected = false;
    assert.equal(cache.byId('node'), null);
    assert.equal(cache.byId('node'), secondNode);
  });

  assert.equal(byIdLookupCount, 3);
});

test('status module creates isolated state instances', { concurrency: false }, async () => {
  await withBrowserGlobals({
    document: {
      getElementById: () => null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const statusModule = await importBrowserModule('dashboard/modules/status.js');
    const first = statusModule.create({ document });
    const second = statusModule.create({ document });

    first.update({
      testMode: true,
      botnessWeights: { geo_risk: 9 },
      configSnapshot: { maze_enabled: true }
    });

    assert.equal(first.getState().testMode, true);
    assert.equal(second.getState().testMode, false);
    assert.equal(second.getState().botnessWeights.geo_risk, 2);

    const snapshot = first.getState();
    snapshot.botnessWeights.geo_risk = 42;
    snapshot.configSnapshot.maze_enabled = false;

    const next = first.getState();
    assert.equal(next.botnessWeights.geo_risk, 9);
    assert.equal(next.configSnapshot.maze_enabled, true);
  });
});

test('config form utils preserve legacy textarea parsing semantics', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const utils = await importBrowserModule('dashboard/modules/config-form-utils.js');
    assert.ok(utils);

    assert.deepEqual(
      toPlain(utils.parseCountryCodesStrict('gb,US,gb')),
      ['GB', 'US']
    );
    assert.equal(utils.normalizeCountryCodesForCompare(' gb,us '), 'GB,US');
    assert.deepEqual(
      toPlain(utils.parseListTextarea(' 10.0.0.1\n10.0.0.1,10.0.0.2')),
      ['10.0.0.1', '10.0.0.2']
    );
    assert.equal(
      utils.formatListTextarea([' 10.0.0.1 ', '', '10.0.0.2']),
      '10.0.0.1\n10.0.0.2'
    );
    assert.deepEqual(
      toPlain(utils.parseBrowserRulesTextarea('chrome,120\nsafari,17')),
      [['chrome', 120], ['safari', 17]]
    );
    assert.equal(
      utils.formatBrowserRulesTextarea([['chrome', 120], ['safari', 17]]),
      'chrome,120\nsafari,17'
    );
    assert.equal(utils.normalizeBrowserRulesForCompare('invalid rule'), '__invalid__');
  });
});

test('input validation module enforces integer, duration, and IP rules', { concurrency: false }, async () => {
  const makeInput = (id, value = '') => ({
    id,
    value,
    dataset: {},
    setCustomValidity: () => {},
    addEventListener: () => {},
    checkValidity: () => true,
    reportValidity: () => {},
    focus: () => {}
  });

  const elements = {
    'pow-difficulty': makeInput('pow-difficulty', '15abc'),
    'dur-honeypot-days': makeInput('dur-honeypot-days', '0'),
    'dur-honeypot-hours': makeInput('dur-honeypot-hours', '1'),
    'dur-honeypot-minutes': makeInput('dur-honeypot-minutes', '0'),
    'ban-duration-days': makeInput('ban-duration-days', '0'),
    'ban-duration-hours': makeInput('ban-duration-hours', '2'),
    'ban-duration-minutes': makeInput('ban-duration-minutes', '0'),
    'ban-ip': makeInput('ban-ip', '203.0.113.9')
  };
  const errors = [];

  await withBrowserGlobals({}, async () => {
    const validation = await importBrowserModule('dashboard/modules/input-validation.js');
    assert.ok(validation);

    const api = validation.create({
      getById: (id) => elements[id] || null,
      setFieldError: (_input, message) => {
        errors.push(String(message || ''));
      },
      integerFieldRules: {
        'pow-difficulty': { min: 12, max: 20, fallback: 15, label: 'PoW difficulty' },
        'dur-honeypot-days': { min: 0, max: 365, fallback: 1, label: 'Honeypot days' },
        'dur-honeypot-hours': { min: 0, max: 23, fallback: 0, label: 'Honeypot hours' },
        'dur-honeypot-minutes': { min: 0, max: 59, fallback: 0, label: 'Honeypot minutes' },
        'ban-duration-days': { min: 0, max: 365, fallback: 0, label: 'Manual days' },
        'ban-duration-hours': { min: 0, max: 23, fallback: 1, label: 'Manual hours' },
        'ban-duration-minutes': { min: 0, max: 59, fallback: 0, label: 'Manual minutes' }
      },
      banDurationBoundsSeconds: { min: 60, max: 31536000 },
      banDurationFields: {
        honeypot: {
          label: 'Honeypot duration',
          fallback: 86400,
          daysId: 'dur-honeypot-days',
          hoursId: 'dur-honeypot-hours',
          minutesId: 'dur-honeypot-minutes'
        }
      },
      manualBanDurationField: {
        label: 'Manual ban duration',
        fallback: 3600,
        daysId: 'ban-duration-days',
        hoursId: 'ban-duration-hours',
        minutesId: 'ban-duration-minutes'
      }
    });

    assert.equal(api.parseIntegerLoose('pow-difficulty'), 15);
    assert.equal(elements['pow-difficulty'].value, '15');

    const honeypot = api.readBanDurationFromInputs('honeypot');
    assert.equal(honeypot.totalSeconds, 3600);
    assert.equal(api.readManualBanDurationSeconds(), 7200);

    assert.equal(api.validateIpFieldById('ban-ip', true, 'Ban IP'), true);
    elements['ban-ip'].value = 'bad-ip';
    assert.equal(api.validateIpFieldById('ban-ip', true, 'Ban IP'), false);
  });

  assert.equal(errors.length > 0, true);
});

test('input validation reports interacting field id to onFieldInteraction callback', { concurrency: false }, async () => {
  const listeners = {};
  const makeInput = (id, value = '') => ({
    id,
    value,
    dataset: {},
    setCustomValidity: () => {},
    checkValidity: () => true,
    reportValidity: () => {},
    focus: () => {},
    addEventListener: (event, handler) => {
      listeners[`${id}:${event}`] = handler;
    }
  });

  const elements = {
    'pow-difficulty': makeInput('pow-difficulty', '15'),
    'ban-ip': makeInput('ban-ip', '203.0.113.10')
  };
  const interactions = [];

  await withBrowserGlobals({}, async () => {
    const validation = await importBrowserModule('dashboard/modules/input-validation.js');
    const api = validation.create({
      getById: (id) => elements[id] || null,
      setFieldError: () => {},
      integerFieldRules: {
        'pow-difficulty': { min: 12, max: 20, fallback: 15, label: 'PoW difficulty' }
      },
      banDurationBoundsSeconds: { min: 60, max: 31536000 },
      banDurationFields: {},
      manualBanDurationField: null,
      onFieldInteraction: (fieldId) => interactions.push(fieldId)
    });
    api.bindIntegerFieldValidation('pow-difficulty');
    api.bindIpFieldValidation('ban-ip', true, 'Ban IP');
  });

  listeners['pow-difficulty:input']();
  listeners['pow-difficulty:blur']();
  listeners['ban-ip:input']();
  listeners['ban-ip:blur']();

  assert.deepEqual(interactions, ['pow-difficulty', 'pow-difficulty', 'ban-ip', 'ban-ip']);
});

test('json object helpers build templates and normalize JSON compare values', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const objectUtils = await importBrowserModule('dashboard/modules/core/json-object.js');
    assert.ok(objectUtils);

    const source = {
      maze_enabled: true,
      provider_backends: { challenge_engine: 'internal', fingerprint_signal: 'external' },
      nested: { deep: { value: 7 } }
    };
    const template = objectUtils.buildTemplateFromPaths(source, [
      'maze_enabled',
      'provider_backends.challenge_engine',
      'nested.deep.value'
    ]);
    assert.deepEqual(toPlain(template), {
      maze_enabled: true,
      provider_backends: { challenge_engine: 'internal' },
      nested: { deep: { value: 7 } }
    });

    assert.equal(objectUtils.normalizeJsonObjectForCompare('{\"a\":1}'), '{\"a\":1}');
    assert.equal(objectUtils.normalizeJsonObjectForCompare('[1,2]'), null);
    assert.equal(objectUtils.normalizeJsonObjectForCompare('{bad json'), null);
  });
});

test('admin endpoint resolver applies loopback override only for local hostnames', { concurrency: false }, async () => {
  await withBrowserGlobals({
    window: {
      location: {
        origin: 'http://127.0.0.1:3000',
        protocol: 'http:',
        host: '127.0.0.1:3000',
        hostname: '127.0.0.1',
        search: '?api_endpoint=http://127.0.0.1:4000/admin'
      }
    }
  }, async () => {
    const endpointModule = await importBrowserModule('dashboard/modules/services/admin-endpoint.js');
    assert.ok(endpointModule);

    const resolve = endpointModule.createAdminEndpointResolver({ window });
    const first = resolve();
    const second = resolve();
    assert.equal(first.endpoint, 'http://127.0.0.1:4000/admin');
    assert.equal(first, second);
  });

  await withBrowserGlobals({
    window: {
      location: {
        origin: 'https://example.com',
        protocol: 'https:',
        host: 'example.com',
        hostname: 'example.com',
        search: '?api_endpoint=http://127.0.0.1:4000/admin'
      }
    }
  }, async () => {
    const endpointModule = await importBrowserModule('dashboard/modules/services/admin-endpoint.js');
    const resolve = endpointModule.createAdminEndpointResolver({ window });
    assert.equal(resolve().endpoint, 'https://example.com');
  });
});

test('tab state view updates element state and dashboard-state hooks', { concurrency: false }, async () => {
  const stateEl = {
    hidden: true,
    textContent: '',
    className: 'tab-state'
  };
  const calls = [];
  const stateStore = {
    setTabLoading: (...args) => calls.push(['setTabLoading', ...args]),
    clearTabError: (...args) => calls.push(['clearTabError', ...args]),
    setTabError: (...args) => calls.push(['setTabError', ...args]),
    setTabEmpty: (...args) => calls.push(['setTabEmpty', ...args]),
    markTabUpdated: (...args) => calls.push(['markTabUpdated', ...args])
  };

  await withBrowserGlobals({}, async () => {
    const tabStateModule = await importBrowserModule('dashboard/modules/tab-state-view.js');
    const view = tabStateModule.create({
      query: (selector) => (selector === '[data-tab-state="monitoring"]' ? stateEl : null),
      getStateStore: () => stateStore
    });

    view.showLoading('monitoring', 'Loading monitoring...');
    assert.equal(stateEl.hidden, false);
    assert.equal(stateEl.className, 'tab-state tab-state--loading');
    assert.equal(stateEl.textContent, 'Loading monitoring...');

    view.showError('monitoring', 'failed');
    assert.equal(stateEl.className, 'tab-state tab-state--error');
    assert.equal(stateEl.textContent, 'failed');

    view.showEmpty('monitoring', 'no data');
    assert.equal(stateEl.className, 'tab-state tab-state--empty');
    assert.equal(stateEl.textContent, 'no data');

    view.clear('monitoring');
    assert.equal(stateEl.hidden, true);
    assert.equal(stateEl.className, 'tab-state');
    assert.equal(stateEl.textContent, '');
  });

  const callNames = calls.map((entry) => entry[0]);
  assert.deepEqual(callNames, [
    'setTabLoading',
    'clearTabError',
    'setTabError',
    'setTabEmpty',
    'setTabEmpty',
    'clearTabError',
    'markTabUpdated',
    'setTabLoading',
    'setTabEmpty',
    'clearTabError',
    'markTabUpdated'
  ]);
});

test('runtime effects request resolves window fetch at call time', { concurrency: false }, async () => {
  let initialFetchCalls = 0;
  let wrappedFetchCalls = 0;
  let replacedUrl = '';
  const hashListeners = [];
  const mockWindow = {
    fetch: async () => {
      initialFetchCalls += 1;
      return { ok: true };
    },
    setTimeout,
    clearTimeout,
    requestAnimationFrame: (task) => {
      task();
      return 9;
    },
    cancelAnimationFrame: () => {},
    location: {
      pathname: '/dashboard/index.html',
      search: '',
      hash: '#monitoring'
    },
    history: {
      replaceState: (_state, _title, url) => {
        replacedUrl = String(url || '');
      }
    },
    addEventListener: (eventName, handler) => {
      if (eventName === 'hashchange') {
        hashListeners.push(handler);
      }
    },
    removeEventListener: (eventName, handler) => {
      if (eventName !== 'hashchange') return;
      const idx = hashListeners.indexOf(handler);
      if (idx >= 0) hashListeners.splice(idx, 1);
    }
  };

  await withBrowserGlobals({
    window: mockWindow,
    navigator: {
      clipboard: {
        writeText: async () => {}
      }
    }
  }, async () => {
    const runtimeEffectsModule = await importBrowserModule('dashboard/modules/services/runtime-effects.js');
    const effects = runtimeEffectsModule.createRuntimeEffects({ window: mockWindow, navigator });

    // Simulate admin-session wrapper installing after runtime effects are created.
    mockWindow.fetch = async () => {
      wrappedFetchCalls += 1;
      return { ok: true };
    };

    await effects.request('/admin/config', { method: 'POST' });
    assert.equal(initialFetchCalls, 0);
    assert.equal(wrappedFetchCalls, 1);

    assert.equal(effects.readHash(), '#monitoring');
    effects.setHash('status');
    assert.equal(mockWindow.location.hash, '#status');
    effects.replaceHash('config');
    assert.equal(replacedUrl.endsWith('#config'), true);
    const offHashChange = effects.onHashChange(() => {});
    assert.equal(hashListeners.length, 1);
    offHashChange();
    assert.equal(hashListeners.length, 0);
  });
});

test('config controls flattens grouped bind options and preserves explicit overrides', { concurrency: false }, async () => {
  const mockDocument = {
    getElementById: () => null,
    querySelector: () => null,
    querySelectorAll: () => []
  };

  await withBrowserGlobals({ document: mockDocument }, async () => {
    const controls = await importBrowserModule('dashboard/modules/config-controls.js');
    assert.ok(controls);

    const grouped = {
      callbacks: { getAdminContext: () => ({ endpoint: 'http://example.test', apikey: 'x' }) },
      readers: { readIntegerFieldValue: () => 42 },
      checks: { checkMazeConfigChanged: () => {} },
      state: { setMazeSavedState: () => {} }
    };
    const flattened = controls._flattenBindOptions(grouped);
    assert.equal(typeof flattened.getAdminContext, 'function');
    assert.equal(typeof flattened.readIntegerFieldValue, 'function');
    assert.equal(typeof flattened.checkMazeConfigChanged, 'function');
    assert.equal(typeof flattened.setMazeSavedState, 'function');

    const explicit = () => 7;
    const explicitWins = controls._flattenBindOptions({
      readIntegerFieldValue: explicit,
      readers: { readIntegerFieldValue: () => 9 }
    });
    assert.equal(explicitWins.readIntegerFieldValue, explicit);
  });
});

test('config controls normalizes typed context into compatibility surface', { concurrency: false }, async () => {
  const mockDocument = {
    getElementById: () => null,
    querySelector: () => null,
    querySelectorAll: () => []
  };

  await withBrowserGlobals({ document: mockDocument }, async () => {
    const controls = await importBrowserModule('dashboard/modules/config-controls.js');
    const normalized = controls._normalizeContextOptions({
      context: {
        statusPanel: { update: () => {}, render: () => {} },
        apiClient: { updateConfig: async () => ({ config: {} }) },
        auth: { getAdminContext: () => ({ endpoint: 'http://x', apikey: 'y' }) },
        readers: { readIntegerFieldValue: () => 1 },
        checks: { checkMazeConfigChanged: () => {} },
        draft: {
          get: (key, fallback) => (key === 'geo' ? { mutable: true } : fallback),
          set: () => {}
        }
      }
    });

    assert.equal(typeof normalized.getAdminContext, 'function');
    assert.equal(typeof normalized.readIntegerFieldValue, 'function');
    assert.equal(typeof normalized.checkMazeConfigChanged, 'function');
    assert.equal(typeof normalized.getGeoSavedState, 'function');
    assert.equal(typeof normalized.setMazeSavedState, 'function');
    assert.deepEqual(normalized.getGeoSavedState(), { mutable: true });
  });
});

test('config controls accepts domainApi without legacy grouped callback bags', { concurrency: false }, async () => {
  const mockDocument = {
    getElementById: () => null,
    querySelector: () => null,
    querySelectorAll: () => []
  };

  await withBrowserGlobals({ document: mockDocument }, async () => {
    const controls = await importBrowserModule('dashboard/modules/config-controls.js');
    const normalized = controls._normalizeContextOptions({
      domainApi: {
        getAdminContext: () => ({ endpoint: 'http://x', apikey: 'y' }),
        readIntegerFieldValue: () => 42,
        setMazeSavedState: () => {},
        checkMazeConfigChanged: () => {}
      }
    });

    assert.equal(typeof normalized.getAdminContext, 'function');
    assert.equal(typeof normalized.readIntegerFieldValue, 'function');
    assert.equal(typeof normalized.setMazeSavedState, 'function');
    assert.equal(typeof normalized.checkMazeConfigChanged, 'function');
  });
});

test('config controls registry save pipeline persists maze settings through generic handler', { concurrency: false }, async () => {
  const saveButton = createMockElement({ disabled: false, dataset: { saving: 'false' } });
  const adminMsg = createMockElement({ className: '' });
  const mazeEnabledToggle = createMockElement({ checked: true });
  const mazeAutoBanToggle = createMockElement({ checked: false });
  const mazeThreshold = createMockElement({ value: '77' });
  const savedStates = [];
  const checks = [];
  const patches = [];

  const elements = {
    'save-maze-config': saveButton,
    'admin-msg': adminMsg,
    'maze-enabled-toggle': mazeEnabledToggle,
    'maze-auto-ban-toggle': mazeAutoBanToggle,
    'maze-threshold': mazeThreshold
  };

  await withBrowserGlobals({
    document: {
      getElementById: (id) => elements[id] || null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const controls = await importBrowserModule('dashboard/modules/config-controls.js');
    controls.bind({
      context: {
        statusPanel: {
          applyPatch: () => {}
        },
        apiClient: {
          updateConfig: async (patch) => {
            patches.push(patch);
            return { config: patch };
          }
        },
        auth: {
          getAdminContext: () => ({ endpoint: 'http://example.test', apikey: 'abc' })
        },
        readers: {
          readIntegerFieldValue: (id) => Number.parseInt(elements[id].value, 10)
        },
        checks: {
          checkMazeConfigChanged: () => checks.push('maze')
        },
        draft: {
          get: () => ({}),
          set: (_, value) => savedStates.push(value)
        },
        effects: {
          setTimer: (task) => {
            task();
            return 1;
          }
        }
      }
    });
  });

  assert.equal(typeof saveButton.onclick, 'function');
  await saveButton.onclick();
  assert.deepEqual(patches, [
    {
      maze_enabled: true,
      maze_auto_ban: false,
      maze_auto_ban_threshold: 77
    }
  ]);
  assert.deepEqual(savedStates.pop(), { enabled: true, autoBan: false, threshold: 77 });
  assert.deepEqual(checks, ['maze']);
  assert.equal(saveButton.textContent, 'Save Maze Settings');
  assert.equal(saveButton.dataset.saving, 'false');
});

test('config ui state declarative bindings hydrate robots controls and reset save buttons', { concurrency: false }, async () => {
  const elements = {
    'robots-enabled-toggle': createMockElement({ checked: false }),
    'robots-block-training-toggle': createMockElement({ checked: false }),
    'robots-block-search-toggle': createMockElement({ checked: false }),
    'robots-allow-search-toggle': createMockElement({ checked: false }),
    'robots-crawl-delay': createMockElement({ value: '0' }),
    'save-robots-config': createMockElement({ disabled: false, dataset: { saving: 'true' } }),
    'save-ai-policy-config': createMockElement({ disabled: false, dataset: { saving: 'true' } })
  };
  const drafts = {};

  await withBrowserGlobals({
    document: {
      getElementById: (id) => elements[id] || null,
      querySelector: () => null,
      querySelectorAll: () => []
    }
  }, async () => {
    const module = await importBrowserModule('dashboard/modules/config-ui-state.js');
    const api = module.create({
      getById: (id) => elements[id] || null,
      setDraft: (section, value) => {
        drafts[section] = JSON.parse(JSON.stringify(value));
      },
      getDraft: () => ({})
    });

    api.updateRobotsConfig({
      robots_enabled: true,
      robots_crawl_delay: 3,
      ai_policy_block_training: true,
      ai_policy_block_search: false,
      ai_policy_allow_search_engines: false
    });
  });

  assert.equal(elements['robots-enabled-toggle'].checked, true);
  assert.equal(elements['robots-crawl-delay'].value, 3);
  assert.equal(elements['robots-block-training-toggle'].checked, true);
  assert.equal(elements['robots-block-search-toggle'].checked, false);
  assert.equal(elements['robots-allow-search-toggle'].checked, true);
  assert.deepEqual(drafts.robots, { enabled: true, crawlDelay: 3 });
  assert.deepEqual(drafts.aiPolicy, { blockTraining: true, blockSearch: false, allowSearch: true });
  assert.equal(elements['save-robots-config'].disabled, true);
  assert.equal(elements['save-robots-config'].dataset.saving, 'false');
  assert.equal(elements['save-ai-policy-config'].disabled, true);
  assert.equal(elements['save-ai-policy-config'].dataset.saving, 'false');
});

test('tables view wires quick-unban callback and detail toggle handlers', { concurrency: false }, async () => {
  const bansTbody = {
    innerHTML: '',
    rows: [],
    appendChild(node) {
      this.rows.push(node);
    }
  };
  const detailRow = {
    classList: {
      _hidden: true,
      toggle(name) {
        if (name === 'hidden') this._hidden = !this._hidden;
      },
      contains(name) {
        return name === 'hidden' ? this._hidden : false;
      }
    }
  };
  const detailsBtn = { dataset: { target: 'ban-detail-2030011310' }, textContent: 'Details' };
  const quickUnbanBtn = { dataset: { ip: '203.0.113.10' } };
  const unbanned = [];

  await withBrowserGlobals({
    document: {
      querySelector: (selector) => (selector === '#bans-table tbody' ? bansTbody : null),
      querySelectorAll: (selector) => {
        if (selector === '.ban-details-toggle') return [detailsBtn];
        if (selector === '.unban-quick') return [quickUnbanBtn];
        return [];
      },
      createElement: () => ({ innerHTML: '', id: '', className: '' }),
      getElementById: (id) => (id === 'ban-detail-2030011310' ? detailRow : null)
    }
  }, async () => {
    const tablesModule = await importBrowserModule('dashboard/modules/tables-view.js');
    const tables = tablesModule.create({
      onQuickUnban: async (ip) => {
        unbanned.push(ip);
      }
    });
    tables.updateBansTable([
      {
        ip: '203.0.113.10',
        reason: 'test_reason',
        banned_at: 1700000000,
        expires: 1900000000,
        fingerprint: { signals: ['ua_transport_mismatch'], score: 5, summary: 'sample' }
      }
    ]);
  });

  assert.equal(typeof detailsBtn.onclick, 'function');
  assert.equal(typeof quickUnbanBtn.onclick, 'function');

  detailsBtn.onclick();
  assert.equal(detailRow.classList.contains('hidden'), false);
  assert.equal(detailsBtn.textContent, 'Hide');

  await quickUnbanBtn.onclick();
  assert.deepEqual(unbanned, ['203.0.113.10']);
});

test('tables view renders empty-state rows for bans and events', { concurrency: false }, async () => {
  const bansTbody = { innerHTML: '', appendChild: () => {} };
  const eventsTbody = { innerHTML: '', appendChild: () => {} };

  await withBrowserGlobals({
    document: {
      querySelector: (selector) => {
        if (selector === '#bans-table tbody') return bansTbody;
        if (selector === '#events tbody') return eventsTbody;
        return null;
      },
      querySelectorAll: () => [],
      createElement: () => ({ innerHTML: '' }),
      getElementById: () => null
    }
  }, async () => {
    const tablesModule = await importBrowserModule('dashboard/modules/tables-view.js');
    const tables = tablesModule.create();
    tables.updateBansTable([]);
    tables.updateEventsTable([]);
  });

  assert.equal(bansTbody.innerHTML.includes('No active bans'), true);
  assert.equal(eventsTbody.innerHTML.includes('No recent events'), true);
});

test('tables view patches monitoring rows in place to reduce refresh churn', { concurrency: false }, async () => {
  const createTableBody = () => ({
    innerHTML: '',
    children: [],
    appendChild(node) {
      this.children.push(node);
    },
    replaceChild(next, prev) {
      const index = this.children.indexOf(prev);
      if (index >= 0) {
        this.children[index] = next;
        return;
      }
      this.children.push(next);
    },
    removeChild(node) {
      const index = this.children.indexOf(node);
      if (index >= 0) {
        this.children.splice(index, 1);
      }
    }
  });

  const eventsTbody = createTableBody();
  const cdpTbody = createTableBody();

  await withBrowserGlobals({
    document: {
      querySelector: (selector) => {
        if (selector === '#events tbody') return eventsTbody;
        if (selector === '#cdp-events tbody') return cdpTbody;
        return null;
      },
      querySelectorAll: () => [],
      createElement: () => ({ innerHTML: '', dataset: {} }),
      getElementById: () => null
    }
  }, async () => {
    const tablesModule = await importBrowserModule('dashboard/modules/tables-view.js');
    const tables = tablesModule.create();

    const firstEvent = {
      ts: 1700000000,
      event: 'Ban',
      ip: '203.0.113.10',
      reason: 'test',
      outcome: 'blocked',
      admin: 'ops'
    };
    const secondEvent = {
      ts: 1700001000,
      event: 'Challenge',
      ip: '203.0.113.11',
      reason: 'risk',
      outcome: 'served',
      admin: 'ops'
    };
    tables.updateEventsTable([firstEvent, secondEvent]);
    const firstEventRowRef = eventsTbody.children[0];

    tables.updateEventsTable([firstEvent]);
    assert.equal(eventsTbody.children.length, 1);
    assert.equal(eventsTbody.children[0], firstEventRowRef);

    const firstCdpEvent = {
      ts: 1700002000,
      ip: '198.51.100.20',
      reason: 'cdp_detected:tier=medium score=75',
      outcome: 'checks:webdriver',
      admin: 'ops'
    };
    const secondCdpEvent = {
      ts: 1700003000,
      ip: '198.51.100.21',
      reason: 'cdp_automation',
      outcome: 'tier=strong score=99',
      admin: 'ops'
    };
    tables.updateCdpEventsTable([firstCdpEvent, secondCdpEvent]);
    const firstCdpRowRef = cdpTbody.children[0];

    tables.updateCdpEventsTable([firstCdpEvent]);
    assert.equal(cdpTbody.children.length, 1);
    assert.equal(cdpTbody.children[0], firstCdpRowRef);
  });
});

test('tables view updates CDP totals and parses tier/score fields', { concurrency: false }, async () => {
  const els = {
    'cdp-total-detections': createMockElement(),
    'cdp-total-auto-bans': createMockElement(),
    'cdp-fp-events': createMockElement(),
    'cdp-fp-flow-violations': createMockElement()
  };

  await withBrowserGlobals({
    document: {
      querySelector: () => null,
      querySelectorAll: () => [],
      createElement: () => ({ innerHTML: '' }),
      getElementById: (id) => els[id] || null
    }
  }, async () => {
    const tablesModule = await importBrowserModule('dashboard/modules/tables-view.js');
    const tables = tablesModule.create();
    tables.updateCdpTotals({
      stats: { total_detections: 12, auto_bans: 3 },
      fingerprint_stats: {
        ua_client_hint_mismatch: 5,
        ua_transport_mismatch: 4,
        temporal_transition: 2,
        flow_violation: 6
      }
    });

    assert.equal(tables._extractCdpField('tier=strong score=98', 'tier'), 'strong');
    assert.equal(tables._extractCdpField('tier=strong score=98', 'score'), '98');
  });

  assert.equal(els['cdp-total-detections'].textContent, '12');
  assert.equal(els['cdp-total-auto-bans'].textContent, '3');
  assert.equal(els['cdp-fp-events'].textContent, '11');
  assert.equal(els['cdp-fp-flow-violations'].textContent, '6');
});

test('tables view monitoring loading state sets CDP placeholders', { concurrency: false }, async () => {
  const els = {
    'cdp-total-detections': createMockElement(),
    'cdp-total-auto-bans': createMockElement(),
    'cdp-fp-events': createMockElement(),
    'cdp-fp-flow-violations': createMockElement()
  };

  await withBrowserGlobals({
    document: {
      querySelector: () => null,
      querySelectorAll: () => [],
      createElement: () => ({ innerHTML: '' }),
      getElementById: (id) => els[id] || null
    }
  }, async () => {
    const tablesModule = await importBrowserModule('dashboard/modules/tables-view.js');
    const tables = tablesModule.create();
    tables.showMonitoringLoadingState();
  });

  assert.equal(els['cdp-total-detections'].textContent, '...');
  assert.equal(els['cdp-total-auto-bans'].textContent, '...');
  assert.equal(els['cdp-fp-events'].textContent, '...');
  assert.equal(els['cdp-fp-flow-violations'].textContent, '...');
});

test('dashboard ESM guardrails forbid legacy global registry and class syntax', () => {
  const dashboardRoot = path.resolve(__dirname, '..', 'dashboard');
  const moduleFiles = listJsFilesRecursively(path.join(dashboardRoot, 'modules'));
  const filesToCheck = [path.join(dashboardRoot, 'dashboard.js'), ...moduleFiles];
  const legacyGlobalPattern = /\b(?:window|globalThis|global)\.ShumaDashboard[A-Za-z0-9_]*\b/;
  const classDeclarationPattern = /\bclass\s+[A-Za-z_$][\w$]*\b/;

  filesToCheck.forEach((filePath) => {
    const source = fs.readFileSync(filePath, 'utf8');
    const analyzable = stripCommentsAndStrings(source);
    assert.equal(
      legacyGlobalPattern.test(analyzable),
      false,
      `legacy dashboard global registry reference found in ${filePath}`
    );
    assert.equal(
      classDeclarationPattern.test(analyzable),
      false,
      `class declaration found in ${filePath}`
    );
  });
});

test('dashboard main wires config UI state through module factory', () => {
  const dashboardPath = path.resolve(__dirname, '..', 'dashboard', 'dashboard.js');
  const source = fs.readFileSync(dashboardPath, 'utf8');

  assert.equal(
    source.includes('configUiState = configUiStateModule.create('),
    true,
    'dashboard.js must initialize configUiState from config-ui-state module'
  );
});

test('dashboard main uses mount-scoped control bindings with explicit teardown', () => {
  const dashboardPath = path.resolve(__dirname, '..', 'dashboard', 'dashboard.js');
  const source = fs.readFileSync(dashboardPath, 'utf8');

  assert.match(
    source,
    /function bindMountScopedDomEvents\(\)/
  );
  assert.match(
    source,
    /teardownControlBindings = bindMountScopedDomEvents\(\);/
  );
  assert.match(
    source,
    /if \(teardownControlBindings\) \{\s*teardownControlBindings\(\);\s*teardownControlBindings = null;\s*\}/m
  );
  assert.match(
    source,
    /const previewRobotsButton = getById\('preview-robots'\);\s*if \(previewRobotsButton\)/m
  );
  assert.match(
    source,
    /const banButton = getById\('ban-btn'\);\s*if \(banButton\)/m
  );
  assert.match(
    source,
    /const unbanButton = getById\('unban-btn'\);\s*if \(unbanButton\)/m
  );
  assert.match(
    source,
    /\]\.forEach\(\(id\) => \{\s*bindFieldEvent\(id, 'input', checkBotnessConfigChanged\);/m
  );
  assert.equal(
    source.includes('banButton.onclick ='),
    false,
    'ban button handler should be bound at mount via addEventListener, not import-time onclick'
  );
  assert.equal(
    source.includes('unbanButton.onclick ='),
    false,
    'unban button handler should be bound at mount via addEventListener, not import-time onclick'
  );
});

test('svelte route guardrails forbid shell injection and bridge-era imports', () => {
  const dashboardRoot = path.resolve(__dirname, '..', 'dashboard');
  const mainRoutePath = path.join(dashboardRoot, 'src', 'routes', '+page.svelte');
  const loginRoutePath = path.join(dashboardRoot, 'src', 'routes', 'login.html', '+page.svelte');
  const mainSource = fs.readFileSync(mainRoutePath, 'utf8');
  const loginSource = fs.readFileSync(loginRoutePath, 'utf8');

  assert.equal(mainSource.includes('{@html'), false);
  assert.equal(loginSource.includes('{@html'), false);
  assert.equal(mainSource.includes('$lib/bridges/'), false);
  assert.equal(loginSource.includes('$lib/bridges/'), false);
  assert.match(mainSource, /mountDashboardRuntime/);
  assert.equal(mainSource.includes('dashboard_runtime'), false);
  assert.equal(mainSource.includes("import('../../../dashboard.js')"), false);
});

test('svelte dashboard actions route login redirect and focus through effects adapters', () => {
  const actionsPath = path.resolve(
    __dirname,
    '..',
    'dashboard',
    'src',
    'lib',
    'runtime',
    'dashboard-actions.js'
  );
  const source = fs.readFileSync(actionsPath, 'utf8');

  assert.equal(source.includes('window.location'), false);
  assert.equal(source.includes('document.getElementById'), false);
  assert.match(source, /effects\.buildLoginRedirectPath/);
  assert.match(source, /effects\.focusTab/);
});

test('svelte tab semantics avoid interactive-role warnings for tablist and tabpanel', () => {
  const dashboardRoot = path.resolve(__dirname, '..', 'dashboard');
  const mainRoutePath = path.join(dashboardRoot, 'src', 'routes', '+page.svelte');
  const componentPaths = [
    path.join(dashboardRoot, 'src', 'lib', 'components', 'dashboard', 'MonitoringTab.svelte'),
    path.join(dashboardRoot, 'src', 'lib', 'components', 'dashboard', 'IpBansTab.svelte'),
    path.join(dashboardRoot, 'src', 'lib', 'components', 'dashboard', 'StatusTab.svelte'),
    path.join(dashboardRoot, 'src', 'lib', 'components', 'dashboard', 'ConfigTab.svelte'),
    path.join(dashboardRoot, 'src', 'lib', 'components', 'dashboard', 'TuningTab.svelte')
  ];

  const mainSource = fs.readFileSync(mainRoutePath, 'utf8');
  assert.equal(mainSource.includes('role="tablist"'), false);

  componentPaths.forEach((filePath) => {
    const source = fs.readFileSync(filePath, 'utf8');
    assert.equal(source.includes('role="tabpanel"'), false);
    assert.equal(source.includes('tabindex={'), false);
  });
});

test('status tab includes runtime performance telemetry cards and threshold guidance', () => {
  const statusTabPath = path.resolve(
    __dirname,
    '..',
    'dashboard',
    'src',
    'lib',
    'components',
    'dashboard',
    'StatusTab.svelte'
  );
  const source = fs.readFileSync(statusTabPath, 'utf8');
  assert.equal(source.includes('Runtime Performance Telemetry'), true);
  assert.equal(source.includes('runtime-fetch-latency-last'), true);
  assert.equal(source.includes('runtime-render-timing-last'), true);
  assert.equal(source.includes('rolling p95 fetch latency'), true);
  assert.equal(source.includes('window:'), true);
  assert.equal(source.includes('p95:'), true);
  assert.equal(source.includes('runtime-polling-skip-count'), true);
  assert.equal(source.includes('runtime-polling-resume-count'), true);
  assert.equal(source.includes('500 ms'), true);
  assert.equal(source.includes('16 ms'), true);
});

test('dashboard runtime adapter enforces single-mount and explicit teardown hooks', () => {
  const runtimePath = path.resolve(
    __dirname,
    '..',
    'dashboard',
    'src',
    'lib',
    'runtime',
    'dashboard-runtime.js'
  );
  const source = fs.readFileSync(runtimePath, 'utf8');
  assert.match(source, /if \(mounted\) return;/);
  assert.match(source, /if \(mountingPromise\) return mountingPromise;/);
  assert.match(source, /const mountOptions = \{\s*\.\.\.\(options \|\| \{\}\)\s*\};/);
  assert.match(source, /delete mountOptions\.mode;/);
  assert.match(source, /module\.mountDashboardExternalRuntime\(mountOptions \|\| \{\}\)/);
  assert.equal(source.includes('module.mountDashboardApp('), false);
  assert.match(source, /runtimeModule\.unmountDashboardApp\(\)/);
  assert.match(source, /refreshExternalDashboardTab/);
  assert.match(source, /setExternalDashboardActiveTab/);
  assert.match(source, /restoreDashboardSession/);
  assert.match(source, /refreshDashboardTab/);
  assert.match(source, /setDashboardActiveTab/);
});

test('svelte dashboard route mounts the external runtime contract only', () => {
  const routePath = path.resolve(
    __dirname,
    '..',
    'dashboard',
    'src',
    'routes',
    '+page.svelte'
  );
  const source = fs.readFileSync(routePath, 'utf8');

  assert.equal(source.includes("mode: 'external'"), false);
  assert.equal(source.includes("mode: 'legacy'"), false);
  assert.match(source, /mountDashboardRuntime\(\{\s*initialTab:/m);
  assert.match(source, /<link rel="preload" href=\{chartLiteSrc\} as="script">/);
  assert.match(source, /<script src=\{chartLiteSrc\} data-shuma-runtime-script=\{chartLiteSrc\}><\/script>/);
  assert.equal(source.includes('function ensureScript('), false);
  assert.equal(source.includes('document.createElement(\'script\')'), false);
  assert.equal(source.includes('useExternalTabPipeline'), false);
  assert.equal(source.includes('useExternalPollingPipeline'), false);
  assert.equal(source.includes('useExternalSessionPipeline'), false);
});

test('dashboard main delegates legacy tab/session/polling orchestration to runtime module', () => {
  const dashboardPath = path.resolve(__dirname, '..', 'dashboard', 'dashboard.js');
  const source = fs.readFileSync(dashboardPath, 'utf8');

  assert.match(source, /createLegacyDashboardTabRuntime/);
  assert.match(source, /createLegacyDashboardSessionRuntime/);
  assert.match(source, /createLegacyAutoRefreshRuntime/);
  assert.match(source, /legacyTabRuntime = createLegacyDashboardTabRuntime\(/);
  assert.match(source, /legacySessionRuntime = createLegacyDashboardSessionRuntime\(/);
  assert.match(source, /legacyAutoRefreshRuntime = createLegacyAutoRefreshRuntime\(/);
  assert.equal(source.includes('function scheduleAutoRefresh()'), false);
  assert.equal(source.includes('function bindVisibilityHandler()'), false);
  assert.equal(source.includes('function clearAutoRefreshTimer()'), false);
});

test('dashboard main delegates config dirty-check orchestration to runtime module', () => {
  const dashboardPath = path.resolve(__dirname, '..', 'dashboard', 'dashboard.js');
  const source = fs.readFileSync(dashboardPath, 'utf8');

  assert.match(source, /createLegacyConfigDirtyRuntime/);
  assert.match(source, /legacyConfigDirtyRuntime = createLegacyConfigDirtyRuntime\(/);
  assert.match(source, /legacyConfigDirtyRuntime\.runCoreChecks\(\)/);
  assert.equal(source.includes('DIRTY_CHECK_REGISTRY'), false);
});

test('monitoring auto-refresh uses consolidated monitoring summary reads to bound poll fan-out', () => {
  const dashboardPath = path.resolve(__dirname, '..', 'dashboard', 'dashboard.js');
  const source = fs.readFileSync(dashboardPath, 'utf8');

  assert.match(source, /if \(isAutoRefresh\) \{/);
  assert.match(
    source,
    /const monitoringData = await dashboardApiClient\.getMonitoring\(\{ hours: 24, limit: 10 \}, requestOptions\);/
  );
  assert.match(source, /dashboardState\.setSnapshot\('monitoring', monitoringData\)/);
  assert.equal(source.includes("const analyticsPromise = reason === 'auto-refresh'"), false);
  assert.match(source, /dashboardApiClient\.getEvents\(24, requestOptions\)/);
  assert.match(source, /dashboardApiClient\.getBans\(requestOptions\)/);
  assert.match(source, /dashboardApiClient\.getMaze\(requestOptions\)/);
  assert.match(source, /dashboardApiClient\.getCdp\(requestOptions\)/);
  assert.match(source, /dashboardApiClient\.getCdpEvents\(\{ hours: 24, limit: 500 \}, requestOptions\)/);
});

test('legacy orchestration runtime provides tab/session/polling primitives', { concurrency: false }, async () => {
  const module = await importBrowserModule('dashboard/src/lib/runtime/dashboard-legacy-orchestration.js');

  const tabState = {
    activeTab: 'monitoring',
    session: { authenticated: false, csrfToken: '' },
    setActiveTab(nextTab) {
      this.activeTab = nextTab;
    },
    getActiveTab() {
      return this.activeTab;
    },
    setSession(nextSession) {
      this.session = { ...nextSession };
    }
  };
  const tabCoordinator = {
    activeTab: 'monitoring',
    activations: [],
    activate(tab, reason) {
      this.activeTab = tab;
      this.activations.push({ tab, reason });
    },
    getActiveTab() {
      return this.activeTab;
    }
  };
  const mountOptions = { useExternalTabPipeline: false };
  const documentStub = {
    visibilityState: 'visible',
    body: { dataset: {} },
    listener: null,
    addEventListener(eventName, handler) {
      if (eventName === 'visibilitychange') this.listener = handler;
    },
    removeEventListener(eventName, handler) {
      if (eventName === 'visibilitychange' && this.listener === handler) {
        this.listener = null;
      }
    }
  };
  const refreshCalls = [];
  const tabRuntime = module.createLegacyDashboardTabRuntime({
    document: documentStub,
    normalizeTab: (tab) => String(tab || 'monitoring'),
    defaultTab: 'monitoring',
    getStateStore: () => tabState,
    getTabCoordinator: () => tabCoordinator,
    getRuntimeMountOptions: () => mountOptions,
    refreshCoreActionButtonsState: () => {},
    refreshDashboardForTab: async (tab, reason) => {
      refreshCalls.push({ tab, reason });
    }
  });

  assert.equal(tabRuntime.setActiveTab('status', 'click'), 'status');
  assert.equal(tabState.activeTab, 'status');
  assert.equal(documentStub.body.dataset.activeDashboardTab, 'status');
  assert.deepEqual(tabCoordinator.activations[0], { tab: 'status', reason: 'click' });

  await tabRuntime.refreshTab('config', 'manual');
  assert.deepEqual(refreshCalls[0], { tab: 'config', reason: 'manual' });

  const sessionController = {
    state: { authenticated: true, csrfToken: 'csrf-token' },
    restoreCount: 0,
    async restoreAdminSession() {
      this.restoreCount += 1;
      return this.state.authenticated === true;
    },
    getState() {
      return { ...this.state };
    }
  };
  let logoutRequest = null;
  const sessionMessage = { textContent: '', className: '' };
  const sessionRuntime = module.createLegacyDashboardSessionRuntime({
    getAdminSessionController: () => sessionController,
    getStateStore: () => tabState,
    refreshCoreActionButtonsState: () => {},
    resolveAdminApiEndpoint: () => ({ endpoint: 'https://example.test' }),
    getRuntimeEffects: () => ({
      request: async (input, init) => {
        logoutRequest = { input, init };
      }
    }),
    getMessageNode: () => sessionMessage
  });

  assert.equal(await sessionRuntime.restoreSession(), true);
  assert.deepEqual(tabState.session, { authenticated: true, csrfToken: 'csrf-token' });
  assert.deepEqual(sessionRuntime.getSessionState(), { authenticated: true, csrfToken: 'csrf-token' });

  await sessionRuntime.logoutSession();
  assert.ok(logoutRequest);
  assert.equal(logoutRequest.input, 'https://example.test/admin/logout');
  assert.equal(logoutRequest.init.method, 'POST');
  assert.equal(logoutRequest.init.credentials, 'same-origin');
  assert.equal(sessionMessage.textContent, 'Logged out');
  assert.equal(sessionMessage.className, 'message success');
  assert.deepEqual(tabState.session, { authenticated: false, csrfToken: '' });

  let timerHandle = null;
  let clearCount = 0;
  const pollingEffects = {
    setTimer(handler, intervalMs) {
      timerHandle = { handler, intervalMs };
      return timerHandle;
    },
    clearTimer() {
      clearCount += 1;
    }
  };
  const pollRefreshes = [];
  const pollingRuntime = module.createLegacyAutoRefreshRuntime({
    effects: pollingEffects,
    document: documentStub,
    tabRefreshIntervals: { monitoring: 30000, config: 60000 },
    defaultTab: 'monitoring',
    normalizeTab: (tab) => String(tab || 'monitoring'),
    getActiveTab: () => tabState.activeTab,
    hasValidApiContext: () => true,
    refreshDashboardForTab: async (tab, reason) => {
      pollRefreshes.push({ tab, reason });
    }
  });

  pollingRuntime.schedule();
  assert.ok(timerHandle);
  assert.equal(timerHandle.intervalMs, 60000);
  await timerHandle.handler();
  assert.deepEqual(pollRefreshes[0], { tab: 'config', reason: 'auto-refresh' });

  pollingRuntime.bindVisibility();
  documentStub.visibilityState = 'hidden';
  if (typeof documentStub.listener === 'function') {
    documentStub.listener();
  }
  assert.ok(clearCount >= 1);

  pollingRuntime.destroy();
});

test('legacy config dirty runtime computes dirty state through capability contracts', { concurrency: false }, async () => {
  const module = await importBrowserModule('dashboard/src/lib/runtime/dashboard-legacy-config-dirty.js');
  const nodes = {
    'save-maze-config': { dataset: {}, disabled: true, textContent: '' },
    'save-robots-config': { dataset: {}, disabled: true, textContent: '' },
    'save-ai-policy-config': { dataset: {}, disabled: true, textContent: '' },
    'maze-enabled-toggle': { checked: true, dataset: {} },
    'maze-auto-ban-toggle': { checked: true, dataset: {} },
    'robots-enabled-toggle': { checked: true, dataset: {} },
    'robots-crawl-delay': { value: '2', dataset: {} },
    'robots-block-training-toggle': { checked: true, dataset: {} },
    'robots-block-search-toggle': { checked: false, dataset: {} },
    'robots-allow-search-toggle': { checked: true, dataset: {} }
  };
  const dirtyCalls = [];
  const runtime = module.createLegacyConfigDirtyRuntime({
    getById: (id) => nodes[id] || null,
    getDraft: (sectionKey) => {
      if (sectionKey === 'maze') return { enabled: false, autoBan: false, threshold: 10 };
      if (sectionKey === 'robots') return { enabled: false, crawlDelay: 1 };
      if (sectionKey === 'aiPolicy') {
        return { blockTraining: false, blockSearch: false, allowSearch: false };
      }
      return {};
    },
    isDraftDirty: () => true,
    hasValidApiContext: () => true,
    validateIntegerFieldById: () => true,
    parseIntegerLoose: () => 25,
    readBanDurationFromInputs: () => ({ totalSeconds: 3600 }),
    validateHoneypotPathsField: () => true,
    validateBrowserRulesField: () => true,
    normalizeListTextareaForCompare: (value) => String(value || '').trim(),
    normalizeBrowserRulesForCompare: (value) => String(value || '').trim(),
    normalizeEdgeIntegrationMode: (value) => String(value || '').trim(),
    setDirtySaveButtonState: (buttonId, changed, apiValid, fieldsValid) => {
      dirtyCalls.push({ buttonId, changed, apiValid, fieldsValid });
    }
  });

  runtime.checkMaze();
  runtime.checkRobots();
  runtime.checkAiPolicy();

  assert.equal(dirtyCalls.some((entry) => entry.buttonId === 'save-maze-config' && entry.changed), true);
  assert.equal(dirtyCalls.some((entry) => entry.buttonId === 'save-robots-config' && entry.changed), true);
  assert.equal(dirtyCalls.some((entry) => entry.buttonId === 'save-ai-policy-config' && entry.changed), true);
  assert.equal(nodes['save-robots-config'].textContent, 'Save robots serving');
  assert.equal(nodes['save-ai-policy-config'].textContent, 'Save AI bot policy');
});

test('dashboard main exports explicit lifecycle entrypoints', () => {
  const dashboardPath = path.resolve(__dirname, '..', 'dashboard', 'dashboard.js');
  const source = fs.readFileSync(dashboardPath, 'utf8');

  assert.match(source, /export function mountDashboardApp\(options = \{\}\)/);
  assert.match(source, /export function unmountDashboardApp\(\)/);
});

test('dashboard module graph is layered (core -> services -> features -> main) with no cycles', () => {
  const dashboardRoot = path.resolve(__dirname, '..', 'dashboard');
  const moduleRoot = path.join(dashboardRoot, 'modules');
  const moduleFiles = listJsFilesRecursively(moduleRoot);
  const allFiles = [path.join(dashboardRoot, 'dashboard.js'), ...moduleFiles];

  const relativeOf = (absolutePath) =>
    path.relative(dashboardRoot, absolutePath).split(path.sep).join('/');
  const rankOf = (relativePath) => {
    if (relativePath === 'dashboard.js') return 3; // main
    if (relativePath.startsWith('modules/core/')) return 0;
    if (relativePath.startsWith('modules/services/')) return 1;
    if (
      relativePath === 'modules/api-client.js' ||
      relativePath === 'modules/admin-session.js' ||
      relativePath === 'modules/dashboard-state.js' ||
      relativePath === 'modules/tab-lifecycle.js' ||
      relativePath === 'modules/config-schema.js' ||
      relativePath === 'modules/config-form-utils.js' ||
      relativePath === 'modules/config-draft-store.js'
    ) {
      return 1;
    }
    if (relativePath.startsWith('modules/')) return 2;
    return 99;
  };

  const knownFiles = new Set(allFiles.map((filePath) => relativeOf(filePath)));
  const adjacency = new Map();
  const rankErrors = [];

  allFiles.forEach((filePath) => {
    const fromRel = relativeOf(filePath);
    const fromDir = path.dirname(filePath);
    const fromRank = rankOf(fromRel);
    const source = fs.readFileSync(filePath, 'utf8');
    const imports = parseRelativeImports(source);
    const edges = [];

    imports.forEach((specifier) => {
      const resolvedAbsolute = path.resolve(fromDir, specifier);
      const withJs = `${resolvedAbsolute}.js`;
      const candidateAbsolute =
        fs.existsSync(resolvedAbsolute) && fs.statSync(resolvedAbsolute).isFile()
          ? resolvedAbsolute
          : (fs.existsSync(withJs) ? withJs : null);
      if (!candidateAbsolute) return;

      const toRel = relativeOf(candidateAbsolute);
      if (!knownFiles.has(toRel)) return;
      edges.push(toRel);

      const toRank = rankOf(toRel);
      if (toRank > fromRank) {
        rankErrors.push(`${fromRel} imports higher layer ${toRel}`);
      }
    });

    adjacency.set(fromRel, edges);
  });

  assert.deepEqual(rankErrors, [], `layering violations:\n${rankErrors.join('\n')}`);
  const cycles = detectCycles(adjacency);
  assert.equal(cycles.length, 0, `module import cycles found:\n${JSON.stringify(cycles, null, 2)}`);
});

test('dashboard state reducer transitions are immutable', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const stateApi = await importBrowserModule('dashboard/modules/dashboard-state.js');

    const initial = stateApi.createInitialState('monitoring');
    const nextActive = stateApi.reduceState(initial, { type: 'set-active-tab', tab: 'status' });
    assert.notEqual(nextActive, initial);
    assert.equal(initial.activeTab, 'monitoring');
    assert.equal(nextActive.activeTab, 'status');

    const nextInvalidated = stateApi.reduceState(nextActive, { type: 'invalidate', scope: 'ip-bans' });
    assert.notEqual(nextInvalidated, nextActive);
    assert.notEqual(nextInvalidated.stale, nextActive.stale);
    assert.equal(nextInvalidated.stale['ip-bans'], true);

    const nextSession = stateApi.reduceState(nextInvalidated, {
      type: 'set-session',
      session: { authenticated: true, csrfToken: 'abc' }
    });
    assert.notEqual(nextSession, nextInvalidated);
    assert.notEqual(nextSession.session, nextInvalidated.session);
    assert.equal(nextSession.session.authenticated, true);
    assert.equal(nextSession.session.csrfToken, 'abc');
  });
});
