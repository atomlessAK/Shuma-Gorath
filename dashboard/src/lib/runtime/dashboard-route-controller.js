// @ts-check

const DEFAULT_TAB_LOADING_MESSAGES = Object.freeze({
  monitoring: 'Loading monitoring data...',
  'ip-bans': 'Loading ban list...',
  status: 'Loading status signals...',
  config: 'Loading config...',
  tuning: 'Loading tuning values...'
});

function defaultNowMs() {
  if (typeof window !== 'undefined' && window.performance && typeof window.performance.now === 'function') {
    return window.performance.now();
  }
  return Date.now();
}

function defaultRequestNextFrame(callback) {
  if (typeof callback !== 'function') return;
  if (typeof window !== 'undefined' && typeof window.requestAnimationFrame === 'function') {
    window.requestAnimationFrame(callback);
    return;
  }
  setTimeout(callback, 0);
}

function defaultReadHashTab() {
  if (typeof window === 'undefined') return '';
  return String(window.location.hash || '').replace(/^#/, '');
}

function defaultWriteHashTab(tab, options = {}) {
  if (typeof window === 'undefined') return;
  const normalized = String(tab || '').replace(/^#/, '');
  if (!normalized) return;
  const nextHash = `#${normalized}`;
  if (window.location.hash === nextHash) return;
  if (options && options.replace === true) {
    const nextUrl = `${window.location.pathname}${window.location.search}${nextHash}`;
    window.history.replaceState(null, '', nextUrl);
    return;
  }
  window.location.hash = nextHash;
}

function defaultIsPageVisible() {
  if (typeof document === 'undefined') return true;
  return document.visibilityState !== 'hidden';
}

function isAbortError(error) {
  if (!error) return false;
  const name = String(error.name || '');
  const message = String(error.message || '');
  return name === 'AbortError' || message.toLowerCase().includes('abort');
}

function normalizeRefreshInterval(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return 0;
  return numeric;
}

export function createDashboardRouteController(options = {}) {
  const tabs = Array.isArray(options.tabs) && options.tabs.length > 0
    ? Object.freeze([...options.tabs])
    : Object.freeze(['monitoring', 'ip-bans', 'status', 'config', 'tuning']);

  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (tab) => String(tab || '');
  const tabLoadingMessages = {
    ...DEFAULT_TAB_LOADING_MESSAGES,
    ...(options.tabLoadingMessages && typeof options.tabLoadingMessages === 'object'
      ? options.tabLoadingMessages
      : {})
  };

  const requestNextFrame =
    typeof options.requestNextFrame === 'function' ? options.requestNextFrame : defaultRequestNextFrame;
  const nowMs = typeof options.nowMs === 'function' ? options.nowMs : defaultNowMs;
  const readHashTab = typeof options.readHashTab === 'function' ? options.readHashTab : defaultReadHashTab;
  const writeHashTab =
    typeof options.writeHashTab === 'function' ? options.writeHashTab : defaultWriteHashTab;
  const isPageVisible =
    typeof options.isPageVisible === 'function' ? options.isPageVisible : defaultIsPageVisible;

  const store = options.store;
  const refreshDashboardTab =
    typeof options.refreshDashboardTab === 'function' ? options.refreshDashboardTab : null;
  const setDashboardActiveTab =
    typeof options.setDashboardActiveTab === 'function' ? options.setDashboardActiveTab : null;

  const restoreDashboardSession =
    typeof options.restoreDashboardSession === 'function' ? options.restoreDashboardSession : null;
  const getDashboardSessionState =
    typeof options.getDashboardSessionState === 'function' ? options.getDashboardSessionState : null;
  const mountDashboardApp =
    typeof options.mountDashboardApp === 'function' ? options.mountDashboardApp : null;

  const recordRefreshMetrics =
    typeof options.recordRefreshMetrics === 'function' ? options.recordRefreshMetrics : () => {};
  const setPollingContext =
    typeof options.setPollingContext === 'function' ? options.setPollingContext : () => {};
  const recordPollingSkip =
    typeof options.recordPollingSkip === 'function' ? options.recordPollingSkip : () => {};
  const recordPollingResume =
    typeof options.recordPollingResume === 'function' ? options.recordPollingResume : () => {};

  const isAuthenticated =
    typeof options.isAuthenticated === 'function'
      ? options.isAuthenticated
      : () => Boolean(store && store.getState && store.getState()?.session?.authenticated === true);

  const redirectToLogin =
    typeof options.redirectToLogin === 'function' ? options.redirectToLogin : () => {};

  const onBootstrapSession =
    typeof options.onBootstrapSession === 'function' ? options.onBootstrapSession : () => {};
  const onRefreshError =
    typeof options.onRefreshError === 'function' ? options.onRefreshError : () => {};

  const selectRefreshInterval =
    typeof options.selectRefreshInterval === 'function'
      ? options.selectRefreshInterval
      : () => 30000;

  let mounted = false;
  let runtimeMounted = false;
  let pollTimer = null;
  let inFlightRefresh = null;
  let pollingPaused = true;

  function clearPolling() {
    if (!pollTimer) return;
    clearTimeout(pollTimer);
    pollTimer = null;
  }

  function abortInFlightRefresh() {
    if (!inFlightRefresh) return;
    inFlightRefresh.controller.abort();
    inFlightRefresh = null;
  }

  async function refreshTab(tab, reason = 'manual') {
    if (!runtimeMounted || typeof refreshDashboardTab !== 'function') return;
    const normalized = normalizeTab(tab);

    if (inFlightRefresh && inFlightRefresh.tab === normalized) {
      if (reason === 'auto-refresh') {
        return inFlightRefresh.promise;
      }
      abortInFlightRefresh();
    }

    const controller = new AbortController();
    const refreshStartedAt = nowMs();
    const showLoadingState = reason !== 'auto-refresh';
    if (showLoadingState && store) {
      store.clearTabError(normalized);
      store.setTabLoading(
        normalized,
        true,
        tabLoadingMessages[normalized] || 'Loading...'
      );
    }

    const refreshPromise = (async () => {
      try {
        await refreshDashboardTab(normalized, reason, { signal: controller.signal });
        const refreshCompletedAt = nowMs();
        await new Promise((resolve) => {
          requestNextFrame(resolve);
        });
        const renderCompletedAt = nowMs();
        recordRefreshMetrics({
          tab: normalized,
          reason,
          fetchLatencyMs: refreshCompletedAt - refreshStartedAt,
          renderTimingMs: renderCompletedAt - refreshCompletedAt
        });
      } catch (error) {
        if (isAbortError(error)) return;
        const message = error && error.message ? error.message : 'Refresh failed';
        if (store) {
          store.setTabError(normalized, message);
        }
        onRefreshError({ tab: normalized, reason, message, error });
      } finally {
        if (showLoadingState && store) {
          store.setTabLoading(normalized, false);
        }
      }
    })();

    inFlightRefresh = {
      tab: normalized,
      controller,
      promise: refreshPromise
    };

    try {
      await refreshPromise;
    } finally {
      if (inFlightRefresh && inFlightRefresh.promise === refreshPromise) {
        inFlightRefresh = null;
      }
    }
  }

  function schedulePolling(resumeReason = 'schedule') {
    clearPolling();
    const activeTab = normalizeTab(store ? store.getState().activeTab : tabs[0]);
    const intervalMs = normalizeRefreshInterval(selectRefreshInterval(activeTab));
    setPollingContext(activeTab, intervalMs);

    if (!mounted) {
      recordPollingSkip('not-mounted', activeTab, intervalMs);
      pollingPaused = true;
      return;
    }
    if (!isAuthenticated()) {
      recordPollingSkip('unauthenticated', activeTab, intervalMs);
      pollingPaused = true;
      return;
    }
    if (!isPageVisible()) {
      recordPollingSkip('page-hidden', activeTab, intervalMs);
      pollingPaused = true;
      return;
    }

    if (pollingPaused) {
      recordPollingResume(resumeReason, activeTab, intervalMs);
      pollingPaused = false;
    }

    pollTimer = setTimeout(async () => {
      pollTimer = null;
      const currentTab = normalizeTab(store ? store.getState().activeTab : tabs[0]);
      const currentInterval = normalizeRefreshInterval(selectRefreshInterval(currentTab));
      if (!mounted || !isAuthenticated() || !isPageVisible()) {
        if (!mounted) {
          recordPollingSkip('not-mounted', currentTab, currentInterval);
        } else if (!isAuthenticated()) {
          recordPollingSkip('unauthenticated', currentTab, currentInterval);
        } else {
          recordPollingSkip('page-hidden', currentTab, currentInterval);
        }
        pollingPaused = true;
        schedulePolling('condition-recheck');
        return;
      }
      await refreshTab(currentTab, 'auto-refresh');
      schedulePolling('cycle');
    }, intervalMs);
  }

  async function applyActiveTab(tab, options = {}) {
    const normalized = normalizeTab(tab);
    const current = normalizeTab(store ? store.getState().activeTab : normalized);
    const changed = current !== normalized;
    const reason = String(options.reason || 'programmatic');

    if (!changed && options.force !== true) {
      if (options.syncHash === true) {
        writeHashTab(normalized, { replace: options.replaceHash === true });
      }
      return;
    }

    if (store) {
      store.setActiveTab(normalized);
    }
    if (runtimeMounted && typeof setDashboardActiveTab === 'function') {
      setDashboardActiveTab(normalized);
    }

    if (changed) {
      abortInFlightRefresh();
    }

    if (options.syncHash === true) {
      writeHashTab(normalized, { replace: options.replaceHash === true });
    }

    if (isAuthenticated()) {
      await refreshTab(normalized, reason);
    }
    schedulePolling('tab-change');
  }

  function syncFromHash(reason = 'hashchange') {
    const rawHashTab = readHashTab();
    const hashTab = normalizeTab(rawHashTab);
    if (rawHashTab !== hashTab) {
      writeHashTab(hashTab, { replace: true });
    }
    void applyActiveTab(hashTab, { reason, syncHash: false });
  }

  function keyNavTarget(currentTab, key) {
    const index = tabs.indexOf(normalizeTab(currentTab));
    const safeIndex = index >= 0 ? index : 0;
    if (key === 'ArrowRight') {
      return tabs[(safeIndex + 1) % tabs.length];
    }
    if (key === 'ArrowLeft') {
      return tabs[(safeIndex - 1 + tabs.length) % tabs.length];
    }
    if (key === 'Home') return tabs[0];
    if (key === 'End') return tabs[tabs.length - 1];
    return null;
  }

  async function bootstrapSession() {
    if (typeof restoreDashboardSession !== 'function' || typeof getDashboardSessionState !== 'function') {
      return false;
    }

    const authenticated = await restoreDashboardSession();
    const runtimeSession = getDashboardSessionState();

    if (store) {
      store.setSession({
        authenticated: runtimeSession.authenticated === true,
        csrfToken: runtimeSession.csrfToken || ''
      });
    }
    onBootstrapSession(runtimeSession);

    if (!authenticated) {
      abortInFlightRefresh();
      clearPolling();
      redirectToLogin();
      return false;
    }

    const currentTab = normalizeTab(store ? store.getState().activeTab : tabs[0]);
    await refreshTab(currentTab, 'session-restored');
    schedulePolling('session-restored');
    return true;
  }

  async function bootstrapRuntime(runtimeOptions = {}) {
    if (typeof mountDashboardApp !== 'function') return false;
    const initialTab = normalizeTab(runtimeOptions.initialTab || readHashTab());

    await mountDashboardApp({
      ...runtimeOptions,
      initialTab,
      store
    });
    runtimeMounted = true;

    if (store) {
      store.setActiveTab(initialTab);
    }
    if (typeof setDashboardActiveTab === 'function') {
      setDashboardActiveTab(initialTab);
    }
    if (readHashTab() !== initialTab) {
      writeHashTab(initialTab, { replace: true });
    }

    return bootstrapSession();
  }

  function handleVisibilityChange() {
    if (!runtimeMounted) return;
    if (isPageVisible()) {
      schedulePolling('visibility-resume');
      return;
    }
    const activeTab = normalizeTab(store ? store.getState().activeTab : tabs[0]);
    const intervalMs = normalizeRefreshInterval(selectRefreshInterval(activeTab));
    setPollingContext(activeTab, intervalMs);
    recordPollingSkip('page-hidden', activeTab, intervalMs);
    pollingPaused = true;
    clearPolling();
  }

  function setMounted(nextMounted) {
    mounted = nextMounted === true;
  }

  function setRuntimeMounted(nextRuntimeMounted) {
    runtimeMounted = nextRuntimeMounted === true;
  }

  function getRuntimeMounted() {
    return runtimeMounted;
  }

  function dispose() {
    mounted = false;
    pollingPaused = true;
    abortInFlightRefresh();
    clearPolling();
    runtimeMounted = false;
  }

  return {
    refreshTab,
    schedulePolling,
    applyActiveTab,
    syncFromHash,
    keyNavTarget,
    bootstrapSession,
    bootstrapRuntime,
    handleVisibilityChange,
    setMounted,
    setRuntimeMounted,
    getRuntimeMounted,
    abortInFlightRefresh,
    clearPolling,
    dispose
  };
}
