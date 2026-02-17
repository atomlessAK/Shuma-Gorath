// @ts-check

export const CONFIG_DRAFT_DEFAULTS = Object.freeze({
  robots: { enabled: true, crawlDelay: 2 },
  aiPolicy: { blockTraining: true, blockSearch: false, allowSearch: false },
  cdp: { enabled: true, autoBan: true, threshold: 0.6 },
  edgeMode: { mode: 'off' },
  rateLimit: { value: 80 },
  jsRequired: { enforced: true },
  maze: { enabled: false, autoBan: false, threshold: 50 },
  banDurations: { honeypot: 86400, rateLimit: 3600, browser: 21600, cdp: 43200, admin: 21600 },
  pow: { enabled: true, difficulty: 15, ttl: 90, mutable: true },
  botness: {
    challengeThreshold: 3,
    mazeThreshold: 6,
    weightJsRequired: 1,
    weightGeoRisk: 2,
    weightRateMedium: 1,
    weightRateHigh: 2,
    mutable: true
  },
  geo: { risk: '', allow: '', challenge: '', maze: '', block: '', mutable: false },
  honeypot: { enabled: true, values: '/instaban' },
  browserPolicy: { block: '', whitelist: '' },
  bypassAllowlists: { network: '', path: '' },
  challengePuzzle: { enabled: true, count: 6, mutable: true },
  advancedConfig: { normalized: '{}' }
});

export const GEO_SCORING_FIELD_IDS = ['geo-risk-list'];
export const GEO_ROUTING_FIELD_IDS = [
  'geo-allow-list',
  'geo-challenge-list',
  'geo-maze-list',
  'geo-block-list'
];
export const GEO_FIELD_IDS = [...GEO_SCORING_FIELD_IDS, ...GEO_ROUTING_FIELD_IDS];

export const sanitizeGeoTextareaValue = (value) =>
  (value || '')
    .replace(/[^a-zA-Z,]/g, '')
    .toUpperCase();

const formatCountryCodes = (list) => {
  if (!Array.isArray(list) || list.length === 0) return '';
  return list.join(',');
};

export const create = (options = {}) => {
  const getById = typeof options.getById === 'function' ? options.getById : () => null;
  const setDraft = typeof options.setDraft === 'function' ? options.setDraft : () => {};
  const getDraft = typeof options.getDraft === 'function' ? options.getDraft : () => ({});
  const statusPanel = options.statusPanel || { update: () => {}, render: () => {} };

  const adminConfigWriteEnabled =
    typeof options.adminConfigWriteEnabled === 'function'
      ? options.adminConfigWriteEnabled
      : () => false;
  const parseBoolLike = typeof options.parseBoolLike === 'function'
    ? options.parseBoolLike
    : (value, fallback = false) => (typeof value === 'boolean' ? value : fallback);
  const normalizeEdgeIntegrationMode =
    typeof options.normalizeEdgeIntegrationMode === 'function'
      ? options.normalizeEdgeIntegrationMode
      : (value) => String(value || '').trim().toLowerCase();

  const normalizeCountryCodesForCompare =
    typeof options.normalizeCountryCodesForCompare === 'function'
      ? options.normalizeCountryCodesForCompare
      : (value) => String(value || '');
  const formatListTextarea =
    typeof options.formatListTextarea === 'function' ? options.formatListTextarea : () => '';
  const normalizeListTextareaForCompare =
    typeof options.normalizeListTextareaForCompare === 'function'
      ? options.normalizeListTextareaForCompare
      : (value) => String(value || '');
  const formatBrowserRulesTextarea =
    typeof options.formatBrowserRulesTextarea === 'function'
      ? options.formatBrowserRulesTextarea
      : () => '';
  const normalizeBrowserRulesForCompare =
    typeof options.normalizeBrowserRulesForCompare === 'function'
      ? options.normalizeBrowserRulesForCompare
      : (value) => String(value || '');
  const setBanDurationInputFromSeconds =
    typeof options.setBanDurationInputFromSeconds === 'function'
      ? options.setBanDurationInputFromSeconds
      : () => {};
  const banDurationFields = options.banDurationFields || {};

  const buildAdvancedConfigTemplate =
    typeof options.buildAdvancedConfigTemplate === 'function'
      ? options.buildAdvancedConfigTemplate
      : () => ({});
  const normalizeJsonObjectForCompare =
    typeof options.normalizeJsonObjectForCompare === 'function'
      ? options.normalizeJsonObjectForCompare
      : () => null;

  const setGeoConfigEditable = (editable) => {
    GEO_FIELD_IDS.forEach((id) => {
      const field = getById(id);
      if (!field) return;
      field.disabled = !editable;
      if (!editable && typeof field.blur === 'function') {
        field.blur();
      }
    });
  };

  const updateBanDurations = (config = {}) => {
    if (!config.ban_durations) return;
    setBanDurationInputFromSeconds('honeypot', config.ban_durations.honeypot);
    setBanDurationInputFromSeconds('rateLimit', config.ban_durations.rate_limit);
    setBanDurationInputFromSeconds('browser', config.ban_durations.browser);
    setBanDurationInputFromSeconds('cdp', config.ban_durations.cdp);
    setBanDurationInputFromSeconds('admin', config.ban_durations.admin);
    setDraft('banDurations', {
      honeypot: Number.parseInt(config.ban_durations.honeypot, 10) || banDurationFields.honeypot.fallback,
      rateLimit: Number.parseInt(config.ban_durations.rate_limit, 10) || banDurationFields.rateLimit.fallback,
      browser: Number.parseInt(config.ban_durations.browser, 10) || banDurationFields.browser.fallback,
      cdp: Number.parseInt(config.ban_durations.cdp, 10) || banDurationFields.cdp.fallback,
      admin: Number.parseInt(config.ban_durations.admin, 10) || banDurationFields.admin.fallback
    });
    const btn = getById('save-durations-btn');
    if (!btn) return;
    btn.dataset.saving = 'false';
    btn.disabled = true;
    btn.textContent = 'Save Durations';
  };

  const updateMazeConfig = (config = {}) => {
    const statusPatch = {};
    const mazeEnabledToggle = getById('maze-enabled-toggle');
    const mazeAutoBanToggle = getById('maze-auto-ban-toggle');
    const mazeThreshold = getById('maze-threshold');
    if (!mazeEnabledToggle || !mazeAutoBanToggle || !mazeThreshold) return;

    if (config.maze_enabled !== undefined) {
      mazeEnabledToggle.checked = config.maze_enabled;
      statusPatch.mazeEnabled = config.maze_enabled === true;
    }
    if (config.maze_auto_ban !== undefined) {
      mazeAutoBanToggle.checked = config.maze_auto_ban;
      statusPatch.mazeAutoBan = config.maze_auto_ban === true;
    }
    if (config.maze_auto_ban_threshold !== undefined) {
      mazeThreshold.value = config.maze_auto_ban_threshold;
    }
    setDraft('maze', {
      enabled: mazeEnabledToggle.checked,
      autoBan: mazeAutoBanToggle.checked,
      threshold: Number.parseInt(mazeThreshold.value, 10) || 50
    });
    const btn = getById('save-maze-config');
    if (btn) {
      btn.dataset.saving = 'false';
      btn.disabled = true;
      btn.textContent = 'Save Maze Settings';
    }
    statusPanel.update(statusPatch);
    statusPanel.render();
  };

  const updateGeoConfig = (config = {}) => {
    const mutable = adminConfigWriteEnabled(config);
    const risk = formatCountryCodes(config.geo_risk);
    const allow = formatCountryCodes(config.geo_allow);
    const challenge = formatCountryCodes(config.geo_challenge);
    const maze = formatCountryCodes(config.geo_maze);
    const block = formatCountryCodes(config.geo_block);

    const riskField = getById('geo-risk-list');
    const allowField = getById('geo-allow-list');
    const challengeField = getById('geo-challenge-list');
    const mazeField = getById('geo-maze-list');
    const blockField = getById('geo-block-list');
    if (!riskField || !allowField || !challengeField || !mazeField || !blockField) return;

    riskField.value = risk;
    allowField.value = allow;
    challengeField.value = challenge;
    mazeField.value = maze;
    blockField.value = block;

    setDraft('geo', {
      risk: normalizeCountryCodesForCompare(risk),
      allow: normalizeCountryCodesForCompare(allow),
      challenge: normalizeCountryCodesForCompare(challenge),
      maze: normalizeCountryCodesForCompare(maze),
      block: normalizeCountryCodesForCompare(block),
      mutable
    });

    statusPanel.update({
      geoRiskCount: Array.isArray(config.geo_risk) ? config.geo_risk.length : 0,
      geoAllowCount: Array.isArray(config.geo_allow) ? config.geo_allow.length : 0,
      geoChallengeCount: Array.isArray(config.geo_challenge) ? config.geo_challenge.length : 0,
      geoMazeCount: Array.isArray(config.geo_maze) ? config.geo_maze.length : 0,
      geoBlockCount: Array.isArray(config.geo_block) ? config.geo_block.length : 0
    });
    statusPanel.render();

    setGeoConfigEditable(mutable);

    const scoringBtn = getById('save-geo-scoring-config');
    if (scoringBtn) {
      scoringBtn.disabled = true;
      scoringBtn.textContent = 'Save GEO Scoring';
    }
    const routingBtn = getById('save-geo-routing-config');
    if (routingBtn) {
      routingBtn.disabled = true;
      routingBtn.textContent = 'Save GEO Routing';
    }
  };

  const updateHoneypotConfig = (config = {}) => {
    const enabledToggle = getById('honeypot-enabled-toggle');
    const field = getById('honeypot-paths');
    if (!field) return;
    if (enabledToggle) {
      enabledToggle.checked = config.honeypot_enabled !== false;
    }
    const formatted = formatListTextarea(config.honeypots);
    field.value = formatted;
    setDraft('honeypot', {
      enabled: enabledToggle ? enabledToggle.checked : true,
      values: normalizeListTextareaForCompare(formatted)
    });
    const btn = getById('save-honeypot-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save Honeypots';
  };

  const updateBrowserPolicyConfig = (config = {}) => {
    const blockField = getById('browser-block-rules');
    const whitelistField = getById('browser-whitelist-rules');
    if (!blockField || !whitelistField) return;

    const blockText = formatBrowserRulesTextarea(config.browser_block);
    const whitelistText = formatBrowserRulesTextarea(config.browser_whitelist);
    blockField.value = blockText;
    whitelistField.value = whitelistText;
    setDraft('browserPolicy', {
      block: normalizeBrowserRulesForCompare(blockText),
      whitelist: normalizeBrowserRulesForCompare(whitelistText)
    });
    const btn = getById('save-browser-policy-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save Browser Policy';
  };

  const updateBypassAllowlistConfig = (config = {}) => {
    const networkField = getById('network-whitelist');
    const pathField = getById('path-whitelist');
    if (!networkField || !pathField) return;

    const networkText = formatListTextarea(config.whitelist);
    const pathText = formatListTextarea(config.path_whitelist);
    networkField.value = networkText;
    pathField.value = pathText;
    setDraft('bypassAllowlists', {
      network: normalizeListTextareaForCompare(networkText),
      path: normalizeListTextareaForCompare(pathText)
    });
    const btn = getById('save-whitelist-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save Allowlists';
  };

  const updateRobotsConfig = (config = {}) => {
    const robotsEnabledToggle = getById('robots-enabled-toggle');
    const robotsBlockTrainingToggle = getById('robots-block-training-toggle');
    const robotsBlockSearchToggle = getById('robots-block-search-toggle');
    const robotsAllowSearchToggle = getById('robots-allow-search-toggle');
    const robotsCrawlDelay = getById('robots-crawl-delay');
    if (
      !robotsEnabledToggle ||
      !robotsBlockTrainingToggle ||
      !robotsBlockSearchToggle ||
      !robotsAllowSearchToggle ||
      !robotsCrawlDelay
    ) {
      return;
    }

    if (config.robots_enabled !== undefined) {
      robotsEnabledToggle.checked = config.robots_enabled;
    }
    const aiBlockTraining = config.ai_policy_block_training ?? config.robots_block_ai_training;
    if (aiBlockTraining !== undefined) {
      robotsBlockTrainingToggle.checked = aiBlockTraining;
    }
    const aiBlockSearch = config.ai_policy_block_search ?? config.robots_block_ai_search;
    if (aiBlockSearch !== undefined) {
      robotsBlockSearchToggle.checked = aiBlockSearch;
    }
    const aiAllowSearch = config.ai_policy_allow_search_engines ?? config.robots_allow_search_engines;
    if (aiAllowSearch !== undefined) {
      robotsAllowSearchToggle.checked = !aiAllowSearch;
    }
    if (config.robots_crawl_delay !== undefined) {
      robotsCrawlDelay.value = config.robots_crawl_delay;
    }
    setDraft('robots', {
      enabled: robotsEnabledToggle.checked,
      crawlDelay: Number.parseInt(robotsCrawlDelay.value, 10) || 2
    });
    setDraft('aiPolicy', {
      blockTraining: robotsBlockTrainingToggle.checked,
      blockSearch: robotsBlockSearchToggle.checked,
      allowSearch: robotsAllowSearchToggle.checked
    });

    const robotsBtn = getById('save-robots-config');
    if (robotsBtn) {
      robotsBtn.disabled = true;
      robotsBtn.textContent = 'Save robots serving';
    }
    const aiBtn = getById('save-ai-policy-config');
    if (aiBtn) {
      aiBtn.disabled = true;
      aiBtn.textContent = 'Save AI bot policy';
    }
  };

  const updateCdpConfig = (config = {}) => {
    const cdpEnabledToggle = getById('cdp-enabled-toggle');
    const cdpAutoBanToggle = getById('cdp-auto-ban-toggle');
    const cdpThresholdSlider = getById('cdp-threshold-slider');
    const cdpThresholdValue = getById('cdp-threshold-value');
    if (!cdpEnabledToggle || !cdpAutoBanToggle || !cdpThresholdSlider || !cdpThresholdValue) return;

    const statusPatch = {};
    if (config.cdp_detection_enabled !== undefined) {
      cdpEnabledToggle.checked = config.cdp_detection_enabled;
      statusPatch.cdpEnabled = config.cdp_detection_enabled === true;
    }
    if (config.cdp_auto_ban !== undefined) {
      cdpAutoBanToggle.checked = config.cdp_auto_ban;
      statusPatch.cdpAutoBan = config.cdp_auto_ban === true;
    }
    if (config.cdp_detection_threshold !== undefined) {
      cdpThresholdSlider.value = config.cdp_detection_threshold;
      cdpThresholdValue.textContent = Number.parseFloat(config.cdp_detection_threshold).toFixed(1);
    }
    setDraft('cdp', {
      enabled: cdpEnabledToggle.checked,
      autoBan: cdpAutoBanToggle.checked,
      threshold: Number.parseFloat(cdpThresholdSlider.value)
    });
    const btn = getById('save-cdp-config');
    if (btn) {
      btn.disabled = true;
      btn.textContent = 'Save CDP Settings';
    }
    statusPanel.update(statusPatch);
    statusPanel.render();
  };

  const updateEdgeIntegrationModeConfig = (config = {}) => {
    const mode = normalizeEdgeIntegrationMode(config.edge_integration_mode);
    const select = getById('edge-integration-mode-select');
    if (!select) return;
    select.value = mode;
    setDraft('edgeMode', { mode });

    const btn = getById('save-edge-integration-mode-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save Edge Integration Mode';
  };

  const updateRateLimitConfig = (config = {}) => {
    const rateLimit = Number.parseInt(config.rate_limit, 10) || 80;
    const field = getById('rate-limit-threshold');
    if (!field) return;
    field.value = rateLimit;
    setDraft('rateLimit', { value: rateLimit });
    statusPanel.update({ rateLimit });
    statusPanel.render();

    const btn = getById('save-rate-limit-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save Rate Limit';
  };

  const updateJsRequiredConfig = (config = {}) => {
    const enforced = parseBoolLike(config.js_required_enforced, true);
    const toggle = getById('js-required-enforced-toggle');
    if (!toggle) return;
    toggle.checked = enforced;
    setDraft('jsRequired', { enforced });
    statusPanel.update({ jsRequiredEnforced: enforced });
    statusPanel.render();

    const btn = getById('save-js-required-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save JS Required';
  };

  const updatePowConfig = (config = {}) => {
    const powEnabled = parseBoolLike(config.pow_enabled, true);
    const difficulty = Number.parseInt(config.pow_difficulty, 10);
    const ttl = Number.parseInt(config.pow_ttl_seconds, 10);
    const difficultyField = getById('pow-difficulty');
    const ttlField = getById('pow-ttl');
    const powEnabledToggle = getById('pow-enabled-toggle');
    if (!difficultyField || !ttlField || !powEnabledToggle) return;

    statusPanel.update({ powEnabled });
    statusPanel.render();

    if (!Number.isNaN(difficulty)) {
      difficultyField.value = difficulty;
    }
    if (!Number.isNaN(ttl)) {
      ttlField.value = ttl;
    }
    powEnabledToggle.checked = powEnabled;

    setDraft('pow', {
      enabled: powEnabledToggle.checked,
      difficulty: Number.parseInt(difficultyField.value, 10) || 15,
      ttl: Number.parseInt(ttlField.value, 10) || 90
    });

    const btn = getById('save-pow-config');
    if (!btn) return;
    btn.disabled = true;
    btn.textContent = 'Save PoW Settings';
  };

  const updateBotnessSignalDefinitions = (signalDefinitions) => {
    const scoredSignals = (signalDefinitions && Array.isArray(signalDefinitions.scored_signals))
      ? signalDefinitions.scored_signals
      : [];
    const terminalSignals = (signalDefinitions && Array.isArray(signalDefinitions.terminal_signals))
      ? signalDefinitions.terminal_signals
      : [];

    const scoredTarget = getById('botness-signal-list');
    const terminalTarget = getById('botness-terminal-list');
    if (!scoredTarget || !terminalTarget) return;

    scoredTarget.innerHTML = scoredSignals.length
      ? scoredSignals.map((signal) => `
      <div class="info-row">
        <span class="info-label">${signal.label}</span>
        <span>${signal.weight}</span>
      </div>
    `).join('')
      : '<p class="text-muted">No scored signals</p>';

    terminalTarget.innerHTML = terminalSignals.length
      ? terminalSignals.map((signal) => `
      <div class="info-row">
        <span class="info-label">${signal.label}</span>
        <span>${signal.action}</span>
      </div>
    `).join('')
      : '<p class="text-muted">No terminal signals</p>';
  };

  const updateChallengeConfig = (config = {}) => {
    const writable = adminConfigWriteEnabled(config);
    const challengeEnabled = config.challenge_puzzle_enabled !== false;
    const challengeTransformCount = Number.parseInt(config.challenge_puzzle_transform_count, 10);
    const challengeThreshold = Number.parseInt(config.challenge_puzzle_risk_threshold, 10);
    const challengeDefault = Number.parseInt(config.challenge_puzzle_risk_threshold_default, 10);
    const mazeThreshold = Number.parseInt(config.botness_maze_threshold, 10);
    const mazeDefault = Number.parseInt(config.botness_maze_threshold_default, 10);
    const weights = config.botness_weights || {};

    const challengeThresholdField = getById('challenge-puzzle-threshold');
    const mazeThresholdField = getById('maze-threshold-score');
    const transformCountField = getById('challenge-puzzle-transform-count');
    const challengeEnabledToggle = getById('challenge-puzzle-enabled-toggle');
    const weightJsRequiredField = getById('weight-js-required');
    const weightGeoRiskField = getById('weight-geo-risk');
    const weightRateMediumField = getById('weight-rate-medium');
    const weightRateHighField = getById('weight-rate-high');
    const botnessStatus = getById('botness-config-status');
    const challengeDefaultLabel = getById('challenge-puzzle-default');
    const mazeDefaultLabel = getById('maze-threshold-default');
    if (
      !challengeThresholdField ||
      !mazeThresholdField ||
      !transformCountField ||
      !challengeEnabledToggle ||
      !weightJsRequiredField ||
      !weightGeoRiskField ||
      !weightRateMediumField ||
      !weightRateHighField ||
      !botnessStatus ||
      !challengeDefaultLabel ||
      !mazeDefaultLabel
    ) {
      return;
    }

    if (!Number.isNaN(challengeThreshold)) {
      challengeThresholdField.value = challengeThreshold;
    }
    if (!Number.isNaN(mazeThreshold)) {
      mazeThresholdField.value = mazeThreshold;
    }
    if (!Number.isNaN(challengeTransformCount)) {
      transformCountField.value = challengeTransformCount;
    }
    challengeEnabledToggle.checked = challengeEnabled;
    weightJsRequiredField.value = Number.parseInt(weights.js_required, 10) || 1;
    weightGeoRiskField.value = Number.parseInt(weights.geo_risk, 10) || 2;
    weightRateMediumField.value = Number.parseInt(weights.rate_medium, 10) || 1;
    weightRateHighField.value = Number.parseInt(weights.rate_high, 10) || 2;

    botnessStatus.textContent = writable ? 'EDITABLE' : 'READ ONLY';
    challengeDefaultLabel.textContent = Number.isNaN(challengeDefault) ? '--' : challengeDefault;
    mazeDefaultLabel.textContent = Number.isNaN(mazeDefault) ? '--' : mazeDefault;

    statusPanel.update({
      challengeEnabled,
      challengeThreshold: Number.isNaN(challengeThreshold) ? 3 : challengeThreshold,
      mazeThreshold: Number.isNaN(mazeThreshold) ? 6 : mazeThreshold,
      botnessWeights: {
        js_required: Number.parseInt(weights.js_required, 10) || 0,
        geo_risk: Number.parseInt(weights.geo_risk, 10) || 0,
        rate_medium: Number.parseInt(weights.rate_medium, 10) || 0,
        rate_high: Number.parseInt(weights.rate_high, 10) || 0
      }
    });

    const editableFields = [
      'challenge-puzzle-threshold',
      'maze-threshold-score',
      'weight-js-required',
      'weight-geo-risk',
      'weight-rate-medium',
      'weight-rate-high'
    ];
    editableFields.forEach((id) => {
      const field = getById(id);
      if (field) field.disabled = !writable;
    });

    setDraft('botness', {
      challengeThreshold: Number.parseInt(challengeThresholdField.value, 10) || 3,
      mazeThreshold: Number.parseInt(mazeThresholdField.value, 10) || 6,
      weightJsRequired: Number.parseInt(weightJsRequiredField.value, 10) || 1,
      weightGeoRisk: Number.parseInt(weightGeoRiskField.value, 10) || 2,
      weightRateMedium: Number.parseInt(weightRateMediumField.value, 10) || 1,
      weightRateHigh: Number.parseInt(weightRateHighField.value, 10) || 2
    });

    updateBotnessSignalDefinitions(config.botness_signal_definitions);

    const btn = getById('save-botness-config');
    if (btn) {
      btn.disabled = true;
      btn.textContent = 'Save Botness Settings';
    }

    setDraft('challengePuzzle', {
      enabled: challengeEnabledToggle.checked,
      count: Number.parseInt(transformCountField.value, 10) || 6
    });
    const challengeBtn = getById('save-challenge-puzzle-config');
    if (challengeBtn) {
      challengeBtn.disabled = true;
      challengeBtn.textContent = 'Save Challenge Puzzle';
    }
    transformCountField.disabled = !writable;
    challengeEnabledToggle.disabled = !writable;
    statusPanel.render();
  };

  const setAdvancedConfigEditorFromConfig = (config, preserveDirty = true) => {
    const field = getById('advanced-config-json');
    if (!field) return;
    const previousBaseline = getDraft('advancedConfig').normalized || '{}';
    const template = buildAdvancedConfigTemplate(config || {});
    const formatted = JSON.stringify(template, null, 2);
    const currentNormalized = normalizeJsonObjectForCompare(field.value);
    const hasUnsavedEdits = field.dataset.dirty === 'true';

    setDraft('advancedConfig', { normalized: normalizeJsonObjectForCompare(formatted) || '{}' });

    const shouldReplace =
      !preserveDirty ||
      !hasUnsavedEdits ||
      currentNormalized === previousBaseline ||
      !String(field.value || '').trim();

    if (shouldReplace) {
      field.value = formatted;
    }
  };

  return {
    updateBanDurations,
    updateMazeConfig,
    updateGeoConfig,
    updateHoneypotConfig,
    updateBrowserPolicyConfig,
    updateBypassAllowlistConfig,
    updateRobotsConfig,
    updateCdpConfig,
    updateEdgeIntegrationModeConfig,
    updateRateLimitConfig,
    updateJsRequiredConfig,
    updatePowConfig,
    updateChallengeConfig,
    setAdvancedConfigEditorFromConfig,
    setGeoConfigEditable
  };
};
