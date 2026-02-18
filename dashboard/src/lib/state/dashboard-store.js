import { derived, get, writable } from 'svelte/store';
import {
  DASHBOARD_TABS,
  DEFAULT_TAB,
  createInitialState,
  reduceState,
  normalizeTab
} from '../../../modules/dashboard-state.js';

export { DASHBOARD_TABS, DEFAULT_TAB, normalizeTab };

export const TAB_REFRESH_INTERVAL_MS = Object.freeze({
  monitoring: 30000,
  'ip-bans': 45000,
  status: 60000,
  config: 60000,
  tuning: 60000
});
export const RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE = 20;

const cloneJson = (value) => JSON.parse(JSON.stringify(value));
const clampMetric = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric < 0) return 0;
  return Number(numeric.toFixed(2));
};
const normalizeWindowSize = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE;
  return Math.max(1, Math.floor(numeric));
};
const calculateP95 = (values = []) => {
  if (!Array.isArray(values) || values.length === 0) return 0;
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil(sorted.length * 0.95) - 1)
  );
  return clampMetric(sorted[index]);
};
const trimWindow = (values = [], limit = RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE) => {
  if (!Array.isArray(values)) return [];
  if (values.length <= limit) return values;
  return values.slice(values.length - limit);
};
const createMetricState = () => ({
  last: 0,
  avg: 0,
  p95: 0,
  max: 0,
  samples: 0,
  totalSamples: 0,
  windowSize: RUNTIME_TELEMETRY_ROLLING_WINDOW_SIZE,
  window: []
});

function createRuntimeTelemetryState() {
  return {
    refresh: {
      lastTab: DEFAULT_TAB,
      lastReason: 'init',
      updatedAt: '',
      fetchLatencyMs: createMetricState(),
      renderTimingMs: createMetricState()
    },
    polling: {
      skips: 0,
      resumes: 0,
      lastSkipReason: '',
      lastSkipAt: '',
      lastResumeReason: '',
      lastResumeAt: '',
      activeTab: DEFAULT_TAB,
      intervalMs: 0
    }
  };
}

const updateMetric = (metric = {}, rawValue) => {
  const value = clampMetric(rawValue);
  const windowSize = normalizeWindowSize(metric.windowSize);
  const currentWindow = Array.isArray(metric.window) ? metric.window : [];
  const nextWindow = trimWindow([...currentWindow, value], windowSize);
  const samples = nextWindow.length;
  const sum = nextWindow.reduce((accumulator, entry) => accumulator + Number(entry || 0), 0);
  const max = nextWindow.reduce((accumulator, entry) => Math.max(accumulator, Number(entry || 0)), 0);
  const totalSamples = Number(metric.totalSamples || 0) + 1;

  return {
    last: value,
    avg: samples > 0 ? clampMetric(sum / samples) : 0,
    p95: calculateP95(nextWindow),
    max: clampMetric(max),
    samples,
    totalSamples,
    windowSize,
    window: nextWindow
  };
};

function createDraftState() {
  return {
    baselines: {},
    drafts: {}
  };
}

function toNormalizedDraft(value) {
  if (value === undefined) return '';
  return JSON.stringify(value);
}

export function createDashboardStore(options = {}) {
  const initialTab = normalizeTab(options.initialTab || DEFAULT_TAB);
  const internal = writable(createInitialState(initialTab));
  const runtimeTelemetryStore = writable(createRuntimeTelemetryState());
  const draftState = createDraftState();

  const dispatch = (event = {}) => {
    let next = null;
    internal.update((state) => {
      next = reduceState(state, event);
      return next;
    });
    return next;
  };

  const getState = () => get(internal);

  const setActiveTab = (tab) => dispatch({ type: 'set-active-tab', tab: normalizeTab(tab) });
  const setSession = (session = {}) => dispatch({ type: 'set-session', session });
  const setSnapshot = (key, value) => dispatch({ type: 'set-snapshot', key, value });
  const setTabLoading = (tab, loading) => dispatch({ type: 'set-tab-loading', tab, loading });
  const setTabError = (tab, message) => dispatch({ type: 'set-tab-error', tab, message });
  const clearTabError = (tab) => dispatch({ type: 'clear-tab-error', tab });
  const setTabEmpty = (tab, empty) => dispatch({ type: 'set-tab-empty', tab, empty });
  const markTabUpdated = (tab) => dispatch({ type: 'mark-tab-updated', tab });
  const invalidate = (scope = 'all') => dispatch({ type: 'invalidate', scope });

  const reset = (tab = DEFAULT_TAB) => {
    internal.set(createInitialState(normalizeTab(tab)));
    resetRuntimeTelemetry();
    draftState.baselines = {};
    draftState.drafts = {};
  };

  const setDraftBaseline = (sectionKey, value) => {
    const key = String(sectionKey || '');
    if (!key) return;
    draftState.baselines[key] = cloneJson(value ?? null);
    if (!Object.prototype.hasOwnProperty.call(draftState.drafts, key)) {
      draftState.drafts[key] = cloneJson(value ?? null);
    }
  };

  const setDraft = (sectionKey, value) => {
    const key = String(sectionKey || '');
    if (!key) return;
    draftState.drafts[key] = cloneJson(value ?? null);
  };

  const getDraft = (sectionKey) => {
    const key = String(sectionKey || '');
    if (!key) return null;
    if (!Object.prototype.hasOwnProperty.call(draftState.drafts, key)) return null;
    return cloneJson(draftState.drafts[key]);
  };

  const isDraftDirty = (sectionKey, currentValue = undefined) => {
    const key = String(sectionKey || '');
    if (!key) return false;
    const baseline = Object.prototype.hasOwnProperty.call(draftState.baselines, key)
      ? draftState.baselines[key]
      : null;
    const candidate = currentValue === undefined
      ? (Object.prototype.hasOwnProperty.call(draftState.drafts, key) ? draftState.drafts[key] : null)
      : currentValue;
    return toNormalizedDraft(candidate) !== toNormalizedDraft(baseline);
  };

  const tabStatus = (tab) => derived(internal, ($state) => {
    const key = normalizeTab(tab);
    const value = $state.tabStatus[key] || {};
    return {
      loading: value.loading === true,
      error: String(value.error || ''),
      empty: value.empty === true,
      updatedAt: String(value.updatedAt || ''),
      stale: $state.stale[key] === true
    };
  });

  const session = derived(internal, ($state) => ({
    authenticated: $state.session.authenticated === true,
    csrfToken: String($state.session.csrfToken || '')
  }));

  const activeTab = derived(internal, ($state) => normalizeTab($state.activeTab));

  const selectRefreshInterval = (tab) => {
    const key = normalizeTab(tab);
    return TAB_REFRESH_INTERVAL_MS[key] || TAB_REFRESH_INTERVAL_MS.monitoring;
  };

  const getRuntimeTelemetry = () => get(runtimeTelemetryStore);
  const resetRuntimeTelemetry = () => {
    runtimeTelemetryStore.set(createRuntimeTelemetryState());
  };

  const setPollingContext = (tab, intervalMs) => {
    runtimeTelemetryStore.update((telemetry) => ({
      ...telemetry,
      polling: {
        ...telemetry.polling,
        activeTab: normalizeTab(tab),
        intervalMs: Number.isFinite(Number(intervalMs)) ? Number(intervalMs) : telemetry.polling.intervalMs
      }
    }));
  };

  const recordRefreshMetrics = (metrics = {}) => {
    runtimeTelemetryStore.update((telemetry) => {
      const fetchLatencyMs = updateMetric(
        telemetry.refresh.fetchLatencyMs,
        metrics.fetchLatencyMs
      );
      const renderTimingMs = updateMetric(
        telemetry.refresh.renderTimingMs,
        metrics.renderTimingMs
      );
      return {
        ...telemetry,
        refresh: {
          ...telemetry.refresh,
          lastTab: normalizeTab(metrics.tab),
          lastReason: String(metrics.reason || 'manual'),
          updatedAt: new Date().toISOString(),
          fetchLatencyMs,
          renderTimingMs
        }
      };
    });
  };

  const recordPollingSkip = (reason = 'unspecified', tab = DEFAULT_TAB, intervalMs = 0) => {
    runtimeTelemetryStore.update((telemetry) => ({
      ...telemetry,
      polling: {
        ...telemetry.polling,
        skips: Number(telemetry.polling.skips || 0) + 1,
        lastSkipReason: String(reason || 'unspecified'),
        lastSkipAt: new Date().toISOString(),
        activeTab: normalizeTab(tab),
        intervalMs: Number.isFinite(Number(intervalMs)) ? Number(intervalMs) : telemetry.polling.intervalMs
      }
    }));
  };

  const recordPollingResume = (reason = 'resume', tab = DEFAULT_TAB, intervalMs = 0) => {
    runtimeTelemetryStore.update((telemetry) => ({
      ...telemetry,
      polling: {
        ...telemetry.polling,
        resumes: Number(telemetry.polling.resumes || 0) + 1,
        lastResumeReason: String(reason || 'resume'),
        lastResumeAt: new Date().toISOString(),
        activeTab: normalizeTab(tab),
        intervalMs: Number.isFinite(Number(intervalMs)) ? Number(intervalMs) : telemetry.polling.intervalMs
      }
    }));
  };

  return {
    subscribe: internal.subscribe,
    getState,
    dispatch,
    reset,
    setActiveTab,
    setSession,
    setSnapshot,
    setTabLoading,
    setTabError,
    clearTabError,
    setTabEmpty,
    markTabUpdated,
    invalidate,
    setDraftBaseline,
    setDraft,
    getDraft,
    isDraftDirty,
    tabStatus,
    session,
    activeTab,
    selectRefreshInterval,
    runtimeTelemetryStore: {
      subscribe: runtimeTelemetryStore.subscribe
    },
    getRuntimeTelemetry,
    resetRuntimeTelemetry,
    setPollingContext,
    recordRefreshMetrics,
    recordPollingSkip,
    recordPollingResume
  };
}
