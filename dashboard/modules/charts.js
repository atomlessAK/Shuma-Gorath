// @ts-check

import { arraysEqualShallow } from './core/format.js';
import { getChartConstructor } from './services/chart-runtime-adapter.js';

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

const sameSeries = (chart, nextLabels, nextData) => {
  if (!chart || !chart.data || !Array.isArray(chart.data.datasets) || chart.data.datasets.length === 0) {
    return false;
  }
  const currentLabels = Array.isArray(chart.data.labels) ? chart.data.labels : [];
  const currentData = Array.isArray(chart.data.datasets[0].data) ? chart.data.datasets[0].data : [];
  return arraysEqualShallow(currentLabels, nextLabels) &&
    arraysEqualShallow(currentData, nextData);
};

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
  if (range === 'month') return now - (30 * 24 * 60 * 60 * 1000);
  return now - (60 * 60 * 1000);
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

const getCanvasContext = (doc, id) => {
  const canvas = doc && typeof doc.getElementById === 'function' ? doc.getElementById(id) : null;
  if (!canvas || typeof canvas.getContext !== 'function') return null;
  return canvas.getContext('2d');
};

export const createDashboardCharts = (options = {}) => {
  const doc = options.document || (typeof document !== 'undefined' ? document : null);

  let eventTypesChart = null;
  let topIpsChart = null;
  let timeSeriesChart = null;
  let currentTimeRange = 'hour';
  let getAdminContext = null;
  let apiClient = null;
  let chartConstructor = null;
  const timeRangeButtonHandlers = [];

  const resolveChartConstructor = () => {
    if (typeof chartConstructor === 'function') return chartConstructor;
    chartConstructor = getChartConstructor({ window: options.window });
    return typeof chartConstructor === 'function' ? chartConstructor : null;
  };

  const fetchEventsFallback = (hours) => {
    if (!doc || typeof getAdminContext !== 'function') {
      return Promise.resolve({ recent_events: [] });
    }
    const ctx = getAdminContext(doc.getElementById('last-updated'));
    if (!ctx) {
      return Promise.resolve({ recent_events: [] });
    }
    const { endpoint, apikey } = ctx;
    return fetch(`${endpoint}/admin/events?hours=${hours}`, {
      headers: { Authorization: `Bearer ${apikey}` }
    }).then((response) => {
      if (!response.ok) throw new Error('Failed to fetch events');
      return response.json();
    });
  };

  const bindTimeRangeButtons = () => {
    if (!doc || typeof doc.querySelectorAll !== 'function') return;

    while (timeRangeButtonHandlers.length > 0) {
      const cleanup = timeRangeButtonHandlers.pop();
      cleanup();
    }

    doc.querySelectorAll('.time-btn').forEach((button) => {
      const onClick = () => {
        doc.querySelectorAll('.time-btn').forEach((entry) => entry.classList.remove('active'));
        button.classList.add('active');
        currentTimeRange = button.dataset.range || 'hour';
        updateTimeSeriesChart();
      };
      button.addEventListener('click', onClick);
      timeRangeButtonHandlers.push(() => {
        button.removeEventListener('click', onClick);
      });
    });
  };

  const destroyCharts = () => {
    if (eventTypesChart && typeof eventTypesChart.destroy === 'function') {
      eventTypesChart.destroy();
    }
    if (topIpsChart && typeof topIpsChart.destroy === 'function') {
      topIpsChart.destroy();
    }
    if (timeSeriesChart && typeof timeSeriesChart.destroy === 'function') {
      timeSeriesChart.destroy();
    }
    eventTypesChart = null;
    topIpsChart = null;
    timeSeriesChart = null;
  };

  const init = (initOptions = {}) => {
    getAdminContext =
      typeof initOptions.getAdminContext === 'function' ? initOptions.getAdminContext : null;
    apiClient = initOptions.apiClient || null;
    chartConstructor =
      typeof initOptions.chartConstructor === 'function' ? initOptions.chartConstructor : null;

    if (!doc) return;

    destroyCharts();
    currentTimeRange = 'hour';

    const ChartCtor = resolveChartConstructor();
    const eventTypesCtx = getCanvasContext(doc, 'eventTypesChart');
    if (eventTypesCtx && ChartCtor) {
      eventTypesChart = new ChartCtor(eventTypesCtx, {
        type: 'doughnut',
        data: {
          labels: [],
          datasets: [{ data: [] }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: true,
          plugins: { legend: { position: 'bottom' } }
        }
      });
    }

    const topIpsCtx = getCanvasContext(doc, 'topIpsChart');
    if (topIpsCtx && ChartCtor) {
      topIpsChart = new ChartCtor(topIpsCtx, {
        type: 'bar',
        data: {
          labels: [],
          datasets: [{ label: 'Events', data: [] }]
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
      });
    }

    const timeSeriesCtx = getCanvasContext(doc, 'timeSeriesChart');
    if (timeSeriesCtx && ChartCtor) {
      timeSeriesChart = new ChartCtor(timeSeriesCtx, {
        type: 'line',
        data: {
          labels: [],
          datasets: [{
            label: 'Events',
            data: [],
            fill: true,
            tension: 0.4,
            borderWidth: 0,
            pointRadius: 0,
            pointHoverRadius: 0
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
      });
    }

    bindTimeRangeButtons();
  };

  const updateEventTypesChart = (eventCounts) => {
    if (!eventTypesChart) return;
    const labels = Object.keys(eventCounts || {});
    const data = Object.values(eventCounts || {});
    if (sameSeries(eventTypesChart, labels, data)) return;

    eventTypesChart.data.labels = labels;
    eventTypesChart.data.datasets[0].data = data;
    const bg = data.map((_, i) => CHART_PALETTE[i % CHART_PALETTE.length]);
    eventTypesChart.data.datasets[0].backgroundColor = bg;
    eventTypesChart.data.datasets[0].borderColor = bg;
    eventTypesChart.update();
  };

  const updateTopIpsChart = (topIps) => {
    if (!topIpsChart) return;
    const labels = (topIps || []).map(([ip]) => ip);
    const data = (topIps || []).map(([, count]) => count);
    if (sameSeries(topIpsChart, labels, data)) return;
    const barColors = data.map((_, i) => CHART_PALETTE[i % CHART_PALETTE.length]);

    topIpsChart.data.labels = labels;
    topIpsChart.data.datasets[0].data = data;
    topIpsChart.data.datasets[0].backgroundColor = barColors;
    topIpsChart.data.datasets[0].borderColor = barColors;
    topIpsChart.update();
  };

  const updateTimeSeriesChart = () => {
    if (!timeSeriesChart) return;
    const hours = hoursForRange(currentTimeRange);
    const loadEvents = apiClient && typeof apiClient.getEvents === 'function'
      ? apiClient.getEvents(hours)
      : fetchEventsFallback(hours);

    loadEvents
      .then((data) => {
        const now = Date.now();
        const cutoffTime = cutoffForRange(currentTimeRange, now);
        const events = data.recent_events || [];
        const filteredEvents = events.filter((entry) => (entry.ts * 1000) >= cutoffTime);
        const bucketSize = bucketSizeForRange(currentTimeRange);

        const buckets = {};
        for (let time = cutoffTime; time <= now; time += bucketSize) {
          const bucketKey = Math.floor(time / bucketSize) * bucketSize;
          buckets[bucketKey] = 0;
        }

        filteredEvents.forEach((entry) => {
          const eventTime = entry.ts * 1000;
          const bucketKey = Math.floor(eventTime / bucketSize) * bucketSize;
          buckets[bucketKey] = (buckets[bucketKey] || 0) + 1;
        });

        const sortedBuckets = Object.keys(buckets)
          .map((key) => parseInt(key, 10))
          .sort((a, b) => a - b);

        const labels = sortedBuckets.map((epochMs) => formatBucketLabel(currentTimeRange, epochMs));
        const counts = sortedBuckets.map((epochMs) => buckets[epochMs]);
        if (sameSeries(timeSeriesChart, labels, counts)) return;

        timeSeriesChart.data.labels = labels;
        timeSeriesChart.data.datasets[0].data = counts;
        timeSeriesChart.data.datasets[0].borderColor = 'rgba(0, 0, 0, 0)';
        timeSeriesChart.data.datasets[0].borderWidth = 0;
        timeSeriesChart.data.datasets[0].backgroundColor = CHART_PALETTE[0];
        timeSeriesChart.update();
      })
      .catch((err) => console.error('Failed to update time series:', err));
  };

  const destroy = () => {
    while (timeRangeButtonHandlers.length > 0) {
      const cleanup = timeRangeButtonHandlers.pop();
      cleanup();
    }
    destroyCharts();
    getAdminContext = null;
    apiClient = null;
    chartConstructor = null;
  };

  return {
    init,
    updateEventTypesChart,
    updateTopIpsChart,
    updateTimeSeriesChart,
    destroy
  };
};
