<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';

  export let loading = false;
  export let cdpDetections = 0;
  export let cdpAutoBans = 0;
  export let cdpFingerprintEvents = 0;
  export let cdpFingerprintFlowViolations = 0;
  export let recentCdpEvents = [];
  export let formatTime = () => '-';
  export let readCdpField = () => '-';
</script>

<div class="section events">
  <h2>CDP Detections</h2>
  <p class="section-desc text-muted">Browser automation detection and bans in the last 24hrs</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Total Detections</h3>
      <div class="stat-value stat-value" id="cdp-total-detections">{loading ? '...' : formatCompactNumber(cdpDetections, '0')}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Auto-Bans</h3>
      <div class="stat-value stat-value" id="cdp-total-auto-bans">{loading ? '...' : formatCompactNumber(cdpAutoBans, '0')}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">FP Mismatch Events</h3>
      <div class="stat-value stat-value" id="cdp-fp-events">{loading ? '...' : formatCompactNumber(cdpFingerprintEvents, '0')}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">FP Flow Violations</h3>
      <div class="stat-value stat-value" id="cdp-fp-flow-violations">{loading ? '...' : formatCompactNumber(cdpFingerprintFlowViolations, '0')}</div>
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
      <tbody>
        {#if recentCdpEvents.length === 0}
          <tr><td colspan="6" style="text-align: center; color: #6b7280;">No CDP detections or auto-bans in the selected window</td></tr>
        {:else}
          {#each recentCdpEvents as ev}
            {@const reason = String(ev.reason || '')}
            {@const outcome = String(ev.outcome || '-')}
            {@const isBan = reason.toLowerCase() === 'cdp_automation'}
            {@const tierSource = isBan ? outcome : reason}
            {@const tier = readCdpField(tierSource, 'tier').toUpperCase()}
            {@const score = readCdpField(tierSource, 'score')}
            {@const details = isBan
              ? `Auto-ban: ${outcome}`
              : (outcome.toLowerCase().startsWith('checks:') ? outcome.replace(/^checks:/i, 'Checks: ') : outcome)}
            <tr>
              <td>{formatTime(ev.ts)}</td>
              <td><code>{ev.ip || '-'}</code></td>
              <td><span class={`badge ${isBan ? 'ban' : 'challenge'}`}>{isBan ? 'BAN' : 'DETECTION'}</span></td>
              <td>{tier}</td>
              <td>{score}</td>
              <td>{details}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>
