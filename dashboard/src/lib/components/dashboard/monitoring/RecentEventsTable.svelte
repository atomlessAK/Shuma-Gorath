<script>
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let recentEvents = [];
  export let formatTime = () => '-';
  export let eventBadgeClass = () => 'badge';
</script>

<div class="section events">
  <h2>Recent Events</h2>
  <p class="section-desc text-muted">Last 100 recorded events</p>
  <TableWrapper>
    <table id="events" class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Time</th>
          <th class="caps-label">Type</th>
          <th class="caps-label">IP</th>
          <th class="caps-label">Reason</th>
          <th class="caps-label">Outcome</th>
          <th class="caps-label">Admin</th>
        </tr>
      </thead>
      <tbody>
        {#if recentEvents.length === 0}
          <tr><td colspan="6" style="text-align: center; color: #6b7280;">No recent events</td></tr>
        {:else}
          {#each recentEvents as ev}
            <tr>
              <td>{formatTime(ev.ts)}</td>
              <td><span class={eventBadgeClass(ev.event)}>{ev.event || '-'}</span></td>
              <td><code>{ev.ip || '-'}</code></td>
              <td>{ev.reason || '-'}</td>
              <td>{ev.outcome || '-'}</td>
              <td>{ev.admin || '-'}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</div>
