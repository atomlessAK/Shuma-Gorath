<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';

  export let loading = false;
  export let geoSummary = {
    totalViolations: '0',
    actionMix: {
      block: 0,
      challenge: 0,
      maze: 0
    }
  };
  export let geoTopCountries = [];
</script>

<div class="section events">
  <h2>GEO Violations</h2>
  <p class="section-desc text-muted">GEO policy actions by route and top country sources.</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Total Violations</h3>
      <div class="stat-value" id="geo-violations-total">{loading ? '...' : geoSummary.totalViolations}</div>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Action Mix</h3>
      <div class="stat-value stat-value--small" id="geo-action-mix">block {geoSummary.actionMix.block} | challenge {geoSummary.actionMix.challenge} | maze {geoSummary.actionMix.maze}</div>
    </div>
  </div>
  <div class="panel panel-border pad-md-b">
    <h3>Top Countries Triggering GEO Actions</h3>
    <div id="geo-top-countries" class="crawler-list">
      {#if geoTopCountries.length === 0}
        <p class="no-data">No GEO violations yet</p>
      {:else}
        {#each geoTopCountries as row}
          <div class="crawler-item panel panel-border">
            <span class="crawler-ip">{row.country}</span>
            <span class="crawler-hits">{formatCompactNumber(row.count, '0')} actions</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
