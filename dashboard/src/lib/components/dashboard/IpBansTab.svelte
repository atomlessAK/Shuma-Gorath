<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TableWrapper from './primitives/TableWrapper.svelte';

  export let managed = false;
  export let isActive = false;
</script>

<section
  id="dashboard-panel-ip-bans"
  class="admin-group admin-group--status"
  data-dashboard-tab-panel="ip-bans"
  aria-labelledby="dashboard-tab-ip-bans"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
          <TabStateMessage tab="ip-bans" />
          <TableWrapper>
            <table id="bans-table" class="panel panel-border bans-table-admin">
              <thead>
                <tr>
                  <th class="caps-label">IP Address</th>
                  <th class="caps-label">Reason</th>
                  <th class="caps-label">Banned At</th>
                  <th class="caps-label">Expires</th>
                  <th class="caps-label">Signals</th>
                  <th class="caps-label">Actions</th>
                </tr>
              </thead>
              <tbody></tbody>
            </table>
          </TableWrapper>
          <div class="controls-grid controls-grid--manual">
            <div class="control-group panel-soft pad-md">
              <h3>Ban IP</h3>
              <input id="ban-ip" class="input-field" type="text" placeholder="IP address" aria-label="IP address to ban" maxlength="45" spellcheck="false" autocomplete="off" />
              <input id="ban-reason" class="input-field" type="text" value="manual_ban" aria-label="Ban reason (fixed)" readonly disabled />
              <label class="control-label" for="ban-duration-days">Duration</label>
              <div class="duration-inputs">
                <label class="duration-input" for="ban-duration-days">
                  <input id="ban-duration-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" value="0" aria-label="Manual ban duration days" />
                  <span class="input-unit">days</span>
                </label>
                <label class="duration-input" for="ban-duration-hours">
                  <input id="ban-duration-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" value="1" aria-label="Manual ban duration hours" />
                  <span class="input-unit">hrs</span>
                </label>
                <label class="duration-input" for="ban-duration-minutes">
                  <input id="ban-duration-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" value="0" aria-label="Manual ban duration minutes" />
                  <span class="input-unit">mins</span>
                </label>
              </div>
              <button id="ban-btn" class="btn btn-submit">Ban</button>
            </div>
            <div class="control-group panel-soft pad-md">
              <h3>Unban IP</h3>
              <input id="unban-ip" class="input-field" type="text" placeholder="IP address" aria-label="IP address to unban" maxlength="45" spellcheck="false" autocomplete="off" />
              <button id="unban-btn" class="btn btn-submit">Unban</button>
            </div>
          </div>
</section>
