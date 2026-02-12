(function (global) {
  let eventTypesChart = null;
  let topIpsChart = null;
  let timeSeriesChart = null;
  let currentTimeRange = 'hour';
  let getAdminContext = null;

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

  function init(options = {}) {
    getAdminContext = typeof options.getAdminContext === 'function' ? options.getAdminContext : null;

    const ctx1 = document.getElementById('eventTypesChart').getContext('2d');
    eventTypesChart = new Chart(ctx1, {
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

    const ctx2 = document.getElementById('topIpsChart').getContext('2d');
    topIpsChart = new Chart(ctx2, {
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

    const ctx3 = document.getElementById('timeSeriesChart').getContext('2d');
    timeSeriesChart = new Chart(ctx3, {
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

    document.querySelectorAll('.time-btn').forEach(btn => {
      btn.addEventListener('click', function () {
        document.querySelectorAll('.time-btn').forEach(b => b.classList.remove('active'));
        this.classList.add('active');
        currentTimeRange = this.dataset.range;
        updateTimeSeriesChart();
      });
    });
  }

  function updateEventTypesChart(eventCounts) {
    if (!eventTypesChart) return;
    const labels = Object.keys(eventCounts || {});
    const data = Object.values(eventCounts || {});

    eventTypesChart.data.labels = labels;
    eventTypesChart.data.datasets[0].data = data;
    const bg = data.map((_, i) => CHART_PALETTE[i % CHART_PALETTE.length]);
    eventTypesChart.data.datasets[0].backgroundColor = bg;
    eventTypesChart.data.datasets[0].borderColor = bg;
    eventTypesChart.update();
  }

  function updateTopIpsChart(topIps) {
    if (!topIpsChart) return;
    const labels = (topIps || []).map(([ip, _]) => ip);
    const data = (topIps || []).map(([_, count]) => count);
    const barColors = data.map((_, i) => CHART_PALETTE[i % CHART_PALETTE.length]);

    topIpsChart.data.labels = labels;
    topIpsChart.data.datasets[0].data = data;
    topIpsChart.data.datasets[0].backgroundColor = barColors;
    topIpsChart.data.datasets[0].borderColor = barColors;
    topIpsChart.update();
  }

  function updateTimeSeriesChart() {
    if (!timeSeriesChart || typeof getAdminContext !== 'function') return;
    const ctx = getAdminContext(document.getElementById('last-updated'));
    if (!ctx) return;
    const { endpoint, apikey } = ctx;

    const hours = currentTimeRange === 'hour' ? 1 :
      currentTimeRange === 'day' ? 24 :
        currentTimeRange === 'week' ? 168 : 720;

    fetch(`${endpoint}/admin/events?hours=${hours}`, {
      headers: { 'Authorization': 'Bearer ' + apikey }
    })
      .then(r => {
        if (!r.ok) throw new Error('Failed to fetch events');
        return r.json();
      })
      .then(data => {
        const now = Date.now();
        let cutoffTime;

        switch (currentTimeRange) {
          case 'hour':
            cutoffTime = now - (60 * 60 * 1000);
            break;
          case 'day':
            cutoffTime = now - (24 * 60 * 60 * 1000);
            break;
          case 'week':
            cutoffTime = now - (7 * 24 * 60 * 60 * 1000);
            break;
          case 'month':
            cutoffTime = now - (30 * 24 * 60 * 60 * 1000);
            break;
          default:
            cutoffTime = now - (60 * 60 * 1000);
            break;
        }

        const events = data.recent_events || [];
        const filteredEvents = events.filter(e => (e.ts * 1000) >= cutoffTime);

        const buckets = {};
        const bucketSize = currentTimeRange === 'hour' ? 300000 :
          currentTimeRange === 'day' ? 3600000 :
            currentTimeRange === 'week' ? 86400000 : 86400000;

        for (let time = cutoffTime; time <= now; time += bucketSize) {
          const bucketKey = Math.floor(time / bucketSize) * bucketSize;
          buckets[bucketKey] = 0;
        }

        filteredEvents.forEach(event => {
          const eventTime = event.ts * 1000;
          const bucketKey = Math.floor(eventTime / bucketSize) * bucketSize;
          buckets[bucketKey] = (buckets[bucketKey] || 0) + 1;
        });

        const sortedBuckets = Object.keys(buckets).map(k => parseInt(k, 10)).sort((a, b) => a - b);

        const labels = sortedBuckets.map(time => {
          const date = new Date(time);
          if (currentTimeRange === 'hour') {
            return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
          }
          if (currentTimeRange === 'day') {
            return date.toLocaleString('en-US', {
              month: 'short',
              day: 'numeric',
              hour: 'numeric',
              minute: '2-digit'
            });
          }
          return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
        });

        const counts = sortedBuckets.map(time => buckets[time]);

        timeSeriesChart.data.labels = labels;
        timeSeriesChart.data.datasets[0].data = counts;
        timeSeriesChart.data.datasets[0].borderColor = 'rgba(0, 0, 0, 0)';
        timeSeriesChart.data.datasets[0].borderWidth = 0;
        timeSeriesChart.data.datasets[0].backgroundColor = CHART_PALETTE[0];
        timeSeriesChart.update();
      })
      .catch(err => console.error('Failed to update time series:', err));
  }

  global.ShumaDashboardCharts = {
    init,
    updateEventTypesChart,
    updateTopIpsChart,
    updateTimeSeriesChart
  };
})(window);
