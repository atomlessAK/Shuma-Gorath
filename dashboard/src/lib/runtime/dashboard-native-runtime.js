// @ts-check

import * as dashboardApiClientModule from '../domain/api-client.js';
import * as adminEndpointModule from '../domain/services/admin-endpoint.js';
import {
  acquireChartRuntime,
  releaseChartRuntime
} from '../domain/services/chart-runtime-adapter.js';
import { createDashboardRefreshRuntime } from './dashboard-runtime-refresh.js';
import {
  normalizeDashboardBasePath,
  resolveDashboardBasePathFromLocation
} from './dashboard-paths.js';

const DASHBOARD_TABS = Object.freeze(['monitoring', 'ip-bans', 'status', 'config', 'tuning']);

const DASHBOARD_STATE_REQUIRED_METHODS = Object.freeze([
  'getState',
  'setActiveTab',
  'getActiveTab',
  'setSession',
  'getSession',
  'setSnapshot',
  'setSnapshots',
  'getSnapshot',
  'getSnapshotVersion',
  'getSnapshotVersions',
  'setTabLoading',
  'setTabError',
  'clearTabError',
  'setTabEmpty',
  'markTabUpdated',
  'invalidate',
  'isTabStale',
  'getDerivedState'
]);

const isObject = (value) => value && typeof value === 'object';

function hasDashboardStateContract(candidate) {
  if (!isObject(candidate)) return false;
  return DASHBOARD_STATE_REQUIRED_METHODS.every(
    (methodName) => typeof candidate[methodName] === 'function'
  );
}

function resolveDashboardStateStore(options = {}) {
  const providedStore = options.store;
  if (hasDashboardStateContract(providedStore)) {
    return providedStore;
  }
  throw new Error('mountDashboardApp requires an injected dashboard store contract.');
}

function normalizeTab(value) {
  const normalized = String(value || '').trim().toLowerCase();
  return DASHBOARD_TABS.includes(normalized) ? normalized : 'monitoring';
}

function parseBoolLike(value, fallback = false) {
  if (typeof value === 'boolean') return value;
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') {
    return true;
  }
  if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') {
    return false;
  }
  return fallback;
}

function deriveMonitoringAnalytics(configSnapshot = {}, analyticsSnapshot = {}) {
  const config = isObject(configSnapshot) ? configSnapshot : {};
  const analytics = isObject(analyticsSnapshot) ? analyticsSnapshot : {};
  return {
    ban_count: Number(analytics.ban_count || 0),
    test_mode: parseBoolLike(config.test_mode, analytics.test_mode === true),
    fail_mode: parseBoolLike(config.kv_store_fail_open, true) ? 'open' : 'closed'
  };
}

function normalizeRuntimeMountOptions(options = {}) {
  const source = options || {};
  const chartRuntimeSrc =
    typeof source.chartRuntimeSrc === 'string'
      ? source.chartRuntimeSrc.trim()
      : '';
  const locationLike = typeof window !== 'undefined' ? window.location : null;
  const basePath = normalizeDashboardBasePath(
    source.basePath || resolveDashboardBasePathFromLocation(locationLike)
  );
  return {
    chartRuntimeSrc,
    basePath,
    initialTab: normalizeTab(source.initialTab || 'monitoring')
  };
}

let runtimeMounted = false;
let runtimeMountOptions = normalizeRuntimeMountOptions({});
let resolveAdminApiEndpoint = () => ({ endpoint: '' });
let dashboardState = null;
let dashboardApiClient = null;
let dashboardRefreshRuntime = null;

const sessionState = {
  authenticated: false,
  csrfToken: ''
};

function setSessionState(authenticated, csrfToken = '') {
  sessionState.authenticated = authenticated === true;
  sessionState.csrfToken = sessionState.authenticated ? String(csrfToken || '') : '';
  if (dashboardState) {
    dashboardState.setSession({
      authenticated: sessionState.authenticated,
      csrfToken: sessionState.csrfToken
    });
  }
}

function resolveEndpoint() {
  const resolved = resolveAdminApiEndpoint();
  if (!resolved || typeof resolved.endpoint !== 'string') return '';
  return resolved.endpoint;
}

function getAdminContext() {
  if (!sessionState.authenticated) return null;
  const endpoint = resolveEndpoint();
  if (!endpoint) return null;
  return {
    endpoint,
    apikey: '',
    sessionAuth: true,
    csrfToken: sessionState.csrfToken
  };
}

function hasRuntimeReadyApiClient() {
  return Boolean(runtimeMounted && dashboardApiClient);
}

function requireApiClient() {
  if (!hasRuntimeReadyApiClient()) {
    throw new Error('Dashboard runtime API client is unavailable.');
  }
  return dashboardApiClient;
}

async function restoreSessionFromServer() {
  const endpoint = resolveEndpoint();
  if (!endpoint) {
    setSessionState(false, '');
    return false;
  }

  try {
    const response = await fetch(`${endpoint}/admin/session`, {
      method: 'GET',
      credentials: 'same-origin'
    });
    if (!response.ok) {
      setSessionState(false, '');
      return false;
    }
    const payload = await response.json().catch(() => ({}));
    const authenticated = payload && payload.authenticated === true;
    const csrfToken = authenticated ? String(payload.csrf_token || '') : '';
    setSessionState(authenticated, csrfToken);
    return authenticated;
  } catch (_error) {
    setSessionState(false, '');
    return false;
  }
}

function invalidateAfterConfigSave(nextConfig = null) {
  if (!dashboardState) return;
  if (isObject(nextConfig)) {
    dashboardState.setSnapshot('config', nextConfig);
  }
  dashboardState.invalidate('securityConfig');
  dashboardState.invalidate('monitoring');
  dashboardState.invalidate('ip-bans');
}

function invalidateAfterBanMutation() {
  if (!dashboardState) return;
  dashboardState.invalidate('ip-bans');
  dashboardState.invalidate('monitoring');
}

export async function mountDashboardApp(options = {}) {
  if (runtimeMounted) return;

  runtimeMountOptions = normalizeRuntimeMountOptions(options);
  dashboardState = resolveDashboardStateStore(options);

  resolveAdminApiEndpoint = adminEndpointModule.createAdminEndpointResolver({ window });

  await acquireChartRuntime({
    window,
    document,
    src: runtimeMountOptions.chartRuntimeSrc || undefined
  });

  dashboardApiClient = dashboardApiClientModule.create({
    getAdminContext,
    onUnauthorized: () => {
      setSessionState(false, '');
    }
  });

  dashboardRefreshRuntime = createDashboardRefreshRuntime({
    normalizeTab,
    getApiClient: () => dashboardApiClient,
    getStateStore: () => dashboardState,
    deriveMonitoringAnalytics
  });

  dashboardState.setActiveTab(runtimeMountOptions.initialTab);
  runtimeMounted = true;
}

export function getDashboardActiveTab() {
  if (!dashboardState) return 'monitoring';
  return normalizeTab(dashboardState.getActiveTab());
}

export function setDashboardActiveTab(tab, _reason = 'external') {
  const normalized = normalizeTab(tab);
  if (dashboardState) {
    dashboardState.setActiveTab(normalized);
  }
  return normalized;
}

export async function refreshDashboardTab(tab, reason = 'manual', options = {}) {
  if (!dashboardRefreshRuntime || typeof dashboardRefreshRuntime.refreshDashboardForTab !== 'function') return;
  return dashboardRefreshRuntime.refreshDashboardForTab(tab, reason, options || {});
}

export async function restoreDashboardSession() {
  return restoreSessionFromServer();
}

export function getDashboardSessionState() {
  return {
    authenticated: sessionState.authenticated === true,
    csrfToken: String(sessionState.csrfToken || '')
  };
}

export async function logoutDashboardSession() {
  const endpoint = resolveEndpoint();
  if (endpoint) {
    const headers = new Headers();
    if (sessionState.csrfToken) {
      headers.set('X-Shuma-CSRF', sessionState.csrfToken);
    }
    try {
      await fetch(`${endpoint}/admin/logout`, {
        method: 'POST',
        headers,
        credentials: 'same-origin'
      });
    } catch (_error) {}
  }
  setSessionState(false, '');
}

export async function updateDashboardConfig(patch) {
  const apiClient = requireApiClient();
  const response = await apiClient.updateConfig(patch || {});
  const nextConfig =
    response && typeof response === 'object' && response.config && typeof response.config === 'object'
      ? response.config
      : response;
  invalidateAfterConfigSave(nextConfig);
  return nextConfig;
}

export async function banDashboardIp(ip, duration, reason = 'manual_ban') {
  const apiClient = requireApiClient();
  const response = await apiClient.banIp(ip, duration, reason);
  invalidateAfterBanMutation();
  return response;
}

export async function unbanDashboardIp(ip) {
  const apiClient = requireApiClient();
  const response = await apiClient.unbanIp(ip);
  invalidateAfterBanMutation();
  return response;
}

export async function getDashboardRobotsPreview() {
  const apiClient = requireApiClient();
  return apiClient.getRobotsPreview();
}

export async function getDashboardEvents(hours = 24, options = {}) {
  const apiClient = requireApiClient();
  return apiClient.getEvents(hours, options || {});
}

export function unmountDashboardApp() {
  if (!runtimeMounted) return;
  runtimeMounted = false;
  runtimeMountOptions = normalizeRuntimeMountOptions({});
  dashboardApiClient = null;
  dashboardState = null;
  dashboardRefreshRuntime = null;
  resolveAdminApiEndpoint = () => ({ endpoint: '' });
  setSessionState(false, '');
  releaseChartRuntime({ window });
}
