<script>
  import { onDestroy, onMount } from 'svelte';
  import IpBansTab from '$lib/components/dashboard/IpBansTab.svelte';
  import MonitoringTab from '$lib/components/dashboard/MonitoringTab.svelte';
  import StatusTab from '$lib/components/dashboard/StatusTab.svelte';
  import {
    buildDashboardLoginPath,
    dashboardIndexPath,
    normalizeDashboardBasePath,
    resolveDashboardAssetPath
  } from '$lib/runtime/dashboard-paths.js';
  import { createDashboardRouteController } from '$lib/runtime/dashboard-route-controller.js';
  import {
    banDashboardIp,
    getDashboardEvents,
    getDashboardRobotsPreview,
    getDashboardSessionState,
    logoutDashboardSession,
    mountDashboardApp,
    refreshDashboardTab,
    setDashboardActiveTab,
    unbanDashboardIp,
    unmountDashboardApp,
    updateDashboardConfig,
    restoreDashboardSession
  } from '$lib/runtime/dashboard-native-runtime.js';
  import {
    createDashboardStore,
    DASHBOARD_TABS,
    normalizeTab
  } from '$lib/state/dashboard-store.js';

  export let data;
  const TAB_LOADING_MESSAGES = Object.freeze({
    monitoring: 'Loading monitoring data...',
    'ip-bans': 'Loading ban list...',
    status: 'Loading status signals...',
    config: 'Loading config...',
    tuning: 'Loading tuning values...'
  });
  const AUTO_REFRESH_INTERVAL_MS = 60000;
  const AUTO_REFRESH_TABS = new Set(['monitoring', 'ip-bans']);
  const AUTO_REFRESH_PREF_KEY = 'shuma_dashboard_auto_refresh_v1';

  const fallbackBasePath = normalizeDashboardBasePath();
  const dashboardBasePath = typeof data?.dashboardBasePath === 'string'
    ? data.dashboardBasePath
    : fallbackBasePath;
  const chartRuntimeSrc = typeof data?.chartRuntimeSrc === 'string'
    ? data.chartRuntimeSrc
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/vendor/chart-lite-1.0.0.min.js');
  const shumaImageSrc = typeof data?.shumaImageSrc === 'string'
    ? data.shumaImageSrc
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/shuma-gorath-pencil.png');

  const dashboardStore = createDashboardStore({ initialTab: 'monitoring' });

  let dashboardState = dashboardStore.getState();
  let runtimeTelemetry = dashboardStore.getRuntimeTelemetry();
  let storeUnsubscribe = () => {};
  let telemetryUnsubscribe = () => {};
  let runtimeReady = false;
  let runtimeError = '';
  let loggingOut = false;
  let autoRefreshEnabled = false;
  let adminMessageText = '';
  let adminMessageKind = 'info';
  let ConfigTabComponent = null;
  let TuningTabComponent = null;
  const tabLinks = {};

  $: activeTabKey = normalizeTab(dashboardState.activeTab);
  $: tabStatus = dashboardState?.tabStatus || {};
  $: activeTabStatus = tabStatus[activeTabKey] || {};
  $: autoRefreshSupported = AUTO_REFRESH_TABS.has(activeTabKey);
  $: refreshNowDisabled =
    !runtimeReady || activeTabStatus.loading === true || autoRefreshSupported !== true;
  $: refreshModeText = autoRefreshSupported
    ? (autoRefreshEnabled
      ? `Auto refresh ON (${Math.floor(AUTO_REFRESH_INTERVAL_MS / 1000)}s cadence)`
      : 'Auto refresh OFF (manual)')
    : 'Manual updates only on this tab';
  $: lastUpdatedText = activeTabStatus.updatedAt
    ? `Last updated: ${new Date(activeTabStatus.updatedAt).toLocaleString()}`
    : 'Last updated: not updated yet';
  $: snapshots = dashboardState?.snapshots || {};
  $: snapshotVersions = dashboardState?.snapshotVersions || {};
  $: analyticsSnapshot = snapshots.analytics || {};
  $: configSnapshot = snapshots.config || {};
  $: testModeEnabled = configSnapshot.test_mode === true || analyticsSnapshot.test_mode === true;

  function registerTabLink(node, tab) {
    let key = normalizeTab(tab);
    tabLinks[key] = node;
    return {
      update(nextTab) {
        delete tabLinks[key];
        key = normalizeTab(nextTab);
        tabLinks[key] = node;
      },
      destroy() {
        delete tabLinks[key];
      }
    };
  }

  function focusTab(tab) {
    const node = tabLinks[normalizeTab(tab)];
    if (node && typeof node.focus === 'function') {
      node.focus();
      return true;
    }
    return false;
  }

  function readAutoRefreshPreference() {
    if (typeof window === 'undefined') return false;
    try {
      return window.localStorage.getItem(AUTO_REFRESH_PREF_KEY) === '1';
    } catch (_error) {
      return false;
    }
  }

  function writeAutoRefreshPreference(nextEnabled) {
    if (typeof window === 'undefined') return;
    try {
      window.localStorage.setItem(AUTO_REFRESH_PREF_KEY, nextEnabled ? '1' : '0');
    } catch (_error) {}
  }

  function resolveLoginRedirectPath() {
    if (typeof window === 'undefined') {
      return buildDashboardLoginPath({ basePath: dashboardBasePath });
    }
    const pathname = String(window.location?.pathname || dashboardIndexPath(dashboardBasePath));
    const search = String(window.location?.search || '');
    const hash = String(window.location?.hash || '');
    return buildDashboardLoginPath({
      basePath: dashboardBasePath,
      nextPath: `${pathname}${search}${hash}`
    });
  }

  function redirectToLogin() {
    if (typeof window === 'undefined') return;
    window.location.replace(resolveLoginRedirectPath());
  }

  const routeController = createDashboardRouteController({
    tabs: DASHBOARD_TABS,
    normalizeTab,
    tabLoadingMessages: TAB_LOADING_MESSAGES,
    store: dashboardStore,
    mountDashboardApp,
    restoreDashboardSession,
    getDashboardSessionState,
    setDashboardActiveTab,
    refreshDashboardTab,
    selectRefreshInterval: (tab) =>
      AUTO_REFRESH_TABS.has(normalizeTab(tab)) ? AUTO_REFRESH_INTERVAL_MS : 0,
    setPollingContext: (tab, intervalMs) => dashboardStore.setPollingContext(tab, intervalMs),
    recordPollingSkip: (reason, tab, intervalMs) =>
      dashboardStore.recordPollingSkip(reason, tab, intervalMs),
    recordPollingResume: (reason, tab, intervalMs) =>
      dashboardStore.recordPollingResume(reason, tab, intervalMs),
    recordRefreshMetrics: (metrics) => dashboardStore.recordRefreshMetrics(metrics),
    isAuthenticated: () => dashboardStore.getState().session.authenticated === true,
    isAutoRefreshEnabled: () => autoRefreshEnabled === true,
    isAutoRefreshTab: (tab) => AUTO_REFRESH_TABS.has(normalizeTab(tab)),
    shouldRefreshOnActivate: ({ tab, store }) => {
      const normalized = normalizeTab(tab);
      if (AUTO_REFRESH_TABS.has(normalized)) return true;
      const state = store && typeof store.getState === 'function' ? store.getState() : null;
      const configSnapshot = state && state.snapshots ? state.snapshots.config : null;
      return !configSnapshot || Object.keys(configSnapshot).length === 0;
    },
    redirectToLogin
  });

  onMount(async () => {
    autoRefreshEnabled = readAutoRefreshPreference();
    routeController.setMounted(true);
    storeUnsubscribe = dashboardStore.subscribe((value) => {
      dashboardState = value;
    });
    telemetryUnsubscribe = dashboardStore.runtimeTelemetryStore.subscribe((value) => {
      runtimeTelemetry = value;
    });

    try {
      const [{ default: loadedConfigTab }, { default: loadedTuningTab }] = await Promise.all([
        import('$lib/components/dashboard/ConfigTab.svelte'),
        import('$lib/components/dashboard/TuningTab.svelte')
      ]);
      ConfigTabComponent = loadedConfigTab;
      TuningTabComponent = loadedTuningTab;

      const bootstrapped = await routeController.bootstrapRuntime({
        initialTab: normalizeTab(data?.initialHashTab || ''),
        chartRuntimeSrc,
        basePath: dashboardBasePath
      });
      runtimeReady = bootstrapped === true;
    } catch (error) {
      runtimeError = error && error.message ? error.message : 'Dashboard bootstrap failed.';
    }
  });

  onDestroy(() => {
    const runtimeWasMounted = routeController.getRuntimeMounted();
    routeController.dispose();
    storeUnsubscribe();
    telemetryUnsubscribe();
    if (runtimeWasMounted) {
      unmountDashboardApp();
    }
  });

  function onTabClick(event, tab) {
    event.preventDefault();
    void routeController.applyActiveTab(tab, { reason: 'click', syncHash: true });
  }

  function onTabKeydown(event, tab) {
    const target = routeController.keyNavTarget(tab, event.key);
    if (!target) return;
    event.preventDefault();
    void routeController.applyActiveTab(target, { reason: 'keyboard', syncHash: true });
    setTimeout(() => {
      focusTab(target);
    }, 0);
  }

  function onWindowHashChange() {
    if (!routeController.getRuntimeMounted()) return;
    routeController.syncFromHash('hashchange');
  }

  function onDocumentVisibilityChange() {
    routeController.handleVisibilityChange();
  }

  function setAdminMessage(text = '', kind = 'info') {
    adminMessageText = String(text || '');
    adminMessageKind = String(kind || 'info');
  }

  function onAutoRefreshToggle(event) {
    const checked = event && event.currentTarget && event.currentTarget.checked === true;
    autoRefreshEnabled = checked;
    writeAutoRefreshPreference(checked);
    routeController.schedulePolling('auto-refresh-toggle');
    if (checked && autoRefreshSupported && runtimeReady) {
      void routeController.refreshTab(activeTabKey, 'manual-refresh');
    }
  }

  async function onRefreshNow(event) {
    if (event && typeof event.preventDefault === 'function') {
      event.preventDefault();
    }
    if (refreshNowDisabled || !autoRefreshSupported) return;
    await routeController.refreshTab(activeTabKey, 'manual-refresh');
  }

  function formatActionError(error, fallback = 'Action failed.') {
    if (error && typeof error.message === 'string' && error.message.trim()) {
      return error.message.trim();
    }
    return fallback;
  }

  async function onSaveConfig(patch, options = {}) {
    const successMessage = options && typeof options.successMessage === 'string'
      ? options.successMessage
      : 'Configuration saved';
    setAdminMessage('Saving configuration...', 'info');
    try {
      const nextConfig = await updateDashboardConfig(patch || {});
      await routeController.refreshTab(activeTabKey, 'config-save');
      setAdminMessage(successMessage, 'success');
      return nextConfig;
    } catch (error) {
      const message = formatActionError(error, 'Failed to save configuration.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onBan(payload = {}) {
    const ip = String(payload.ip || '').trim();
    const duration = Number(payload.duration || 0);
    if (!ip || !Number.isFinite(duration) || duration <= 0) return;
    setAdminMessage(`Banning ${ip}...`, 'info');
    try {
      await banDashboardIp(ip, duration, 'manual_ban');
      await routeController.refreshTab('ip-bans', 'ban-save');
      setAdminMessage(`Banned ${ip} for ${duration}s`, 'success');
    } catch (error) {
      const message = formatActionError(error, 'Failed to ban IP.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onUnban(payload = {}) {
    const ip = String(payload.ip || '').trim();
    if (!ip) return;
    setAdminMessage(`Unbanning ${ip}...`, 'info');
    try {
      await unbanDashboardIp(ip);
      await routeController.refreshTab('ip-bans', 'unban-save');
      setAdminMessage(`Unbanned ${ip}`, 'success');
    } catch (error) {
      const message = formatActionError(error, 'Failed to unban IP.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onRobotsPreview() {
    return getDashboardRobotsPreview();
  }

  async function onFetchEventsRange(hours, options = {}) {
    return getDashboardEvents(hours, options || {});
  }

  async function onLogoutClick(event) {
    if (!routeController.getRuntimeMounted()) return;
    event.preventDefault();
    if (loggingOut) return;
    loggingOut = true;
    try {
      routeController.abortInFlightRefresh();
      await logoutDashboardSession();
      dashboardStore.setSession({ authenticated: false, csrfToken: '' });
      routeController.clearPolling();
      redirectToLogin();
    } finally {
      loggingOut = false;
    }
  }
</script>
<svelte:head>
  <title>Shuma-Gorath Dashboard</title>
</svelte:head>
<svelte:window on:hashchange={onWindowHashChange} />
<svelte:document on:visibilitychange={onDocumentVisibilityChange} />
<div class="container panel panel-border" data-dashboard-runtime-mode="native">
  <div id="test-mode-banner" class="test-mode-banner" class:hidden={!testModeEnabled}>
    TEST MODE ACTIVE - Logging only, no blocking
  </div>
  <button
    id="logout-btn"
    class="btn btn-subtle dashboard-logout"
    aria-label="Log out of admin session"
    disabled={loggingOut || dashboardState.session.authenticated !== true}
    on:click={onLogoutClick}
  >Logout</button>
  <header>
    <div class="shuma-image-wrapper">
      <img src={shumaImageSrc} alt="Shuma-Gorath" class="shuma-gorath-img">
    </div>
    <h1>Shuma-Gorath</h1>
    <p class="subtitle text-muted">Multi-Dimensional Bot Defence</p>
    <nav class="dashboard-tabs" aria-label="Dashboard sections">
      {#each DASHBOARD_TABS as tab}
        {@const tabKey = normalizeTab(tab)}
        {@const selected = activeTabKey === tabKey}
        <a
          id={`dashboard-tab-${tab}`}
          class="dashboard-tab-link"
          class:active={selected}
          data-dashboard-tab-link={tab}
          href={`#${tab}`}
          role="tab"
          aria-selected={selected ? 'true' : 'false'}
          aria-controls={`dashboard-panel-${tab}`}
          tabindex={selected ? 0 : -1}
          on:click={(event) => onTabClick(event, tab)}
          on:keydown={(event) => onTabKeydown(event, tab)}
          use:registerTabLink={tab}
        >
          {tab === 'ip-bans' ? 'IP Bans' : tab.charAt(0).toUpperCase() + tab.slice(1)}
        </a>
      {/each}
    </nav>
    {#if autoRefreshSupported}
      <div id="dashboard-refresh-controls" class="dashboard-refresh-controls">
        <div class="dashboard-refresh-meta">
          <span id="last-updated" class="text-muted">{lastUpdatedText}</span>
          {#if !autoRefreshEnabled}
            <button
              id="refresh-now-btn"
              class="btn btn-subtle"
              aria-label="Refresh now"
              title="Refresh now"
              disabled={refreshNowDisabled}
              on:click={onRefreshNow}
            >↻</button>
          {/if}
        </div>
        <div class="dashboard-refresh-auto">
          <span id="refresh-mode" class="text-muted">{refreshModeText}</span>
          <div class="toggle-row dashboard-refresh-toggle">
            <label class="toggle-switch" for="auto-refresh-toggle">
              <input
                id="auto-refresh-toggle"
                type="checkbox"
                aria-label="Enable automatic refresh for current tab"
                checked={autoRefreshEnabled}
                on:change={onAutoRefreshToggle}
              >
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      </div>
    {/if}
  </header>

  <MonitoringTab
    managed={true}
    isActive={activeTabKey === 'monitoring'}
    autoRefreshEnabled={autoRefreshEnabled}
    tabStatus={tabStatus.monitoring || {}}
    analyticsSnapshot={snapshots.analytics}
    eventsSnapshot={snapshots.events}
    bansSnapshot={snapshots.bans}
    mazeSnapshot={snapshots.maze}
    cdpSnapshot={snapshots.cdp}
    cdpEventsSnapshot={snapshots.cdpEvents}
    monitoringSnapshot={snapshots.monitoring}
    onFetchEventsRange={onFetchEventsRange}
  />

  <div
    id="dashboard-admin-section"
    class="section admin-section"
    hidden={activeTabKey === 'monitoring'}
    aria-hidden={activeTabKey === 'monitoring' ? 'true' : 'false'}
  >
    <div class="admin-groups">
      <IpBansTab
        managed={true}
        isActive={activeTabKey === 'ip-bans'}
        tabStatus={tabStatus['ip-bans'] || {}}
        bansSnapshot={snapshots.bans}
        configSnapshot={snapshots.config}
        onBan={onBan}
        onUnban={onUnban}
      />
      <StatusTab
        managed={true}
        isActive={activeTabKey === 'status'}
        runtimeTelemetry={runtimeTelemetry}
        tabStatus={tabStatus.status || {}}
        configSnapshot={snapshots.config}
      />
      {#if ConfigTabComponent}
        <svelte:component
          this={ConfigTabComponent}
          managed={true}
          isActive={activeTabKey === 'config'}
          tabStatus={tabStatus.config || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
          onFetchRobotsPreview={onRobotsPreview}
        />
      {:else}
        <section
          id="dashboard-panel-config"
          class="admin-group"
          data-dashboard-tab-panel="config"
          aria-labelledby="dashboard-tab-config"
          hidden={activeTabKey !== 'config'}
          aria-hidden={activeTabKey === 'config' ? 'false' : 'true'}
        >
          <p class="message info">Loading config controls...</p>
        </section>
      {/if}
      {#if TuningTabComponent}
        <svelte:component
          this={TuningTabComponent}
          managed={true}
          isActive={activeTabKey === 'tuning'}
          tabStatus={tabStatus.tuning || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-tuning"
          class="admin-group"
          data-dashboard-tab-panel="tuning"
          aria-labelledby="dashboard-tab-tuning"
          hidden={activeTabKey !== 'tuning'}
          aria-hidden={activeTabKey === 'tuning' ? 'false' : 'true'}
        >
          <p class="message info">Loading tuning controls...</p>
        </section>
      {/if}
    </div>
    <div id="admin-msg" class={`message ${adminMessageKind}`}>{adminMessageText}</div>
  </div>

  {#if runtimeError}
    <p class="message error">{runtimeError}</p>
  {/if}
  {#if !runtimeReady && !runtimeError}
    <p class="message info">Loading dashboard runtime...</p>
  {/if}
</div>
