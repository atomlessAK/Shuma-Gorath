// @ts-check

export const CONFIG_DRAFT_DEFAULTS = Object.freeze({
  robots: { enabled: true, crawlDelay: 2 },
  aiPolicy: { blockTraining: true, blockSearch: false, allowSearch: false },
  cdp: { enabled: true, autoBan: true, threshold: 0.6 },
  edgeMode: { mode: 'off' },
  rateLimit: { value: 80 },
  jsRequired: { enforced: true },
  maze: { enabled: true, autoBan: true, threshold: 50 },
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

const setControlProperty = (control, prop, value) => {
  if (!control || value === undefined) return;
  control[prop] = value;
};

const applyControlBindings = ({ getById, config, bindings }) => {
  const values = {};
  for (const binding of bindings) {
    const control = getById(binding.id);
    if (!control) {
      return { ok: false, values };
    }
    const raw = typeof binding.read === 'function' ? binding.read(config) : config[binding.key];
    if (raw === undefined) continue;
    const next = typeof binding.coerce === 'function' ? binding.coerce(raw, config) : raw;
    setControlProperty(control, binding.prop || 'value', next);
    values[binding.name || binding.id] = next;
  }
  return { ok: true, values };
};

const resetSaveButton = (getById, buttonId, label) => {
  const button = getById(buttonId);
  if (!button) return;
  button.dataset.saving = 'false';
  button.disabled = true;
  button.textContent = label;
};

const renderInfoRows = ({ container, rows, emptyText, valueKey }) => {
  if (!container) return;
  container.textContent = '';
  if (!Array.isArray(rows) || rows.length === 0) {
    const empty = document.createElement('p');
    empty.className = 'text-muted';
    empty.textContent = emptyText;
    container.appendChild(empty);
    return;
  }
  rows.forEach((row) => {
    const wrapper = document.createElement('div');
    wrapper.className = 'info-row';

    const label = document.createElement('span');
    label.className = 'info-label';
    label.textContent = String(row.label || '--');

    const value = document.createElement('span');
    value.textContent = String(row[valueKey] ?? '--');

    wrapper.appendChild(label);
    wrapper.appendChild(value);
    container.appendChild(wrapper);
  });
};

export const create = (options = {}) => {
  const getById = typeof options.getById === 'function' ? options.getById : () => null;
  const setDraft = typeof options.setDraft === 'function' ? options.setDraft : () => {};
  const getDraft = typeof options.getDraft === 'function' ? options.getDraft : () => ({});

  const statusPanel = options.statusPanel || { update: () => {}, render: () => {} };
  const applyStatusPatch =
    typeof statusPanel.applyPatch === 'function'
      ? statusPanel.applyPatch.bind(statusPanel)
      : (patch) => {
        statusPanel.update(patch);
        statusPanel.render();
      };

  const adminConfigWriteEnabled =
    typeof options.adminConfigWriteEnabled === 'function'
      ? options.adminConfigWriteEnabled
      : () => false;
  const parseBoolLike =
    typeof options.parseBoolLike === 'function'
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

  const isElementFocused = (element) => Boolean(element && document.activeElement === element);
  const hasUnsavedSectionEdits = (buttonId) => {
    const button = getById(buttonId);
    if (!button) return false;
    return button.disabled === false && button.dataset.saving !== 'true';
  };
  const shouldPreserveInProgressEdits = (buttonId, fields = []) =>
    hasUnsavedSectionEdits(buttonId) || fields.some(isElementFocused);

  const updateBanDurations = (config = {}) => {
    if (!config.ban_durations) return;
    setBanDurationInputFromSeconds('honeypot', config.ban_durations.honeypot);
    setBanDurationInputFromSeconds('rateLimit', config.ban_durations.rate_limit);
    setBanDurationInputFromSeconds('browser', config.ban_durations.browser);
    setBanDurationInputFromSeconds('cdp', config.ban_durations.cdp);
    setBanDurationInputFromSeconds('admin', config.ban_durations.admin);

    setDraft('banDurations', {
      honeypot:
        Number.parseInt(config.ban_durations.honeypot, 10) || banDurationFields.honeypot.fallback,
      rateLimit:
        Number.parseInt(config.ban_durations.rate_limit, 10) || banDurationFields.rateLimit.fallback,
      browser:
        Number.parseInt(config.ban_durations.browser, 10) || banDurationFields.browser.fallback,
      cdp: Number.parseInt(config.ban_durations.cdp, 10) || banDurationFields.cdp.fallback,
      admin: Number.parseInt(config.ban_durations.admin, 10) || banDurationFields.admin.fallback
    });

    resetSaveButton(getById, 'save-durations-btn', 'Save Durations');
  };

  const updateMazeConfig = (config = {}) => {
    const mazeEnabledToggle = getById('maze-enabled-toggle');
    const mazeAutoBanToggle = getById('maze-auto-ban-toggle');
    const mazeThreshold = getById('maze-threshold');

    if (!mazeEnabledToggle || !mazeAutoBanToggle || !mazeThreshold) return;

    if (
      shouldPreserveInProgressEdits('save-maze-config', [
        mazeEnabledToggle,
        mazeAutoBanToggle,
        mazeThreshold
      ])
    ) {
      return;
    }

    const { ok } = applyControlBindings({
      getById,
      config,
      bindings: [
        { id: 'maze-enabled-toggle', name: 'enabled', key: 'maze_enabled', prop: 'checked' },
        { id: 'maze-auto-ban-toggle', name: 'autoBan', key: 'maze_auto_ban', prop: 'checked' },
        {
          id: 'maze-threshold',
          name: 'threshold',
          key: 'maze_auto_ban_threshold',
          prop: 'value'
        }
      ]
    });
    if (!ok) return;

    setDraft('maze', {
      enabled: mazeEnabledToggle.checked,
      autoBan: mazeAutoBanToggle.checked,
      threshold: Number.parseInt(mazeThreshold.value, 10) || 50
    });

    applyStatusPatch({
      mazeEnabled: mazeEnabledToggle.checked,
      mazeAutoBan: mazeAutoBanToggle.checked
    });

    resetSaveButton(getById, 'save-maze-config', 'Save Maze Settings');
  };

  const updateGeoConfig = (config = {}) => {
    const mutable = adminConfigWriteEnabled(config);
    const risk = formatCountryCodes(config.geo_risk);
    const allow = formatCountryCodes(config.geo_allow);
    const challenge = formatCountryCodes(config.geo_challenge);
    const maze = formatCountryCodes(config.geo_maze);
    const block = formatCountryCodes(config.geo_block);

    const { ok } = applyControlBindings({
      getById,
      config: {
        risk,
        allow,
        challenge,
        maze,
        block
      },
      bindings: [
        { id: 'geo-risk-list', key: 'risk', prop: 'value' },
        { id: 'geo-allow-list', key: 'allow', prop: 'value' },
        { id: 'geo-challenge-list', key: 'challenge', prop: 'value' },
        { id: 'geo-maze-list', key: 'maze', prop: 'value' },
        { id: 'geo-block-list', key: 'block', prop: 'value' }
      ]
    });
    if (!ok) return;

    setDraft('geo', {
      risk: normalizeCountryCodesForCompare(risk),
      allow: normalizeCountryCodesForCompare(allow),
      challenge: normalizeCountryCodesForCompare(challenge),
      maze: normalizeCountryCodesForCompare(maze),
      block: normalizeCountryCodesForCompare(block),
      mutable
    });

    applyStatusPatch({
      geoRiskCount: Array.isArray(config.geo_risk) ? config.geo_risk.length : 0,
      geoAllowCount: Array.isArray(config.geo_allow) ? config.geo_allow.length : 0,
      geoChallengeCount: Array.isArray(config.geo_challenge) ? config.geo_challenge.length : 0,
      geoMazeCount: Array.isArray(config.geo_maze) ? config.geo_maze.length : 0,
      geoBlockCount: Array.isArray(config.geo_block) ? config.geo_block.length : 0
    });

    setGeoConfigEditable(mutable);
    resetSaveButton(getById, 'save-geo-scoring-config', 'Save GEO Scoring');
    resetSaveButton(getById, 'save-geo-routing-config', 'Save GEO Routing');
  };

  const updateHoneypotConfig = (config = {}) => {
    const enabled = config.honeypot_enabled !== false;
    const formatted = formatListTextarea(config.honeypots);

    const { ok } = applyControlBindings({
      getById,
      config: {
        enabled,
        values: formatted
      },
      bindings: [
        { id: 'honeypot-enabled-toggle', key: 'enabled', prop: 'checked' },
        { id: 'honeypot-paths', key: 'values', prop: 'value' }
      ]
    });
    if (!ok) return;

    setDraft('honeypot', {
      enabled,
      values: normalizeListTextareaForCompare(formatted)
    });

    resetSaveButton(getById, 'save-honeypot-config', 'Save Honeypots');
  };

  const updateBrowserPolicyConfig = (config = {}) => {
    const blockText = formatBrowserRulesTextarea(config.browser_block);
    const whitelistText = formatBrowserRulesTextarea(config.browser_whitelist);

    const { ok } = applyControlBindings({
      getById,
      config: {
        block: blockText,
        whitelist: whitelistText
      },
      bindings: [
        { id: 'browser-block-rules', key: 'block', prop: 'value' },
        { id: 'browser-whitelist-rules', key: 'whitelist', prop: 'value' }
      ]
    });
    if (!ok) return;

    setDraft('browserPolicy', {
      block: normalizeBrowserRulesForCompare(blockText),
      whitelist: normalizeBrowserRulesForCompare(whitelistText)
    });

    resetSaveButton(getById, 'save-browser-policy-config', 'Save Browser Policy');
  };

  const updateBypassAllowlistConfig = (config = {}) => {
    const networkText = formatListTextarea(config.whitelist);
    const pathText = formatListTextarea(config.path_whitelist);

    const { ok } = applyControlBindings({
      getById,
      config: {
        network: networkText,
        path: pathText
      },
      bindings: [
        { id: 'network-whitelist', key: 'network', prop: 'value' },
        { id: 'path-whitelist', key: 'path', prop: 'value' }
      ]
    });
    if (!ok) return;

    setDraft('bypassAllowlists', {
      network: normalizeListTextareaForCompare(networkText),
      path: normalizeListTextareaForCompare(pathText)
    });

    resetSaveButton(getById, 'save-whitelist-config', 'Save Allowlists');
  };

  const updateRobotsConfig = (config = {}) => {
    const aiBlockTraining = config.ai_policy_block_training ?? config.robots_block_ai_training;
    const aiBlockSearch = config.ai_policy_block_search ?? config.robots_block_ai_search;
    const aiAllowSearch =
      config.ai_policy_allow_search_engines ?? config.robots_allow_search_engines;

    const { ok } = applyControlBindings({
      getById,
      config: {
        robotsEnabled: config.robots_enabled,
        blockTraining: aiBlockTraining,
        blockSearch: aiBlockSearch,
        allowSearchToggle: aiAllowSearch === undefined ? undefined : !aiAllowSearch,
        crawlDelay: config.robots_crawl_delay
      },
      bindings: [
        { id: 'robots-enabled-toggle', key: 'robotsEnabled', prop: 'checked' },
        { id: 'robots-block-training-toggle', key: 'blockTraining', prop: 'checked' },
        { id: 'robots-block-search-toggle', key: 'blockSearch', prop: 'checked' },
        { id: 'robots-allow-search-toggle', key: 'allowSearchToggle', prop: 'checked' },
        { id: 'robots-crawl-delay', key: 'crawlDelay', prop: 'value' }
      ]
    });
    if (!ok) return;

    const robotsEnabledToggle = getById('robots-enabled-toggle');
    const robotsCrawlDelay = getById('robots-crawl-delay');
    const robotsBlockTrainingToggle = getById('robots-block-training-toggle');
    const robotsBlockSearchToggle = getById('robots-block-search-toggle');
    const robotsAllowSearchToggle = getById('robots-allow-search-toggle');

    setDraft('robots', {
      enabled: robotsEnabledToggle.checked,
      crawlDelay: Number.parseInt(robotsCrawlDelay.value, 10) || 2
    });
    setDraft('aiPolicy', {
      blockTraining: robotsBlockTrainingToggle.checked,
      blockSearch: robotsBlockSearchToggle.checked,
      allowSearch: robotsAllowSearchToggle.checked
    });

    resetSaveButton(getById, 'save-robots-config', 'Save robots serving');
    resetSaveButton(getById, 'save-ai-policy-config', 'Save AI bot policy');
  };

  const updateCdpConfig = (config = {}) => {
    const { ok } = applyControlBindings({
      getById,
      config,
      bindings: [
        {
          id: 'cdp-enabled-toggle',
          key: 'cdp_detection_enabled',
          prop: 'checked'
        },
        {
          id: 'cdp-auto-ban-toggle',
          key: 'cdp_auto_ban',
          prop: 'checked'
        },
        {
          id: 'cdp-threshold-slider',
          key: 'cdp_detection_threshold',
          prop: 'value'
        },
        {
          id: 'cdp-threshold-value',
          key: 'cdp_detection_threshold',
          prop: 'textContent',
          coerce: (value) => Number.parseFloat(value).toFixed(1)
        }
      ]
    });
    if (!ok) return;

    const cdpEnabledToggle = getById('cdp-enabled-toggle');
    const cdpAutoBanToggle = getById('cdp-auto-ban-toggle');
    const cdpThresholdSlider = getById('cdp-threshold-slider');

    setDraft('cdp', {
      enabled: cdpEnabledToggle.checked,
      autoBan: cdpAutoBanToggle.checked,
      threshold: Number.parseFloat(cdpThresholdSlider.value)
    });

    applyStatusPatch({
      cdpEnabled: cdpEnabledToggle.checked,
      cdpAutoBan: cdpAutoBanToggle.checked
    });

    resetSaveButton(getById, 'save-cdp-config', 'Save CDP Settings');
  };

  const updateEdgeIntegrationModeConfig = (config = {}) => {
    const mode = normalizeEdgeIntegrationMode(config.edge_integration_mode);
    const { ok } = applyControlBindings({
      getById,
      config: { mode },
      bindings: [{ id: 'edge-integration-mode-select', key: 'mode', prop: 'value' }]
    });
    if (!ok) return;

    setDraft('edgeMode', { mode });
    resetSaveButton(getById, 'save-edge-integration-mode-config', 'Save Edge Integration Mode');
  };

  const updateRateLimitConfig = (config = {}) => {
    const rateLimit = Number.parseInt(config.rate_limit, 10) || 80;

    const { ok } = applyControlBindings({
      getById,
      config: { rateLimit },
      bindings: [{ id: 'rate-limit-threshold', key: 'rateLimit', prop: 'value' }]
    });
    if (!ok) return;

    setDraft('rateLimit', { value: rateLimit });
    applyStatusPatch({ rateLimit });
    resetSaveButton(getById, 'save-rate-limit-config', 'Save Rate Limit');
  };

  const updateJsRequiredConfig = (config = {}) => {
    const enforced = parseBoolLike(config.js_required_enforced, true);

    const { ok } = applyControlBindings({
      getById,
      config: { enforced },
      bindings: [{ id: 'js-required-enforced-toggle', key: 'enforced', prop: 'checked' }]
    });
    if (!ok) return;

    setDraft('jsRequired', { enforced });
    applyStatusPatch({ jsRequiredEnforced: enforced });
    resetSaveButton(getById, 'save-js-required-config', 'Save JS Required');
  };

  const updatePowConfig = (config = {}) => {
    const powEnabled = parseBoolLike(config.pow_enabled, true);
    const difficulty = Number.parseInt(config.pow_difficulty, 10);
    const ttl = Number.parseInt(config.pow_ttl_seconds, 10);

    const { ok } = applyControlBindings({
      getById,
      config: {
        enabled: powEnabled,
        difficulty: Number.isNaN(difficulty) ? undefined : difficulty,
        ttl: Number.isNaN(ttl) ? undefined : ttl
      },
      bindings: [
        { id: 'pow-enabled-toggle', key: 'enabled', prop: 'checked' },
        { id: 'pow-difficulty', key: 'difficulty', prop: 'value' },
        { id: 'pow-ttl', key: 'ttl', prop: 'value' }
      ]
    });
    if (!ok) return;

    const difficultyField = getById('pow-difficulty');
    const ttlField = getById('pow-ttl');
    const powEnabledToggle = getById('pow-enabled-toggle');

    setDraft('pow', {
      enabled: powEnabledToggle.checked,
      difficulty: Number.parseInt(difficultyField.value, 10) || 15,
      ttl: Number.parseInt(ttlField.value, 10) || 90
    });

    applyStatusPatch({ powEnabled });
    resetSaveButton(getById, 'save-pow-config', 'Save PoW Settings');
  };

  const updateBotnessSignalDefinitions = (signalDefinitions) => {
    const scoredSignals =
      signalDefinitions && Array.isArray(signalDefinitions.scored_signals)
        ? signalDefinitions.scored_signals
        : [];
    const terminalSignals =
      signalDefinitions && Array.isArray(signalDefinitions.terminal_signals)
        ? signalDefinitions.terminal_signals
        : [];

    const scoredTarget = getById('botness-signal-list');
    const terminalTarget = getById('botness-terminal-list');
    if (!scoredTarget || !terminalTarget) return;

    renderInfoRows({
      container: scoredTarget,
      rows: scoredSignals,
      emptyText: 'No scored signals',
      valueKey: 'weight'
    });
    renderInfoRows({
      container: terminalTarget,
      rows: terminalSignals,
      emptyText: 'No terminal signals',
      valueKey: 'action'
    });
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

    const { ok } = applyControlBindings({
      getById,
      config: {
        challengeThreshold: Number.isNaN(challengeThreshold) ? undefined : challengeThreshold,
        mazeThreshold: Number.isNaN(mazeThreshold) ? undefined : mazeThreshold,
        transformCount: Number.isNaN(challengeTransformCount)
          ? undefined
          : challengeTransformCount,
        challengeEnabled,
        weightJsRequired: Number.parseInt(weights.js_required, 10) || 1,
        weightGeoRisk: Number.parseInt(weights.geo_risk, 10) || 2,
        weightRateMedium: Number.parseInt(weights.rate_medium, 10) || 1,
        weightRateHigh: Number.parseInt(weights.rate_high, 10) || 2
      },
      bindings: [
        { id: 'challenge-puzzle-threshold', key: 'challengeThreshold', prop: 'value' },
        { id: 'maze-threshold-score', key: 'mazeThreshold', prop: 'value' },
        { id: 'challenge-puzzle-transform-count', key: 'transformCount', prop: 'value' },
        { id: 'challenge-puzzle-enabled-toggle', key: 'challengeEnabled', prop: 'checked' },
        { id: 'weight-js-required', key: 'weightJsRequired', prop: 'value' },
        { id: 'weight-geo-risk', key: 'weightGeoRisk', prop: 'value' },
        { id: 'weight-rate-medium', key: 'weightRateMedium', prop: 'value' },
        { id: 'weight-rate-high', key: 'weightRateHigh', prop: 'value' }
      ]
    });
    if (!ok) return;

    botnessStatus.textContent = writable ? 'EDITABLE' : 'READ ONLY';
    challengeDefaultLabel.textContent = Number.isNaN(challengeDefault) ? '--' : challengeDefault;
    mazeDefaultLabel.textContent = Number.isNaN(mazeDefault) ? '--' : mazeDefault;

    applyStatusPatch({
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

    resetSaveButton(getById, 'save-botness-config', 'Save Botness Settings');

    setDraft('challengePuzzle', {
      enabled: challengeEnabledToggle.checked,
      count: Number.parseInt(transformCountField.value, 10) || 6
    });
    resetSaveButton(getById, 'save-challenge-puzzle-config', 'Save Challenge Puzzle');

    transformCountField.disabled = !writable;
    challengeEnabledToggle.disabled = !writable;
  };

  const setAdvancedConfigEditorFromConfig = (config, preserveDirty = true) => {
    const field = getById('advanced-config-json');
    if (!field) return;

    const previousBaseline = getDraft('advancedConfig').normalized || '{}';
    const template = buildAdvancedConfigTemplate(config || {});
    const formatted = JSON.stringify(template, null, 2);
    const currentNormalized = normalizeJsonObjectForCompare(field.value);
    const hasUnsavedEdits = field.dataset.dirty === 'true';

    setDraft('advancedConfig', {
      normalized: normalizeJsonObjectForCompare(formatted) || '{}'
    });

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
