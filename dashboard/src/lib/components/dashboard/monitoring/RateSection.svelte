<script>
  export let loading = false;
  export let rateSummary = {
    totalViolations: '0',
    uniqueOffenders: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let rateOutcomeRows = [];
</script>

<div class="section events">
  <h2>Rate Limiting Violations</h2>
  <p class="section-desc text-muted">Rate-limit outcomes and top offender bucket.</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label">Total Violations</h3>
      <div class="stat-value" id="rate-violations-total">{loading ? '...' : rateSummary.totalViolations}</div>
    </div>
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label">Unique Offenders</h3>
      <div class="stat-value" id="rate-violations-unique">{loading ? '...' : rateSummary.uniqueOffenders}</div>
    </div>
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label" id="rate-top-offender-label">{loading ? 'Top Offender' : rateSummary.topOffender.label}</h3>
      <div class="stat-value" id="rate-top-offender">{loading ? '...' : rateSummary.topOffender.value}</div>
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
