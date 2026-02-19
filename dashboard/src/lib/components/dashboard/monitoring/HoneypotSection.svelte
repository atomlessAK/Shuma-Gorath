<script>
  export let loading = false;
  export let honeypot = {
    totalHits: '0',
    uniqueCrawlers: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let topPaths = [];
</script>

<div class="section events">
  <h2>Honeypot Hits</h2>
  <p class="section-desc text-muted">Structured honeypot telemetry (hits, offender buckets, and trap paths).</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label">Total Hits</h3>
      <span class="stat-value" id="honeypot-total-hits">{loading ? '...' : honeypot.totalHits}</span>
    </div>
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label">Unique Crawlers</h3>
      <span class="stat-value" id="honeypot-unique-crawlers">{loading ? '...' : honeypot.uniqueCrawlers}</span>
    </div>
    <div class="card panel panel-border pad-md">
      <h3 class="caps-label" id="honeypot-top-offender-label">{loading ? 'Top Offender' : honeypot.topOffender.label}</h3>
      <span class="stat-value" id="honeypot-top-offender">{loading ? '...' : honeypot.topOffender.value}</span>
    </div>
  </div>
  <div class="panel panel-border pad-md">
    <h3>Top Honeypot Paths</h3>
    <div id="honeypot-top-paths" class="crawler-list">
      {#if topPaths.length === 0}
        <p class="no-data">No honeypot path data yet</p>
      {:else}
        {#each topPaths as row}
          <div class="crawler-item panel panel-border">
            <span class="crawler-ip">{row.path}</span>
            <span class="crawler-hits">{row.count.toLocaleString()} hits</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
