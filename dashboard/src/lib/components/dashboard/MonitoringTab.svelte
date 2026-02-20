<script>
  import { browser } from '$app/environment';
  import { onDestroy, onMount } from 'svelte';
  import {
    CHALLENGE_REASON_LABELS,
    NOT_A_BOT_OUTCOME_LABELS,
    NOT_A_BOT_LATENCY_LABELS,
    POW_REASON_LABELS,
    RATE_OUTCOME_LABELS,
    deriveMazeStatsViewModel,
    deriveMonitoringSummaryViewModel,
    derivePrometheusHelperViewModel,
    formatMetricLabel
  } from './monitoring-view-model.js';
  import { arraysEqualShallow } from '../../domain/core/format.js';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import OverviewStats from './monitoring/OverviewStats.svelte';
  import PrimaryCharts from './monitoring/PrimaryCharts.svelte';
  import RecentEventsTable from './monitoring/RecentEventsTable.svelte';
  import CdpSection from './monitoring/CdpSection.svelte';
  import MazeSection from './monitoring/MazeSection.svelte';
  import HoneypotSection from './monitoring/HoneypotSection.svelte';
  import ChallengeSection from './monitoring/ChallengeSection.svelte';
  import PowSection from './monitoring/PowSection.svelte';
  import RateSection from './monitoring/RateSection.svelte';
  import GeoSection from './monitoring/GeoSection.svelte';
  import ExternalMonitoringSection from './monitoring/ExternalMonitoringSection.svelte';

  const EVENT_ROW_RENDER_LIMIT = 100;
  const CDP_ROW_RENDER_LIMIT = 500;
  const MONITORING_LIST_LIMIT = 10;
  const MONITORING_TREND_POINT_LIMIT = 720;
  const RANGE_EVENTS_FETCH_LIMIT = 5000;
  const RANGE_EVENTS_REQUEST_TIMEOUT_MS = 10000;
  const RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS = 180000;
  const CHART_RESIZE_REDRAW_DEBOUNCE_MS = 180;
  const MAX_SAFE_COUNT = 1_000_000_000;
  const CHALLENGE_TREND_COLOR = 'rgba(122, 114, 255, 0.35)';
  const POW_TREND_COLOR = 'rgba(255, 130, 92, 0.35)';
  const POW_OUTCOME_LABELS = Object.freeze({
    success: 'Success',
    failure: 'Failure'
  });
  const CHART_PALETTE = [
    'rgb(255,205,235)',
    'rgb(225,175,205)',
    'rgb(205, 155, 185)',
    'rgb(190, 140, 170)',
    'rgb(175, 125, 155)',
    'rgb(160, 110, 140)',
    'rgb(147, 97, 127)',
    'rgb(135, 85, 115)'
  ];
  const TIME_RANGES = Object.freeze(['hour', 'day', 'week', 'month']);

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
  export let onFetchEventsRange = null;
  export let autoRefreshEnabled = false;

  let eventTypesCanvas = null;
  let topIpsCanvas = null;
  let timeSeriesCanvas = null;
  let challengeTrendCanvas = null;
  let powTrendCanvas = null;
  let eventTypesChart = null;
  let topIpsChart = null;
  let timeSeriesChart = null;
  let challengeTrendChart = null;
  let powTrendChart = null;

  let selectedTimeRange = 'hour';
  let rangeEventsSnapshot = { range: '', recent_events: [] };
  let rangeEventsAbortController = null;
  let lastRequestedRange = '';
  let rangeEventsLastFetchedAtMs = 0;
  let lastRangeTabUpdateAnchor = '';

  let copyButtonLabel = 'Copy JS Example';
  let copyCurlButtonLabel = 'Copy Curl Example';
  let copyButtonTimer = null;
  let copyCurlButtonTimer = null;
  let resizeRedrawTimer = null;
  let chartRefreshNonce = 0;
  let wasActive = false;
  let detachColorSchemeListener = () => {};

  const defaultMonitoringSummary = deriveMonitoringSummaryViewModel({});
  const defaultMazeStats = deriveMazeStatsViewModel({});
  const defaultPrometheusHelper = derivePrometheusHelperViewModel({}, '');

  const clearTimer = (timerId) => {
    if (timerId === null) return null;
    clearTimeout(timerId);
    return null;
  };
  const clampCount = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric < 0) return 0;
    return Math.min(MAX_SAFE_COUNT, Math.floor(numeric));
  };
  const sanitizeText = (value, fallback = '-') => {
    const text = String(value || '').replace(/[\u0000-\u001f\u007f]/g, '').trim();
    return text || fallback;
  };
  const shouldFetchRange = (range) => range === 'week' || range === 'month';
  const hoursForRange = (range) => {
    if (range === 'hour') return 1;
    if (range === 'day') return 24;
    if (range === 'week') return 168;
    return 720;
  };
  const cutoffForRange = (range, now) => {
    if (range === 'hour') return now - (60 * 60 * 1000);
    if (range === 'day') return now - (24 * 60 * 60 * 1000);
    if (range === 'week') return now - (7 * 24 * 60 * 60 * 1000);
    return now - (30 * 24 * 60 * 60 * 1000);
  };
  const bucketSizeForRange = (range) =>
    range === 'hour' ? 300000 : range === 'day' ? 3600000 : 86400000;
  const formatBucketLabel = (range, epochMs) => {
    const date = new Date(epochMs);
    if (range === 'hour') {
      return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
    }
    if (range === 'day') {
      return date.toLocaleString('en-US', {
        month: 'short',
        day: 'numeric',
        hour: 'numeric',
        minute: '2-digit'
      });
    }
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };
  const sameSeries = (chart, nextLabels, nextData) => {
    if (!chart || !chart.data || !Array.isArray(chart.data.datasets) || chart.data.datasets.length === 0) {
      return false;
    }
    const currentLabels = Array.isArray(chart.data.labels) ? chart.data.labels : [];
    const currentData = Array.isArray(chart.data.datasets[0].data) ? chart.data.datasets[0].data : [];
    return arraysEqualShallow(currentLabels, nextLabels) && arraysEqualShallow(currentData, nextData);
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
    return rows.slice(0, MONITORING_LIST_LIMIT).map(([key, value]) => ({
      key: sanitizeText(key),
      label: sanitizeText(formatMetricLabel(key, labels)),
      count: clampCount(value)
    }));
  };

  const normalizePairRows = (rows, labels) => {
    if (!Array.isArray(rows)) return [];
    return rows.slice(0, MONITORING_LIST_LIMIT).map(([key, value]) => ({
      key: sanitizeText(key),
      label: sanitizeText(formatMetricLabel(key, labels)),
      count: clampCount(value)
    }));
  };

  const normalizeTopPaths = (paths) => {
    if (!Array.isArray(paths)) return [];
    return paths.slice(0, MONITORING_LIST_LIMIT).map((entry) => ({
      path: sanitizeText(entry.path, '-'),
      count: clampCount(entry.count)
    }));
  };

  const normalizeTopCountries = (rows) => {
    if (!Array.isArray(rows)) return [];
    return rows.slice(0, MONITORING_LIST_LIMIT).map((entry) => ({
      country: sanitizeText(entry.country, '-'),
      count: clampCount(entry.count)
    }));
  };

  const normalizeTrendSeries = (series) => {
    const labels = Array.isArray(series?.labels) ? series.labels : [];
    const data = Array.isArray(series?.data) ? series.data : [];
    const pointCount = Math.min(labels.length, data.length);
    const start = Math.max(0, pointCount - MONITORING_TREND_POINT_LIMIT);
    const nextLabels = [];
    const nextData = [];
    for (let index = start; index < pointCount; index += 1) {
      nextLabels.push(sanitizeText(labels[index], '-'));
      nextData.push(clampCount(data[index]));
    }
    return {
      labels: nextLabels,
      data: nextData
    };
  };

  const getChartConstructor = () => {
    if (!browser || !window || typeof window.Chart !== 'function') return null;
    return window.Chart;
  };

  const chartNeedsRefresh = (chart, refreshNonce) =>
    Number(chart?.__shumaRefreshNonce || 0) !== Number(refreshNonce || 0);

  const stampChartRefresh = (chart, refreshNonce) => {
    if (chart && typeof chart === 'object') {
      chart.__shumaRefreshNonce = Number(refreshNonce || 0);
    }
    return chart;
  };

  const requestChartRefresh = () => {
    chartRefreshNonce += 1;
  };

  const scheduleChartRefreshAfterResize = () => {
    resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    resizeRedrawTimer = setTimeout(() => {
      resizeRedrawTimer = null;
      if (!isActive) return;
      requestChartRefresh();
    }, CHART_RESIZE_REDRAW_DEBOUNCE_MS);
  };

  const attachColorSchemeChangeListener = () => {
    if (!browser || !window || typeof window.matchMedia !== 'function') {
      return () => {};
    }
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => {
      if (!isActive) return;
      requestChartRefresh();
    };
    if (typeof mediaQuery.addEventListener === 'function') {
      mediaQuery.addEventListener('change', onChange);
      return () => {
        mediaQuery.removeEventListener('change', onChange);
      };
    }
    if (typeof mediaQuery.addListener === 'function') {
      mediaQuery.addListener(onChange);
      return () => {
        mediaQuery.removeListener(onChange);
      };
    }
    return () => {};
  };

  const updateTrendChart = (chart, canvas, title, color, trendSeries, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
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
      }), refreshNonce);
    }

    if (!chartNeedsRefresh(chart, refreshNonce) && sameSeries(chart, trendSeries.labels, trendSeries.data)) {
      return chart;
    }
    chart.data.labels = trendSeries.labels;
    chart.data.datasets[0].data = trendSeries.data;
    chart.update();
    return stampChartRefresh(chart, refreshNonce);
  };

  const updateDoughnutChart = (chart, canvas, counts, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    const labels = Object.keys(counts || {});
    const data = Object.values(counts || {});
    const colors = data.map((_, index) => CHART_PALETTE[index % CHART_PALETTE.length]);

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
        type: 'doughnut',
        data: {
          labels,
          datasets: [{ data, backgroundColor: colors, borderColor: colors }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          plugins: { legend: { position: 'bottom' } }
        }
      }), refreshNonce);
    }

    if (!chartNeedsRefresh(chart, refreshNonce) && sameSeries(chart, labels, data)) {
      return chart;
    }
    chart.data.labels = labels;
    chart.data.datasets[0].data = data;
    chart.data.datasets[0].backgroundColor = colors;
    chart.data.datasets[0].borderColor = colors;
    chart.update();
    return stampChartRefresh(chart, refreshNonce);
  };

  const updateTopIpsChart = (chart, canvas, topIps, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;
    const pairs = Array.isArray(topIps) ? topIps : [];
    const labels = pairs.map(([ip]) => String(ip || '-'));
    const data = pairs.map(([, count]) => Number(count || 0));
    const colors = data.map((_, index) => CHART_PALETTE[index % CHART_PALETTE.length]);

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
        type: 'bar',
        data: {
          labels,
          datasets: [{ label: 'Events', data, backgroundColor: colors, borderColor: colors }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          scales: {
            y: {
              beginAtZero: true,
              ticks: { stepSize: 1 }
            }
          },
          plugins: { legend: { display: false } }
        }
      }), refreshNonce);
    }

    if (!chartNeedsRefresh(chart, refreshNonce) && sameSeries(chart, labels, data)) {
      return chart;
    }
    chart.data.labels = labels;
    chart.data.datasets[0].data = data;
    chart.data.datasets[0].backgroundColor = colors;
    chart.data.datasets[0].borderColor = colors;
    chart.update();
    return stampChartRefresh(chart, refreshNonce);
  };

  const buildTimeSeries = (events, range) => {
    const now = Date.now();
    const cutoffTime = cutoffForRange(range, now);
    const bucketSize = bucketSizeForRange(range);
    const normalized = Array.isArray(events) ? events : [];
    const filteredEvents = normalized.filter((entry) => (Number(entry?.ts || 0) * 1000) >= cutoffTime);
    const boundedEvents = filteredEvents.length > RANGE_EVENTS_FETCH_LIMIT
      ? filteredEvents.slice(0, RANGE_EVENTS_FETCH_LIMIT)
      : filteredEvents;
    const buckets = {};
    for (let time = cutoffTime; time <= now; time += bucketSize) {
      const bucketKey = Math.floor(time / bucketSize) * bucketSize;
      buckets[bucketKey] = 0;
    }
    boundedEvents.forEach((entry) => {
      const eventTime = Number(entry?.ts || 0) * 1000;
      if (!Number.isFinite(eventTime) || eventTime <= 0) return;
      const bucketKey = Math.floor(eventTime / bucketSize) * bucketSize;
      buckets[bucketKey] = (buckets[bucketKey] || 0) + 1;
    });
    const sortedBuckets = Object.keys(buckets)
      .map((key) => Number.parseInt(key, 10))
      .sort((left, right) => left - right);
    return {
      labels: sortedBuckets.map((epochMs) => formatBucketLabel(range, epochMs)),
      data: sortedBuckets.map((epochMs) => buckets[epochMs] || 0)
    };
  };

  const updateTimeSeriesChart = (chart, canvas, series, refreshNonce = 0) => {
    const chartCtor = getChartConstructor();
    if (!canvas || !chartCtor) return chart;
    const ctx = canvas.getContext('2d');
    if (!ctx) return chart;

    if (!chart) {
      return stampChartRefresh(new chartCtor(ctx, {
        type: 'line',
        data: {
          labels: series.labels,
          datasets: [{
            label: 'Events',
            data: series.data,
            fill: true,
            tension: 0.4,
            borderWidth: 0,
            pointRadius: 0,
            pointHoverRadius: 0,
            borderColor: 'rgba(0, 0, 0, 0)',
            backgroundColor: CHART_PALETTE[0]
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          scales: {
            y: {
              beginAtZero: true,
              ticks: { stepSize: 1 }
            }
          },
          plugins: { legend: { display: false } }
        }
      }), refreshNonce);
    }

    if (!chartNeedsRefresh(chart, refreshNonce) && sameSeries(chart, series.labels, series.data)) {
      return chart;
    }
    chart.data.labels = series.labels;
    chart.data.datasets[0].data = series.data;
    chart.update();
    return stampChartRefresh(chart, refreshNonce);
  };

  function selectTimeRange(range) {
    if (!TIME_RANGES.includes(range)) return;
    if (selectedTimeRange === range) return;
    selectedTimeRange = range;
    if (!shouldFetchRange(range)) return;
    if (rangeEventsSnapshot.range === range) return;
    lastRequestedRange = '';
  }

  function abortRangeEventsFetch() {
    if (!rangeEventsAbortController) return;
    rangeEventsAbortController.abort();
    rangeEventsAbortController = null;
  }

  async function fetchRangeEvents(range) {
    if (!browser || !shouldFetchRange(range)) return;
    const hours = hoursForRange(range);
    if (!Number.isFinite(hours)) return;
    if (typeof onFetchEventsRange !== 'function') {
      rangeEventsSnapshot = { range, recent_events: [] };
      return;
    }
    abortRangeEventsFetch();
    const abortController = new AbortController();
    const timeoutId = setTimeout(() => {
      abortController.abort();
    }, RANGE_EVENTS_REQUEST_TIMEOUT_MS);
    rangeEventsAbortController = abortController;
    try {
      const payload = await onFetchEventsRange(hours, {
        signal: abortController.signal
      });
      if (rangeEventsAbortController !== abortController) return;
      rangeEventsSnapshot = {
        range,
        recent_events: Array.isArray(payload?.recent_events)
          ? payload.recent_events.slice(0, RANGE_EVENTS_FETCH_LIMIT)
          : []
      };
      rangeEventsLastFetchedAtMs = Date.now();
    } catch (error) {
      if (error && error.name === 'AbortError') return;
      if (rangeEventsAbortController !== abortController) return;
      rangeEventsSnapshot = { range, recent_events: [] };
      rangeEventsLastFetchedAtMs = Date.now();
    } finally {
      clearTimeout(timeoutId);
      if (rangeEventsAbortController === abortController) {
        rangeEventsAbortController = null;
      }
    }
  }

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
  $: notABotOutcomeRows = normalizeReasonRows(
    monitoringSummary.notABot.outcomes,
    NOT_A_BOT_OUTCOME_LABELS
  );
  $: notABotLatencyRows = normalizeReasonRows(
    monitoringSummary.notABot.latencyBuckets,
    NOT_A_BOT_LATENCY_LABELS
  );
  $: powReasonRows = normalizeReasonRows(monitoringSummary.pow.reasons, POW_REASON_LABELS);
  $: powOutcomeRows = normalizePairRows(monitoringSummary.pow.outcomes, POW_OUTCOME_LABELS);
  $: rateOutcomeRows = normalizePairRows(monitoringSummary.rate.outcomes, RATE_OUTCOME_LABELS);
  $: geoTopCountries = normalizeTopCountries(monitoringSummary.geo.topCountries);

  $: challengeTrendSeries = normalizeTrendSeries(monitoringSummary.challenge.trend);
  $: powTrendSeries = normalizeTrendSeries(monitoringSummary.pow.trend);

  $: defaultRangeEvents = Array.isArray(events.recent_events)
    ? events.recent_events.slice(0, RANGE_EVENTS_FETCH_LIMIT)
    : [];
  $: selectedRangeEvents = shouldFetchRange(selectedTimeRange)
    ? (rangeEventsSnapshot.range === selectedTimeRange ? rangeEventsSnapshot.recent_events : [])
    : defaultRangeEvents;
  $: timeSeries = buildTimeSeries(selectedRangeEvents, selectedTimeRange);

  $: if (browser && !isActive) {
    abortRangeEventsFetch();
  }

  $: if (browser && !autoRefreshEnabled) {
    lastRangeTabUpdateAnchor = '';
  }

  $: if (browser && isActive && autoRefreshEnabled && shouldFetchRange(selectedTimeRange)) {
    const currentUpdatedAt = String(tabStatus?.updatedAt || '');
    if (currentUpdatedAt && currentUpdatedAt !== lastRangeTabUpdateAnchor) {
      lastRangeTabUpdateAnchor = currentUpdatedAt;
      if ((Date.now() - rangeEventsLastFetchedAtMs) >= RANGE_EVENTS_AUTO_REFRESH_INTERVAL_MS) {
        lastRequestedRange = '';
      }
    }
  }

  $: if (browser && isActive && shouldFetchRange(selectedTimeRange) && lastRequestedRange !== selectedTimeRange) {
    lastRequestedRange = selectedTimeRange;
    void fetchRangeEvents(selectedTimeRange);
  }

  $: if (browser) {
    const nextActive = isActive === true;
    if (nextActive && !wasActive) {
      requestChartRefresh();
    }
    wasActive = nextActive;
  }

  $: if (browser && eventTypesCanvas) {
    eventTypesChart = updateDoughnutChart(
      eventTypesChart,
      eventTypesCanvas,
      events.event_counts || {},
      chartRefreshNonce
    );
  }

  $: if (browser && topIpsCanvas) {
    topIpsChart = updateTopIpsChart(topIpsChart, topIpsCanvas, events.top_ips || [], chartRefreshNonce);
  }

  $: if (browser && timeSeriesCanvas) {
    timeSeriesChart = updateTimeSeriesChart(timeSeriesChart, timeSeriesCanvas, timeSeries, chartRefreshNonce);
  }

  $: if (browser && challengeTrendCanvas) {
    challengeTrendChart = updateTrendChart(
      challengeTrendChart,
      challengeTrendCanvas,
      'Challenge Failures',
      CHALLENGE_TREND_COLOR,
      challengeTrendSeries,
      chartRefreshNonce
    );
  }

  $: if (browser && powTrendCanvas) {
    powTrendChart = updateTrendChart(
      powTrendChart,
      powTrendCanvas,
      'PoW Failures',
      POW_TREND_COLOR,
      powTrendSeries,
      chartRefreshNonce
    );
  }

  onMount(() => {
    if (!browser || !window) return undefined;
    const onResize = () => {
      scheduleChartRefreshAfterResize();
    };
    window.addEventListener('resize', onResize, { passive: true });
    detachColorSchemeListener = attachColorSchemeChangeListener();
    return () => {
      window.removeEventListener('resize', onResize);
      if (typeof detachColorSchemeListener === 'function') {
        detachColorSchemeListener();
        detachColorSchemeListener = () => {};
      }
      resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    };
  });

  onDestroy(() => {
    copyButtonTimer = clearTimer(copyButtonTimer);
    copyCurlButtonTimer = clearTimer(copyCurlButtonTimer);
    resizeRedrawTimer = clearTimer(resizeRedrawTimer);
    if (typeof detachColorSchemeListener === 'function') {
      detachColorSchemeListener();
      detachColorSchemeListener = () => {};
    }
    abortRangeEventsFetch();
    if (eventTypesChart && typeof eventTypesChart.destroy === 'function') {
      eventTypesChart.destroy();
    }
    if (topIpsChart && typeof topIpsChart.destroy === 'function') {
      topIpsChart.destroy();
    }
    if (timeSeriesChart && typeof timeSeriesChart.destroy === 'function') {
      timeSeriesChart.destroy();
    }
    if (challengeTrendChart && typeof challengeTrendChart.destroy === 'function') {
      challengeTrendChart.destroy();
    }
    if (powTrendChart && typeof powTrendChart.destroy === 'function') {
      powTrendChart.destroy();
    }
    eventTypesChart = null;
    topIpsChart = null;
    timeSeriesChart = null;
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

  <OverviewStats
    loading={tabStatus?.loading === true}
    {totalBans}
    {activeBans}
    {eventCount}
    {uniqueIps}
  />

  <PrimaryCharts
    {selectedTimeRange}
    onSelectTimeRange={selectTimeRange}
    bind:eventTypesCanvas
    bind:topIpsCanvas
    bind:timeSeriesCanvas
  />

  <RecentEventsTable
    {recentEvents}
    {formatTime}
    {eventBadgeClass}
  />

  <CdpSection
    loading={tabStatus?.loading === true}
    {cdpDetections}
    {cdpAutoBans}
    {cdpFingerprintEvents}
    {cdpFingerprintFlowViolations}
    {recentCdpEvents}
    {formatTime}
    {readCdpField}
  />

  <MazeSection
    loading={tabStatus?.loading === true}
    {mazeStats}
  />

  <HoneypotSection
    loading={tabStatus?.loading === true}
    honeypot={monitoringSummary.honeypot}
    topPaths={honeypotTopPaths}
  />

  <ChallengeSection
    loading={tabStatus?.loading === true}
    challengeSummary={monitoringSummary.challenge}
    notABotSummary={monitoringSummary.notABot}
    {challengeReasonRows}
    {notABotOutcomeRows}
    {notABotLatencyRows}
    bind:challengeTrendCanvas
  />

  <PowSection
    loading={tabStatus?.loading === true}
    powSummary={monitoringSummary.pow}
    {powReasonRows}
    {powOutcomeRows}
    bind:powTrendCanvas
  />

  <RateSection
    loading={tabStatus?.loading === true}
    rateSummary={monitoringSummary.rate}
    {rateOutcomeRows}
  />

  <GeoSection
    loading={tabStatus?.loading === true}
    geoSummary={monitoringSummary.geo}
    {geoTopCountries}
  />

  <ExternalMonitoringSection
    {prometheusHelper}
    {copyButtonLabel}
    copyCurlButtonLabel={copyCurlButtonLabel}
    onCopyJs={(text) => copyToClipboard(text, 'js')}
    onCopyCurl={(text) => copyToClipboard(text, 'curl')}
  />
</section>
