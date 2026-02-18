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

  const chartRuntimeSrc = `${base}/assets/vendor/chart-lite-1.0.0.min.js`;

  const dashboardStore = createDashboardStore({ initialTab: 'monitoring' });

  let dashboardState = dashboardStore.getState();
  let runtimeTelemetry = dashboardStore.getRuntimeTelemetry();
  let storeUnsubscribe = () => {};
  let telemetryUnsubscribe = () => {};
  let dashboardActions = null;
  let runtimeReady = false;
  let runtimeError = '';
  let loggingOut = false;

  async function bootstrapNativeRuntime() {
    await mountDashboardRuntime({
      initialTab: normalizeTab(window.location.hash.replace(/^#/, '')),
      chartRuntimeSrc
    });

    const effects = createDashboardEffects();
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

  const isTabActive = (tab) => normalizeTab(dashboardState.activeTab) === normalizeTab(tab);
</script>

<svelte:head>
  <title>Shuma-Gorath Dashboard</title>
</svelte:head>

<span id="last-updated" class="text-muted"></span>
<div class="container panel panel-border" data-dashboard-runtime-mode="native">
  <header>
    <div class="shuma-image-wrapper">
      <img src="/assets/shuma-gorath-pencil.png" alt="Shuma-Gorath" class="shuma-gorath-img">
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
        <a
          id={`dashboard-tab-${tab}`}
          class="dashboard-tab-link"
          class:active={isTabActive(tab)}
          data-dashboard-tab-link={tab}
          href={`#${tab}`}
          role="tab"
          aria-selected={isTabActive(tab) ? 'true' : 'false'}
          aria-controls={`dashboard-panel-${tab}`}
          tabindex={isTabActive(tab) ? 0 : -1}
          on:click={(event) => onTabClick(event, tab)}
          on:keydown={(event) => onTabKeydown(event, tab)}
        >
          {tab === 'ip-bans' ? 'IP Bans' : tab.charAt(0).toUpperCase() + tab.slice(1)}
        </a>
      {/each}
    </nav>
  </header>
  <div id="test-mode-banner" class="test-mode-banner hidden">TEST MODE ACTIVE - Logging only, no blocking</div>

  <MonitoringTab managed={true} isActive={isTabActive('monitoring')} />

  <div
    id="dashboard-admin-section"
    class="section admin-section"
    hidden={isTabActive('monitoring')}
    aria-hidden={isTabActive('monitoring') ? 'true' : 'false'}
  >
    <div class="admin-groups">
      <IpBansTab managed={true} isActive={isTabActive('ip-bans')} />
      <StatusTab managed={true} isActive={isTabActive('status')} runtimeTelemetry={runtimeTelemetry} />
      <ConfigTab managed={true} isActive={isTabActive('config')} />
      <TuningTab managed={true} isActive={isTabActive('tuning')} />
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
