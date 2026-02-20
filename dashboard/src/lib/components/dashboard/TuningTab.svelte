<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;

  let notABotThreshold = 2;
  let challengeThreshold = 3;
  let mazeThreshold = 6;
  let weightJsRequired = 1;
  let weightGeoRisk = 2;
  let weightRateMedium = 1;
  let weightRateHigh = 2;
  let savingBotness = false;

  let baseline = {
    notABotThreshold: 2,
    challengeThreshold: 3,
    mazeThreshold: 6,
    weightJsRequired: 1,
    weightGeoRisk: 2,
    weightRateMedium: 1,
    weightRateHigh: 2
  };
  let lastAppliedConfigVersion = -1;

  const parseInteger = (value, fallback) => {
    const parsed = Number.parseInt(value, 10);
    return Number.isInteger(parsed) ? parsed : fallback;
  };

  const inRange = (value, min, max) => {
    const parsed = Number.parseInt(value, 10);
    return Number.isInteger(parsed) && parsed >= min && parsed <= max;
  };

  const toBotnessBaseline = (config = {}) => {
    const weights = config && typeof config.botness_weights === 'object'
      ? config.botness_weights
      : {};
    return {
      notABotThreshold: parseInteger(config.not_a_bot_risk_threshold, 2),
      challengeThreshold: parseInteger(config.challenge_puzzle_risk_threshold, 3),
      mazeThreshold: parseInteger(config.botness_maze_threshold, 6),
      weightJsRequired: parseInteger(weights.js_required, 1),
      weightGeoRisk: parseInteger(weights.geo_risk, 2),
      weightRateMedium: parseInteger(weights.rate_medium, 1),
      weightRateHigh: parseInteger(weights.rate_high, 2)
    };
  };

  function applyConfig(config = {}) {
    const next = toBotnessBaseline(config);
    baseline = next;
    notABotThreshold = next.notABotThreshold;
    challengeThreshold = next.challengeThreshold;
    mazeThreshold = next.mazeThreshold;
    weightJsRequired = next.weightJsRequired;
    weightGeoRisk = next.weightGeoRisk;
    weightRateMedium = next.weightRateMedium;
    weightRateHigh = next.weightRateHigh;
  }

  async function saveBotness() {
    if (!botnessValid || !botnessDirty || !writable || typeof onSaveConfig !== 'function') return;
    savingBotness = true;
    const payload = {
      not_a_bot_risk_threshold: Number(notABotThreshold),
      challenge_puzzle_risk_threshold: Number(challengeThreshold),
      botness_maze_threshold: Number(mazeThreshold),
      botness_weights: {
        js_required: Number(weightJsRequired),
        geo_risk: Number(weightGeoRisk),
        rate_medium: Number(weightRateMedium),
        rate_high: Number(weightRateHigh)
      }
    };

    try {
      await onSaveConfig(payload, { successMessage: 'Botness scoring saved' });
      baseline = {
        notABotThreshold: Number(notABotThreshold),
        challengeThreshold: Number(challengeThreshold),
        mazeThreshold: Number(mazeThreshold),
        weightJsRequired: Number(weightJsRequired),
        weightGeoRisk: Number(weightGeoRisk),
        weightRateMedium: Number(weightRateMedium),
        weightRateHigh: Number(weightRateHigh)
      };
    } finally {
      savingBotness = false;
    }
  }

  $: writable = configSnapshot && configSnapshot.admin_config_write_enabled === true;
  $: notABotDefault = parseInteger(configSnapshot?.not_a_bot_risk_threshold_default, 2);
  $: challengeDefault = parseInteger(configSnapshot?.challenge_puzzle_risk_threshold_default, 3);
  $: mazeDefault = parseInteger(configSnapshot?.botness_maze_threshold_default, 6);
  $: signalDefinitions = configSnapshot && typeof configSnapshot.botness_signal_definitions === 'object'
    ? configSnapshot.botness_signal_definitions
    : {};
  $: scoredSignals = Array.isArray(signalDefinitions.scored_signals)
    ? signalDefinitions.scored_signals
    : [];
  $: terminalSignals = Array.isArray(signalDefinitions.terminal_signals)
    ? signalDefinitions.terminal_signals
    : [];

  $: botnessValid = (
    inRange(notABotThreshold, 1, 10) &&
    inRange(challengeThreshold, 1, 10) &&
    (Number(challengeThreshold) <= 1 || Number(notABotThreshold) < Number(challengeThreshold)) &&
    inRange(mazeThreshold, 1, 10) &&
    inRange(weightJsRequired, 0, 10) &&
    inRange(weightGeoRisk, 0, 10) &&
    inRange(weightRateMedium, 0, 10) &&
    inRange(weightRateHigh, 0, 10)
  );
  $: botnessDirty = (
    Number(notABotThreshold) !== baseline.notABotThreshold ||
    Number(challengeThreshold) !== baseline.challengeThreshold ||
    Number(mazeThreshold) !== baseline.mazeThreshold ||
    Number(weightJsRequired) !== baseline.weightJsRequired ||
    Number(weightGeoRisk) !== baseline.weightGeoRisk ||
    Number(weightRateMedium) !== baseline.weightRateMedium ||
    Number(weightRateHigh) !== baseline.weightRateHigh
  );
  $: saveBotnessDisabled = !writable || !botnessDirty || !botnessValid || savingBotness;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
    }
  }
</script>

<section
  id="dashboard-panel-tuning"
  class="admin-group config-edit-pane"
  data-dashboard-tab-panel="tuning"
  aria-labelledby="dashboard-tab-tuning"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="tuning" status={tabStatus} />
  <div class="controls-grid controls-grid--tuning">
    <div class="control-group panel-soft pad-md">
      <h3>Botness Scoring</h3>
      <p class="control-desc text-muted">Weighted signals form a unified score. Moderate scores get the challenge; higher scores route to maze.</p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label" for="not-a-bot-threshold-score">Not-a-Bot (score)</label>
          <input class="input-field" type="number" id="not-a-bot-threshold-score" min="1" max="10" step="1" inputmode="numeric" aria-label="Not-a-Bot risk threshold" bind:value={notABotThreshold} disabled={!writable}>
        </div>
        <div class="input-row">
          <label class="control-label" for="challenge-puzzle-threshold">Challenge (score)</label>
          <input class="input-field" type="number" id="challenge-puzzle-threshold" min="1" max="10" step="1" inputmode="numeric" aria-label="Challenge risk threshold" bind:value={challengeThreshold} disabled={!writable}>
        </div>
        <div class="input-row">
          <label class="control-label" for="maze-threshold-score">Maze (score)</label>
          <input class="input-field" type="number" id="maze-threshold-score" min="1" max="10" step="1" inputmode="numeric" aria-label="Maze risk threshold" bind:value={mazeThreshold} disabled={!writable}>
        </div>
        <div class="input-row">
          <label class="control-label" for="weight-js-required">Weight: JS (points)</label>
          <input class="input-field" type="number" id="weight-js-required" min="0" max="10" step="1" inputmode="numeric" aria-label="Weight for JS verification required" bind:value={weightJsRequired} disabled={!writable}>
        </div>
        <div class="input-row">
          <label class="control-label" for="weight-geo-risk">Weight: Geo (points)</label>
          <input class="input-field" type="number" id="weight-geo-risk" min="0" max="10" step="1" inputmode="numeric" aria-label="Weight for high-risk geography" bind:value={weightGeoRisk} disabled={!writable}>
        </div>
        <div class="input-row">
          <label class="control-label" for="weight-rate-medium">Weight: Rate 50% (points)</label>
          <input class="input-field" type="number" id="weight-rate-medium" min="0" max="10" step="1" inputmode="numeric" aria-label="Weight for medium rate pressure" bind:value={weightRateMedium} disabled={!writable}>
        </div>
        <div class="input-row">
          <label class="control-label" for="weight-rate-high">Weight: Rate 80% (points)</label>
          <input class="input-field" type="number" id="weight-rate-high" min="0" max="10" step="1" inputmode="numeric" aria-label="Weight for high rate pressure" bind:value={weightRateHigh} disabled={!writable}>
        </div>
        <div class="info-panel panel-muted pad-sm">
          <h4>Status</h4>
          <div class="info-row">
            <span class="info-label text-muted">Config:</span>
            <span id="botness-config-status" class="status-value">{writable ? 'EDITABLE' : 'READ ONLY'}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Default Not-a-Bot:</span>
            <span id="not-a-bot-default">{notABotDefault}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Default Challenge:</span>
            <span id="challenge-puzzle-default">{challengeDefault}</span>
          </div>
          <div class="info-row">
            <span class="info-label text-muted">Default Maze:</span>
            <span id="maze-threshold-default">{mazeDefault}</span>
          </div>
        </div>
        <div class="info-panel panel-muted pad-sm">
          <h4>Scored Signals</h4>
          <div id="botness-signal-list">
            {#if scoredSignals.length === 0}
              <p class="text-muted">No scored signals</p>
            {:else}
              {#each scoredSignals as signal}
                <div class="info-row">
                  <span class="info-label">{signal.label || '--'}</span>
                  <span>{signal.weight ?? '--'}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
        <div class="info-panel panel-muted pad-sm">
          <h4>Terminal Signals</h4>
          <div id="botness-terminal-list">
            {#if terminalSignals.length === 0}
              <p class="text-muted">No terminal signals</p>
            {:else}
              {#each terminalSignals as signal}
                <div class="info-row">
                  <span class="info-label">{signal.label || '--'}</span>
                  <span>{signal.action ?? '--'}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
      <button id="save-botness-config" class="btn btn-submit" disabled={saveBotnessDisabled} on:click={saveBotness}>Save Botness Settings</button>
    </div>
  </div>
</section>
