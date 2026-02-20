<script>
  export let prometheusHelper = {
    facts: [],
    exampleJs: '',
    copyCurlText: '',
    exampleOutput: '',
    exampleStats: '',
    exampleWindowed: '',
    exampleSummaryStats: '',
    observabilityLink: '',
    apiLink: ''
  };
  export let copyButtonLabel = 'Copy JS Example';
  export let copyCurlButtonLabel = 'Copy Curl Example';
  export let onCopyJs = null;
  export let onCopyCurl = null;

  const copyJs = () => {
    if (typeof onCopyJs === 'function') {
      onCopyJs(prometheusHelper.exampleJs);
    }
  };

  const copyCurl = () => {
    if (typeof onCopyCurl === 'function') {
      onCopyCurl(prometheusHelper.copyCurlText);
    }
  };
</script>

<div class="section panel panel-border">
  <h2>External Monitoring</h2>
  <ul id="monitoring-prometheus-facts" class="prometheus-facts text-muted">
    {#each prometheusHelper.facts as fact}
      <li>{fact}</li>
    {/each}
  </ul>
  <h3 class="caps-label">1) Fetch Full Prometheus Text Payload (Javascript)</h3>
  <pre id="monitoring-prometheus-example" class="prometheus-example prometheus-example--code">{prometheusHelper.exampleJs}</pre>
  <div class="prometheus-copy-actions">
    <button id="monitoring-prometheus-copy" class="btn btn-subtle" type="button" on:click={copyJs}>{copyButtonLabel}</button>
    <button id="monitoring-prometheus-copy-curl" class="btn btn-subtle" type="button" data-copy-text={prometheusHelper.copyCurlText} on:click={copyCurl}>{copyCurlButtonLabel}</button>
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
