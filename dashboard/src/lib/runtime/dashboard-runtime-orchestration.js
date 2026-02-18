export function createDashboardTabRuntime(options = {}) {
  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (value) => String(value || '');
  const defaultTab = String(options.defaultTab || 'monitoring');
  const documentRef = options.document || null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;
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
    return normalized;
  }

  function getActiveTab() {
    const stateStore = getStateStore();
    if (stateStore && typeof stateStore.getActiveTab === 'function') {
      return stateStore.getActiveTab();
    }
    return defaultTab;
  }

  async function refreshTab(tab, reason = 'manual', runtimeOptions = {}) {
    const normalized = normalizeTab(tab);
    return refreshDashboardForTab(normalized, reason, runtimeOptions || {});
  }

  return {
    setActiveTab,
    getActiveTab,
    refreshTab
  };
}

export function createDashboardSessionRuntime(options = {}) {
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
