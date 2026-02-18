<script>
  import { base } from '$app/paths';
  import { onDestroy, onMount } from 'svelte';
  import ConfigTab from '$lib/components/dashboard/ConfigTab.svelte';
  import IpBansTab from '$lib/components/dashboard/IpBansTab.svelte';
  import MonitoringTab from '$lib/components/dashboard/MonitoringTab.svelte';
  import StatusTab from '$lib/components/dashboard/StatusTab.svelte';
  import TuningTab from '$lib/components/dashboard/TuningTab.svelte';
  import { createDashboardActions } from '$lib/runtime/dashboard-actions.js';
  import { createDashboardEffects } from '$lib/runtime/dashboard-effects.js';
  import {
    normalizeDashboardBasePath,
    resolveDashboardAssetPath
  } from '$lib/runtime/dashboard-paths.js';
  import {
    getDashboardSessionState,
    logoutDashboardSession,
    mountDashboardRuntime,
    refreshDashboardTab,
    restoreDashboardSession,
    setDashboardActiveTab,
    unmountDashboardRuntime
  } from '$lib/runtime/dashboard-runtime.js';
  import {
    createDashboardStore,
    DASHBOARD_TABS,
    normalizeTab
  } from '$lib/state/dashboard-store.js';

  const dashboardBasePath = normalizeDashboardBasePath(base);
  const chartRuntimeSrc = resolveDashboardAssetPath(dashboardBasePath, 'assets/vendor/chart-lite-1.0.0.min.js');
  const shumaImageSrc = resolveDashboardAssetPath(dashboardBasePath, 'assets/shuma-gorath-pencil.png');

  const dashboardStore = createDashboardStore({ initialTab: 'monitoring' });

  let dashboardState = dashboardStore.getState();
  let runtimeTelemetry = dashboardStore.getRuntimeTelemetry();
  let storeUnsubscribe = () => {};
  let telemetryUnsubscribe = () => {};
  let dashboardActions = null;
  let runtimeReady = false;
  let runtimeError = '';
  let loggingOut = false;

  $: activeTabKey = normalizeTab(dashboardState.activeTab);
  $: tabStatus = dashboardState?.tabStatus || {};
  $: activeTabStatus = tabStatus[activeTabKey] || {};
  $: lastUpdatedText = activeTabStatus.updatedAt ? `updated: ${activeTabStatus.updatedAt}` : '';
  $: snapshots = dashboardState?.snapshots || {};
  $: analyticsSnapshot = snapshots.analytics || {};
  $: testModeEnabled = analyticsSnapshot.test_mode === true;

  async function bootstrapNativeRuntime() {
    await mountDashboardRuntime({
      initialTab: normalizeTab(window.location.hash.replace(/^#/, '')),
      chartRuntimeSrc,
      basePath: dashboardBasePath,
      store: dashboardStore
    });

    const effects = createDashboardEffects({ basePath: dashboardBasePath });
    dashboardActions = createDashboardActions({
      store: dashboardStore,
      effects,
      runtime: {
        refreshTab: refreshDashboardTab,
        setActiveTab: setDashboardActiveTab,
        restoreSession: restoreDashboardSession,
        getSessionState: getDashboardSessionState,
        logout: logoutDashboardSession
      }
    });

    dashboardActions.init();
    const authenticated = await dashboardActions.bootstrapSession();
    if (!authenticated) return;
    runtimeReady = true;
  }

  onMount(async () => {
    storeUnsubscribe = dashboardStore.subscribe((value) => {
      dashboardState = value;
    });
    telemetryUnsubscribe = dashboardStore.runtimeTelemetryStore.subscribe((value) => {
      runtimeTelemetry = value;
    });

    try {
      await bootstrapNativeRuntime();
    } catch (error) {
      runtimeError = error && error.message ? error.message : 'Dashboard bootstrap failed.';
    }
  });

  onDestroy(() => {
    if (dashboardActions) {
      dashboardActions.destroy();
      dashboardActions = null;
    }
    storeUnsubscribe();
    telemetryUnsubscribe();
    unmountDashboardRuntime();
  });

  function onTabClick(event, tab) {
    if (!dashboardActions) return;
    dashboardActions.onTabClick(event, tab);
  }

  function onTabKeydown(event, tab) {
    if (!dashboardActions) return;
    dashboardActions.onTabKeydown(event, tab);
  }

  async function onLogoutClick(event) {
    if (!dashboardActions) return;
    event.preventDefault();
    if (loggingOut) return;
    loggingOut = true;
    try {
      await dashboardActions.logout();
    } finally {
      loggingOut = false;
    }
  }

</script>

<svelte:head>
  <title>Shuma-Gorath Dashboard</title>
</svelte:head>

<span id="last-updated" class="text-muted">{lastUpdatedText}</span>
<div class="container panel panel-border" data-dashboard-runtime-mode="native">
  <header>
    <div class="shuma-image-wrapper">
      <img src={shumaImageSrc} alt="Shuma-Gorath" class="shuma-gorath-img">
    </div>
    <h1>Shuma-Gorath</h1>
    <p class="subtitle text-muted">Multi-Dimensional Bot Defence</p>
    <button
      id="logout-btn"
      class="btn btn-subtle dashboard-logout"
      aria-label="Log out of admin session"
      disabled={loggingOut || dashboardState.session.authenticated !== true}
      on:click={onLogoutClick}
    >Logout</button>
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
        >
          {tab === 'ip-bans' ? 'IP Bans' : tab.charAt(0).toUpperCase() + tab.slice(1)}
        </a>
      {/each}
    </nav>
  </header>
  <div id="test-mode-banner" class="test-mode-banner" class:hidden={!testModeEnabled}>
    TEST MODE ACTIVE - Logging only, no blocking
  </div>

  <MonitoringTab
    managed={true}
    isActive={activeTabKey === 'monitoring'}
    tabStatus={tabStatus.monitoring || {}}
    analyticsSnapshot={snapshots.analytics}
    eventsSnapshot={snapshots.events}
    bansSnapshot={snapshots.bans}
    mazeSnapshot={snapshots.maze}
    cdpSnapshot={snapshots.cdp}
    cdpEventsSnapshot={snapshots.cdpEvents}
    monitoringSnapshot={snapshots.monitoring}
  />

  <div
    id="dashboard-admin-section"
    class="section admin-section"
    hidden={activeTabKey === 'monitoring'}
    aria-hidden={activeTabKey === 'monitoring' ? 'true' : 'false'}
  >
    <div class="admin-groups">
      <IpBansTab managed={true} isActive={activeTabKey === 'ip-bans'} tabStatus={tabStatus['ip-bans'] || {}} />
      <StatusTab
        managed={true}
        isActive={activeTabKey === 'status'}
        runtimeTelemetry={runtimeTelemetry}
        tabStatus={tabStatus.status || {}}
      />
      <ConfigTab
        managed={true}
        isActive={activeTabKey === 'config'}
        tabStatus={tabStatus.config || {}}
        analyticsSnapshot={snapshots.analytics}
      />
      <TuningTab managed={true} isActive={activeTabKey === 'tuning'} tabStatus={tabStatus.tuning || {}} />
    </div>
    <div id="admin-msg" class="message"></div>
  </div>

  {#if runtimeError}
    <p class="message error">{runtimeError}</p>
  {/if}
  {#if !runtimeReady && !runtimeError}
    <p class="message info">Loading dashboard runtime...</p>
  {/if}
</div>
