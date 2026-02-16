// @ts-check

(function (global) {
  const DASHBOARD_TABS = Object.freeze(['monitoring', 'ip-bans', 'status', 'config', 'tuning']);
  const DEFAULT_TAB = 'monitoring';

  /**
   * @param {string} raw
   * @returns {string}
   */
  function normalizeTab(raw) {
    const tab = String(raw || '').trim().toLowerCase();
    return DASHBOARD_TABS.includes(tab) ? tab : DEFAULT_TAB;
  }

  function createTabFlags(initialValue) {
    const next = {};
    DASHBOARD_TABS.forEach((tab) => {
      next[tab] = Boolean(initialValue);
    });
    return next;
  }

  function createTabStatusState() {
    const state = {};
    DASHBOARD_TABS.forEach((tab) => {
      state[tab] = {
        loading: false,
        error: '',
        empty: false,
        updatedAt: ''
      };
    });
    return state;
  }

  function create(options = {}) {
    const initialTab = normalizeTab(options.initialTab || DEFAULT_TAB);

    const state = {
      activeTab: initialTab,
      stale: createTabFlags(true),
      session: {
        authenticated: false,
        csrfToken: ''
      },
      snapshots: {
        analytics: null,
        events: null,
        bans: null,
        maze: null,
        cdp: null,
        cdpEvents: null,
        monitoring: null,
        config: null
      },
      tabStatus: createTabStatusState()
    };

    const invalidationScopes = Object.freeze({
      all: DASHBOARD_TABS,
      monitoring: ['monitoring'],
      'ip-bans': ['ip-bans'],
      status: ['status'],
      config: ['config'],
      tuning: ['tuning'],
      securityConfig: ['status', 'config', 'tuning']
    });

    function setActiveTab(tabName) {
      state.activeTab = normalizeTab(tabName);
    }

    function getActiveTab() {
      return state.activeTab;
    }

    function setSession(nextSession = {}) {
      state.session = {
        authenticated: nextSession.authenticated === true,
        csrfToken: nextSession.authenticated === true ? String(nextSession.csrfToken || '') : ''
      };
    }

    function getSession() {
      return {
        authenticated: state.session.authenticated,
        csrfToken: state.session.csrfToken
      };
    }

    function setSnapshot(key, value) {
      if (!Object.prototype.hasOwnProperty.call(state.snapshots, key)) return;
      state.snapshots[key] = value;
    }

    function getSnapshot(key) {
      return Object.prototype.hasOwnProperty.call(state.snapshots, key) ? state.snapshots[key] : null;
    }

    function setTabLoading(tabName, loading) {
      const tab = normalizeTab(tabName);
      state.tabStatus[tab].loading = Boolean(loading);
      if (loading) state.tabStatus[tab].error = '';
    }

    function setTabError(tabName, message) {
      const tab = normalizeTab(tabName);
      state.tabStatus[tab].error = String(message || '');
      state.tabStatus[tab].loading = false;
      state.tabStatus[tab].updatedAt = new Date().toISOString();
    }

    function clearTabError(tabName) {
      const tab = normalizeTab(tabName);
      state.tabStatus[tab].error = '';
    }

    function setTabEmpty(tabName, empty) {
      const tab = normalizeTab(tabName);
      state.tabStatus[tab].empty = Boolean(empty);
    }

    function markTabUpdated(tabName) {
      const tab = normalizeTab(tabName);
      state.tabStatus[tab].updatedAt = new Date().toISOString();
      state.tabStatus[tab].loading = false;
      state.tabStatus[tab].error = '';
      state.stale[tab] = false;
    }

    /**
     * Explicit invalidation rules:
     * - `monitoring`: stats/charts/events/maze summaries
     * - `ip-bans`: bans table and quick actions
     * - `status|config|tuning`: shared config snapshot consumers
     * - `securityConfig`: alias for status+config+tuning invalidation after config writes
     * - `all`: complete dashboard invalidation
     *
     * @param {string} scope
     */
    function invalidate(scope = 'all') {
      const tabs = invalidationScopes[scope] || invalidationScopes.all;
      tabs.forEach((tab) => {
        state.stale[tab] = true;
      });
    }

    function isTabStale(tabName) {
      const tab = normalizeTab(tabName);
      return state.stale[tab] === true;
    }

    function getTabStatus(tabName) {
      const tab = normalizeTab(tabName);
      return {
        loading: state.tabStatus[tab].loading,
        error: state.tabStatus[tab].error,
        empty: state.tabStatus[tab].empty,
        updatedAt: state.tabStatus[tab].updatedAt,
        stale: state.stale[tab] === true
      };
    }

    function getDerivedState() {
      const analytics = state.snapshots.analytics || {};
      const events = state.snapshots.events || {};
      const bans = state.snapshots.bans || {};
      const maze = state.snapshots.maze || {};
      const monitoringEmpty =
        (Array.isArray(events.recent_events) ? events.recent_events.length : 0) === 0 &&
        (Array.isArray(bans.bans) ? bans.bans.length : 0) === 0 &&
        Number(maze.total_hits || 0) === 0;

      return {
        monitoringEmpty,
        hasConfigSnapshot: Boolean(state.snapshots.config),
        activeTab: state.activeTab
      };
    }

    return {
      DASHBOARD_TABS,
      DEFAULT_TAB,
      normalizeTab,
      setActiveTab,
      getActiveTab,
      setSession,
      getSession,
      setSnapshot,
      getSnapshot,
      setTabLoading,
      setTabError,
      clearTabError,
      setTabEmpty,
      markTabUpdated,
      invalidate,
      isTabStale,
      getTabStatus,
      getDerivedState
    };
  }

  global.ShumaDashboardState = {
    create,
    normalizeTab,
    DASHBOARD_TABS,
    DEFAULT_TAB
  };
})(window);
