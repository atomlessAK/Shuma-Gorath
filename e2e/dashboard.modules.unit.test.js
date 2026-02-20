const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { pathToFileURL } = require('node:url');

const CHART_LITE_PATH = 'dashboard/static/assets/vendor/chart-lite-1.0.0.min.js';
const DASHBOARD_ROOT = path.resolve(__dirname, '..', 'dashboard');
const DASHBOARD_NATIVE_RUNTIME_PATH = path.join(
  DASHBOARD_ROOT,
  'src',
  'lib',
  'runtime',
  'dashboard-native-runtime.js'
);
const DASHBOARD_REFRESH_RUNTIME_PATH = path.join(
  DASHBOARD_ROOT,
  'src',
  'lib',
  'runtime',
  'dashboard-runtime-refresh.js'
);

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
    createElement: () => ({
      innerHTML: '',
      classList: { add() {}, remove() {}, toggle() {}, contains() { return false; } }
    })
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
  if (windowValue.fetch) restoreFns.push(setGlobalValue('fetch', windowValue.fetch));
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
  if (sandbox.document && !sandbox.window.document) sandbox.window.document = sandbox.document;
  if (sandbox.location && !sandbox.window.location) sandbox.window.location = sandbox.location;
  if (sandbox.navigator && !sandbox.window.navigator) sandbox.window.navigator = sandbox.navigator;
  sandbox.globalThis = sandbox.window;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: absolutePath });
  return sandbox.window;
}

test('dashboard API adapters normalize sparse payloads safely', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const api = await importBrowserModule('dashboard/src/lib/domain/api-client.js');

    const analytics = api.adaptAnalytics({ ban_count: '7', test_mode: true });
    assert.equal(analytics.ban_count, 7);
    assert.equal(analytics.test_mode, true);
    assert.equal(analytics.fail_mode, 'open');

    const events = api.adaptEvents({
      recent_events: [{ ip: '198.51.100.1' }, null, 'ignored'],
      top_ips: [['198.51.100.1', '9'], ['198.51.100.2', 4], ['bad']],
      unique_ips: '11'
    });
    assert.equal(events.recent_events.length, 1);
    assert.deepEqual(toPlain(events.top_ips), [
      ['198.51.100.1', 9],
      ['198.51.100.2', 4]
    ]);
    assert.equal(events.unique_ips, 11);
  });
});

test('dashboard API client parses JSON payloads when content-type is missing', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const requests = [];
    const client = apiModule.create({
      getAdminContext: () => ({ endpoint: 'https://edge.local', apikey: '', sessionAuth: false }),
      request: async (url, init = {}) => {
        requests.push({ url, init });
        return {
          ok: true,
          status: 200,
          headers: new Headers(),
          text: async () => '{"recent_events":[{"ip":"203.0.113.7"}],"top_ips":[["203.0.113.7",3]],"unique_ips":1}',
          json: async () => ({
            recent_events: [{ ip: '203.0.113.7' }],
            top_ips: [['203.0.113.7', 3]],
            unique_ips: 1
          })
        };
      }
    });

    const events = await client.getEvents(24);
    assert.equal(events.recent_events.length, 1);
    assert.deepEqual(toPlain(events.top_ips), [['203.0.113.7', 3]]);
    assert.equal(events.unique_ips, 1);
    assert.equal(requests.length, 1);
  });
});

test('dashboard API client adds CSRF + same-origin for session-auth writes and strips empty bearer', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const calls = [];
    const client = apiModule.create({
      getAdminContext: () => ({
        endpoint: 'https://edge.local',
        apikey: '   ',
        sessionAuth: true,
        csrfToken: 'csrf-token'
      }),
      request: async (url, init = {}) => {
        calls.push({ url, init });
        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          json: async () => ({ config: { maze_enabled: true } }),
          text: async () => JSON.stringify({ config: { maze_enabled: true } })
        };
      }
    });

    await client.updateConfig({ maze_enabled: true });
    assert.equal(calls.length, 1);

    const headers = calls[0].init.headers;
    assert.equal(headers.get('authorization'), null);
    assert.equal(headers.get('x-shuma-csrf'), 'csrf-token');
    assert.equal(calls[0].init.credentials, 'same-origin');
  });
});

test('dashboard API client times out stalled requests with DashboardApiError', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const apiModule = await importBrowserModule('dashboard/src/lib/domain/api-client.js');
    const errors = [];
    const client = apiModule.create({
      getAdminContext: () => ({ endpoint: 'https://edge.local', apikey: '', sessionAuth: false }),
      onApiError: (error) => {
        errors.push(error);
      },
      request: async (_url, init = {}) =>
        new Promise((_resolve, reject) => {
          if (init.signal && typeof init.signal.addEventListener === 'function') {
            init.signal.addEventListener('abort', () => {
              const abortError = new Error('Request aborted');
              abortError.name = 'AbortError';
              reject(abortError);
            }, { once: true });
          }
        })
    });

    await assert.rejects(
      () => client.getEvents(24, { timeoutMs: 25 }),
      (error) => {
        assert.equal(error.name, 'DashboardApiError');
        assert.match(String(error.message || ''), /timed out/i);
        return true;
      }
    );
    assert.equal(errors.length, 1);
    assert.equal(errors[0].name, 'DashboardApiError');
  });
});

test('chart-lite renders doughnut legend labels and dark center fill', () => {
  const script = loadClassicBrowserScript(CHART_LITE_PATH, {});
  const Chart = script.Chart;
  const { ctx, calls } = createMockCanvasContext();

  new Chart(ctx, {
    type: 'doughnut',
    data: {
      labels: ['Allow', 'Challenge', 'Block'],
      datasets: [{ data: [10, 4, 1] }]
    },
    options: { theme: 'dark' }
  });

  assert.equal(calls.fillText.includes('Allow'), true);
  assert.equal(calls.fillStyle.length > 0, true);
  assert.equal(calls.fillStyle.every((entry) => String(entry).toLowerCase() === '#ffffff'), false);
});

test('chart runtime adapter lazily loads once and tears down on final release', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const adapter = await importBrowserModule('dashboard/src/lib/domain/services/chart-runtime-adapter.js');
    const win = { location: { pathname: '/dashboard/index.html' }, Chart: undefined };

    const appendedScripts = [];
    const scriptNode = {
      dataset: {},
      attributes: {},
      parentNode: null,
      setAttribute(name, value) {
        this.attributes[name] = value;
      },
      getAttribute(name) {
        return this.attributes[name] || '';
      },
      addEventListener(event, handler) {
        if (event === 'load') {
          this._onload = handler;
        }
      },
      removeEventListener() {}
    };

    const doc = {
      head: {
        appendChild(node) {
          appendedScripts.push(node);
          node.parentNode = this;
          win.Chart = function ChartMock() {};
          if (typeof node._onload === 'function') node._onload();
        },
        removeChild(node) {
          node.parentNode = null;
        }
      },
      body: null,
      createElement() {
        return { ...scriptNode, dataset: {}, attributes: {} };
      },
      querySelectorAll() {
        return [];
      }
    };

    const one = await adapter.acquireChartRuntime({ window: win, document: doc, src: '/dashboard/assets/chart-lite.js' });
    const two = await adapter.acquireChartRuntime({ window: win, document: doc, src: '/dashboard/assets/chart-lite.js' });

    assert.equal(typeof one, 'function');
    assert.equal(one, two);
    assert.equal(appendedScripts.length, 1);

    adapter.releaseChartRuntime({ window: win, document: doc });
    assert.equal(typeof win.Chart, 'function');
    adapter.releaseChartRuntime({ window: win, document: doc });
    assert.equal(win.Chart, undefined);
  });
});

test('dashboard state and store contracts remain immutable and bounded', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const stateModule = await importBrowserModule('dashboard/src/lib/domain/dashboard-state.js');
    const storeModule = await importBrowserModule('dashboard/src/lib/state/dashboard-store.js');

    const initial = stateModule.createInitialState('monitoring');
    const next = stateModule.reduceState(initial, { type: 'set-active-tab', tab: 'config' });
    assert.notEqual(initial, next);
    assert.equal(initial.activeTab, 'monitoring');
    assert.equal(next.activeTab, 'config');

    const store = storeModule.createDashboardStore({ initialTab: 'monitoring' });
    store.recordRefreshMetrics({ tab: 'monitoring', reason: 'manual', fetchLatencyMs: 100, renderTimingMs: 10 });
    store.recordRefreshMetrics({ tab: 'monitoring', reason: 'manual', fetchLatencyMs: 200, renderTimingMs: 20 });
    store.recordRefreshMetrics({ tab: 'status', reason: 'manual', fetchLatencyMs: 999, renderTimingMs: 999 });

    const telemetry = store.getRuntimeTelemetry();
    assert.equal(telemetry.refresh.fetchLatencyMs.last, 200);
    assert.equal(telemetry.refresh.renderTimingMs.last, 20);
    assert.equal(telemetry.refresh.lastTab, 'monitoring');
    assert.equal(telemetry.refresh.fetchLatencyMs.totalSamples, 2);
    assert.equal(telemetry.refresh.fetchLatencyMs.window.length > 0, true);
  });
});

test('monitoring view model and status module remain pure snapshot transforms', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const monitoringModelModule = await importBrowserModule('dashboard/src/lib/components/dashboard/monitoring-view-model.js');
    const ipRangePolicyModule = await importBrowserModule('dashboard/src/lib/domain/ip-range-policy.js');
    const statusModule = await importBrowserModule('dashboard/src/lib/domain/status.js');

    const summary = monitoringModelModule.deriveMonitoringSummaryViewModel({
      honeypot: {
        total_hits: 120,
        unique_crawlers: 4,
        top_crawlers: [{ label: 'bot-a', count: 40 }, { label: 'bot-b', count: 30 }]
      },
      challenge: {
        total_failures: 10,
        unique_offenders: 5,
        top_offenders: [{ label: 'hash-a', count: 4 }],
        reasons: { timeout: 3 },
        trend: []
      },
      not_a_bot: {
        served: 20,
        submitted: 18,
        pass: 12,
        escalate: 4,
        fail: 2,
        replay: 1,
        abandonments_estimated: 2,
        abandonment_ratio: 0.1,
        outcomes: { pass: 12, escalate: 4, fail: 2, replay: 1 },
        solve_latency_buckets: { lt_1s: 3, '1_3s': 8, '3_10s': 5, '10s_plus': 2 }
      },
      pow: {
        total_failures: 5,
        total_successes: 5,
        total_attempts: 10,
        success_ratio: 0.5,
        unique_offenders: 2,
        top_offenders: [{ label: 'hash-pow', count: 3 }],
        reasons: { invalid_proof: 5 },
        outcomes: { success: 5, failure: 5 },
        trend: []
      },
      rate: {
        total_violations: 12,
        unique_offenders: 2,
        top_offenders: [{ label: 'ip-a', count: 8 }],
        outcomes: { block: 7 }
      },
      geo: {
        total_violations: 5,
        actions: { block: 3, challenge: 2, maze: 0 },
        top_countries: [['US', 3]]
      }
    });
    assert.equal(summary.honeypot.totalHits, '120');
    assert.equal(summary.challenge.totalFailures, '10');
    assert.equal(summary.notABot.served, '20');
    assert.equal(summary.notABot.pass, '12');
    assert.equal(summary.notABot.abandonmentRate, '10.0%');
    assert.equal(summary.pow.totalFailures, '5');
    assert.equal(summary.pow.totalSuccesses, '5');
    assert.equal(summary.pow.totalAttempts, '10');
    assert.equal(summary.pow.successRate, '50.0%');
    assert.equal(
      summary.pow.outcomes.some((row) => row[0] === 'success' && Number(row[1]) === 5),
      true
    );
    const helper = monitoringModelModule.derivePrometheusHelperViewModel({
      docs: {
        observability: 'javascript:alert(1)',
        api: 'https://example.com/api'
      }
    });
    assert.equal(helper.observabilityLink, '');
    assert.equal(helper.apiLink, 'https://example.com/api');

    const parsedOutcome = ipRangePolicyModule.parseIpRangeOutcome(
      'source=managed source_id=openai-gptbot action=forbidden_403 matched_cidr=203.0.113.0/24 taxonomy[level=L11 action=A_DENY_HARD detection=D_IP_RANGE_FORBIDDEN signals=S_IP_RANGE_MANAGED]'
    );
    assert.equal(parsedOutcome.source, 'managed');
    assert.equal(parsedOutcome.sourceId, 'openai-gptbot');
    assert.equal(parsedOutcome.action, 'forbidden_403');
    assert.equal(parsedOutcome.detection, 'D_IP_RANGE_FORBIDDEN');
    assert.deepEqual(toPlain(parsedOutcome.signals), ['S_IP_RANGE_MANAGED']);

    const ipRangeSummary = monitoringModelModule.deriveIpRangeMonitoringViewModel([
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_forbidden',
        outcome: 'source=managed source_id=openai-gptbot action=forbidden_403 matched_cidr=203.0.113.0/24 taxonomy[level=L11 action=A_DENY_HARD detection=D_IP_RANGE_FORBIDDEN signals=S_IP_RANGE_MANAGED]'
      },
      {
        ts: Math.floor(Date.now() / 1000),
        reason: 'ip_range_policy_maze_fallback_block',
        outcome: 'source=custom source_id=manual-bad-range action=maze matched_cidr=198.51.100.0/24 taxonomy[level=L10 action=A_DENY_TEMP detection=D_IP_RANGE_MAZE signals=S_IP_RANGE_CUSTOM]'
      }
    ], {
      ip_range_policy_mode: 'enforce',
      ip_range_emergency_allowlist: ['198.51.100.7/32'],
      ip_range_custom_rules: [{ id: 'manual-bad-range', enabled: true }],
      ip_range_managed_policies: [{ set_id: 'openai-gptbot', enabled: true }],
      ip_range_managed_max_staleness_hours: 24,
      ip_range_allow_stale_managed_enforce: false,
      ip_range_managed_catalog_version: '2026-02-20',
      ip_range_managed_catalog_generated_at: '2026-02-20T00:00:00Z',
      ip_range_managed_catalog_generated_at_unix: Math.floor(Date.now() / 1000) - 3600,
      ip_range_managed_sets: [{ set_id: 'openai-gptbot', provider: 'openai', stale: false, entry_count: 42 }]
    });
    assert.equal(ipRangeSummary.totalMatches, 2);
    assert.equal(ipRangeSummary.mode, 'enforce');
    assert.equal(
      ipRangeSummary.actions.some(([label, count]) => label === 'forbidden_403' && Number(count) === 1),
      true
    );
    assert.equal(ipRangeSummary.catalog.managedSetCount, 1);

    const configSnapshot = {
      kv_store_fail_open: true,
      test_mode: false,
      pow_enabled: true,
      not_a_bot_enabled: true,
      not_a_bot_risk_threshold: 2,
      challenge_puzzle_enabled: true,
      challenge_puzzle_transform_count: 6,
      challenge_puzzle_risk_threshold: 3,
      ip_range_policy_mode: 'advisory',
      ip_range_emergency_allowlist: ['198.51.100.0/24'],
      ip_range_custom_rules: [{ id: 'custom-1', enabled: true }],
      ip_range_managed_policies: [{ set_id: 'openai-gptbot', enabled: true }],
      ip_range_managed_max_staleness_hours: 24,
      ip_range_allow_stale_managed_enforce: false,
      ip_range_managed_catalog_version: '2026-02-20',
      ip_range_managed_catalog_generated_at_unix: Math.floor(Date.now() / 1000) - 7200,
      ip_range_managed_sets: [{ set_id: 'openai-gptbot', stale: false }],
      botness_maze_threshold: 6,
      botness_weights: {
        js_required: 1,
        geo_risk: 2,
        rate_medium: 1,
        rate_high: 2
      }
    };
    const before = JSON.stringify(configSnapshot);
    const derived = statusModule.deriveStatusSnapshot(configSnapshot);
    assert.equal(String(derived.failMode).toLowerCase(), 'open');
    assert.equal(derived.powEnabled, true);
    assert.equal(derived.notABotEnabled, true);
    assert.equal(JSON.stringify(configSnapshot), before);

    const statusItems = statusModule.buildFeatureStatusItems(derived);
    const challengePuzzleItem = statusItems.find((item) => item.title === 'Challenge Puzzle');
    const challengeNotABotItem = statusItems.find((item) => item.title === 'Challenge Not-A-Bot');
    const ipRangeItem = statusItems.find((item) => item.title === 'IP Range Policy');
    assert.equal(Boolean(challengePuzzleItem), true);
    assert.equal(Boolean(challengeNotABotItem), true);
    assert.equal(Boolean(ipRangeItem), true);
    assert.equal(challengePuzzleItem?.status, 'ENABLED');
    assert.equal(challengeNotABotItem?.status, 'ENABLED');
    assert.equal(ipRangeItem?.status, 'ADVISORY');
    assert.equal(statusItems.some((item) => item.title === 'Challenge'), false);
  });
});

test('config form utils and JSON object helpers preserve parser contracts', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const formUtils = await importBrowserModule('dashboard/src/lib/domain/config-form-utils.js');
    const json = await importBrowserModule('dashboard/src/lib/domain/core/json-object.js');
    const schema = await importBrowserModule('dashboard/src/lib/domain/config-schema.js');

    assert.deepEqual(formUtils.parseCountryCodesStrict('GB,US'), ['GB', 'US']);
    assert.equal(formUtils.normalizeListTextareaForCompare('a\na\nb'), 'a\nb');
    assert.deepEqual(formUtils.parseBrowserRulesTextarea('Chrome,120\nFirefox,115'), [
      ['Chrome', 120],
      ['Firefox', 115]
    ]);

    const template = json.buildTemplateFromPaths(
      { pow_enabled: true, botness_weights: { geo_risk: 3 }, extra: 1 },
      ['pow_enabled', 'botness_weights.geo_risk']
    );
    assert.deepEqual(toPlain(template), { pow_enabled: true, botness_weights: { geo_risk: 3 } });
    assert.equal(json.normalizeJsonObjectForCompare('{"ok":true}'), '{"ok":true}');
    assert.equal(Array.isArray(schema.advancedConfigTemplatePaths), true);
  });
});

test('admin endpoint resolver applies loopback override only for local hostnames', { concurrency: false }, async () => {
  await withBrowserGlobals({}, async () => {
    const endpointModule = await importBrowserModule('dashboard/src/lib/domain/services/admin-endpoint.js');

    const localResolver = endpointModule.createAdminEndpointResolver({
      window: {
        location: {
          origin: 'http://127.0.0.1:3000',
          hostname: '127.0.0.1',
          search: '?api_endpoint=http://localhost:7777',
          protocol: 'http:',
          host: '127.0.0.1:3000'
        }
      }
    });
    assert.equal(localResolver().endpoint, 'http://localhost:7777');

    const remoteResolver = endpointModule.createAdminEndpointResolver({
      window: {
        location: {
          origin: 'https://example.com',
          hostname: 'example.com',
          search: '?api_endpoint=http://localhost:7777',
          protocol: 'https:',
          host: 'example.com'
        }
      }
    });
    assert.equal(remoteResolver().endpoint, 'https://example.com');
  });
});

test('ip bans, config, and tuning tabs are declarative and callback-driven', () => {
  const ipBansSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/IpBansTab.svelte'),
    'utf8'
  );
  const configSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/ConfigTab.svelte'),
    'utf8'
  );
  const tuningSource = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/TuningTab.svelte'),
    'utf8'
  );

  assert.match(ipBansSource, /export let onBan = null;/);
  assert.match(ipBansSource, /export let onUnban = null;/);
  assert.match(ipBansSource, /export let configSnapshot = null;/);
  assert.match(ipBansSource, /let banFilter = 'all';/);
  assert.match(ipBansSource, /id="ip-ban-filter"/);
  assert.match(ipBansSource, /isIpRangeBanLike/);
  assert.match(ipBansSource, /config\.ban_durations\.admin/);
  assert.match(ipBansSource, /applyConfiguredBanDuration\(configSnapshot\)/);
  assert.match(ipBansSource, /disabled=\{!canBan\}/);
  assert.match(ipBansSource, /disabled=\{!canUnban\}/);
  assert.equal(ipBansSource.includes('querySelectorAll('), false);

  assert.match(configSource, /export let onSaveConfig = null;/);
  assert.match(configSource, /export let onFetchRobotsPreview = null;/);
  assert.match(configSource, /let testMode = false;/);
  assert.match(configSource, /let ipRangePolicyMode = 'off';/);
  assert.match(configSource, /let restrictSearchEngines = false;/);
  assert.equal(configSource.includes('robotsAllowSearch'), false);
  assert.match(configSource, /onTestModeToggleChange/);
  assert.match(configSource, /await onSaveConfig\(/);
  assert.match(configSource, /\$: testModeToggleText = testMode \? 'Test Mode On' : 'Test Mode Off';/);
  assert.match(configSource, /id="preview-challenge-puzzle-link"/);
  assert.match(configSource, /id="preview-not-a-bot-link"/);
  assert.match(configSource, /id="save-config-all"/);
  assert.match(configSource, /saveAllConfig\(/);
  assert.match(configSource, /window\.addEventListener\('beforeunload'/);
  assert.match(configSource, /id="ip-range-policy-mode"/);
  assert.match(configSource, /ip_range_policy_mode/);
  assert.match(configSource, /\(LOGGING ONLY\)/);
  assert.match(configSource, /\(BLOCKING ACTIVE\)/);
  assert.equal(configSource.includes('Test Mode Active'), false);
  assert.equal(configSource.includes('ENABLED (LOGGING ONLY)'), false);
  assert.equal(configSource.includes('DISABLED (BLOCKING ACTIVE)'), false);
  assert.equal(configSource.includes('id="save-js-required-config"'), false);
  assert.equal(configSource.includes('id="save-test-mode-config"'), false);
  assert.equal(configSource.includes('id="save-advanced-config"'), false);
  assert.equal(configSource.includes('{@html'), false);

  assert.match(tuningSource, /export let onSaveConfig = null;/);
  assert.match(tuningSource, /await onSaveConfig\(payload/);
  assert.match(tuningSource, /id="save-tuning-all"/);
  assert.match(tuningSource, /window\.addEventListener\('beforeunload'/);
  assert.equal(tuningSource.includes('id="save-botness-config"'), false);
});

test('dashboard route lazily loads heavy tabs and keeps orchestration local', () => {
  const source = fs.readFileSync(path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'), 'utf8');

  assert.match(source, /import\('\$lib\/components\/dashboard\/ConfigTab\.svelte'\)/);
  assert.match(source, /import\('\$lib\/components\/dashboard\/TuningTab\.svelte'\)/);
  assert.match(source, /\$lib\/runtime\/dashboard-route-controller\.js/);
  assert.match(source, /<svelte:window on:hashchange=\{onWindowHashChange\} \/>/);
  assert.match(source, /<svelte:document on:visibilitychange=\{onDocumentVisibilityChange\} \/>/);
  assert.match(source, /use:registerTabLink=\{tab\}/);
  assert.match(source, /buildDashboardLoginPath/);
  assert.match(source, /const AUTO_REFRESH_INTERVAL_MS = 60000;/);
  assert.match(source, /isAutoRefreshEnabled: \(\) => autoRefreshEnabled === true/);
  assert.match(source, /shouldRefreshOnActivate: \(\{ tab, store \}\) =>/);
  assert.equal(source.includes('requestNextFrame,'), false);
  assert.equal(source.includes('nowMs,'), false);
  assert.equal(source.includes('readHashTab,'), false);
  assert.equal(source.includes('writeHashTab,'), false);
  assert.equal(source.includes('isPageVisible,'), false);
  assert.equal(source.includes('createDashboardActions'), false);
  assert.equal(source.includes('createDashboardEffects'), false);
  assert.match(source, /onSaveConfig=\{onSaveConfig\}/);
  assert.match(source, /onBan=\{onBan\}/);
  assert.match(source, /onUnban=\{onUnban\}/);
  assert.match(source, /configSnapshot=\{snapshots\.config\}/);
  assert.match(source, /id="admin-msg"/);
});

test('monitoring tab applies bounded sanitization and redraw guards', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/MonitoringTab.svelte'),
    'utf8'
  );

  assert.match(source, /const MONITORING_LIST_LIMIT = 10;/);
  assert.match(source, /const MONITORING_TREND_POINT_LIMIT = 720;/);
  assert.match(source, /const RANGE_EVENTS_FETCH_LIMIT = 5000;/);
  assert.match(source, /const RANGE_EVENTS_REQUEST_TIMEOUT_MS = 10000;/);
  assert.match(source, /const RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS = 180000;/);
  assert.match(source, /export let autoRefreshEnabled = false;/);
  assert.match(source, /sameSeries\(chart, trendSeries\.labels, trendSeries\.data\)/);
  assert.match(source, /abortRangeEventsFetch\(\);/);
  assert.match(source, /clampCount\(/);
});

test('monitoring tab is decomposed into focused subsection components', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard/MonitoringTab.svelte'),
    'utf8'
  );

  assert.match(source, /import OverviewStats from '\.\/monitoring\/OverviewStats\.svelte';/);
  assert.match(source, /import PrimaryCharts from '\.\/monitoring\/PrimaryCharts\.svelte';/);
  assert.match(source, /import RecentEventsTable from '\.\/monitoring\/RecentEventsTable\.svelte';/);
  assert.match(source, /import ExternalMonitoringSection from '\.\/monitoring\/ExternalMonitoringSection\.svelte';/);
  assert.match(source, /import IpRangeSection from '\.\/monitoring\/IpRangeSection\.svelte';/);
  assert.match(source, /<OverviewStats/);
  assert.match(source, /<PrimaryCharts/);
  assert.match(source, /<ChallengeSection/);
  assert.match(source, /<PowSection/);
  assert.match(source, /<IpRangeSection/);
  assert.match(source, /<ExternalMonitoringSection/);
});

test('dashboard runtime is slim and free of legacy DOM-id wiring layers', () => {
  const source = fs.readFileSync(DASHBOARD_NATIVE_RUNTIME_PATH, 'utf8');

  assert.equal(source.includes('document.getElementById'), false);
  assert.equal(source.includes('config-controls'), false);
  assert.equal(source.includes('config-ui-state'), false);
  assert.equal(source.includes('input-validation'), false);
  assert.equal(source.includes('tab-lifecycle'), false);
  assert.equal(source.includes('createDashboardSessionRuntime'), false);
  assert.equal(source.includes('createDashboardTabRuntime'), false);
  assert.match(source, /createDashboardRefreshRuntime/);
  assert.equal(source.includes('createDashboardTabStateRuntime'), false);
  assert.match(source, /export async function updateDashboardConfig/);
  assert.match(source, /export async function banDashboardIp/);
  assert.match(source, /export async function unbanDashboardIp/);
  assert.match(source, /dashboardRefreshRuntime\.clearAllCaches/);
});

test('dashboard refresh runtime remains snapshot-only and excludes legacy config UI glue', () => {
  const source = fs.readFileSync(DASHBOARD_REFRESH_RUNTIME_PATH, 'utf8');

  assert.equal(source.includes('updateConfigModeUi'), false);
  assert.equal(source.includes('invokeConfigUiState'), false);
  assert.equal(source.includes('refreshAllDirtySections'), false);
  assert.equal(source.includes('refreshDirtySections'), false);
  assert.equal(source.includes('getMessageNode'), false);
  assert.match(source, /const MONITORING_CACHE_KEY = 'shuma_dashboard_cache_monitoring_v1';/);
  assert.match(source, /const IP_BANS_CACHE_KEY = 'shuma_dashboard_cache_ip_bans_v1';/);
  assert.equal(source.includes('shuma_dashboard_cache_config_v1'), false);
  assert.match(source, /const MONITORING_CACHE_MAX_RECENT_EVENTS = 25;/);
  assert.match(source, /const MONITORING_CACHE_MAX_CDP_EVENTS = 50;/);
  assert.match(source, /const MONITORING_CACHE_MAX_BANS = 100;/);
  assert.match(source, /function clearAllCaches\(\) \{/);
  assert.match(source, /writeCache\(MONITORING_CACHE_KEY, \{ monitoring: compactMonitoring \}\);/);
  assert.match(source, /if \(hasConfigSnapshot\(existingConfig\)\) \{/);
  assert.equal(source.includes("? { monitoring: monitoringData }"), false);
  assert.match(source, /const refreshConfigTab = \(reason = 'manual'/);
  assert.match(source, /const includeConfigRefresh = reason !== 'auto-refresh';/);
  assert.match(
    source,
    /includeConfigRefresh \? refreshSharedConfig\(reason, runtimeOptions\) : Promise\.resolve\(null\)/
  );
});

test('dashboard route imports native runtime actions directly', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    'utf8'
  );

  assert.equal(source.includes("$lib/runtime/dashboard-runtime.js"), false);
  assert.match(source, /\$lib\/runtime\/dashboard-native-runtime\.js/);
  assert.match(source, /\$lib\/runtime\/dashboard-route-controller\.js/);
  assert.match(source, /updateDashboardConfig/);
  assert.match(source, /banDashboardIp/);
  assert.match(source, /unbanDashboardIp/);
  assert.match(source, /getDashboardRobotsPreview/);
});

test('dashboard route controller gates polling to auto-enabled eligible tabs', () => {
  const source = fs.readFileSync(
    path.join(DASHBOARD_ROOT, 'src/lib/runtime/dashboard-route-controller.js'),
    'utf8'
  );

  assert.match(source, /const isAutoRefreshEnabled =/);
  assert.match(source, /const isAutoRefreshTab =/);
  assert.match(source, /recordPollingSkip\('auto-refresh-disabled'/);
  assert.match(source, /recordPollingSkip\('tab-not-auto-refreshable'/);
  assert.match(source, /const shouldRefreshOnActivate =/);
});

test('dashboard module graph is layered with no cycles', () => {
  const moduleRoot = path.join(DASHBOARD_ROOT, 'src/lib/domain');
  const moduleFiles = listJsFilesRecursively(moduleRoot);
  const runtimeFiles = listJsFilesRecursively(path.join(DASHBOARD_ROOT, 'src/lib/runtime'));
  const allFiles = [DASHBOARD_NATIVE_RUNTIME_PATH, ...runtimeFiles, ...moduleFiles];

  const relativeOf = (absolutePath) =>
    path.relative(DASHBOARD_ROOT, absolutePath).split(path.sep).join('/');
  const rankOf = (relativePath) => {
    if (relativePath === 'src/lib/runtime/dashboard-native-runtime.js') return 4;
    if (relativePath.startsWith('src/lib/runtime/')) return 3;
    if (relativePath.startsWith('src/lib/domain/core/')) return 0;
    if (relativePath.startsWith('src/lib/domain/services/')) return 1;
    if (
      relativePath === 'src/lib/domain/api-client.js' ||
      relativePath === 'src/lib/domain/dashboard-state.js' ||
      relativePath === 'src/lib/domain/config-schema.js' ||
      relativePath === 'src/lib/domain/config-form-utils.js'
    ) {
      return 1;
    }
    if (relativePath.startsWith('src/lib/domain/')) return 2;
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

test('dashboard modules are reachable from route/runtime entry graph (no dead wrappers)', () => {
  const moduleRoot = path.join(DASHBOARD_ROOT, 'src/lib/domain');
  const moduleFiles = listJsFilesRecursively(moduleRoot);

  const routeAndRuntimeEntries = [
    path.join(DASHBOARD_ROOT, 'src/routes/+page.svelte'),
    path.join(DASHBOARD_ROOT, 'src/routes/login.html/+page.svelte'),
    ...fs
      .readdirSync(path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard'))
      .filter((name) => name.endsWith('.svelte'))
      .map((name) => path.join(DASHBOARD_ROOT, 'src/lib/components/dashboard', name)),
    ...listJsFilesRecursively(path.join(DASHBOARD_ROOT, 'src/lib/runtime')),
    ...listJsFilesRecursively(path.join(DASHBOARD_ROOT, 'src/lib/state'))
  ];

  const queue = routeAndRuntimeEntries.filter((absolutePath) => fs.existsSync(absolutePath));
  const visited = new Set();

  const resolveRelativeImport = (fromPath, specifier) => {
    const base = path.resolve(path.dirname(fromPath), specifier);
    const candidates = [base, `${base}.js`, `${base}.svelte`, path.join(base, 'index.js')];
    for (const candidate of candidates) {
      if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) return candidate;
    }
    return null;
  };

  while (queue.length > 0) {
    const current = queue.pop();
    if (visited.has(current)) continue;
    visited.add(current);
    const source = fs.readFileSync(current, 'utf8');
    const imports = parseRelativeImports(source);
    imports.forEach((specifier) => {
      const resolved = resolveRelativeImport(current, specifier);
      if (resolved && !visited.has(resolved)) {
        queue.push(resolved);
      }
    });
  }

  const unreachableModules = moduleFiles
    .filter((absolutePath) => !visited.has(absolutePath))
    .map((absolutePath) => path.relative(DASHBOARD_ROOT, absolutePath).split(path.sep).join('/'));

  assert.deepEqual(
    unreachableModules,
    [],
    `unreachable dashboard modules found:\n${unreachableModules.join('\n')}`
  );
});
