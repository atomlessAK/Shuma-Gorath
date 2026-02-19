<script>
  export let loading = false;
  export let challengeSummary = {
    totalFailures: '0',
    uniqueOffenders: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let challengeReasonRows = [];
  export let challengeTrendCanvas = null;
</script>

<div class="section events">
  <h2>Challenge Failures</h2>
  <p class="section-desc text-muted">Challenge submit failures by reason class with trend and top offender.</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label">Total Failures</h3>
      <div class="stat-value" id="challenge-failures-total">{loading ? '...' : challengeSummary.totalFailures}</div>
    </div>
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label">Unique Offenders</h3>
      <div class="stat-value" id="challenge-failures-unique">{loading ? '...' : challengeSummary.uniqueOffenders}</div>
    </div>
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label" id="challenge-top-offender-label">{loading ? 'Top Offender' : challengeSummary.topOffender.label}</h3>
      <div class="stat-value" id="challenge-top-offender">{loading ? '...' : challengeSummary.topOffender.value}</div>
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
