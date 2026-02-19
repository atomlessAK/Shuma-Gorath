<script>
  import StatCard from '../primitives/StatCard.svelte';

  export let loading = false;
  export let totalBans = 0;
  export let activeBans = 0;
  export let eventCount = 0;
  export let uniqueIps = 0;
</script>

<div class="stats-cards">
  <StatCard title="Total Bans">
    <div class="stat-value" id="total-bans">{loading ? '...' : totalBans.toLocaleString()}</div>
  </StatCard>
  <StatCard title="Active Bans">
    <div class="stat-value" id="active-bans">{loading ? '...' : activeBans.toLocaleString()}</div>
  </StatCard>
  <StatCard title="Events (24h)">
    <div class="stat-value" id="total-events">{loading ? '...' : eventCount.toLocaleString()}</div>
  </StatCard>
  <StatCard title="Unique IPs">
    <div class="stat-value" id="unique-ips">{loading ? '...' : uniqueIps.toLocaleString()}</div>
  </StatCard>
</div>
