<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';

  export let loading = false;
  export let powSummary = {
    totalAttempts: '0',
    totalSuccesses: '0',
    totalFailures: '0',
    uniqueOffenders: '0',
    successRate: '-',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let powReasonRows = [];
  export let powOutcomeRows = [];
  export let powTrendCanvas = null;
</script>

<div class="section events">
  <h2>PoW Verification</h2>
  <p class="section-desc text-muted">PoW verify outcomes with failure reasons, trend, and top offender.</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Total Verifies</h3>
      <div class="stat-value" id="pow-total-attempts">{loading ? '...' : powSummary.totalAttempts}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Total Failures</h3>
      <div class="stat-value" id="pow-failures-total">{loading ? '...' : powSummary.totalFailures}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Unique Offenders</h3>
      <div class="stat-value" id="pow-failures-unique">{loading ? '...' : powSummary.uniqueOffenders}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label" id="pow-top-offender-label">{loading ? 'Top Offender' : powSummary.topOffender.label}</h3>
      <div class="stat-value" id="pow-top-offender">{loading ? '...' : powSummary.topOffender.value}</div>
    </div>
  </div>
  <div class="chart-container panel-soft panel-border pad-md-trb">
    <canvas id="powFailuresTrendChart" bind:this={powTrendCanvas}></canvas>
  </div>
  <div class="panel panel-border">
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
                <td>{formatCompactNumber(row.count, '0')}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
  <div class="panel panel-border pad-md-b">
    <h3>Verify Outcomes</h3>
    <ul id="pow-outcomes-list" class="metric-list">
      {#if powOutcomeRows.length === 0}
        <li class="text-muted">No verify outcomes yet</li>
      {:else}
        {#each powOutcomeRows as row}
          <li><strong>{row.label}:</strong> {formatCompactNumber(row.count, '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>
</div>
