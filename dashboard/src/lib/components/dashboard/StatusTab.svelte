<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let runtimeTelemetry = null;

  const formatMetricMs = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return '-';
    return `${numeric.toFixed(2)} ms`;
  };

  const formatTimestamp = (value) => {
    const raw = String(value || '').trim();
    if (!raw) return '-';
    return raw;
  };

  $: refresh = runtimeTelemetry && runtimeTelemetry.refresh ? runtimeTelemetry.refresh : {};
  $: polling = runtimeTelemetry && runtimeTelemetry.polling ? runtimeTelemetry.polling : {};
</script>

<section
  id="dashboard-panel-status"
  class="admin-group admin-group--status"
  data-dashboard-tab-panel="status"
  aria-labelledby="dashboard-tab-status"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
  <TabStateMessage tab="status" />
  <div class="controls-grid controls-grid--status">
    <div class="control-group panel-soft pad-md">
      <div id="status-items"></div>
    </div>
    <div class="control-group panel-soft pad-md status-inventory-group">
      <h3>Runtime Variable Inventory</h3>
      <p class="control-desc text-muted">
        Complete runtime snapshot of active configuration variables, grouped by concern.
        Rows with highlighted background are runtime admin-writable variables.
      </p>
      <div id="status-vars-groups" class="status-var-groups"></div>
    </div>
    <div class="control-group panel-soft pad-md">
      <h3>Runtime Performance Telemetry</h3>
      <p class="control-desc text-muted">
        Operator thresholds: keep rolling p95 fetch latency under <strong>500 ms</strong>, rolling p95 render
        timing under <strong>16 ms</strong>, and investigate sustained polling skip/resume churn.
      </p>
      <div class="stats-cards stats-cards--compact">
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Fetch Latency (Last / Rolling)</h3>
          <div id="runtime-fetch-latency-last" class="stat-value stat-value--small">
            {formatMetricMs(refresh.fetchLatencyMs?.last)}
          </div>
          <p id="runtime-fetch-latency-avg" class="text-muted">
            avg: {formatMetricMs(refresh.fetchLatencyMs?.avg)} | p95: {formatMetricMs(refresh.fetchLatencyMs?.p95)}
            (window: {refresh.fetchLatencyMs?.samples || 0}/{refresh.fetchLatencyMs?.windowSize || 0}, total: {refresh.fetchLatencyMs?.totalSamples || 0})
          </p>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Render Timing (Last / Rolling)</h3>
          <div id="runtime-render-timing-last" class="stat-value stat-value--small">
            {formatMetricMs(refresh.renderTimingMs?.last)}
          </div>
          <p id="runtime-render-timing-avg" class="text-muted">
            avg: {formatMetricMs(refresh.renderTimingMs?.avg)} | p95: {formatMetricMs(refresh.renderTimingMs?.p95)}
            (window: {refresh.renderTimingMs?.samples || 0}/{refresh.renderTimingMs?.windowSize || 0}, total: {refresh.renderTimingMs?.totalSamples || 0})
          </p>
        </div>
        <div class="card panel panel-border pad-md">
          <h3 class="caps-label">Polling Skip / Resume</h3>
          <div id="runtime-polling-skip-count" class="stat-value stat-value--small">{polling.skips || 0}</div>
          <p id="runtime-polling-resume-count" class="text-muted">resumes: {polling.resumes || 0}</p>
          <p id="runtime-polling-last-skip-reason" class="text-muted">
            last skip: {polling.lastSkipReason || '-'}
          </p>
          <p id="runtime-polling-last-resume-at" class="text-muted">
            last resume: {formatTimestamp(polling.lastResumeAt)}
          </p>
        </div>
      </div>
    </div>
  </div>
</section>
