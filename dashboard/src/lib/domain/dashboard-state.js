// @ts-check

export const DASHBOARD_TABS = Object.freeze(['monitoring', 'ip-bans', 'status', 'config', 'tuning']);
export const DEFAULT_TAB = 'monitoring';

const SNAPSHOT_KEYS = Object.freeze([
  'analytics',
  'events',
  'bans',
  'maze',
  'cdp',
  'cdpEvents',
  'monitoring',
  'config'
]);

const TAB_STATUS_DEFAULT = Object.freeze({
  loading: false,
  error: '',
  message: '',
  empty: false,
  updatedAt: ''
});

const INVALIDATION_SCOPES = Object.freeze({
  all: DASHBOARD_TABS,
  monitoring: ['monitoring'],
  'ip-bans': ['ip-bans'],
  status: ['status'],
  config: ['config'],
  tuning: ['tuning'],
  securityConfig: ['status', 'config', 'tuning']
});

const cloneTabFlags = (value) => {
  const next = {};
  DASHBOARD_TABS.forEach((tab) => {
    next[tab] = Boolean(value);
  });
  return next;
};

const createTabStatusState = () => {
  const next = {};
  DASHBOARD_TABS.forEach((tab) => {
    next[tab] = { ...TAB_STATUS_DEFAULT };
  });
  return next;
};

const createSnapshotVersionState = () => {
  const next = {};
  SNAPSHOT_KEYS.forEach((key) => {
    next[key] = 0;
  });
  return next;
};

export const normalizeTab = (raw) => {
  const tab = String(raw || '').trim().toLowerCase();
  return DASHBOARD_TABS.includes(tab) ? tab : DEFAULT_TAB;
};

export const createInitialState = (initialTab = DEFAULT_TAB) => ({
  activeTab: normalizeTab(initialTab),
  stale: cloneTabFlags(true),
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
  snapshotVersions: createSnapshotVersionState(),
  tabStatus: createTabStatusState()
});

const timestampNow = () => new Date().toISOString();

export const actions = Object.freeze({
  setActiveTab: (tab) => ({ type: 'set-active-tab', tab }),
  setSession: (session) => ({ type: 'set-session', session }),
  setSnapshot: (key, value) => ({ type: 'set-snapshot', key, value }),
  setSnapshots: (snapshots) => ({ type: 'set-snapshots', snapshots }),
  setTabLoading: (tab, loading) => ({ type: 'set-tab-loading', tab, loading }),
  setTabError: (tab, message) => ({ type: 'set-tab-error', tab, message }),
  clearTabError: (tab) => ({ type: 'clear-tab-error', tab }),
  setTabEmpty: (tab, empty) => ({ type: 'set-tab-empty', tab, empty }),
  markTabUpdated: (tab) => ({ type: 'mark-tab-updated', tab }),
  invalidate: (scope = 'all') => ({ type: 'invalidate', scope })
});

export const reduceState = (prevState, event = {}) => {
  const prev = prevState || createInitialState();
  const type = String(event.type || 'noop');
  switch (type) {
    case 'set-active-tab': {
      return {
        ...prev,
        activeTab: normalizeTab(event.tab)
      };
    }
    case 'set-session': {
      const authenticated = event.session && event.session.authenticated === true;
      const csrfToken = authenticated ? String(event.session.csrfToken || '') : '';
      return {
        ...prev,
        session: {
          authenticated,
          csrfToken
        }
      };
    }
    case 'set-snapshot': {
      const key = String(event.key || '');
      if (!SNAPSHOT_KEYS.includes(key)) return prev;
      if (Object.is(prev.snapshots[key], event.value)) return prev;
      return {
        ...prev,
        snapshots: {
          ...prev.snapshots,
          [key]: event.value
        },
        snapshotVersions: {
          ...prev.snapshotVersions,
          [key]: Number(prev.snapshotVersions[key] || 0) + 1
        }
      };
    }
    case 'set-snapshots': {
      const updates =
        event && event.snapshots && typeof event.snapshots === 'object'
          ? event.snapshots
          : null;
      if (!updates) return prev;

      let changed = false;
      const nextSnapshots = { ...prev.snapshots };
      const nextVersions = { ...prev.snapshotVersions };

      SNAPSHOT_KEYS.forEach((key) => {
        if (!Object.prototype.hasOwnProperty.call(updates, key)) return;
        const nextValue = updates[key];
        if (Object.is(prev.snapshots[key], nextValue)) return;
        changed = true;
        nextSnapshots[key] = nextValue;
        nextVersions[key] = Number(nextVersions[key] || 0) + 1;
      });

      if (!changed) return prev;

      return {
        ...prev,
        snapshots: nextSnapshots,
        snapshotVersions: nextVersions
      };
    }
    case 'mark-tab-updated': {
      const tab = normalizeTab(event.tab);
      const updatedAt = String(event.updatedAt || timestampNow());
      if (prev.stale[tab] === false && prev.tabStatus[tab].updatedAt === updatedAt) {
        return prev;
      }
      return {
        ...prev,
        stale: {
          ...prev.stale,
          [tab]: false
        },
        tabStatus: {
          ...prev.tabStatus,
          [tab]: {
            ...prev.tabStatus[tab],
            loading: false,
            updatedAt
          }
        }
      };
    }
    case 'set-tab-loading': {
      const tab = normalizeTab(event.tab);
      const loading = event.loading === true;
      const hasExplicitMessage = Object.prototype.hasOwnProperty.call(event, 'message');
      const message = loading
        ? String(event.message || 'Loading...')
        : (hasExplicitMessage ? String(event.message || '') : prev.tabStatus[tab].message);
      return {
        ...prev,
        tabStatus: {
          ...prev.tabStatus,
          [tab]: {
            ...prev.tabStatus[tab],
            loading,
            error: loading ? '' : prev.tabStatus[tab].error,
            message
          }
        }
      };
    }
    case 'set-tab-error': {
      const tab = normalizeTab(event.tab);
      return {
        ...prev,
        tabStatus: {
          ...prev.tabStatus,
          [tab]: {
            ...prev.tabStatus[tab],
            error: String(event.message || ''),
            message: String(event.message || ''),
            loading: false,
            updatedAt: String(event.updatedAt || timestampNow())
          }
        }
      };
    }
    case 'clear-tab-error': {
      const tab = normalizeTab(event.tab);
      return {
        ...prev,
        tabStatus: {
          ...prev.tabStatus,
          [tab]: {
            ...prev.tabStatus[tab],
            error: '',
            message: ''
          }
        }
      };
    }
    case 'set-tab-empty': {
      const tab = normalizeTab(event.tab);
      return {
        ...prev,
        tabStatus: {
          ...prev.tabStatus,
          [tab]: {
            ...prev.tabStatus[tab],
            empty: event.empty === true,
            message: event.empty === true ? String(event.message || 'No data.') : ''
          }
        }
      };
    }
    case 'invalidate': {
      const scope = String(event.scope || 'all');
      const tabs = INVALIDATION_SCOPES[scope] || INVALIDATION_SCOPES.all;
      const stale = { ...prev.stale };
      tabs.forEach((tab) => {
        stale[tab] = true;
      });
      return {
        ...prev,
        stale
      };
    }
    default:
      return prev;
  }
};

const deriveMonitoringEmpty = (state) => {
  const events = state.snapshots.events || {};
  const bans = state.snapshots.bans || {};
  const maze = state.snapshots.maze || {};
  return (
    (Array.isArray(events.recent_events) ? events.recent_events.length : 0) === 0 &&
    (Array.isArray(bans.bans) ? bans.bans.length : 0) === 0 &&
    Number(maze.total_hits || 0) === 0
  );
};

export const selectors = Object.freeze({
  activeTab: (state) => state.activeTab,
  session: (state) => ({
    authenticated: state.session.authenticated,
    csrfToken: state.session.csrfToken
  }),
  snapshot: (state, key) => (Object.prototype.hasOwnProperty.call(state.snapshots, key)
    ? state.snapshots[key]
    : null),
  snapshotVersion: (state, key) =>
    (Object.prototype.hasOwnProperty.call(state.snapshotVersions, key)
      ? Number(state.snapshotVersions[key] || 0)
      : 0),
  snapshotVersions: (state) => ({ ...state.snapshotVersions }),
  tabStatus: (state, tabName) => {
    const tab = normalizeTab(tabName);
    return {
      loading: state.tabStatus[tab].loading,
      error: state.tabStatus[tab].error,
      message: state.tabStatus[tab].message,
      empty: state.tabStatus[tab].empty,
      updatedAt: state.tabStatus[tab].updatedAt,
      stale: state.stale[tab] === true
    };
  },
  tabIsStale: (state, tabName) => state.stale[normalizeTab(tabName)] === true,
  monitoringEmpty: (state) => deriveMonitoringEmpty(state),
  hasConfigSnapshot: (state) => Boolean(state.snapshots.config)
});

export const create = (options = {}) => {
  let state = createInitialState(options.initialTab || DEFAULT_TAB);

  const apply = (event) => {
    state = reduceState(state, event);
    return state;
  };

  const getState = () => state;

  const setActiveTab = (tabName) => {
    apply(actions.setActiveTab(tabName));
  };

  const getActiveTab = () => selectors.activeTab(state);

  const setSession = (nextSession = {}) => {
    apply(actions.setSession(nextSession));
  };

  const getSession = () => selectors.session(state);

  const setSnapshot = (key, value) => {
    apply(actions.setSnapshot(key, value));
  };

  const setSnapshots = (updates = {}) => {
    apply(actions.setSnapshots(updates));
  };

  const getSnapshot = (key) => {
    return selectors.snapshot(state, key);
  };

  const getSnapshotVersion = (key) => {
    return selectors.snapshotVersion(state, key);
  };

  const getSnapshotVersions = () => {
    return selectors.snapshotVersions(state);
  };

  const setTabLoading = (tabName, loading, message = undefined) => {
    const event = actions.setTabLoading(tabName, loading);
    if (message !== undefined) {
      event.message = message;
    }
    apply(event);
  };

  const setTabError = (tabName, message) => {
    apply(actions.setTabError(tabName, message));
  };

  const clearTabError = (tabName) => {
    apply(actions.clearTabError(tabName));
  };

  const setTabEmpty = (tabName, empty, message = undefined) => {
    const event = actions.setTabEmpty(tabName, empty);
    if (message !== undefined) {
      event.message = message;
    }
    apply(event);
  };

  const markTabUpdated = (tabName) => {
    apply(actions.markTabUpdated(tabName));
  };

  const invalidate = (scope = 'all') => {
    apply(actions.invalidate(scope));
  };

  const isTabStale = (tabName) => {
    return selectors.tabIsStale(state, tabName);
  };

  const getTabStatus = (tabName) => {
    return selectors.tabStatus(state, tabName);
  };

  const getDerivedState = () => ({
    monitoringEmpty: selectors.monitoringEmpty(state),
    hasConfigSnapshot: selectors.hasConfigSnapshot(state),
    activeTab: selectors.activeTab(state)
  });

  return {
    DASHBOARD_TABS,
    DEFAULT_TAB,
    normalizeTab,
    createInitialState,
    actions,
    selectors,
    reduceState,
    getState,
    setActiveTab,
    getActiveTab,
    setSession,
    getSession,
    setSnapshot,
    setSnapshots,
    getSnapshot,
    getSnapshotVersion,
    getSnapshotVersions,
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
};
