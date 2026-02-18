<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
</script>

<section
  id="dashboard-panel-tuning"
  class="admin-group config-edit-pane hidden"
  data-dashboard-tab-panel="tuning"
  aria-labelledby="dashboard-tab-tuning"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
          <TabStateMessage tab="tuning" />
          <div class="controls-grid controls-grid--tuning">
            <div class="control-group panel-soft pad-md">
              <h3>Botness Scoring</h3>
              <p class="control-desc text-muted">Weighted signals form a unified score. Moderate scores get the challenge; higher scores route to maze.</p>
              <div class="admin-controls">
                <div class="input-row">
                  <label class="control-label" for="challenge-puzzle-threshold">Challenge (score)</label>
                  <input class="input-field" type="number" id="challenge-puzzle-threshold" min="1" max="10" step="1" inputmode="numeric" value="3" aria-label="Challenge risk threshold">
                </div>
                <div class="input-row">
                  <label class="control-label" for="maze-threshold-score">Maze (score)</label>
                  <input class="input-field" type="number" id="maze-threshold-score" min="1" max="10" step="1" inputmode="numeric" value="6" aria-label="Maze risk threshold">
                </div>
                <div class="input-row">
                  <label class="control-label" for="weight-js-required">Weight: JS (points)</label>
                  <input class="input-field" type="number" id="weight-js-required" min="0" max="10" step="1" inputmode="numeric" value="1" aria-label="Weight for JS verification required">
                </div>
                <div class="input-row">
                  <label class="control-label" for="weight-geo-risk">Weight: Geo (points)</label>
                  <input class="input-field" type="number" id="weight-geo-risk" min="0" max="10" step="1" inputmode="numeric" value="2" aria-label="Weight for high-risk geography">
                </div>
                <div class="input-row">
                  <label class="control-label" for="weight-rate-medium">Weight: Rate 50% (points)</label>
                  <input class="input-field" type="number" id="weight-rate-medium" min="0" max="10" step="1" inputmode="numeric" value="1" aria-label="Weight for medium rate pressure">
                </div>
                <div class="input-row">
                  <label class="control-label" for="weight-rate-high">Weight: Rate 80% (points)</label>
                  <input class="input-field" type="number" id="weight-rate-high" min="0" max="10" step="1" inputmode="numeric" value="2" aria-label="Weight for high rate pressure">
                </div>
                <div class="info-panel panel-muted pad-sm">
                  <h4>Status</h4>
                  <div class="info-row">
                    <span class="info-label text-muted">Config:</span>
                    <span id="botness-config-status" class="status-value">--</span>
                  </div>
                  <div class="info-row">
                    <span class="info-label text-muted">Default Challenge:</span>
                    <span id="challenge-puzzle-default">--</span>
                  </div>
                  <div class="info-row">
                    <span class="info-label text-muted">Default Maze:</span>
                    <span id="maze-threshold-default">--</span>
                  </div>
                </div>
                <div class="info-panel panel-muted pad-sm">
                  <h4>Scored Signals</h4>
                  <div id="botness-signal-list"></div>
                </div>
                <div class="info-panel panel-muted pad-sm">
                  <h4>Terminal Signals</h4>
                  <div id="botness-terminal-list"></div>
                </div>
              </div>
              <button id="save-botness-config" class="btn btn-submit" disabled>Save Botness Settings</button>
            </div>
          </div>
        </section>
