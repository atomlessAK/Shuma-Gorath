let tabRuntime = null;
let sessionRuntime = null;
let normalizeTab = (value) => String(value || 'monitoring');
let defaultTab = 'monitoring';

function resolveTab(tab) {
  try {
    return normalizeTab(tab);
  } catch (_error) {
    return defaultTab;
  }
}

export function configureDashboardExternalAdapters(options = {}) {
  tabRuntime = options.tabRuntime || null;
  sessionRuntime = options.sessionRuntime || null;
  if (typeof options.normalizeTab === 'function') {
    normalizeTab = options.normalizeTab;
  }
  if (options.defaultTab) {
    defaultTab = String(options.defaultTab);
  }
}

export function clearDashboardExternalAdapters() {
  tabRuntime = null;
  sessionRuntime = null;
}

export function setDashboardActiveTab(tab, reason = 'external') {
  if (!tabRuntime || typeof tabRuntime.setActiveTab !== 'function') {
    return resolveTab(tab);
  }
  return tabRuntime.setActiveTab(tab, reason);
}

export function getDashboardActiveTab() {
  if (!tabRuntime || typeof tabRuntime.getActiveTab !== 'function') {
    return defaultTab;
  }
  return tabRuntime.getActiveTab();
}

export async function refreshDashboardTab(tab, reason = 'manual', options = {}) {
  if (!tabRuntime || typeof tabRuntime.refreshTab !== 'function') return;
  return tabRuntime.refreshTab(tab, reason, options || {});
}

export async function restoreDashboardSession() {
  if (!sessionRuntime || typeof sessionRuntime.restoreSession !== 'function') return false;
  return sessionRuntime.restoreSession();
}

export function getDashboardSessionState() {
  if (!sessionRuntime || typeof sessionRuntime.getSessionState !== 'function') {
    return { authenticated: false, csrfToken: '' };
  }
  return sessionRuntime.getSessionState();
}

export async function logoutDashboardSession() {
  if (!sessionRuntime || typeof sessionRuntime.logoutSession !== 'function') return;
  await sessionRuntime.logoutSession();
}
