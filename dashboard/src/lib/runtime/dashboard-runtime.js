import {
  getDashboardSessionState as getExternalDashboardSessionState,
  logoutDashboardSession as logoutExternalDashboardSession,
  refreshDashboardTab as refreshExternalDashboardTab,
  restoreDashboardSession as restoreExternalDashboardSession,
  setDashboardActiveTab as setExternalDashboardActiveTab
} from './dashboard-external-adapters.js';
let mountingPromise = null;
let mounted = false;
let runtimeModule = null;

async function resolveRuntimeModule() {
  if (runtimeModule) return runtimeModule;
  runtimeModule = await import('../../../dashboard.js');
  return runtimeModule;
}

export async function mountDashboardRuntime(options = {}) {
  const mountOptions = { ...(options || {}) };
  delete mountOptions.mode;

  if (mounted) return;
  if (mountingPromise) return mountingPromise;

  mountingPromise = resolveRuntimeModule()
    .then((module) => {
      if (typeof module.mountDashboardExternalRuntime !== 'function') {
        throw new Error('Dashboard runtime entrypoint is missing mountDashboardExternalRuntime()');
      }
      module.mountDashboardExternalRuntime(mountOptions || {});
      mounted = true;
    })
    .finally(() => {
      mountingPromise = null;
    });

  return mountingPromise;
}

export function unmountDashboardRuntime() {
  if (!mounted) return;
  if (runtimeModule && typeof runtimeModule.unmountDashboardApp === 'function') {
    runtimeModule.unmountDashboardApp();
  }
  mounted = false;
}

export async function restoreDashboardSession() {
  return restoreExternalDashboardSession();
}

export function getDashboardSessionState() {
  return getExternalDashboardSessionState();
}

export async function refreshDashboardTab(tab, reason = 'manual', options = {}) {
  return refreshExternalDashboardTab(tab, reason, options || {});
}

export function setDashboardActiveTab(tab) {
  setExternalDashboardActiveTab(tab);
}

export async function logoutDashboardSession() {
  return logoutExternalDashboardSession();
}
