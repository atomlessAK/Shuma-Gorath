export function createDashboardRefreshRuntime(options = {}) {
  const MONITORING_CACHE_KEY = 'shuma_dashboard_cache_monitoring_v1';
  const IP_BANS_CACHE_KEY = 'shuma_dashboard_cache_ip_bans_v1';
  const DEFAULT_CACHE_TTL_MS = 60000;
  const MONITORING_CACHE_MAX_RECENT_EVENTS = 25;
  const MONITORING_CACHE_MAX_CDP_EVENTS = 50;
  const MONITORING_CACHE_MAX_BANS = 100;
  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (value) => String(value || '');
  const getApiClient =
    typeof options.getApiClient === 'function' ? options.getApiClient : () => null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;
  const deriveMonitoringAnalytics =
    typeof options.deriveMonitoringAnalytics === 'function'
      ? options.deriveMonitoringAnalytics
      : () => ({ ban_count: 0, test_mode: false, fail_mode: 'open' });
  const storage = options.storage && typeof options.storage === 'object'
    ? options.storage
    : (typeof window !== 'undefined' && window.localStorage ? window.localStorage : null);
  const cacheTtlMs = (() => {
    const numeric = Number(options.cacheTtlMs);
    if (!Number.isFinite(numeric) || numeric <= 0) return DEFAULT_CACHE_TTL_MS;
    return Math.max(1000, Math.floor(numeric));
  })();

  const isConfigSnapshotEmpty = (config) =>
    !config || typeof config !== 'object' || Object.keys(config).length === 0;
  const hasConfigSnapshot = (config) => !isConfigSnapshotEmpty(config);
  const toArray = (value) => (Array.isArray(value) ? value : []);

  function compactBansSnapshot(bansData = {}) {
    const source = bansData && typeof bansData === 'object' ? bansData : {};
    return {
      ...source,
      bans: toArray(source.bans).slice(0, MONITORING_CACHE_MAX_BANS)
    };
  }

  function compactMonitoringSnapshot(monitoringData = {}) {
    const source = monitoringData && typeof monitoringData === 'object' ? monitoringData : {};
    const details = source.details && typeof source.details === 'object' ? source.details : {};
    const events = details.events && typeof details.events === 'object' ? details.events : {};
    const cdpEvents = details.cdp_events && typeof details.cdp_events === 'object' ? details.cdp_events : {};
    const bans = details.bans && typeof details.bans === 'object' ? details.bans : {};
    return {
      ...source,
      details: {
        ...details,
        events: {
          ...events,
          recent_events: toArray(events.recent_events).slice(0, MONITORING_CACHE_MAX_RECENT_EVENTS)
        },
        bans: compactBansSnapshot(bans),
        cdp_events: {
          ...cdpEvents,
          events: toArray(cdpEvents.events).slice(0, MONITORING_CACHE_MAX_CDP_EVENTS),
          limit: Math.min(
            MONITORING_CACHE_MAX_CDP_EVENTS,
            Number.isFinite(Number(cdpEvents.limit)) && Number(cdpEvents.limit) > 0
              ? Math.floor(Number(cdpEvents.limit))
              : MONITORING_CACHE_MAX_CDP_EVENTS
          )
        }
      }
    };
  }

  function buildMonitoringSnapshots(monitoringData = {}, configSnapshot = {}) {
    const monitoring = monitoringData && typeof monitoringData === 'object' ? monitoringData : {};
    const monitoringDetails =
      monitoring && typeof monitoring.details === 'object' ? monitoring.details : {};
    const analyticsResponse = monitoringDetails.analytics || {};
    const events = monitoringDetails.events || {};
    const bansData = monitoringDetails.bans || { bans: [] };
    const mazeData = monitoringDetails.maze || {};
    const cdpData = monitoringDetails.cdp || {};
    const cdpEventsData = monitoringDetails.cdp_events || { events: [] };
    const analytics = deriveMonitoringAnalytics(configSnapshot, analyticsResponse);
    if (Array.isArray(bansData.bans)) {
      analytics.ban_count = bansData.bans.length;
    }
    return {
      monitoring,
      analytics,
      events,
      bans: bansData,
      maze: mazeData,
      cdp: cdpData,
      cdpEvents: cdpEventsData
    };
  }

  function shouldReadFromCache(reason = 'manual') {
    return !(
      reason === 'auto-refresh' ||
      reason === 'manual-refresh' ||
      reason === 'config-save' ||
      reason === 'ban-save' ||
      reason === 'unban-save'
    );
  }

  function readCache(cacheKey) {
    if (!storage) return null;
    try {
      const raw = storage.getItem(cacheKey);
      if (!raw) return null;
      const parsed = JSON.parse(raw);
      const cachedAt = Number(parsed?.cachedAt || 0);
      if (!Number.isFinite(cachedAt) || cachedAt <= 0 || (Date.now() - cachedAt) > cacheTtlMs) {
        storage.removeItem(cacheKey);
        return null;
      }
      const payload = parsed && typeof parsed.payload === 'object' ? parsed.payload : null;
      return payload && typeof payload === 'object' ? payload : null;
    } catch (_error) {
      return null;
    }
  }

  function writeCache(cacheKey, payload) {
    if (!storage || !payload || typeof payload !== 'object') return;
    try {
      storage.setItem(cacheKey, JSON.stringify({
        cachedAt: Date.now(),
        payload
      }));
    } catch (_error) {}
  }

  function clearCache(cacheKey) {
    if (!storage) return;
    try {
      storage.removeItem(cacheKey);
    } catch (_error) {}
  }

  function clearAllCaches() {
    clearCache(MONITORING_CACHE_KEY);
    clearCache(IP_BANS_CACHE_KEY);
  }

  function toRequestOptions(runtimeOptions = {}) {
    return runtimeOptions && runtimeOptions.signal ? { signal: runtimeOptions.signal } : {};
  }

  function applySnapshots(updates = {}) {
    const dashboardState = getStateStore();
    if (!dashboardState || !updates || typeof updates !== 'object') return;
    if (typeof dashboardState.setSnapshots === 'function') {
      dashboardState.setSnapshots(updates);
      return;
    }
    Object.entries(updates).forEach(([key, value]) => {
      dashboardState.setSnapshot(key, value);
    });
  }

  function showTabLoading(tab, message = 'Loading...') {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.clearTabError(tab);
    dashboardState.setTabEmpty(tab, false, '');
    dashboardState.setTabLoading(tab, true, message);
  }

  function showTabError(tab, message) {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.setTabEmpty(tab, false, '');
    dashboardState.setTabError(tab, message);
  }

  function showTabEmpty(tab, message) {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.clearTabError(tab);
    dashboardState.setTabLoading(tab, false, '');
    dashboardState.setTabEmpty(tab, true, message);
  }

  function clearTabStateMessage(tab) {
    const dashboardState = getStateStore();
    if (!dashboardState) return;
    dashboardState.setTabLoading(tab, false, '');
    dashboardState.setTabEmpty(tab, false, '');
    dashboardState.clearTabError(tab);
  }

  async function refreshSharedConfig(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    const dashboardState = getStateStore();
    const requestOptions = toRequestOptions(runtimeOptions);
    const existingConfig = dashboardState ? dashboardState.getSnapshot('config') : null;

    if (!dashboardApiClient) {
      return existingConfig;
    }
    if (hasConfigSnapshot(existingConfig)) {
      return existingConfig;
    }

    const config = await dashboardApiClient.getConfig(requestOptions);
    applySnapshots({ config });
    return config;
  }

  async function refreshMonitoringTab(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return;

    const isAutoRefresh = reason === 'auto-refresh';
    if (!isAutoRefresh) {
      showTabLoading('monitoring', 'Loading monitoring data...');
    }

    const dashboardState = getStateStore();
    if (shouldReadFromCache(reason)) {
      const cachedMonitoring = readCache(MONITORING_CACHE_KEY);
      if (cachedMonitoring) {
        const configSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
        const monitoringData =
          cachedMonitoring && typeof cachedMonitoring.monitoring === 'object'
            ? cachedMonitoring.monitoring
            : (cachedMonitoring && typeof cachedMonitoring === 'object'
              ? cachedMonitoring
              : {});
        applySnapshots(buildMonitoringSnapshots(monitoringData, configSnapshot));
        if (dashboardState && dashboardState.getDerivedState().monitoringEmpty) {
          showTabEmpty('monitoring', 'No operational events yet. Monitoring will populate as traffic arrives.');
        } else {
          clearTabStateMessage('monitoring');
        }
        return;
      }
    }

    const requestOptions = toRequestOptions(runtimeOptions);
    const monitoringData = await dashboardApiClient.getMonitoring({ hours: 24, limit: 10 }, requestOptions);
    const configSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
    const monitoringSnapshots = buildMonitoringSnapshots(monitoringData, configSnapshot);
    const compactMonitoring = compactMonitoringSnapshot(monitoringData);
    const compactBans = compactBansSnapshot(monitoringSnapshots.bans);
    if (dashboardState) {
      applySnapshots(monitoringSnapshots);
      writeCache(MONITORING_CACHE_KEY, { monitoring: compactMonitoring });
      writeCache(IP_BANS_CACHE_KEY, { bans: compactBans });
    } else {
      writeCache(MONITORING_CACHE_KEY, { monitoring: compactMonitoring });
      writeCache(IP_BANS_CACHE_KEY, { bans: compactBans });
    }

    if (dashboardState && dashboardState.getDerivedState().monitoringEmpty) {
      showTabEmpty('monitoring', 'No operational events yet. Monitoring will populate as traffic arrives.');
    } else {
      clearTabStateMessage('monitoring');
    }
  }

  async function refreshIpBansTab(reason = 'manual', runtimeOptions = {}) {
    const dashboardApiClient = getApiClient();
    if (!dashboardApiClient) return;
    const includeConfigRefresh = reason !== 'auto-refresh';
    if (reason !== 'auto-refresh') {
      showTabLoading('ip-bans', 'Loading ban list...');
    }
    const dashboardState = getStateStore();
    if (shouldReadFromCache(reason)) {
      const cachedIpBans = readCache(IP_BANS_CACHE_KEY);
      if (cachedIpBans) {
        applySnapshots(cachedIpBans);
        clearTabStateMessage('ip-bans');
        return;
      }
    }

    const requestOptions = toRequestOptions(runtimeOptions);
    const [bansData, configSnapshot] = await Promise.all([
      dashboardApiClient.getBans(requestOptions),
      includeConfigRefresh ? refreshSharedConfig(reason, runtimeOptions) : Promise.resolve(null)
    ]);
    const compactBans = compactBansSnapshot(bansData);
    if (dashboardState) {
      applySnapshots({ bans: bansData });
      if (hasConfigSnapshot(configSnapshot)) {
        applySnapshots({ config: configSnapshot });
      }
      writeCache(IP_BANS_CACHE_KEY, { bans: compactBans });
    } else {
      writeCache(IP_BANS_CACHE_KEY, { bans: compactBans });
    }

    if (reason === 'ban-save' || reason === 'unban-save') {
      clearCache(MONITORING_CACHE_KEY);
    }

    clearTabStateMessage('ip-bans');
  }

  async function refreshConfigBackedTab(
    tab,
    reason = 'manual',
    loadingMessage,
    emptyMessage,
    runtimeOptions = {}
  ) {
    if (reason !== 'auto-refresh') {
      showTabLoading(tab, loadingMessage);
    }
    const config = await refreshSharedConfig(reason, runtimeOptions);
    if (isConfigSnapshotEmpty(config)) {
      showTabEmpty(tab, emptyMessage);
    } else {
      clearTabStateMessage(tab);
    }
  }

  const refreshStatusTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'status',
      reason,
      'Loading status signals...',
      'No status config snapshot available yet.',
      runtimeOptions
    );

  const refreshConfigTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'config',
      reason,
      'Loading config...',
      'No config snapshot available yet.',
      runtimeOptions
    );

  const refreshTuningTab = (reason = 'manual', runtimeOptions = {}) =>
    refreshConfigBackedTab(
      'tuning',
      reason,
      'Loading tuning values...',
      'No tuning config snapshot available yet.',
      runtimeOptions
    );

  const TAB_REFRESH_HANDLERS = Object.freeze({
    monitoring: async (reason = 'manual', runtimeOptions = {}) => {
      await refreshMonitoringTab(reason, runtimeOptions);
      if (reason !== 'auto-refresh') {
        await refreshSharedConfig(reason, runtimeOptions);
      }
    },
    'ip-bans': refreshIpBansTab,
    status: refreshStatusTab,
    config: refreshConfigTab,
    tuning: refreshTuningTab
  });

  async function refreshDashboardForTab(tab, reason = 'manual', runtimeOptions = {}) {
    const activeTab = normalizeTab(tab);
    try {
      const handler = TAB_REFRESH_HANDLERS[activeTab] || TAB_REFRESH_HANDLERS.monitoring;
      await handler(reason, runtimeOptions);
      const dashboardState = getStateStore();
      if (dashboardState) {
        dashboardState.markTabUpdated(activeTab);
      }
    } catch (error) {
      if (error && error.name === 'AbortError') {
        return;
      }
      const message = error && error.message ? error.message : 'Refresh failed';
      console.error(`Dashboard refresh error (${activeTab}):`, error);
      showTabError(activeTab, message);
    }
  }

  function refreshActiveTab(reason = 'manual') {
    const dashboardState = getStateStore();
    const activeTab = dashboardState ? dashboardState.getActiveTab() : 'monitoring';
    return refreshDashboardForTab(activeTab, reason);
  }

  return {
    clearAllCaches,
    refreshSharedConfig,
    refreshMonitoringTab,
    refreshIpBansTab,
    refreshStatusTab,
    refreshConfigTab,
    refreshTuningTab,
    refreshDashboardForTab,
    refreshActiveTab
  };
}
