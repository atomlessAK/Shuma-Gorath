<script>
  import StatCard from '../primitives/StatCard.svelte';
  import { formatCompactNumber } from '../../../domain/core/format.js';

  export let loading = false;
  export let totalBans = 0;
  export let activeBans = 0;
  export let eventCount = 0;
  export let uniqueIps = 0;
</script>

<div class="stats-cards">
  <StatCard title="Total Bans">
    <div class="stat-value" id="total-bans">{loading ? '...' : formatCompactNumber(totalBans, '0')}</div>
  </StatCard>
  <StatCard title="Active Bans">
    <div class="stat-value" id="active-bans">{loading ? '...' : formatCompactNumber(activeBans, '0')}</div>
  </StatCard>
  <StatCard title="Events (24h)">
    <div class="stat-value" id="total-events">{loading ? '...' : formatCompactNumber(eventCount, '0')}</div>
  </StatCard>
  <StatCard title="Unique IPs">
    <div class="stat-value" id="unique-ips">{loading ? '...' : formatCompactNumber(uniqueIps, '0')}</div>
  </StatCard>
</div>
