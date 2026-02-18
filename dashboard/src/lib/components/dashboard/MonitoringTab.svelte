<script>
  import StatCard from './primitives/StatCard.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TableWrapper from './primitives/TableWrapper.svelte';

  export let managed = false;
  export let isActive = true;
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
    <TabStateMessage tab="monitoring" />
    <!-- Stats Summary Cards -->
    <div class="stats-cards">
      <StatCard title="Total Bans">
        <div class="stat-value" id="total-bans">-</div>
      </StatCard>
      <StatCard title="Active Bans">
        <div class="stat-value" id="active-bans">-</div>
      </StatCard>
      <StatCard title="Events (24h)">
        <div class="stat-value" id="total-events">-</div>
      </StatCard>
      <StatCard title="Unique IPs">
        <div class="stat-value" id="unique-ips">-</div>
      </StatCard>
    </div>

    <!-- Charts Row -->
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

    <!-- Events Over Time -->
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

    <!-- Recent Events -->
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
          <tbody></tbody>
        </table>
      </TableWrapper>
    </div>

    <!-- CDP Detections & Bans -->
    <div class="section events">
      <h2>CDP Detections</h2>
      <p class="section-desc text-muted">Browser automation detection and bans in the last 24hrs</p>
      <div class="stats-cards stats-cards--compact">
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Total Detections</h3>
          <div class="stat-value stat-value" id="cdp-total-detections">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Auto-Bans</h3>
          <div class="stat-value stat-value" id="cdp-total-auto-bans">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">FP Mismatch Events</h3>
          <div class="stat-value stat-value" id="cdp-fp-events">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">FP Flow Violations</h3>
          <div class="stat-value stat-value" id="cdp-fp-flow-violations">-</div>
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
          <tbody></tbody>
        </table>
      </div>
    </div>

    <!-- Maze Activity Section -->
    <div class="section events">
      <h2>Maze</h2>
      <p class="section-desc text-muted">Crawlers trapped in infinite fake pages</p>
      <div class="stats-cards stats-cards--compact">
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Total Hits</h3>
          <span class="stat-value" id="maze-total-hits">-</span>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Unique Crawlers</h3>
          <span class="stat-value" id="maze-unique-crawlers">-</span>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Auto-Banned</h3>
          <span class="stat-value" id="maze-auto-bans">-</span>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label" id="maze-top-offender-label">Top Offender</h3>
          <span class="stat-value" id="maze-top-offender">-</span>
        </div>
      </div>
    </div>

    <div class="section events">
      <h2>Honeypot Hits</h2>
      <p class="section-desc text-muted">Structured honeypot telemetry (hits, offender buckets, and trap paths).</p>
      <div class="stats-cards stats-cards--compact">
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Total Hits</h3>
          <span class="stat-value" id="honeypot-total-hits">-</span>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Unique Crawlers</h3>
          <span class="stat-value" id="honeypot-unique-crawlers">-</span>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label" id="honeypot-top-offender-label">Top Offender</h3>
          <span class="stat-value" id="honeypot-top-offender">-</span>
        </div>
      </div>
      <div class="panel panel-border pad-md">
        <h3>Top Honeypot Paths</h3>
        <div id="honeypot-top-paths" class="crawler-list">
          <p class="no-data">No honeypot path data yet</p>
        </div>
      </div>
    </div>

    <div class="section events">
      <h2>Challenge Failures</h2>
      <p class="section-desc text-muted">Challenge submit failures by reason class with trend and top offender.</p>
      <div class="stats-cards stats-cards--compact">
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Total Failures</h3>
          <div class="stat-value" id="challenge-failures-total">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Unique Offenders</h3>
          <div class="stat-value" id="challenge-failures-unique">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label" id="challenge-top-offender-label">Top Offender</h3>
          <div class="stat-value" id="challenge-top-offender">-</div>
        </div>
      </div>
      <div class="chart-container panel-soft panel-border pad-md">
        <canvas id="challengeFailuresTrendChart"></canvas>
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
            <tbody id="challenge-failure-reasons"></tbody>
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
          <div class="stat-value" id="pow-failures-total">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Unique Offenders</h3>
          <div class="stat-value" id="pow-failures-unique">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label" id="pow-top-offender-label">Top Offender</h3>
          <div class="stat-value" id="pow-top-offender">-</div>
        </div>
      </div>
      <div class="chart-container panel-soft panel-border pad-md">
        <canvas id="powFailuresTrendChart"></canvas>
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
            <tbody id="pow-failure-reasons"></tbody>
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
          <div class="stat-value" id="rate-violations-total">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Unique Offenders</h3>
          <div class="stat-value" id="rate-violations-unique">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label" id="rate-top-offender-label">Top Offender</h3>
          <div class="stat-value" id="rate-top-offender">-</div>
        </div>
      </div>
      <div class="panel panel-border pad-md">
        <h3>Enforcement Outcomes</h3>
        <ul id="rate-outcomes-list" class="metric-list"></ul>
      </div>
    </div>

    <div class="section events">
      <h2>GEO Violations</h2>
      <p class="section-desc text-muted">GEO policy actions by route and top country sources.</p>
      <div class="stats-cards stats-cards--compact">
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Total Violations</h3>
          <div class="stat-value" id="geo-violations-total">-</div>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Action Mix</h3>
          <div class="stat-value stat-value--small" id="geo-action-mix">-</div>
        </div>
      </div>
      <div class="panel panel-border pad-md">
        <h3>Top Countries Triggering GEO Actions</h3>
        <div id="geo-top-countries" class="crawler-list">
          <p class="no-data">No GEO violations yet</p>
        </div>
      </div>
    </div>

    <div class="section panel panel-border pad-md">
      <h2>External Monitoring</h2>
      <ul id="monitoring-prometheus-facts" class="prometheus-facts text-muted">
        <li>Loading monitoring guidance...</li>
      </ul>
      <h3 class="caps-label">1) Fetch Full Prometheus Text Payload (Javascript)</h3>
      <pre id="monitoring-prometheus-example" class="prometheus-example prometheus-example--code">// Loading example...</pre>
      <div class="prometheus-copy-actions">
        <button id="monitoring-prometheus-copy" class="btn btn-subtle" type="button">Copy JS Example</button>
        <button id="monitoring-prometheus-copy-curl" class="btn btn-subtle" type="button">Copy Curl Example</button>
      </div>
      <h3 class="caps-label">2) Example Prometheus Text Payload (Truncated Output)</h3>
      <pre id="monitoring-prometheus-output" class="prometheus-example prometheus-example--output"># Loading example output...</pre>
      <h3 class="caps-label">3) Read Selected Metrics From Prometheus Text Payload (Javascript)</h3>
      <pre id="monitoring-prometheus-stats" class="prometheus-example prometheus-example--code">// Loading parser example...</pre>
      <h3 class="caps-label">4) Request JSON Format Bounded Summary &#123;<code>hours</code>/<code>limit</code>&#125; (Javascript)</h3>
      <pre id="monitoring-prometheus-windowed" class="prometheus-example prometheus-example--code">// Loading bounded query example...</pre>
      <h3 class="caps-label">5) Read Specific Summary Stats from JSON. (Javascript)</h3>
      <pre id="monitoring-prometheus-summary-stats" class="prometheus-example prometheus-example--code">// Loading summary field example...</pre>
      <p class="section-desc text-muted">Detailed docs: <a id="monitoring-prometheus-observability-link" href="https://github.com/atomless/Shuma-Gorath/blob/main/docs/observability.md" target="_blank" rel="noopener noreferrer">Observability</a> and <a id="monitoring-prometheus-api-link" href="https://github.com/atomless/Shuma-Gorath/blob/main/docs/api.md" target="_blank" rel="noopener noreferrer">API</a>.</p>
    </div>
    </section>
