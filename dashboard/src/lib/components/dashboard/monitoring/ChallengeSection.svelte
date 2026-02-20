<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';

  export let loading = false;
  export let challengeSummary = {
    totalFailures: '0',
    uniqueOffenders: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let notABotSummary = {
    served: '0',
    submitted: '0',
    pass: '0',
    escalate: '0',
    fail: '0',
    replay: '0',
    abandonmentsEstimated: '0',
    abandonmentRate: '0.0%'
  };
  export let challengeReasonRows = [];
  export let notABotOutcomeRows = [];
  export let notABotLatencyRows = [];
  export let challengeTrendCanvas = null;
</script>

<div class="section events">
  <h2>Challenge Failures</h2>
  <p class="section-desc text-muted">Challenge submit failures by reason class with trend and top offender.</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Total Failures</h3>
      <div class="stat-value" id="challenge-failures-total">{loading ? '...' : challengeSummary.totalFailures}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Unique Offenders</h3>
      <div class="stat-value" id="challenge-failures-unique">{loading ? '...' : challengeSummary.uniqueOffenders}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label" id="challenge-top-offender-label">{loading ? 'Top Offender' : challengeSummary.topOffender.label}</h3>
      <div class="stat-value" id="challenge-top-offender">{loading ? '...' : challengeSummary.topOffender.value}</div>
    </div>
  </div>
  <div class="panel panel-border">
    <h3 class="caps-label">Not-a-Bot Outcomes (24h)</h3>
    <div class="stats-cards stats-cards--compact">
      <div class="card panel panel-border pad-md-b">
        <h4 class="caps-label">Served</h4>
        <div class="stat-value stat-value--small">{loading ? '...' : notABotSummary.served}</div>
      </div>
      <div class="card panel panel-border pad-md-b">
        <h4 class="caps-label">Submitted</h4>
        <div class="stat-value stat-value--small">{loading ? '...' : notABotSummary.submitted}</div>
      </div>
      <div class="card panel panel-border pad-md-b">
        <h4 class="caps-label">Pass / Escalate</h4>
        <div class="stat-value stat-value--small">{loading ? '...' : `${notABotSummary.pass} / ${notABotSummary.escalate}`}</div>
      </div>
      <div class="card panel panel-border pad-md-b">
        <h4 class="caps-label">Fail / Replay</h4>
        <div class="stat-value stat-value--small">{loading ? '...' : `${notABotSummary.fail} / ${notABotSummary.replay}`}</div>
      </div>
      <div class="card panel panel-border pad-md-b">
        <h4 class="caps-label">Abandonment</h4>
        <div class="stat-value stat-value--small">{loading ? '...' : `${notABotSummary.abandonmentsEstimated} (${notABotSummary.abandonmentRate})`}</div>
      </div>
    </div>
    <div class="table-wrapper">
      <table class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Outcome</th>
            <th class="caps-label">Count</th>
          </tr>
        </thead>
        <tbody id="not-a-bot-outcomes">
          {#if notABotOutcomeRows.length === 0}
            <tr><td colspan="2" style="text-align: center; color: #6b7280;">No not-a-bot submissions in window</td></tr>
          {:else}
            {#each notABotOutcomeRows as row}
              <tr>
                <td>{row.label}</td>
                <td>{formatCompactNumber(row.count, '0')}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
      <table class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Solve Latency</th>
            <th class="caps-label">Count</th>
          </tr>
        </thead>
        <tbody id="not-a-bot-latency">
          {#if notABotLatencyRows.length === 0}
            <tr><td colspan="2" style="text-align: center; color: #6b7280;">No solve latency data in window</td></tr>
          {:else}
            {#each notABotLatencyRows as row}
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
  <div class="chart-container panel-soft panel-border pad-md-trb">
    <canvas id="challengeFailuresTrendChart" bind:this={challengeTrendCanvas}></canvas>
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
        <tbody id="challenge-failure-reasons">
          {#if challengeReasonRows.length === 0}
            <tr><td colspan="2" style="text-align: center; color: #6b7280;">No failures in window</td></tr>
          {:else}
            {#each challengeReasonRows as row}
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
</div>
