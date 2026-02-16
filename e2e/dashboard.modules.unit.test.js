const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

function loadBrowserModule(relativePath, overrides = {}) {
  const absolutePath = path.resolve(__dirname, '..', relativePath);
  const source = fs.readFileSync(absolutePath, 'utf8');
  const sandbox = {
    window: {
      ...overrides
    },
    fetch: overrides.fetch || (typeof fetch === 'undefined' ? undefined : fetch),
    console,
    URL,
    Headers: typeof Headers === 'undefined' ? function HeadersShim() {} : Headers,
    Request: typeof Request === 'undefined' ? function RequestShim() {} : Request,
    Response: typeof Response === 'undefined' ? function ResponseShim() {} : Response
  };
  sandbox.globalThis = sandbox.window;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: absolutePath });
  return sandbox.window;
}

function toPlain(value) {
  return JSON.parse(JSON.stringify(value));
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

test('dashboard API adapters normalize sparse payloads safely', () => {
  const browser = loadBrowserModule('dashboard/modules/api-client.js');
  const api = browser.ShumaDashboardApiClient;
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

test('dashboard API client parses JSON payloads when content-type is missing', async () => {
  const payload = {
    recent_events: [{ event: 'AdminAction', ts: 1700000000 }],
    event_counts: { AdminAction: 1 },
    top_ips: [['198.51.100.8', 1]],
    unique_ips: 1
  };
  let requestUrl = '';
  const browser = loadBrowserModule('dashboard/modules/api-client.js', {
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
  });
  const api = browser.ShumaDashboardApiClient.create({
    getAdminContext: () => ({ endpoint: 'http://example.test', apikey: '' })
  });

  const events = await api.getEvents(24);
  assert.equal(requestUrl, 'http://example.test/admin/events?hours=24');
  assert.equal(events.recent_events.length, 1);
  assert.equal(events.unique_ips, 1);
  assert.deepEqual(toPlain(events.top_ips), [['198.51.100.8', 1]]);
});

test('chart-lite renders doughnut legend labels', () => {
  const browser = loadBrowserModule('dashboard/assets/vendor/chart-lite-1.0.0.min.js', {
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
  const browser = loadBrowserModule('dashboard/assets/vendor/chart-lite-1.0.0.min.js', {
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
  const browser = loadBrowserModule('dashboard/assets/vendor/chart-lite-1.0.0.min.js', {
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

test('dashboard state invalidation scopes are explicit and bounded', () => {
  const browser = loadBrowserModule('dashboard/modules/dashboard-state.js');
  const stateApi = browser.ShumaDashboardState;
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

test('tab lifecycle normalizes unknown tabs to monitoring default', () => {
  const browser = loadBrowserModule('dashboard/modules/tab-lifecycle.js');
  const lifecycle = browser.ShumaDashboardTabLifecycle;
  assert.ok(lifecycle);

  assert.equal(lifecycle.normalizeTab('ip-bans'), 'ip-bans');
  assert.equal(lifecycle.normalizeTab('IP-BANS'), 'ip-bans');
  assert.equal(lifecycle.normalizeTab('unknown-tab'), 'monitoring');
  assert.equal(lifecycle.normalizeTab(''), 'monitoring');
});
