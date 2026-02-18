export function createDashboardRefreshRuntime(options = {}) {
  const normalizeTab =
    typeof options.normalizeTab === 'function' ? options.normalizeTab : (value) => String(value || '');
  const getApiClient =
    typeof options.getApiClient === 'function' ? options.getApiClient : () => null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;
  const getTablesView =
    typeof options.getTablesView === 'function' ? options.getTablesView : () => null;
  const getChartsRuntime =
    typeof options.getChartsRuntime === 'function' ? options.getChartsRuntime : () => null;
  const getMessageNode =
    typeof options.getMessageNode === 'function' ? options.getMessageNode : () => null;
  const runDomWriteBatch =
    typeof options.runDomWriteBatch === 'function' ? options.runDomWriteBatch : async (task) => task();
  const updateConfigModeUi =
    typeof options.updateConfigModeUi === 'function' ? options.updateConfigModeUi : () => {};
  const invokeConfigUiState =
    typeof options.invokeConfigUiState === 'function' ? options.invokeConfigUiState : () => {};
  const refreshAllDirtySections =
    typeof options.refreshAllDirtySections === 'function' ? options.refreshAllDirtySections : () => {};
  const refreshDirtySections =
    typeof options.refreshDirtySections === 'function' ? options.refreshDirtySections : () => {};
  const refreshCoreActionButtonsState =
    typeof options.refreshCoreActionButtonsState === 'function'
      ? options.refreshCoreActionButtonsState
      : () => {};
  const tabState =
    options.tabState && typeof options.tabState === 'object' ? options.tabState : {};
  const deriveMonitoringAnalytics =
    typeof options.deriveMonitoringAnalytics === 'function'
      ? options.deriveMonitoringAnalytics
      : () => ({ ban_count: 0, test_mode: false, fail_mode: 'open' });
  const configUiRefreshMethods = Array.isArray(options.configUiRefreshMethods)
    ? options.configUiRefreshMethods
    : [];
  const dirtySectionsByTab =
    options.dirtySectionsByTab && typeof options.dirtySectionsByTab === 'object'
      ? options.dirtySectionsByTab
      : {};

  const showTabLoading =
    typeof tabState.showTabLoading === 'function' ? tabState.showTabLoading : () => {};
  const showTabError =
    typeof tabState.showTabError === 'function' ? tabState.showTabError : () => {};
  const showTabEmpty =
    typeof tabState.showTabEmpty === 'function' ? tabState.showTabEmpty : () => {};
  const clearTabStateMessage =
    typeof tabState.clearTabStateMessage === 'function' ? tabState.clearTabStateMessage : () => {};

  const isConfigSnapshotEmpty = (config) =>
    !config || typeof config !== 'object' || Object.keys(config).length === 0;

  function toRequestOptions(runtimeOptions = {}) {
    return runtimeOptions && runtimeOptions.signal ? { signal: runtimeOptions.signal } : {};
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
    if (dashboardState) {
      dashboardState.setSnapshot('config', config);
    }
    await runDomWriteBatch(() => {
      updateConfigModeUi(config, { configSnapshot: config });
      configUiRefreshMethods.forEach((methodName) => invokeConfigUiState(methodName, config));
      invokeConfigUiState('setAdvancedConfigEditorFromConfig', config, true);
      refreshAllDirtySections();
    });
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
      dashboardState.setSnapshot('monitoring', monitoringData);
      if (!isAutoRefresh) {
        dashboardState.setSnapshot('analytics', analytics);
        dashboardState.setSnapshot('events', events);
        dashboardState.setSnapshot('bans', bansData);
        dashboardState.setSnapshot('maze', mazeData);
        dashboardState.setSnapshot('cdp', cdpData);
        dashboardState.setSnapshot('cdpEvents', cdpEventsData);
      }
    }

    if (!isAutoRefresh) {
      await runDomWriteBatch(() => {
        const chartsRuntime = getChartsRuntime();
        if (chartsRuntime) {
          chartsRuntime.updateEventTypesChart(events.event_counts || {});
          chartsRuntime.updateTopIpsChart(events.top_ips || []);
          chartsRuntime.updateTimeSeriesChart();
        }
      });
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
    if (reason !== 'auto-refresh') {
      showTabLoading('ip-bans', 'Loading ban list...');
    }
    const dashboardState = getStateStore();
    const requestOptions = toRequestOptions(runtimeOptions);
    const bansData = await dashboardApiClient.getBans(requestOptions);
    if (dashboardState) {
      dashboardState.setSnapshot('bans', bansData);
    }
    await runDomWriteBatch(() => {
      const tablesView = getTablesView();
      if (tablesView) {
        tablesView.updateBansTable(bansData.bans || []);
      }
    });
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
      refreshCoreActionButtonsState();
      refreshDirtySections(dirtySectionsByTab[activeTab] || []);
    } catch (error) {
      if (error && error.name === 'AbortError') {
        return;
      }
      const message = error && error.message ? error.message : 'Refresh failed';
      console.error(`Dashboard refresh error (${activeTab}):`, error);
      showTabError(activeTab, message);
      const messageNode = getMessageNode();
      if (messageNode) {
        messageNode.textContent = `Refresh failed: ${message}`;
        messageNode.className = 'message error';
      }
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
