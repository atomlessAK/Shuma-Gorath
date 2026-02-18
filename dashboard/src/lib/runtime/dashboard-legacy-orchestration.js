export function createLegacyDashboardTabRuntime(options = {}) {
  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (value) => String(value || '');
  const defaultTab = String(options.defaultTab || 'monitoring');
  const documentRef = options.document || null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;
  const getTabCoordinator =
    typeof options.getTabCoordinator === 'function' ? options.getTabCoordinator : () => null;
  const getRuntimeMountOptions =
    typeof options.getRuntimeMountOptions === 'function'
      ? options.getRuntimeMountOptions
      : () => ({ useExternalTabPipeline: false });
  const refreshCoreActionButtonsState =
    typeof options.refreshCoreActionButtonsState === 'function'
      ? options.refreshCoreActionButtonsState
      : () => {};
  const refreshDashboardForTab =
    typeof options.refreshDashboardForTab === 'function' ? options.refreshDashboardForTab : async () => {};

  function setActiveTab(tab, reason = 'external') {
    const normalized = normalizeTab(tab);
    const stateStore = getStateStore();
    if (stateStore && typeof stateStore.setActiveTab === 'function') {
      stateStore.setActiveTab(normalized);
    }
    if (documentRef && documentRef.body && documentRef.body.dataset) {
      documentRef.body.dataset.activeDashboardTab = normalized;
    }
    refreshCoreActionButtonsState();
    const mountOptions = getRuntimeMountOptions() || {};
    const tabCoordinator = getTabCoordinator();
    if (
      mountOptions.useExternalTabPipeline !== true &&
      tabCoordinator &&
      typeof tabCoordinator.activate === 'function'
    ) {
      tabCoordinator.activate(normalized, reason);
    }
    return normalized;
  }

  function getActiveTab() {
    const tabCoordinator = getTabCoordinator();
    if (tabCoordinator && typeof tabCoordinator.getActiveTab === 'function') {
      return tabCoordinator.getActiveTab();
    }
    const stateStore = getStateStore();
    if (stateStore && typeof stateStore.getActiveTab === 'function') {
      return stateStore.getActiveTab();
    }
    return defaultTab;
  }

  async function refreshTab(tab, reason = 'manual', runtimeOptions = {}) {
    const normalized = setActiveTab(tab, reason);
    return refreshDashboardForTab(normalized, reason, runtimeOptions || {});
  }

  return {
    setActiveTab,
    getActiveTab,
    refreshTab
  };
}

export function createLegacyDashboardSessionRuntime(options = {}) {
  const getAdminSessionController =
    typeof options.getAdminSessionController === 'function'
      ? options.getAdminSessionController
      : () => null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;
  const refreshCoreActionButtonsState =
    typeof options.refreshCoreActionButtonsState === 'function'
      ? options.refreshCoreActionButtonsState
      : () => {};
  const resolveAdminApiEndpoint =
    typeof options.resolveAdminApiEndpoint === 'function' ? options.resolveAdminApiEndpoint : () => null;
  const getRuntimeEffects =
    typeof options.getRuntimeEffects === 'function' ? options.getRuntimeEffects : () => null;
  const getMessageNode =
    typeof options.getMessageNode === 'function' ? options.getMessageNode : () => null;

  function getSessionState() {
    const sessionController = getAdminSessionController();
    if (!sessionController || typeof sessionController.getState !== 'function') {
      return { authenticated: false, csrfToken: '' };
    }
    const state = sessionController.getState();
    return {
      authenticated: state.authenticated === true,
      csrfToken: state.csrfToken || ''
    };
  }

  async function restoreSession() {
    const sessionController = getAdminSessionController();
    if (!sessionController || typeof sessionController.restoreAdminSession !== 'function') {
      return false;
    }
    const authenticated = await sessionController.restoreAdminSession();
    const stateStore = getStateStore();
    if (stateStore && typeof stateStore.setSession === 'function') {
      const sessionState = getSessionState();
      stateStore.setSession({
        authenticated: sessionState.authenticated === true,
        csrfToken: sessionState.csrfToken || ''
      });
    }
    refreshCoreActionButtonsState();
    return authenticated;
  }

  async function logoutSession() {
    const resolved = resolveAdminApiEndpoint();
    const endpoint = resolved && resolved.endpoint ? resolved.endpoint : '';
    const effects = getRuntimeEffects();
    if (!endpoint || !effects || typeof effects.request !== 'function') return;

    const sessionState = getSessionState();
    const headers = new Headers();
    if (sessionState.csrfToken) {
      headers.set('X-Shuma-CSRF', sessionState.csrfToken);
    }

    try {
      await effects.request(`${endpoint}/admin/logout`, {
        method: 'POST',
        headers,
        credentials: 'same-origin'
      });
    } catch (_error) {}

    const sessionController = getAdminSessionController();
    if (sessionController && typeof sessionController.restoreAdminSession === 'function') {
      await sessionController.restoreAdminSession();
    }
    const stateStore = getStateStore();
    if (stateStore && typeof stateStore.setSession === 'function') {
      stateStore.setSession({ authenticated: false, csrfToken: '' });
    }
    const messageNode = getMessageNode();
    if (messageNode) {
      messageNode.textContent = 'Logged out';
      messageNode.className = 'message success';
    }
    refreshCoreActionButtonsState();
  }

  return {
    getSessionState,
    restoreSession,
    logoutSession
  };
}

export function createLegacyAutoRefreshRuntime(options = {}) {
  const effects = options.effects;
  const documentRef = options.document || null;
  const tabRefreshIntervals = options.tabRefreshIntervals || {};
  const defaultTab = String(options.defaultTab || 'monitoring');
  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (value) => String(value || defaultTab);
  const getActiveTab =
    typeof options.getActiveTab === 'function' ? options.getActiveTab : () => defaultTab;
  const hasValidApiContext =
    typeof options.hasValidApiContext === 'function' ? options.hasValidApiContext : () => false;
  const refreshDashboardForTab =
    typeof options.refreshDashboardForTab === 'function' ? options.refreshDashboardForTab : async () => {};

  if (!effects || typeof effects.setTimer !== 'function' || typeof effects.clearTimer !== 'function') {
    throw new Error('createLegacyAutoRefreshRuntime requires runtime effects with setTimer/clearTimer.');
  }

  let timer = null;
  let visibilityChangeListener = null;
  let pageVisible = documentRef ? documentRef.visibilityState !== 'hidden' : true;

  function clear() {
    if (!timer) return;
    effects.clearTimer(timer);
    timer = null;
  }

  function schedule() {
    clear();
    if (!hasValidApiContext() || !pageVisible) return;
    const activeTab = normalizeTab(getActiveTab());
    const intervalMs = tabRefreshIntervals[activeTab] || tabRefreshIntervals.monitoring || 30000;
    timer = effects.setTimer(async () => {
      timer = null;
      if (hasValidApiContext() && pageVisible) {
        await refreshDashboardForTab(activeTab, 'auto-refresh');
      }
      schedule();
    }, intervalMs);
  }

  function unbindVisibility() {
    if (!visibilityChangeListener || !documentRef) return;
    documentRef.removeEventListener('visibilitychange', visibilityChangeListener);
    visibilityChangeListener = null;
  }

  function bindVisibility() {
    if (!documentRef || typeof documentRef.addEventListener !== 'function') return;
    unbindVisibility();
    visibilityChangeListener = () => {
      pageVisible = documentRef.visibilityState !== 'hidden';
      if (pageVisible) {
        schedule();
      } else {
        clear();
      }
    };
    documentRef.addEventListener('visibilitychange', visibilityChangeListener);
  }

  function destroy() {
    clear();
    unbindVisibility();
  }

  return {
    schedule,
    clear,
    bindVisibility,
    unbindVisibility,
    destroy,
    isPageVisible: () => pageVisible
  };
}
