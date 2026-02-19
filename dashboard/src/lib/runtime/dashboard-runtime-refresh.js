export function createDashboardRefreshRuntime(options = {}) {
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

  const isConfigSnapshotEmpty = (config) =>
    !config || typeof config !== 'object' || Object.keys(config).length === 0;

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

    if (!dashboardApiClient) {
      return dashboardState ? dashboardState.getSnapshot('config') : null;
    }
    if (dashboardState && reason === 'auto-refresh' && !dashboardState.isTabStale('config')) {
      return dashboardState.getSnapshot('config');
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
    const requestOptions = toRequestOptions(runtimeOptions);
    const monitoringData = await dashboardApiClient.getMonitoring({ hours: 24, limit: 10 }, requestOptions);
    const configSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
    const monitoringDetails = monitoringData && typeof monitoringData === 'object'
      ? (monitoringData.details || {})
      : {};
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

    if (dashboardState) {
      applySnapshots(
        isAutoRefresh
          ? { monitoring: monitoringData }
          : {
            monitoring: monitoringData,
            analytics,
            events,
            bans: bansData,
            maze: mazeData,
            cdp: cdpData,
            cdpEvents: cdpEventsData
          }
      );
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
    const requestOptions = toRequestOptions(runtimeOptions);
    const [bansData] = await Promise.all([
      dashboardApiClient.getBans(requestOptions),
      includeConfigRefresh ? refreshSharedConfig(reason, runtimeOptions) : Promise.resolve(null)
    ]);
    if (dashboardState) applySnapshots({ bans: bansData });
    if (!Array.isArray(bansData.bans) || bansData.bans.length === 0) {
      showTabEmpty('ip-bans', 'No active bans.');
    } else {
      clearTabStateMessage('ip-bans');
    }
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
