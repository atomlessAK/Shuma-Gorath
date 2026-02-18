<script>
  import { browser } from '$app/environment';
  import { onDestroy } from 'svelte';
  import {
    CHALLENGE_REASON_LABELS,
    POW_REASON_LABELS,
    RATE_OUTCOME_LABELS,
    deriveMazeStatsViewModel,
    deriveMonitoringSummaryViewModel,
    derivePrometheusHelperViewModel,
    formatMetricLabel
  } from './monitoring-view-model.js';
  import StatCard from './primitives/StatCard.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TableWrapper from './primitives/TableWrapper.svelte';

  const EVENT_ROW_RENDER_LIMIT = 100;
  const CDP_ROW_RENDER_LIMIT = 500;
  const CHALLENGE_TREND_COLOR = 'rgba(122, 114, 255, 0.35)';
  const POW_TREND_COLOR = 'rgba(255, 130, 92, 0.35)';

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
  export let analyticsSnapshot = null;
  export let eventsSnapshot = null;
  export let bansSnapshot = null;
  export let mazeSnapshot = null;
  export let cdpSnapshot = null;
  export let cdpEventsSnapshot = null;
  export let monitoringSnapshot = null;

  let challengeTrendCanvas = null;
  let powTrendCanvas = null;
  let challengeTrendChart = null;
  let powTrendChart = null;

  let copyButtonLabel = 'Copy JS Example';
  let copyCurlButtonLabel = 'Copy Curl Example';
  let copyButtonTimer = null;
  let copyCurlButtonTimer = null;

  const defaultMonitoringSummary = deriveMonitoringSummaryViewModel({});
  const defaultMazeStats = deriveMazeStatsViewModel({});
  const defaultPrometheusHelper = derivePrometheusHelperViewModel({}, '');

  const clearTimer = (timerId) => {
    if (timerId === null) return null;
    clearTimeout(timerId);
    return null;
  };

  const scheduleCopyLabelReset = (kind) => {
    if (kind === 'js') {
      copyButtonTimer = clearTimer(copyButtonTimer);
      copyButtonTimer = setTimeout(() => {
        copyButtonLabel = 'Copy JS Example';
      }, 1200);
      return;
    }
    copyCurlButtonTimer = clearTimer(copyCurlButtonTimer);
    copyCurlButtonTimer = setTimeout(() => {
      copyCurlButtonLabel = 'Copy Curl Example';
    }, 1200);
  };

  const copyToClipboard = async (text, kind) => {
    if (!browser) return;
    const value = String(text || '');
    try {
      await navigator.clipboard.writeText(value);
      if (kind === 'js') {
        copyButtonLabel = 'Copied';
      } else {
        copyCurlButtonLabel = 'Copied';
      }
    } catch (_error) {
      if (kind === 'js') {
        copyButtonLabel = 'Copy Failed';
      } else {
        copyCurlButtonLabel = 'Copy Failed';
      }
    }
    scheduleCopyLabelReset(kind);
  };

  const eventBadgeClass = (eventType) => {
    const normalized = String(eventType || '').toLowerCase().replace(/[^a-z_]/g, '');
    return normalized ? `badge ${normalized}` : 'badge';
  };

  const formatTime = (rawTs) => {
    const ts = Number(rawTs || 0);
    if (!Number.isFinite(ts) || ts <= 0) return '-';
    return new Date(ts * 1000).toLocaleString();
  };

  const readCdpField = (text, key) => {
    const match = new RegExp(`${key}=([^\\s]+)`, 'i').exec(String(text || ''));
    return match ? match[1] : '-';
  };

  const normalizeReasonRows = (rows, labels) => {
    if (!Array.isArray(rows)) return [];
    return rows.map(([key, value]) => ({
      key,
      label: formatMetricLabel(key, labels),
      count: Number(value || 0)
    }));
  };

  const normalizePairRows = (rows, labels) => {
    if (!Array.isArray(rows)) return [];
    return rows.map(([key, value]) => ({
      key,
      label: formatMetricLabel(key, labels),
      count: Number(value || 0)
    }));
  };

  const normalizeTopPaths = (paths) => {
    if (!Array.isArray(paths)) return [];
    return paths.map((entry) => ({
      path: String(entry.path || '-'),
      count: Number(entry.count || 0)
    }));
  };

  const normalizeTopCountries = (rows) => {
    if (!Array.isArray(rows)) return [];
    return rows.map((entry) => ({
      country: String(entry.country || '-'),
      count: Number(entry.count || 0)
    }));
  };

  const getChartConstructor = () => {
    if (!browser || !window || typeof window.Chart !== 'function') return null;
    return window.Chart;
  };

  const updateTrendChart = (chart, canvas, title, color, trendSeries) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;

    if (!chart) {
      return new chartCtor(ctx, {
        type: 'line',
        data: {
          labels: trendSeries.labels,
          datasets: [{
            label: title,
            data: trendSeries.data,
            backgroundColor: color,
            fill: true,
            tension: 0.35,
            pointRadius: 0,
            pointHoverRadius: 0,
            borderWidth: 0
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          plugins: { legend: { display: false } },
          scales: {
            y: {
              beginAtZero: true,
              ticks: { stepSize: 1 }
            }
          }
        }
      });
    }

    chart.data.labels = trendSeries.labels;
    chart.data.datasets[0].data = trendSeries.data;
    chart.update();
    return chart;
  };

  $: analytics = analyticsSnapshot && typeof analyticsSnapshot === 'object' ? analyticsSnapshot : {};
  $: events = eventsSnapshot && typeof eventsSnapshot === 'object' ? eventsSnapshot : {};
  $: bans = Array.isArray(bansSnapshot?.bans) ? bansSnapshot.bans : [];
  $: maze = mazeSnapshot && typeof mazeSnapshot === 'object' ? mazeSnapshot : {};
  $: cdp = cdpSnapshot && typeof cdpSnapshot === 'object' ? cdpSnapshot : {};
  $: cdpEventsData = cdpEventsSnapshot && typeof cdpEventsSnapshot === 'object'
    ? cdpEventsSnapshot
    : {};
  $: monitoring = monitoringSnapshot && typeof monitoringSnapshot === 'object'
    ? monitoringSnapshot
    : {};

  $: recentEvents = Array.isArray(events.recent_events)
    ? events.recent_events.slice(0, EVENT_ROW_RENDER_LIMIT)
    : [];
  $: recentCdpEvents = Array.isArray(cdpEventsData.events)
    ? cdpEventsData.events.slice(0, CDP_ROW_RENDER_LIMIT)
    : [];

  $: eventCount = recentEvents.length;
  $: totalBans = Number.isFinite(Number(analytics.ban_count))
    ? Number(analytics.ban_count)
    : bans.length;
  $: activeBans = bans.length;
  $: uniqueIps = Number.isFinite(Number(events.unique_ips))
    ? Number(events.unique_ips)
    : (Array.isArray(events.top_ips) ? events.top_ips.length : 0);
  $: testModeEnabled = analytics.test_mode === true;

  $: cdpDetections = Number(cdp?.stats?.total_detections || 0);
  $: cdpAutoBans = Number(cdp?.stats?.auto_bans || 0);
  $: cdpFingerprintEvents =
    Number(cdp?.fingerprint_stats?.ua_client_hint_mismatch || 0) +
    Number(cdp?.fingerprint_stats?.ua_transport_mismatch || 0) +
    Number(cdp?.fingerprint_stats?.temporal_transition || 0);
  $: cdpFingerprintFlowViolations = Number(cdp?.fingerprint_stats?.flow_violation || 0);

  $: mazeStats = deriveMazeStatsViewModel(maze || {}) || defaultMazeStats;
  $: monitoringSummary =
    deriveMonitoringSummaryViewModel(monitoring.summary || {}) || defaultMonitoringSummary;
  $: prometheusHelper = derivePrometheusHelperViewModel(
    monitoring.prometheus || {},
    browser && window?.location?.origin ? window.location.origin : ''
  ) || defaultPrometheusHelper;

  $: honeypotTopPaths = normalizeTopPaths(monitoringSummary.honeypot.topPaths);
  $: challengeReasonRows = normalizeReasonRows(
    monitoringSummary.challenge.reasons,
    CHALLENGE_REASON_LABELS
  );
  $: powReasonRows = normalizeReasonRows(monitoringSummary.pow.reasons, POW_REASON_LABELS);
  $: rateOutcomeRows = normalizePairRows(monitoringSummary.rate.outcomes, RATE_OUTCOME_LABELS);
  $: geoTopCountries = normalizeTopCountries(monitoringSummary.geo.topCountries);

  $: challengeTrendSeries = monitoringSummary.challenge.trend || { labels: [], data: [] };
  $: powTrendSeries = monitoringSummary.pow.trend || { labels: [], data: [] };

  $: if (browser && challengeTrendCanvas) {
    challengeTrendChart = updateTrendChart(
      challengeTrendChart,
      challengeTrendCanvas,
      'Challenge Failures',
      CHALLENGE_TREND_COLOR,
      challengeTrendSeries
    );
  }

  $: if (browser && powTrendCanvas) {
    powTrendChart = updateTrendChart(
      powTrendChart,
      powTrendCanvas,
      'PoW Failures',
      POW_TREND_COLOR,
      powTrendSeries
    );
  }

  onDestroy(() => {
    copyButtonTimer = clearTimer(copyButtonTimer);
    copyCurlButtonTimer = clearTimer(copyCurlButtonTimer);
    if (challengeTrendChart && typeof challengeTrendChart.destroy === 'function') {
      challengeTrendChart.destroy();
    }
    if (powTrendChart && typeof powTrendChart.destroy === 'function') {
      powTrendChart.destroy();
    }
    challengeTrendChart = null;
    powTrendChart = null;
  });
</script>

<section
  id="dashboard-panel-monitoring"
  class="dashboard-tab-panel"
  data-dashboard-tab-panel="monitoring"
  aria-labelledby="dashboard-tab-monitoring"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'false'}
  tabindex="-1"
>
  <TabStateMessage tab="monitoring" status={tabStatus} />

  <div class="stats-cards">
    <StatCard title="Total Bans">
      <div class="stat-value" id="total-bans">{tabStatus?.loading ? '...' : totalBans.toLocaleString()}</div>
    </StatCard>
    <StatCard title="Active Bans">
      <div class="stat-value" id="active-bans">{tabStatus?.loading ? '...' : activeBans.toLocaleString()}</div>
    </StatCard>
    <StatCard title="Events (24h)">
      <div class="stat-value" id="total-events">{tabStatus?.loading ? '...' : eventCount.toLocaleString()}</div>
    </StatCard>
    <StatCard title="Unique IPs">
      <div class="stat-value" id="unique-ips">{tabStatus?.loading ? '...' : uniqueIps.toLocaleString()}</div>
    </StatCard>
  </div>

  <div class="charts-row">
    <div class="chart-container panel-soft panel-border pad-md">
      <h2>Event Types (24h)</h2>
      <canvas id="eventTypesChart"></canvas>
    </div>
    <div class="chart-container panel-soft panel-border pad-md">
      <h2>Top 10 IPs by Events</h2>
      <canvas id="topIpsChart"></canvas>
    </div>
  </div>

  <div class="section">
    <h2>Events Over Time</h2>
    <p class="section-desc text-muted">Recent events plotted over various time windows</p>
    <div class="chart-header">
      <div class="time-range-buttons">
        <button class="btn time-btn active" data-range="hour">60 Mins</button>
        <button class="btn time-btn" data-range="day">24 Hours</button>
        <button class="btn time-btn" data-range="week">7 Days</button>
        <button class="btn time-btn" data-range="month">30 Days</button>
      </div>
    </div>
    <div class="chart-container panel-soft panel-border pad-md">
      <canvas id="timeSeriesChart"></canvas>
    </div>
  </div>

  <div class="section events">
    <h2>Recent Events</h2>
    <p class="section-desc text-muted">Last 100 recorded events</p>
    <TableWrapper>
      <table id="events" class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Time</th>
            <th class="caps-label">Type</th>
            <th class="caps-label">IP</th>
            <th class="caps-label">Reason</th>
            <th class="caps-label">Outcome</th>
            <th class="caps-label">Admin</th>
          </tr>
        </thead>
        <tbody>
          {#if recentEvents.length === 0}
            <tr><td colspan="6" style="text-align: center; color: #6b7280;">No recent events</td></tr>
          {:else}
            {#each recentEvents as ev}
              <tr>
                <td>{formatTime(ev.ts)}</td>
                <td><span class={eventBadgeClass(ev.event)}>{ev.event || '-'}</span></td>
                <td><code>{ev.ip || '-'}</code></td>
                <td>{ev.reason || '-'}</td>
                <td>{ev.outcome || '-'}</td>
                <td>{ev.admin || '-'}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </TableWrapper>
  </div>

  <div class="section events">
    <h2>CDP Detections</h2>
    <p class="section-desc text-muted">Browser automation detection and bans in the last 24hrs</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Detections</h3>
        <div class="stat-value stat-value" id="cdp-total-detections">{tabStatus?.loading ? '...' : cdpDetections.toLocaleString()}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Auto-Bans</h3>
        <div class="stat-value stat-value" id="cdp-total-auto-bans">{tabStatus?.loading ? '...' : cdpAutoBans.toLocaleString()}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">FP Mismatch Events</h3>
        <div class="stat-value stat-value" id="cdp-fp-events">{tabStatus?.loading ? '...' : cdpFingerprintEvents.toLocaleString()}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">FP Flow Violations</h3>
        <div class="stat-value stat-value" id="cdp-fp-flow-violations">{tabStatus?.loading ? '...' : cdpFingerprintFlowViolations.toLocaleString()}</div>
      </div>
    </div>
    <div class="table-wrapper">
      <table id="cdp-events" class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Time</th>
            <th class="caps-label">IP</th>
            <th class="caps-label">Type</th>
            <th class="caps-label">Tier</th>
            <th class="caps-label">Score</th>
            <th class="caps-label">Details</th>
          </tr>
        </thead>
        <tbody>
          {#if recentCdpEvents.length === 0}
            <tr><td colspan="6" style="text-align: center; color: #6b7280;">No CDP detections or auto-bans in the selected window</td></tr>
          {:else}
            {#each recentCdpEvents as ev}
              {@const reason = String(ev.reason || '')}
              {@const outcome = String(ev.outcome || '-')}
              {@const isBan = reason.toLowerCase() === 'cdp_automation'}
              {@const tierSource = isBan ? outcome : reason}
              {@const tier = readCdpField(tierSource, 'tier').toUpperCase()}
              {@const score = readCdpField(tierSource, 'score')}
              {@const details = isBan
                ? `Auto-ban: ${outcome}`
                : (outcome.toLowerCase().startsWith('checks:') ? outcome.replace(/^checks:/i, 'Checks: ') : outcome)}
              <tr>
                <td>{formatTime(ev.ts)}</td>
                <td><code>{ev.ip || '-'}</code></td>
                <td><span class={`badge ${isBan ? 'ban' : 'challenge'}`}>{isBan ? 'BAN' : 'DETECTION'}</span></td>
                <td>{tier}</td>
                <td>{score}</td>
                <td>{details}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>

  <div class="section events">
    <h2>Maze</h2>
    <p class="section-desc text-muted">Crawlers trapped in infinite fake pages</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Hits</h3>
        <span class="stat-value" id="maze-total-hits">{tabStatus?.loading ? '...' : mazeStats.totalHits}</span>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Unique Crawlers</h3>
        <span class="stat-value" id="maze-unique-crawlers">{tabStatus?.loading ? '...' : mazeStats.uniqueCrawlers}</span>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Auto-Banned</h3>
        <span class="stat-value" id="maze-auto-bans">{tabStatus?.loading ? '...' : mazeStats.mazeAutoBans}</span>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label" id="maze-top-offender-label">{tabStatus?.loading ? 'Top Offender' : mazeStats.topOffender.label}</h3>
        <span class="stat-value" id="maze-top-offender">{tabStatus?.loading ? '...' : mazeStats.topOffender.value}</span>
      </div>
    </div>
  </div>

  <div class="section events">
    <h2>Honeypot Hits</h2>
    <p class="section-desc text-muted">Structured honeypot telemetry (hits, offender buckets, and trap paths).</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Hits</h3>
        <span class="stat-value" id="honeypot-total-hits">{tabStatus?.loading ? '...' : monitoringSummary.honeypot.totalHits}</span>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Unique Crawlers</h3>
        <span class="stat-value" id="honeypot-unique-crawlers">{tabStatus?.loading ? '...' : monitoringSummary.honeypot.uniqueCrawlers}</span>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label" id="honeypot-top-offender-label">{tabStatus?.loading ? 'Top Offender' : monitoringSummary.honeypot.topOffender.label}</h3>
        <span class="stat-value" id="honeypot-top-offender">{tabStatus?.loading ? '...' : monitoringSummary.honeypot.topOffender.value}</span>
      </div>
    </div>
    <div class="panel panel-border pad-md">
      <h3>Top Honeypot Paths</h3>
      <div id="honeypot-top-paths" class="crawler-list">
        {#if honeypotTopPaths.length === 0}
          <p class="no-data">No honeypot path data yet</p>
        {:else}
          {#each honeypotTopPaths as row}
            <div class="crawler-item panel panel-border">
              <span class="crawler-ip">{row.path}</span>
              <span class="crawler-hits">{row.count.toLocaleString()} hits</span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>

  <div class="section events">
    <h2>Challenge Failures</h2>
    <p class="section-desc text-muted">Challenge submit failures by reason class with trend and top offender.</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Failures</h3>
        <div class="stat-value" id="challenge-failures-total">{tabStatus?.loading ? '...' : monitoringSummary.challenge.totalFailures}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Unique Offenders</h3>
        <div class="stat-value" id="challenge-failures-unique">{tabStatus?.loading ? '...' : monitoringSummary.challenge.uniqueOffenders}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label" id="challenge-top-offender-label">{tabStatus?.loading ? 'Top Offender' : monitoringSummary.challenge.topOffender.label}</h3>
        <div class="stat-value" id="challenge-top-offender">{tabStatus?.loading ? '...' : monitoringSummary.challenge.topOffender.value}</div>
      </div>
    </div>
    <div class="chart-container panel-soft panel-border pad-md">
      <canvas id="challengeFailuresTrendChart" bind:this={challengeTrendCanvas}></canvas>
    </div>
    <div class="panel panel-border pad-md">
      <div class="table-wrapper">
        <table class="panel panel-border">
          <thead>
            <tr>
              <th class="caps-label">Reason</th>
              <th class="caps-label">Count</th>
            </tr>
          </thead>
          <tbody id="challenge-failure-reasons">
            {#if challengeReasonRows.length === 0}
              <tr><td colspan="2" style="text-align: center; color: #6b7280;">No failures in window</td></tr>
            {:else}
              {#each challengeReasonRows as row}
                <tr>
                  <td>{row.label}</td>
                  <td>{row.count.toLocaleString()}</td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <div class="section events">
    <h2>PoW Failures</h2>
    <p class="section-desc text-muted">PoW verification failures by reason class with trend and top offender.</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Failures</h3>
        <div class="stat-value" id="pow-failures-total">{tabStatus?.loading ? '...' : monitoringSummary.pow.totalFailures}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Unique Offenders</h3>
        <div class="stat-value" id="pow-failures-unique">{tabStatus?.loading ? '...' : monitoringSummary.pow.uniqueOffenders}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label" id="pow-top-offender-label">{tabStatus?.loading ? 'Top Offender' : monitoringSummary.pow.topOffender.label}</h3>
        <div class="stat-value" id="pow-top-offender">{tabStatus?.loading ? '...' : monitoringSummary.pow.topOffender.value}</div>
      </div>
    </div>
    <div class="chart-container panel-soft panel-border pad-md">
      <canvas id="powFailuresTrendChart" bind:this={powTrendCanvas}></canvas>
    </div>
    <div class="panel panel-border pad-md">
      <div class="table-wrapper">
        <table class="panel panel-border">
          <thead>
            <tr>
              <th class="caps-label">Reason</th>
              <th class="caps-label">Count</th>
            </tr>
          </thead>
          <tbody id="pow-failure-reasons">
            {#if powReasonRows.length === 0}
              <tr><td colspan="2" style="text-align: center; color: #6b7280;">No failures in window</td></tr>
            {:else}
              {#each powReasonRows as row}
                <tr>
                  <td>{row.label}</td>
                  <td>{row.count.toLocaleString()}</td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <div class="section events">
    <h2>Rate Limiting Violations</h2>
    <p class="section-desc text-muted">Rate-limit outcomes and top offender bucket.</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Violations</h3>
        <div class="stat-value" id="rate-violations-total">{tabStatus?.loading ? '...' : monitoringSummary.rate.totalViolations}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Unique Offenders</h3>
        <div class="stat-value" id="rate-violations-unique">{tabStatus?.loading ? '...' : monitoringSummary.rate.uniqueOffenders}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label" id="rate-top-offender-label">{tabStatus?.loading ? 'Top Offender' : monitoringSummary.rate.topOffender.label}</h3>
        <div class="stat-value" id="rate-top-offender">{tabStatus?.loading ? '...' : monitoringSummary.rate.topOffender.value}</div>
      </div>
    </div>
    <div class="panel panel-border pad-md">
      <h3>Enforcement Outcomes</h3>
      <ul id="rate-outcomes-list" class="metric-list">
        {#if rateOutcomeRows.length === 0}
          <li class="text-muted">No outcomes yet</li>
        {:else}
          {#each rateOutcomeRows as row}
            <li><strong>{row.label}:</strong> {row.count.toLocaleString()}</li>
          {/each}
        {/if}
      </ul>
    </div>
  </div>

  <div class="section events">
    <h2>GEO Violations</h2>
    <p class="section-desc text-muted">GEO policy actions by route and top country sources.</p>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Total Violations</h3>
        <div class="stat-value" id="geo-violations-total">{tabStatus?.loading ? '...' : monitoringSummary.geo.totalViolations}</div>
      </div>
      <div class="card panel panel-border pad-md">
        <h3 class="caps-label">Action Mix</h3>
        <div class="stat-value stat-value--small" id="geo-action-mix">block {monitoringSummary.geo.actionMix.block} | challenge {monitoringSummary.geo.actionMix.challenge} | maze {monitoringSummary.geo.actionMix.maze}</div>
      </div>
    </div>
    <div class="panel panel-border pad-md">
      <h3>Top Countries Triggering GEO Actions</h3>
      <div id="geo-top-countries" class="crawler-list">
        {#if geoTopCountries.length === 0}
          <p class="no-data">No GEO violations yet</p>
        {:else}
          {#each geoTopCountries as row}
            <div class="crawler-item panel panel-border">
              <span class="crawler-ip">{row.country}</span>
              <span class="crawler-hits">{row.count.toLocaleString()} actions</span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>

  <div class="section panel panel-border pad-md">
    <h2>External Monitoring</h2>
    <ul id="monitoring-prometheus-facts" class="prometheus-facts text-muted">
      {#each prometheusHelper.facts as fact}
        <li>{fact}</li>
      {/each}
    </ul>
    <h3 class="caps-label">1) Fetch Full Prometheus Text Payload (Javascript)</h3>
    <pre id="monitoring-prometheus-example" class="prometheus-example prometheus-example--code">{prometheusHelper.exampleJs}</pre>
    <div class="prometheus-copy-actions">
      <button id="monitoring-prometheus-copy" class="btn btn-subtle" type="button" on:click={() => copyToClipboard(prometheusHelper.exampleJs, 'js')}>{copyButtonLabel}</button>
      <button id="monitoring-prometheus-copy-curl" class="btn btn-subtle" type="button" data-copy-text={prometheusHelper.copyCurlText} on:click={() => copyToClipboard(prometheusHelper.copyCurlText, 'curl')}>{copyCurlButtonLabel}</button>
    </div>
    <h3 class="caps-label">2) Example Prometheus Text Payload (Truncated Output)</h3>
    <pre id="monitoring-prometheus-output" class="prometheus-example prometheus-example--output">{prometheusHelper.exampleOutput}</pre>
    <h3 class="caps-label">3) Read Selected Metrics From Prometheus Text Payload (Javascript)</h3>
    <pre id="monitoring-prometheus-stats" class="prometheus-example prometheus-example--code">{prometheusHelper.exampleStats}</pre>
    <h3 class="caps-label">4) Request JSON Format Bounded Summary &#123;<code>hours</code>/<code>limit</code>&#125; (Javascript)</h3>
    <pre id="monitoring-prometheus-windowed" class="prometheus-example prometheus-example--code">{prometheusHelper.exampleWindowed}</pre>
    <h3 class="caps-label">5) Read Specific Summary Stats from JSON. (Javascript)</h3>
    <pre id="monitoring-prometheus-summary-stats" class="prometheus-example prometheus-example--code">{prometheusHelper.exampleSummaryStats}</pre>
    <p class="section-desc text-muted">Detailed docs: <a id="monitoring-prometheus-observability-link" href={prometheusHelper.observabilityLink || 'https://github.com/atomless/Shuma-Gorath/blob/main/docs/observability.md'} target="_blank" rel="noopener noreferrer">Observability</a> and <a id="monitoring-prometheus-api-link" href={prometheusHelper.apiLink || 'https://github.com/atomless/Shuma-Gorath/blob/main/docs/api.md'} target="_blank" rel="noopener noreferrer">API</a>.</p>
  </div>
</section>
