<script>
  export let loading = false;
  export let mazeStats = {
    totalHits: '0',
    uniqueCrawlers: '0',
    mazeAutoBans: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
</script>

<div class="section events">
  <h2>Maze</h2>
  <p class="section-desc text-muted">Crawlers trapped in infinite fake pages</p>
  <div class="stats-cards stats-cards--compact">
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Total Hits</h3>
      <span class="stat-value" id="maze-total-hits">{loading ? '...' : mazeStats.totalHits}</span>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Unique Crawlers</h3>
      <span class="stat-value" id="maze-unique-crawlers">{loading ? '...' : mazeStats.uniqueCrawlers}</span>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label">Auto-Banned</h3>
      <span class="stat-value" id="maze-auto-bans">{loading ? '...' : mazeStats.mazeAutoBans}</span>
    </div>
    <div class="card panel panel-border pad-md-b">
      <h3 class="caps-label" id="maze-top-offender-label">{loading ? 'Top Offender' : mazeStats.topOffender.label}</h3>
      <span class="stat-value" id="maze-top-offender">{loading ? '...' : mazeStats.topOffender.value}</span>
    </div>
  </div>
</div>
