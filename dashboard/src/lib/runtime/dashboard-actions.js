import { DASHBOARD_TABS, normalizeTab } from '../state/dashboard-store.js';
import { buildDashboardLoginPath } from './dashboard-paths.js';

export function createDashboardActions(options = {}) {
  const store = options.store;
  const effects = options.effects;
  const runtime = options.runtime;

  if (!store || !effects || !runtime) {
    throw new Error('createDashboardActions requires store, effects, and runtime.');
  }

  let hashCleanup = null;
  let visibilityCleanup = null;
  let pollTimer = null;
  let mounted = false;
  let inFlightRefresh = null;
  let pollingPaused = true;
  const TAB_LOADING_MESSAGES = Object.freeze({
    monitoring: 'Loading monitoring data...',
    'ip-bans': 'Loading ban list...',
    status: 'Loading status signals...',
    config: 'Loading config...',
    tuning: 'Loading tuning values...'
  });
  const now =
    typeof effects.now === 'function'
      ? () => effects.now()
      : () => Date.now();

  function isAbortError(error) {
    if (!error) return false;
    const name = String(error.name || '');
    const message = String(error.message || '');
    return name === 'AbortError' || message.toLowerCase().includes('abort');
  }

  function clearPolling() {
    if (!pollTimer) return;
    effects.clearTimer(pollTimer);
    pollTimer = null;
  }

  function setPollingContext(tab, intervalMs) {
    if (typeof store.setPollingContext !== 'function') return;
    store.setPollingContext(tab, intervalMs);
  }

  function recordPollingSkip(reason, tab, intervalMs) {
    if (typeof store.recordPollingSkip !== 'function') return;
    store.recordPollingSkip(reason, tab, intervalMs);
  }

  function recordPollingResume(reason, tab, intervalMs) {
    if (typeof store.recordPollingResume !== 'function') return;
    store.recordPollingResume(reason, tab, intervalMs);
  }

  function recordRefreshMetrics(metrics = {}) {
    if (typeof store.recordRefreshMetrics !== 'function') return;
    store.recordRefreshMetrics(metrics);
  }

  function resolveLoginRedirectPath() {
    if (typeof effects.buildLoginRedirectPath === 'function') {
      return effects.buildLoginRedirectPath();
    }
    return buildDashboardLoginPath();
  }

  function isAuthenticated() {
    const session = store.getState().session || {};
    return session.authenticated === true;
  }

  function abortInFlightRefresh() {
    if (!inFlightRefresh) return;
    inFlightRefresh.controller.abort();
    inFlightRefresh = null;
  }

  async function refreshTab(tab, reason = 'manual') {
    const normalized = normalizeTab(tab);
    if (inFlightRefresh && inFlightRefresh.tab === normalized) {
      if (reason === 'auto-refresh') {
        return inFlightRefresh.promise;
      }
      abortInFlightRefresh();
    }

    const controller = new AbortController();
    const refreshStartedAt = now();
    const showLoadingState = reason !== 'auto-refresh';
    if (showLoadingState) {
      store.clearTabError(normalized);
      store.setTabLoading(normalized, true, TAB_LOADING_MESSAGES[normalized] || 'Loading...');
    }

    const refreshPromise = (async () => {
      try {
        await runtime.refreshTab(normalized, reason, { signal: controller.signal });
        const refreshCompletedAt = now();
        await new Promise((resolve) => {
          effects.requestFrame(() => resolve());
        });
        const renderCompletedAt = now();
        recordRefreshMetrics({
          tab: normalized,
          reason,
          fetchLatencyMs: refreshCompletedAt - refreshStartedAt,
          renderTimingMs: renderCompletedAt - refreshCompletedAt
        });
        store.markTabUpdated(normalized);
      } catch (error) {
        if (isAbortError(error)) {
          return;
        }
        const message = error && error.message ? error.message : 'Refresh failed';
        store.setTabError(normalized, message);
      } finally {
        if (showLoadingState) {
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
    const activeTab = normalizeTab(store.getState().activeTab);
    const intervalMs = store.selectRefreshInterval(activeTab);
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
    if (!effects.isPageVisible()) {
      recordPollingSkip('page-hidden', activeTab, intervalMs);
      pollingPaused = true;
      return;
    }

    if (pollingPaused) {
      recordPollingResume(resumeReason, activeTab, intervalMs);
      pollingPaused = false;
    }

    pollTimer = effects.setTimer(async () => {
      pollTimer = null;
      if (!mounted || !isAuthenticated() || !effects.isPageVisible()) {
        if (!mounted) {
          recordPollingSkip('not-mounted', activeTab, intervalMs);
        } else if (!isAuthenticated()) {
          recordPollingSkip('unauthenticated', activeTab, intervalMs);
        } else {
          recordPollingSkip('page-hidden', activeTab, intervalMs);
        }
        pollingPaused = true;
        schedulePolling('condition-recheck');
        return;
      }
      await refreshTab(activeTab, 'auto-refresh');
      schedulePolling('cycle');
    }, intervalMs);
  }

  async function applyActiveTab(tab, reason = 'programmatic', opts = {}) {
    const normalized = normalizeTab(tab);
    const current = normalizeTab(store.getState().activeTab);
    const changed = current !== normalized;

    if (!changed && opts.force !== true) {
      if (opts.syncHash === true) {
        effects.writeHashTab(normalized, { replace: opts.replaceHash === true });
      }
      return;
    }

    store.setActiveTab(normalized);
    runtime.setActiveTab(normalized);
    if (changed) {
      abortInFlightRefresh();
    }

    if (opts.syncHash === true) {
      effects.writeHashTab(normalized, { replace: opts.replaceHash === true });
    }

    if (isAuthenticated()) {
      await refreshTab(normalized, reason);
    }

    schedulePolling('tab-change');
  }

  function syncFromHash(reason = 'hash') {
    const hashTab = normalizeTab(effects.readHashTab());
    if (effects.readHashTab() !== hashTab) {
      effects.writeHashTab(hashTab, { replace: true });
    }
    void applyActiveTab(hashTab, reason, { syncHash: false });
  }

  function keyNavTarget(currentTab, key) {
    const index = DASHBOARD_TABS.indexOf(normalizeTab(currentTab));
    const safeIndex = index >= 0 ? index : 0;
    if (key === 'ArrowRight') {
      return DASHBOARD_TABS[(safeIndex + 1) % DASHBOARD_TABS.length];
    }
    if (key === 'ArrowLeft') {
      return DASHBOARD_TABS[(safeIndex - 1 + DASHBOARD_TABS.length) % DASHBOARD_TABS.length];
    }
    if (key === 'Home') return DASHBOARD_TABS[0];
    if (key === 'End') return DASHBOARD_TABS[DASHBOARD_TABS.length - 1];
    return null;
  }

  async function bootstrapSession() {
    const authenticated = await runtime.restoreSession();
    const runtimeSession = runtime.getSessionState();

    store.setSession({
      authenticated: runtimeSession.authenticated === true,
      csrfToken: runtimeSession.csrfToken || ''
    });

    if (!authenticated) {
      abortInFlightRefresh();
      clearPolling();
      effects.redirect(resolveLoginRedirectPath());
      return false;
    }

    await refreshTab(store.getState().activeTab, 'session-restored');
    schedulePolling('session-restored');
    return true;
  }

  async function logout() {
    abortInFlightRefresh();
    await runtime.logout();
    store.setSession({ authenticated: false, csrfToken: '' });
    pollingPaused = true;
    clearPolling();
    effects.redirect(resolveLoginRedirectPath());
  }

  function onTabKeydown(event, tab) {
    const target = keyNavTarget(tab, event.key);
    if (!target) return;
    event.preventDefault();
    void applyActiveTab(target, 'keyboard', { syncHash: true });
    effects.requestFrame(() => {
      if (typeof effects.focusTab === 'function') {
        effects.focusTab(target);
      }
    });
  }

  function onTabClick(event, tab) {
    event.preventDefault();
    void applyActiveTab(tab, 'click', { syncHash: true });
  }

  function init() {
    if (mounted) return;
    mounted = true;

    const initialHash = normalizeTab(effects.readHashTab());
    if (effects.readHashTab() !== initialHash) {
      effects.writeHashTab(initialHash, { replace: true });
    }
    void applyActiveTab(initialHash, 'initial-hash', { syncHash: false, force: true });

    hashCleanup = effects.onHashChange(() => {
      syncFromHash('hashchange');
    });

    visibilityCleanup = effects.onVisibilityChange(() => {
      if (effects.isPageVisible()) {
        schedulePolling('visibility-resume');
      } else {
        const activeTab = normalizeTab(store.getState().activeTab);
        const intervalMs = store.selectRefreshInterval(activeTab);
        setPollingContext(activeTab, intervalMs);
        recordPollingSkip('page-hidden', activeTab, intervalMs);
        pollingPaused = true;
        clearPolling();
      }
    });
  }

  function destroy() {
    mounted = false;
    pollingPaused = true;
    abortInFlightRefresh();
    clearPolling();
    if (hashCleanup) {
      hashCleanup();
      hashCleanup = null;
    }
    if (visibilityCleanup) {
      visibilityCleanup();
      visibilityCleanup = null;
    }
  }

  return {
    init,
    destroy,
    bootstrapSession,
    logout,
    applyActiveTab,
    onTabKeydown,
    onTabClick,
    refreshTab
  };
}
