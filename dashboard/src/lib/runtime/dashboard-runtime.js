let mountingPromise = null;
let mounted = false;
let runtimeModule = null;

async function resolveRuntimeModule() {
  if (runtimeModule) return runtimeModule;
  runtimeModule = await import('./dashboard-native-runtime.js');
  return runtimeModule;
}

export async function mountDashboardRuntime(options = {}) {
  if (mounted) return;
  if (mountingPromise) return mountingPromise;

  mountingPromise = resolveRuntimeModule()
    .then(async (module) => {
      if (typeof module.mountDashboardApp !== 'function') {
        throw new Error('Dashboard runtime entrypoint is missing mountDashboardApp()');
      }
      await module.mountDashboardApp({ ...(options || {}) });
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

async function ensureMountedRuntime() {
  if (mountingPromise) {
    await mountingPromise;
  }
  return runtimeModule;
}

export async function restoreDashboardSession() {
  const module = await ensureMountedRuntime();
  if (!module || typeof module.restoreDashboardSession !== 'function') return false;
  return module.restoreDashboardSession();
}

export function getDashboardSessionState() {
  if (!runtimeModule || typeof runtimeModule.getDashboardSessionState !== 'function') {
    return { authenticated: false, csrfToken: '' };
  }
  return runtimeModule.getDashboardSessionState();
}

export async function refreshDashboardTab(tab, reason = 'manual', options = {}) {
  const module = await ensureMountedRuntime();
  if (!module || typeof module.refreshDashboardTab !== 'function') return;
  return module.refreshDashboardTab(tab, reason, options || {});
}

export function setDashboardActiveTab(tab) {
  if (!runtimeModule || typeof runtimeModule.setDashboardActiveTab !== 'function') return;
  runtimeModule.setDashboardActiveTab(tab);
}

export async function logoutDashboardSession() {
  const module = await ensureMountedRuntime();
  if (!module || typeof module.logoutDashboardSession !== 'function') return;
  return module.logoutDashboardSession();
}
