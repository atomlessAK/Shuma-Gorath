// Global chart instances
let eventTypesChart = null;
let topIpsChart = null;
let timeSeriesChart = null;
let currentTimeRange = 'hour';

// Initialize charts
function initCharts() {
  const ctx1 = document.getElementById('eventTypesChart').getContext('2d');
  eventTypesChart = new Chart(ctx1, {
    type: 'doughnut',
    data: {
      labels: [],
      datasets: [{
        data: []
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      plugins: {
        legend: {
          position: 'bottom'
        },
        colorschemes: {
          scheme: 'brewer.Reds3'
        }
      }
    }
  });

  const ctx2 = document.getElementById('topIpsChart').getContext('2d');
  topIpsChart = new Chart(ctx2, {
    type: 'bar',
    data: {
      labels: [],
      datasets: [{
        label: 'Events',
        data: []
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      scales: {
        y: {
          beginAtZero: true,
          ticks: {
            stepSize: 1
          }
        }
      },
      plugins: {
        legend: {
          display: false
        },
        colorschemes: {
          scheme: 'brewer.Reds3'
        }
      }
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
        tension: 0.4
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      scales: {
        y: {
          beginAtZero: true,
          ticks: {
            stepSize: 1
          }
        }
      },
      plugins: {
        legend: {
          display: false
        },
        colorschemes: {
          scheme: 'brewer.Reds3'
        }
      }
    }
  });

  // Setup time range button handlers
  document.querySelectorAll('.time-btn').forEach(btn => {
    btn.addEventListener('click', function() {
      document.querySelectorAll('.time-btn').forEach(b => b.classList.remove('active'));
      this.classList.add('active');
      currentTimeRange = this.dataset.range;
      updateTimeSeriesChart();
    });
  });
}

// Update stat cards
function updateStatCards(analytics, events, bans) {
  document.getElementById('total-bans').textContent = analytics.ban_count || 0;
  document.getElementById('active-bans').textContent = bans.length || 0;
  document.getElementById('total-events').textContent = (events.recent_events || []).length;
  document.getElementById('unique-ips').textContent = (events.top_ips || []).length;
  
  // Update test mode banner and toggle
  const testMode = analytics.test_mode === true;
  const banner = document.getElementById('test-mode-banner');
  const toggle = document.getElementById('test-mode-toggle');
  const status = document.getElementById('test-mode-status');
  
  if (testMode) {
    banner.classList.remove('hidden');
    status.textContent = 'Enabled (logging only)';
    status.style.color = '#d97706';
  } else {
    banner.classList.add('hidden');
    status.textContent = 'Disabled (blocking active)';
    status.style.color = '#10b981';
  }
  toggle.checked = testMode;
}

// Update ban duration fields from config
function updateBanDurations(config) {
  if (config.ban_durations) {
    document.getElementById('dur-honeypot').value = config.ban_durations.honeypot || 86400;
    document.getElementById('dur-rate-limit').value = config.ban_durations.rate_limit || 3600;
    document.getElementById('dur-browser').value = config.ban_durations.browser || 21600;
    document.getElementById('dur-admin').value = config.ban_durations.admin || 21600;
  }
}

// Update event types chart
function updateEventTypesChart(eventCounts) {
  const labels = Object.keys(eventCounts);
  const data = Object.values(eventCounts);
  
  eventTypesChart.data.labels = labels;
  eventTypesChart.data.datasets[0].data = data;
  eventTypesChart.update();
}

// Update top IPs chart
function updateTopIpsChart(topIps) {
  const labels = topIps.map(([ip, _]) => ip);
  const data = topIps.map(([_, count]) => count);

  // Reds3 palette (ColorBrewer 3-class reds)
  const reds3 = ["#fee0d2", "#fc9272", "#de2d26"];
  const barColors = data.map((_, i) => reds3[i % reds3.length]);

  topIpsChart.data.labels = labels;
  topIpsChart.data.datasets[0].data = data;
  topIpsChart.data.datasets[0].backgroundColor = barColors;
  topIpsChart.data.datasets[0].borderColor = barColors;
  topIpsChart.update();
}

// Update time series chart
function updateTimeSeriesChart() {
  const endpoint = document.getElementById('endpoint').value;
  const apikey = document.getElementById('apikey').value;

  fetch(`${endpoint}/admin/events?limit=1000`, {
    headers: { 'Authorization': 'Bearer ' + apikey }
  })
  .then(r => {
    if (!r.ok) throw new Error('Failed to fetch events');
    return r.json();
  })
  .then(data => {
    const now = Date.now();
    let cutoffTime;
    
    switch(currentTimeRange) {
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
    }

    // Filter events by time range
    const events = data.recent_events || [];
    const filteredEvents = events.filter(e => {
      const eventTime = e.ts * 1000; // ts is in seconds, convert to milliseconds
      return eventTime >= cutoffTime;
    });

    // Group events by time bucket
    const buckets = {};
    const bucketSize = currentTimeRange === 'hour' ? 300000 : // 5 mins for hour
                       currentTimeRange === 'day' ? 3600000 : // 1 hour for day
                       currentTimeRange === 'week' ? 86400000 : // 1 day for week
                       86400000; // 1 day for month

    // Pre-fill buckets to ensure full time range is shown
    for (let time = cutoffTime; time <= now; time += bucketSize) {
      const bucketKey = Math.floor(time / bucketSize) * bucketSize;
      buckets[bucketKey] = 0;
    }

    // Count events in buckets
    filteredEvents.forEach(event => {
      const eventTime = event.ts * 1000; // ts is in seconds, convert to milliseconds
      const bucketKey = Math.floor(eventTime / bucketSize) * bucketSize;
      buckets[bucketKey] = (buckets[bucketKey] || 0) + 1;
    });

    // Sort by time and prepare chart data
    const sortedBuckets = Object.keys(buckets)
      .map(k => parseInt(k))
      .sort((a, b) => a - b);

    const labels = sortedBuckets.map(time => {
      const date = new Date(time);
      if (currentTimeRange === 'hour') {
        // Hour view: just time
        return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
      } else if (currentTimeRange === 'day') {
        // Day view: date + time
        return date.toLocaleString('en-US', { 
          month: 'short', 
          day: 'numeric', 
          hour: 'numeric', 
          minute: '2-digit' 
        });
      } else {
        // Week/month view: just date
        return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
      }
    });

    const counts = sortedBuckets.map(time => buckets[time]);

    // Use Reds3 palette for time series
    const reds3 = ["#fee0d2", "#fc9272", "#de2d26"];
    timeSeriesChart.data.labels = labels;
    timeSeriesChart.data.datasets[0].data = counts;
    timeSeriesChart.data.datasets[0].borderColor = reds3[2];
    timeSeriesChart.data.datasets[0].backgroundColor = reds3[0];
    timeSeriesChart.update();
  })
  .catch(err => console.error('Failed to update time series:', err));
}

// Update bans table
function updateBansTable(bans) {
  const tbody = document.querySelector('#bans-table tbody');
  tbody.innerHTML = '';
  
  if (bans.length === 0) {
    tbody.innerHTML = '<tr><td colspan="5" style="text-align: center; color: #6b7280;">No active bans</td></tr>';
    return;
  }
  
  for (const ban of bans) {
    const tr = document.createElement('tr');
    const now = Math.floor(Date.now() / 1000);
    const isExpired = ban.expires < now;
    const bannedAt = new Date((ban.expires - 3600) * 1000).toLocaleString(); // Assuming 1h default
    const expiresAt = new Date(ban.expires * 1000).toLocaleString();
    
    tr.innerHTML = `
      <td><code>${ban.ip}</code></td>
      <td>${ban.reason || 'unknown'}</td>
      <td>${bannedAt}</td>
      <td class="${isExpired ? 'expired' : ''}">${isExpired ? 'Expired' : expiresAt}</td>
      <td><button class="unban-quick" data-ip="${ban.ip}">Unban</button></td>
    `;
    tbody.appendChild(tr);
  }
  
  // Add click handlers for quick unban buttons
  document.querySelectorAll('.unban-quick').forEach(btn => {
    btn.onclick = async function() {
      const ip = this.dataset.ip;
      const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
      const apikey = document.getElementById('apikey').value;
      const msg = document.getElementById('admin-msg');
      
      msg.textContent = `Unbanning ${ip}...`;
      msg.className = 'message info';
      
      try {
        await window.unbanIp(endpoint, apikey, ip);
        msg.textContent = `✓ Unbanned ${ip}`;
        msg.className = 'message success';
        setTimeout(() => document.getElementById('refresh').click(), 500);
      } catch (e) {
        msg.textContent = '✗ Error: ' + e.message;
        msg.className = 'message error';
      }
    };
  });
}

// Update events table
function updateEventsTable(events) {
  const tbody = document.querySelector('#events tbody');
  tbody.innerHTML = '';
  
  if (!events || events.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No recent events</td></tr>';
    return;
  }
  
  for (const ev of events) {
    const tr = document.createElement('tr');
    const eventClass = ev.event.toLowerCase();
    tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><span class="badge ${eventClass}">${ev.event}</span></td>
      <td><code>${ev.ip || '-'}</code></td>
      <td>${ev.reason || '-'}</td>
      <td>${ev.outcome || '-'}</td>
      <td>${ev.admin || '-'}</td>
    `;
    tbody.appendChild(tr);
  }
}

// Update maze stats section
function updateMazeStats(data) {
  document.getElementById('maze-total-hits').textContent = 
    data.total_hits?.toLocaleString() || '0';
  document.getElementById('maze-unique-crawlers').textContent = 
    data.unique_crawlers?.toLocaleString() || '0';
  document.getElementById('maze-auto-bans').textContent = 
    data.maze_auto_bans?.toLocaleString() || '0';
  
  // Update crawler list
  const crawlerList = document.getElementById('maze-crawler-list');
  const crawlers = data.top_crawlers || [];
  
  if (crawlers.length === 0) {
    crawlerList.innerHTML = '<p class="no-data">No crawlers in maze yet</p>';
    return;
  }
  
  crawlerList.innerHTML = crawlers.map(crawler => {
    const isHigh = crawler.hits >= 30;
    return `
      <div class="crawler-item">
        <span class="crawler-ip">${crawler.ip}</span>
        <span class="crawler-hits ${isHigh ? 'high' : ''}">${crawler.hits} pages</span>
      </div>
    `;
  }).join('');
}

// Update maze config controls from loaded config
function updateMazeConfig(config) {
  if (config.maze_enabled !== undefined) {
    document.getElementById('maze-enabled-toggle').checked = config.maze_enabled;
  }
  if (config.maze_auto_ban !== undefined) {
    document.getElementById('maze-auto-ban-toggle').checked = config.maze_auto_ban;
  }
  if (config.maze_auto_ban_threshold !== undefined) {
    document.getElementById('maze-threshold').value = config.maze_auto_ban_threshold;
  }
}

// Update robots.txt config controls from loaded config
// Track saved state for change detection
let robotsSavedState = {
  enabled: true,
  blockTraining: true,
  blockSearch: false,
  allowSearch: false,  // This is the toggle state (inverted from allow_search_engines)
  crawlDelay: 2
};

// Track CDP detection saved state for change detection
let cdpSavedState = {
  enabled: true,
  autoBan: true,
  threshold: 0.6
};

function updateRobotsConfig(config) {
  // Update toggles from server config
  if (config.robots_enabled !== undefined) {
    document.getElementById('robots-enabled-toggle').checked = config.robots_enabled;
  }
  if (config.robots_block_ai_training !== undefined) {
    document.getElementById('robots-block-training-toggle').checked = config.robots_block_ai_training;
  }
  if (config.robots_block_ai_search !== undefined) {
    document.getElementById('robots-block-search-toggle').checked = config.robots_block_ai_search;
  }
  if (config.robots_allow_search_engines !== undefined) {
    // Invert: toggle ON = restrict (allow=false), toggle OFF = allow (allow=true)
    document.getElementById('robots-allow-search-toggle').checked = !config.robots_allow_search_engines;
  }
  if (config.robots_crawl_delay !== undefined) {
    document.getElementById('robots-crawl-delay').value = config.robots_crawl_delay;
  }
  // Store saved state for change detection (read from DOM after updates)
  robotsSavedState = {
    enabled: document.getElementById('robots-enabled-toggle').checked,
    blockTraining: document.getElementById('robots-block-training-toggle').checked,
    blockSearch: document.getElementById('robots-block-search-toggle').checked,
    allowSearch: document.getElementById('robots-allow-search-toggle').checked,
    crawlDelay: parseInt(document.getElementById('robots-crawl-delay').value) || 2
  };
  // Reset button state
  const btn = document.getElementById('save-robots-config');
  btn.disabled = true;
  btn.textContent = '📝 Update Policy';
}

// Check if robots config has changed from saved state
function checkRobotsConfigChanged() {
  const current = {
    enabled: document.getElementById('robots-enabled-toggle').checked,
    blockTraining: document.getElementById('robots-block-training-toggle').checked,
    blockSearch: document.getElementById('robots-block-search-toggle').checked,
    allowSearch: document.getElementById('robots-allow-search-toggle').checked,
    crawlDelay: parseInt(document.getElementById('robots-crawl-delay').value) || 2
  };
  const changed = (
    current.enabled !== robotsSavedState.enabled ||
    current.blockTraining !== robotsSavedState.blockTraining ||
    current.blockSearch !== robotsSavedState.blockSearch ||
    current.allowSearch !== robotsSavedState.allowSearch ||
    current.crawlDelay !== robotsSavedState.crawlDelay
  );
  const btn = document.getElementById('save-robots-config');
  btn.disabled = !changed;
  if (changed) {
    btn.textContent = '📝 Update Policy';
  }
}

// Add change listeners for robots config controls
['robots-enabled-toggle', 'robots-block-training-toggle', 'robots-block-search-toggle', 'robots-allow-search-toggle'].forEach(id => {
  document.getElementById(id).addEventListener('change', checkRobotsConfigChanged);
});
document.getElementById('robots-crawl-delay').addEventListener('input', checkRobotsConfigChanged);

// Save maze configuration
document.getElementById('save-maze-config').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const btn = this;
  
  const mazeEnabled = document.getElementById('maze-enabled-toggle').checked;
  const mazeAutoBan = document.getElementById('maze-auto-ban-toggle').checked;
  const mazeThreshold = parseInt(document.getElementById('maze-threshold').value) || 50;
  
  btn.textContent = 'Saving...';
  btn.disabled = true;
  
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        maze_enabled: mazeEnabled,
        maze_auto_ban: mazeAutoBan,
        maze_auto_ban_threshold: mazeThreshold
      })
    });
    
    if (!resp.ok) throw new Error('Failed to save config');
    
    btn.textContent = '✓ Saved!';
    setTimeout(() => {
      btn.textContent = '💾 Save Maze Settings';
      btn.disabled = false;
    }, 1500);
  } catch (e) {
    btn.textContent = '✗ Error';
    console.error('Failed to save maze config:', e);
    setTimeout(() => {
      btn.textContent = '💾 Save Maze Settings';
      btn.disabled = false;
    }, 2000);
  }
};

// Save robots.txt configuration
document.getElementById('save-robots-config').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const btn = this;
  
  const robotsEnabled = document.getElementById('robots-enabled-toggle').checked;
  const blockTraining = document.getElementById('robots-block-training-toggle').checked;
  const blockSearch = document.getElementById('robots-block-search-toggle').checked;
  // Invert: toggle ON = restrict (allow=false), toggle OFF = allow (allow=true)
  const allowSearchEngines = !document.getElementById('robots-allow-search-toggle').checked;
  const crawlDelay = parseInt(document.getElementById('robots-crawl-delay').value) || 2;
  
  btn.textContent = 'Saving...';
  btn.disabled = true;
  
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        robots_enabled: robotsEnabled,
        robots_block_ai_training: blockTraining,
        robots_block_ai_search: blockSearch,
        robots_allow_search_engines: allowSearchEngines,
        robots_crawl_delay: crawlDelay
      })
    });
    
    if (!resp.ok) throw new Error('Failed to save config');
    
    btn.textContent = '✓ Updated!';
    // Update saved state to current values (store toggle states, not server values)
    robotsSavedState = {
      enabled: robotsEnabled,
      blockTraining: blockTraining,
      blockSearch: blockSearch,
      allowSearch: document.getElementById('robots-allow-search-toggle').checked,  // Store toggle state
      crawlDelay: crawlDelay
    };
    // Refresh preview if it's visible
    const preview = document.getElementById('robots-preview');
    if (!preview.classList.contains('hidden')) {
      refreshRobotsPreview();
    }
    setTimeout(() => {
      btn.textContent = '📝 Update Policy';
      btn.disabled = true; // Disable since we just saved
    }, 1500);
  } catch (e) {
    btn.textContent = '✗ Error';
    console.error('Failed to save robots config:', e);
    setTimeout(() => {
      btn.textContent = '📝 Update Policy';
      btn.disabled = false; // Keep enabled so they can retry
    }, 2000);
  }
};

// Fetch and update robots.txt preview content
async function refreshRobotsPreview() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const previewContent = document.getElementById('robots-preview-content');
  
  try {
    const resp = await fetch(endpoint + '/admin/robots', {
      headers: { 'Authorization': 'Bearer ' + apikey }
    });
    
    if (!resp.ok) throw new Error('Failed to fetch robots preview');
    
    const data = await resp.json();
    previewContent.textContent = data.preview || '# No preview available';
  } catch (e) {
    previewContent.textContent = '# Error loading preview: ' + e.message;
    console.error('Failed to load robots preview:', e);
  }
}

// Toggle robots.txt preview visibility
document.getElementById('preview-robots').onclick = async function() {
  const preview = document.getElementById('robots-preview');
  const btn = this;
  
  if (preview.classList.contains('hidden')) {
    // Show preview
    btn.textContent = 'Loading...';
    btn.disabled = true;
    await refreshRobotsPreview();
    preview.classList.remove('hidden');
    btn.textContent = '🙈 Hide robots.txt';
    btn.disabled = false;
  } else {
    // Hide preview
    preview.classList.add('hidden');
    btn.textContent = '👁️ Show robots.txt';
  }
};

// Update CDP detection config controls from loaded config
function updateCdpConfig(config) {
  if (config.cdp_detection_enabled !== undefined) {
    document.getElementById('cdp-enabled-toggle').checked = config.cdp_detection_enabled;
  }
  if (config.cdp_auto_ban !== undefined) {
    document.getElementById('cdp-auto-ban-toggle').checked = config.cdp_auto_ban;
  }
  if (config.cdp_detection_threshold !== undefined) {
    document.getElementById('cdp-threshold-slider').value = config.cdp_detection_threshold;
     document.getElementById('cdp-threshold-value').textContent = parseFloat(config.cdp_detection_threshold).toFixed(1);
  }
  // Store saved state for change detection
  cdpSavedState = {
    enabled: document.getElementById('cdp-enabled-toggle').checked,
    autoBan: document.getElementById('cdp-auto-ban-toggle').checked,
    threshold: parseFloat(document.getElementById('cdp-threshold-slider').value)
  };
  // Reset button state
  const btn = document.getElementById('save-cdp-config');
  btn.disabled = true;
  btn.textContent = '💾 Save CDP Settings';
}

// Check if CDP config has changed from saved state
function checkCdpConfigChanged() {
  const current = {
    enabled: document.getElementById('cdp-enabled-toggle').checked,
    autoBan: document.getElementById('cdp-auto-ban-toggle').checked,
    threshold: parseFloat(document.getElementById('cdp-threshold-slider').value)
  };
  const changed = (
    current.enabled !== cdpSavedState.enabled ||
    current.autoBan !== cdpSavedState.autoBan ||
    current.threshold !== cdpSavedState.threshold
  );
  const btn = document.getElementById('save-cdp-config');
  btn.disabled = !changed;
}

// Update threshold display when slider moves
document.getElementById('cdp-threshold-slider').addEventListener('input', function() {
  document.getElementById('cdp-threshold-value').textContent = this.value;
    document.getElementById('cdp-threshold-value').textContent = parseFloat(this.value).toFixed(1);
    checkCdpConfigChanged();
});

// Add change listeners for CDP config controls
['cdp-enabled-toggle', 'cdp-auto-ban-toggle'].forEach(id => {
  document.getElementById(id).addEventListener('change', checkCdpConfigChanged);
});

// Save CDP detection configuration
document.getElementById('save-cdp-config').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const btn = this;
  
  const cdpEnabled = document.getElementById('cdp-enabled-toggle').checked;
  const cdpAutoBan = document.getElementById('cdp-auto-ban-toggle').checked;
  const cdpThreshold = parseFloat(document.getElementById('cdp-threshold-slider').value);
  
  btn.textContent = 'Saving...';
  btn.disabled = true;
  
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        cdp_detection_enabled: cdpEnabled,
        cdp_auto_ban: cdpAutoBan,
        cdp_detection_threshold: cdpThreshold
      })
    });
    
    if (!resp.ok) throw new Error('Failed to save config');
    
    btn.textContent = '✓ Saved!';
    // Update saved state to current values
    cdpSavedState = {
      enabled: cdpEnabled,
      autoBan: cdpAutoBan,
      threshold: cdpThreshold
    };
    setTimeout(() => {
      btn.textContent = '💾 Save CDP Settings';
      btn.disabled = true;
    }, 1500);
  } catch (e) {
    btn.textContent = '✗ Error';
    console.error('Failed to save CDP config:', e);
    setTimeout(() => {
      btn.textContent = '💾 Save CDP Settings';
      btn.disabled = false;
    }, 2000);
  }
};

// Fetch CDP stats from admin endpoint
async function refreshCdpStats() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  
  try {
    const resp = await fetch(endpoint + '/admin/cdp', {
      headers: { 'Authorization': 'Bearer ' + apikey }
    });
    
    if (!resp.ok) return;
    
    const data = await resp.json();
    if (data.stats) {
      document.getElementById('cdp-total-detections').textContent = data.stats.total_detections || 0;
      document.getElementById('cdp-auto-bans').textContent = data.stats.auto_bans || 0;
    }
  } catch (e) {
    console.error('Failed to load CDP stats:', e);
  }
}

// Main refresh function
document.getElementById('refresh').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  
  // Show loading state
  document.getElementById('total-bans').textContent = '...';
  document.getElementById('active-bans').textContent = '...';
  document.getElementById('total-events').textContent = '...';
  document.getElementById('unique-ips').textContent = '...';

  try {
    // Fetch all data in parallel
    const [analyticsResp, eventsResp, bansResp, mazeResp] = await Promise.all([
      fetch(endpoint + '/admin/analytics', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/events?hours=24', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/ban', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/maze', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      })
    ]);

    if (!analyticsResp.ok || !eventsResp.ok || !bansResp.ok) {
      throw new Error('Failed to fetch data. Check API key and endpoint.');
    }

    const analytics = await analyticsResp.json();
    const events = await eventsResp.json();
    const bansData = await bansResp.json();
    const mazeData = mazeResp.ok ? await mazeResp.json() : null;

    // Update all sections
    updateStatCards(analytics, events, bansData.bans || []);
    updateEventTypesChart(events.event_counts || {});
    updateTopIpsChart(events.top_ips || []);
    updateTimeSeriesChart();
    updateBansTable(bansData.bans || []);
    updateEventsTable(events.recent_events || []);
    
    // Update maze stats
    if (mazeData) {
      updateMazeStats(mazeData);
    }
    
    // Fetch and update ban durations from config
    try {
      const configResp = await fetch(endpoint + '/admin/config', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      });
      if (configResp.ok) {
        const config = await configResp.json();
        updateBanDurations(config);
        updateMazeConfig(config);
        updateRobotsConfig(config);
        updateCdpConfig(config);
      }
    } catch (e) {
      console.error('Failed to load config:', e);
    }
    
    // Fetch CDP stats
    refreshCdpStats();
    
    // Update last updated time
    document.getElementById('last-updated').textContent = 
      'Last updated: ' + new Date().toLocaleTimeString();
    document.getElementById('last-updated').style.color = '#10b981';
    
  } catch (e) {
    console.error('Dashboard refresh error:', e);
    document.getElementById('last-updated').textContent = 'Error: ' + e.message;
    document.getElementById('last-updated').style.color = '#ef4444';
  }
};

// Admin controls - Ban IP
document.getElementById('ban-btn').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const ip = document.getElementById('ban-ip').value.trim();
  const reason = document.getElementById('ban-reason').value.trim() || 'manual_ban';
  const duration = parseInt(document.getElementById('ban-duration').value) || 3600;
  const msg = document.getElementById('admin-msg');
  
  if (!ip) { 
    msg.textContent = '⚠ Enter an IP to ban.';
    msg.className = 'message warning';
    return;
  }
  
  msg.textContent = `Banning ${ip}...`;
  msg.className = 'message info';
  
  try {
    await window.banIp(endpoint, apikey, ip, reason, duration);
    msg.textContent = `✓ Banned ${ip} for ${duration}s`;
    msg.className = 'message success';
    document.getElementById('ban-ip').value = '';
    setTimeout(() => document.getElementById('refresh').click(), 500);
  } catch (e) {
    msg.textContent = '✗ Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Admin controls - Unban IP
document.getElementById('unban-btn').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const ip = document.getElementById('unban-ip').value.trim();
  const msg = document.getElementById('admin-msg');
  
  if (!ip) {
    msg.textContent = '⚠ Enter an IP to unban.';
    msg.className = 'message warning';
    return;
  }
  
  msg.textContent = `Unbanning ${ip}...`;
  msg.className = 'message info';
  
  try {
    await window.unbanIp(endpoint, apikey, ip);
    msg.textContent = `✓ Unbanned ${ip}`;
    msg.className = 'message success';
    document.getElementById('unban-ip').value = '';
    setTimeout(() => document.getElementById('refresh').click(), 500);
  } catch (e) {
    msg.textContent = '✗ Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Save Ban Durations Handler
document.getElementById('save-durations-btn').onclick = async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const msg = document.getElementById('admin-msg');
  
  const ban_durations = {
    honeypot: parseInt(document.getElementById('dur-honeypot').value) || 86400,
    rate_limit: parseInt(document.getElementById('dur-rate-limit').value) || 3600,
    browser: parseInt(document.getElementById('dur-browser').value) || 21600,
    admin: parseInt(document.getElementById('dur-admin').value) || 21600
  };
  
  msg.textContent = 'Saving ban durations...';
  msg.className = 'message info';
  
  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ ban_durations })
    });
    
    if (!resp.ok) {
      throw new Error('Failed to save config');
    }
    
    const data = await resp.json();
    msg.textContent = '✓ Ban durations saved';
    msg.className = 'message success';
  } catch (e) {
    msg.textContent = '✗ Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Initialize charts and load data on page load
initCharts();
document.getElementById('refresh').click();

// Test Mode Toggle Handler
document.getElementById('test-mode-toggle').addEventListener('change', async function() {
  const endpoint = document.getElementById('endpoint').value.replace(/\/$/, '');
  const apikey = document.getElementById('apikey').value;
  const msg = document.getElementById('admin-msg');
  const testMode = this.checked;
  
  msg.textContent = `${testMode ? 'Enabling' : 'Disabling'} test mode...`;
  msg.className = 'message info';
  
  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ test_mode: testMode })
    });
    
    if (!resp.ok) {
      throw new Error('Failed to update config');
    }
    
    const data = await resp.json();
    msg.textContent = `✓ Test mode ${data.config.test_mode ? 'enabled' : 'disabled'}`;
    msg.className = 'message success';
    
    // Refresh dashboard to update banner
    setTimeout(() => document.getElementById('refresh').click(), 500);
  } catch (e) {
    msg.textContent = '✗ Error: ' + e.message;
    msg.className = 'message error';
    // Revert toggle on error
    this.checked = !testMode;
  }
});

// Auto-refresh every 30 seconds
setInterval(() => {
  document.getElementById('refresh').click();
}, 30000);
