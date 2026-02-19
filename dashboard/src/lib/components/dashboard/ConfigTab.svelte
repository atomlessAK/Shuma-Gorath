<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import {
    formatBrowserRulesTextarea,
    formatListTextarea,
    normalizeBrowserRulesForCompare,
    normalizeCountryCodesForCompare,
    normalizeListTextareaForCompare,
    parseBrowserRulesTextarea,
    parseCountryCodesStrict,
    parseHoneypotPathsTextarea,
    parseListTextarea
  } from '../../domain/config-form-utils.js';
  import { advancedConfigTemplatePaths } from '../../domain/config-schema.js';
  import {
    buildTemplateFromPaths,
    normalizeJsonObjectForCompare
  } from '../../domain/core/json-object.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let analyticsSnapshot = null;
  export let configSnapshot = null;
  export let configVersion = 0;
  export let onSaveConfig = null;
  export let onFetchRobotsPreview = null;

  const MAX_DURATION_SECONDS = 31536000;
  const MIN_DURATION_SECONDS = 60;
  const EDGE_MODES = new Set(['off', 'advisory', 'authoritative']);

  let writable = false;
  let hasConfigSnapshot = false;
  let configLoaded = true;
  let lastAppliedConfigVersion = -1;

  let saving = {
    jsRequired: false,
    pow: false,
    challenge: false,
    rateLimit: false,
    honeypot: false,
    browserPolicy: false,
    whitelist: false,
    maze: false,
    cdp: false,
    edgeMode: false,
    geoScoring: false,
    geoRouting: false,
    durations: false,
    robots: false,
    aiPolicy: false,
    advanced: false
  };

  let robotsPreviewOpen = false;
  let robotsPreviewLoading = false;
  let robotsPreviewContent = '';

  let jsRequiredEnforced = true;

  let powEnabled = true;
  let powDifficulty = 15;
  let powTtl = 90;

  let challengePuzzleEnabled = true;
  let challengePuzzleTransformCount = 6;

  let rateLimitThreshold = 80;

  let honeypotEnabled = true;
  let honeypotPaths = '';

  let browserBlockRules = '';
  let browserWhitelistRules = '';

  let networkWhitelist = '';
  let pathWhitelist = '';

  let mazeEnabled = true;
  let mazeAutoBan = true;
  let mazeThreshold = 50;

  let cdpEnabled = true;
  let cdpAutoBan = true;
  let cdpThreshold = 0.6;

  let edgeIntegrationMode = 'off';

  let geoRiskList = '';
  let geoAllowList = '';
  let geoChallengeList = '';
  let geoMazeList = '';
  let geoBlockList = '';

  let durHoneypotDays = 1;
  let durHoneypotHours = 0;
  let durHoneypotMinutes = 0;
  let durRateLimitDays = 0;
  let durRateLimitHours = 1;
  let durRateLimitMinutes = 0;
  let durBrowserDays = 0;
  let durBrowserHours = 6;
  let durBrowserMinutes = 0;
  let durCdpDays = 0;
  let durCdpHours = 12;
  let durCdpMinutes = 0;
  let durAdminDays = 0;
  let durAdminHours = 6;
  let durAdminMinutes = 0;

  let robotsEnabled = true;
  let robotsCrawlDelay = 2;
  let robotsBlockTraining = true;
  let robotsBlockSearch = false;
  let robotsAllowSearch = false;

  let advancedConfigJson = '{}';

  let baseline = {
    jsRequired: { enforced: true },
    pow: { enabled: true, difficulty: 15, ttl: 90 },
    challenge: { enabled: true, count: 6 },
    rateLimit: { value: 80 },
    honeypot: { enabled: true, values: '' },
    browserPolicy: { block: '', whitelist: '' },
    whitelist: { network: '', path: '' },
    maze: { enabled: true, autoBan: true, threshold: 50 },
    cdp: { enabled: true, autoBan: true, threshold: 0.6 },
    edgeMode: { mode: 'off' },
    geo: { risk: '', allow: '', challenge: '', maze: '', block: '' },
    durations: {
      honeypot: 86400,
      rateLimit: 3600,
      browser: 21600,
      cdp: 43200,
      admin: 21600
    },
    robots: { enabled: true, crawlDelay: 2 },
    aiPolicy: { blockTraining: true, blockSearch: false, allowSearch: false },
    advanced: { normalized: '{}' }
  };

  const parseInteger = (value, fallback) => {
    const parsed = Number.parseInt(value, 10);
    return Number.isInteger(parsed) ? parsed : fallback;
  };

  const parseFloatNumber = (value, fallback) => {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  };

  const formatCountryCodes = (values) => {
    if (!Array.isArray(values) || values.length === 0) return '';
    return values
      .map((value) => String(value || '').trim().toUpperCase())
      .filter(Boolean)
      .join(',');
  };

  const normalizeEdgeMode = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return EDGE_MODES.has(normalized) ? normalized : 'off';
  };

  const durationPartsFromSeconds = (seconds, fallbackSeconds) => {
    const source = Number.parseInt(seconds, 10);
    const safe = Number.isFinite(source) && source > 0 ? source : fallbackSeconds;
    const days = Math.floor(safe / 86400);
    const remainingAfterDays = safe - (days * 86400);
    const hours = Math.floor(remainingAfterDays / 3600);
    const remainingAfterHours = remainingAfterDays - (hours * 3600);
    const minutes = Math.floor(remainingAfterHours / 60);
    return {
      days,
      hours,
      minutes
    };
  };

  const durationSeconds = (days, hours, minutes) => {
    const d = parseInteger(days, 0);
    const h = parseInteger(hours, 0);
    const m = parseInteger(minutes, 0);
    return (d * 86400) + (h * 3600) + (m * 60);
  };

  const inRange = (value, min, max) => {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) && parsed >= min && parsed <= max;
  };

  const isDurationTupleValid = (days, hours, minutes) => {
    if (!inRange(days, 0, 365)) return false;
    if (!inRange(hours, 0, 23)) return false;
    if (!inRange(minutes, 0, 59)) return false;
    const total = durationSeconds(days, hours, minutes);
    return total >= MIN_DURATION_SECONDS && total <= MAX_DURATION_SECONDS;
  };

  function setSaving(key, value) {
    saving = {
      ...saving,
      [key]: value === true
    };
  }

  function resetRobotsPreview() {
    robotsPreviewOpen = false;
    robotsPreviewLoading = false;
    robotsPreviewContent = '';
  }

  async function withSave(key, task) {
    if (saving[key]) return null;
    setSaving(key, true);
    try {
      return await task();
    } finally {
      setSaving(key, false);
    }
  }

  function applyConfig(config = {}) {
    configLoaded = true;
    hasConfigSnapshot = config && typeof config === 'object' && Object.keys(config).length > 0;
    writable = config.admin_config_write_enabled === true;

    jsRequiredEnforced = config.js_required_enforced !== false;

    powEnabled = config.pow_enabled !== false;
    powDifficulty = parseInteger(config.pow_difficulty, 15);
    powTtl = parseInteger(config.pow_ttl_seconds, 90);

    challengePuzzleEnabled = config.challenge_puzzle_enabled !== false;
    challengePuzzleTransformCount = parseInteger(config.challenge_puzzle_transform_count, 6);

    rateLimitThreshold = parseInteger(config.rate_limit, 80);

    honeypotEnabled = config.honeypot_enabled !== false;
    honeypotPaths = formatListTextarea(config.honeypots);

    browserBlockRules = formatBrowserRulesTextarea(config.browser_block);
    browserWhitelistRules = formatBrowserRulesTextarea(config.browser_whitelist);

    networkWhitelist = formatListTextarea(config.whitelist);
    pathWhitelist = formatListTextarea(config.path_whitelist);

    mazeEnabled = config.maze_enabled !== false;
    mazeAutoBan = config.maze_auto_ban !== false;
    mazeThreshold = parseInteger(config.maze_auto_ban_threshold, 50);

    cdpEnabled = config.cdp_detection_enabled !== false;
    cdpAutoBan = config.cdp_auto_ban !== false;
    cdpThreshold = Number(parseFloatNumber(config.cdp_detection_threshold, 0.6).toFixed(1));

    edgeIntegrationMode = normalizeEdgeMode(config.edge_integration_mode);

    geoRiskList = formatCountryCodes(config.geo_risk);
    geoAllowList = formatCountryCodes(config.geo_allow);
    geoChallengeList = formatCountryCodes(config.geo_challenge);
    geoMazeList = formatCountryCodes(config.geo_maze);
    geoBlockList = formatCountryCodes(config.geo_block);

    const banDurations = config && typeof config.ban_durations === 'object'
      ? config.ban_durations
      : {};

    const honeypotParts = durationPartsFromSeconds(banDurations.honeypot, 86400);
    durHoneypotDays = honeypotParts.days;
    durHoneypotHours = honeypotParts.hours;
    durHoneypotMinutes = honeypotParts.minutes;

    const rateLimitParts = durationPartsFromSeconds(banDurations.rate_limit, 3600);
    durRateLimitDays = rateLimitParts.days;
    durRateLimitHours = rateLimitParts.hours;
    durRateLimitMinutes = rateLimitParts.minutes;

    const browserParts = durationPartsFromSeconds(banDurations.browser, 21600);
    durBrowserDays = browserParts.days;
    durBrowserHours = browserParts.hours;
    durBrowserMinutes = browserParts.minutes;

    const cdpParts = durationPartsFromSeconds(banDurations.cdp, 43200);
    durCdpDays = cdpParts.days;
    durCdpHours = cdpParts.hours;
    durCdpMinutes = cdpParts.minutes;

    const adminParts = durationPartsFromSeconds(banDurations.admin, 21600);
    durAdminDays = adminParts.days;
    durAdminHours = adminParts.hours;
    durAdminMinutes = adminParts.minutes;

    robotsEnabled = config.robots_enabled !== false;
    robotsCrawlDelay = parseInteger(config.robots_crawl_delay, 2);

    robotsBlockTraining = (config.ai_policy_block_training ?? config.robots_block_ai_training) !== false;
    robotsBlockSearch = (config.ai_policy_block_search ?? config.robots_block_ai_search) === true;
    const aiAllowSearchEngines = config.ai_policy_allow_search_engines ?? config.robots_allow_search_engines;
    robotsAllowSearch = aiAllowSearchEngines === undefined ? false : aiAllowSearchEngines !== true;

    const advancedTemplate = buildTemplateFromPaths(config, advancedConfigTemplatePaths || []);
    const advancedText = JSON.stringify(advancedTemplate, null, 2);
    advancedConfigJson = advancedText;

    baseline = {
      jsRequired: { enforced: jsRequiredEnforced },
      pow: {
        enabled: powEnabled,
        difficulty: Number(powDifficulty),
        ttl: Number(powTtl)
      },
      challenge: {
        enabled: challengePuzzleEnabled,
        count: Number(challengePuzzleTransformCount)
      },
      rateLimit: { value: Number(rateLimitThreshold) },
      honeypot: {
        enabled: honeypotEnabled,
        values: normalizeListTextareaForCompare(honeypotPaths)
      },
      browserPolicy: {
        block: normalizeBrowserRulesForCompare(browserBlockRules),
        whitelist: normalizeBrowserRulesForCompare(browserWhitelistRules)
      },
      whitelist: {
        network: normalizeListTextareaForCompare(networkWhitelist),
        path: normalizeListTextareaForCompare(pathWhitelist)
      },
      maze: {
        enabled: mazeEnabled,
        autoBan: mazeAutoBan,
        threshold: Number(mazeThreshold)
      },
      cdp: {
        enabled: cdpEnabled,
        autoBan: cdpAutoBan,
        threshold: Number(cdpThreshold)
      },
      edgeMode: {
        mode: edgeIntegrationMode
      },
      geo: {
        risk: normalizeCountryCodesForCompare(geoRiskList),
        allow: normalizeCountryCodesForCompare(geoAllowList),
        challenge: normalizeCountryCodesForCompare(geoChallengeList),
        maze: normalizeCountryCodesForCompare(geoMazeList),
        block: normalizeCountryCodesForCompare(geoBlockList)
      },
      durations: {
        honeypot: durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes),
        rateLimit: durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes),
        browser: durationSeconds(durBrowserDays, durBrowserHours, durBrowserMinutes),
        cdp: durationSeconds(durCdpDays, durCdpHours, durCdpMinutes),
        admin: durationSeconds(durAdminDays, durAdminHours, durAdminMinutes)
      },
      robots: {
        enabled: robotsEnabled,
        crawlDelay: Number(robotsCrawlDelay)
      },
      aiPolicy: {
        blockTraining: robotsBlockTraining,
        blockSearch: robotsBlockSearch,
        allowSearch: robotsAllowSearch
      },
      advanced: {
        normalized: normalizeJsonObjectForCompare(advancedText) || '{}'
      }
    };

    resetRobotsPreview();
  }

  async function submitConfigPatch(sectionKey, patch, successMessage) {
    if (typeof onSaveConfig !== 'function') return;
    const nextConfig = await withSave(sectionKey, async () =>
      onSaveConfig(patch, { successMessage })
    );
    if (nextConfig && typeof nextConfig === 'object') {
      applyConfig(nextConfig);
    }
  }

  async function saveJsRequiredConfig() {
    if (saveJsRequiredDisabled) return;
    await submitConfigPatch('jsRequired', {
      js_required_enforced: jsRequiredEnforced
    }, 'JS required policy saved');
  }

  async function savePowConfig() {
    if (savePowDisabled) return;
    await submitConfigPatch('pow', {
      pow_enabled: powEnabled,
      pow_difficulty: Number(powDifficulty),
      pow_ttl_seconds: Number(powTtl)
    }, 'PoW settings saved');
  }

  async function saveChallengePuzzleConfig() {
    if (saveChallengePuzzleDisabled) return;
    await submitConfigPatch('challenge', {
      challenge_puzzle_enabled: challengePuzzleEnabled,
      challenge_puzzle_transform_count: Number(challengePuzzleTransformCount)
    }, 'Challenge puzzle settings saved');
  }

  async function saveRateLimitConfig() {
    if (saveRateLimitDisabled) return;
    await submitConfigPatch('rateLimit', {
      rate_limit: Number(rateLimitThreshold)
    }, 'Rate limit saved');
  }

  async function saveHoneypotConfig() {
    if (saveHoneypotDisabled) return;
    const honeypots = parseHoneypotPathsTextarea(honeypotPaths);
    await submitConfigPatch('honeypot', {
      honeypot_enabled: honeypotEnabled,
      honeypots
    }, 'Honeypot settings saved');
  }

  async function saveBrowserPolicyConfig() {
    if (saveBrowserPolicyDisabled) return;
    await submitConfigPatch('browserPolicy', {
      browser_block: parseBrowserRulesTextarea(browserBlockRules),
      browser_whitelist: parseBrowserRulesTextarea(browserWhitelistRules)
    }, 'Browser policy saved');
  }

  async function saveWhitelistConfig() {
    if (saveWhitelistDisabled) return;
    await submitConfigPatch('whitelist', {
      whitelist: parseListTextarea(networkWhitelist),
      path_whitelist: parseListTextarea(pathWhitelist)
    }, 'Bypass allowlists saved');
  }

  async function saveMazeConfig() {
    if (saveMazeDisabled) return;
    await submitConfigPatch('maze', {
      maze_enabled: mazeEnabled,
      maze_auto_ban: mazeAutoBan,
      maze_auto_ban_threshold: Number(mazeThreshold)
    }, 'Maze settings saved');
  }

  async function saveCdpConfig() {
    if (saveCdpDisabled) return;
    await submitConfigPatch('cdp', {
      cdp_detection_enabled: cdpEnabled,
      cdp_auto_ban: cdpAutoBan,
      cdp_detection_threshold: Number(cdpThreshold)
    }, 'CDP settings saved');
  }

  async function saveEdgeIntegrationModeConfig() {
    if (saveEdgeModeDisabled) return;
    await submitConfigPatch('edgeMode', {
      edge_integration_mode: edgeIntegrationMode
    }, 'Edge integration mode saved');
  }

  async function saveGeoScoringConfig() {
    if (saveGeoScoringDisabled) return;
    await submitConfigPatch('geoScoring', {
      geo_risk: parseCountryCodesStrict(geoRiskList)
    }, 'GEO scoring saved');
  }

  async function saveGeoRoutingConfig() {
    if (saveGeoRoutingDisabled) return;
    await submitConfigPatch('geoRouting', {
      geo_allow: parseCountryCodesStrict(geoAllowList),
      geo_challenge: parseCountryCodesStrict(geoChallengeList),
      geo_maze: parseCountryCodesStrict(geoMazeList),
      geo_block: parseCountryCodesStrict(geoBlockList)
    }, 'GEO routing saved');
  }

  async function saveDurationsConfig() {
    if (saveDurationsDisabled) return;
    await submitConfigPatch('durations', {
      ban_durations: {
        honeypot: durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes),
        rate_limit: durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes),
        browser: durationSeconds(durBrowserDays, durBrowserHours, durBrowserMinutes),
        cdp: durationSeconds(durCdpDays, durCdpHours, durCdpMinutes),
        admin: durationSeconds(durAdminDays, durAdminHours, durAdminMinutes)
      }
    }, 'Ban durations saved');
  }

  async function saveRobotsConfig() {
    if (saveRobotsDisabled) return;
    await submitConfigPatch('robots', {
      robots_enabled: robotsEnabled,
      robots_crawl_delay: Number(robotsCrawlDelay)
    }, 'robots.txt serving saved');
    if (robotsPreviewOpen) {
      await refreshRobotsPreview();
    }
  }

  async function saveAiPolicyConfig() {
    if (saveAiPolicyDisabled) return;
    await submitConfigPatch('aiPolicy', {
      ai_policy_block_training: robotsBlockTraining,
      ai_policy_block_search: robotsBlockSearch,
      ai_policy_allow_search_engines: !robotsAllowSearch
    }, 'AI bot policy saved');
    if (robotsPreviewOpen) {
      await refreshRobotsPreview();
    }
  }

  async function saveAdvancedConfig() {
    if (saveAdvancedDisabled) return;
    const parsed = JSON.parse(advancedConfigJson);
    await submitConfigPatch('advanced', parsed, 'Advanced config saved');
  }

  async function refreshRobotsPreview() {
    if (typeof onFetchRobotsPreview !== 'function') return;
    robotsPreviewLoading = true;
    try {
      const payload = await onFetchRobotsPreview();
      robotsPreviewContent = payload && typeof payload.content === 'string'
        ? payload.content
        : '# No preview available';
    } catch (error) {
      robotsPreviewContent = `# Error loading preview: ${error && error.message ? error.message : 'Unknown error'}`;
    } finally {
      robotsPreviewLoading = false;
    }
  }

  async function toggleRobotsPreview() {
    if (robotsPreviewOpen) {
      robotsPreviewOpen = false;
      return;
    }
    robotsPreviewOpen = true;
    await refreshRobotsPreview();
  }

  const readBool = (value) => value === true;

  $: hasAnalyticsSnapshot = analyticsSnapshot && typeof analyticsSnapshot === 'object';
  $: testModeEnabled = hasAnalyticsSnapshot ? analyticsSnapshot.test_mode === true : null;
  $: testModeStatusText = testModeEnabled === null
    ? 'LOADING...'
    : (testModeEnabled ? 'ENABLED (LOGGING ONLY)' : 'DISABLED (BLOCKING ACTIVE)');
  $: testModeStatusClass = `text-muted status-value ${
    testModeEnabled === null
      ? ''
      : (testModeEnabled ? 'test-mode-status--enabled' : 'test-mode-status--disabled')
  }`.trim();

  $: configModeText = !configLoaded
    ? 'Admin page configuration state is LOADING.'
    : (hasConfigSnapshot
      ? (writable
      ? 'Admin page configuration enabled. Saved changes persist across builds. Set SHUMA_ADMIN_CONFIG_WRITE_ENABLED to false in deployment env to disable.'
      : 'Admin page configuration disabled. Set SHUMA_ADMIN_CONFIG_WRITE_ENABLED to true to enable.')
      : 'Admin page configuration loaded, but the snapshot is empty.');

  $: jsRequiredDirty = readBool(jsRequiredEnforced) !== baseline.jsRequired.enforced;
  $: saveJsRequiredDisabled = !writable || !jsRequiredDirty || saving.jsRequired;

  $: powValid = inRange(powDifficulty, 12, 20) && inRange(powTtl, 30, 300);
  $: powDirty = (
    readBool(powEnabled) !== baseline.pow.enabled ||
    Number(powDifficulty) !== baseline.pow.difficulty ||
    Number(powTtl) !== baseline.pow.ttl
  );
  $: savePowDisabled = !writable || !powDirty || !powValid || saving.pow;

  $: challengePuzzleValid = inRange(challengePuzzleTransformCount, 4, 8);
  $: challengePuzzleDirty = (
    readBool(challengePuzzleEnabled) !== baseline.challenge.enabled ||
    Number(challengePuzzleTransformCount) !== baseline.challenge.count
  );
  $: saveChallengePuzzleDisabled = !writable || !challengePuzzleDirty || !challengePuzzleValid || saving.challenge;

  $: rateLimitValid = inRange(rateLimitThreshold, 1, 1000000);
  $: rateLimitDirty = Number(rateLimitThreshold) !== baseline.rateLimit.value;
  $: saveRateLimitDisabled = !writable || !rateLimitDirty || !rateLimitValid || saving.rateLimit;

  $: honeypotNormalized = normalizeListTextareaForCompare(honeypotPaths);
  $: honeypotValid = (() => {
    try {
      parseHoneypotPathsTextarea(honeypotPaths);
      return true;
    } catch (_error) {
      return false;
    }
  })();
  $: honeypotDirty = (
    readBool(honeypotEnabled) !== baseline.honeypot.enabled ||
    honeypotNormalized !== baseline.honeypot.values
  );
  $: saveHoneypotDisabled = !writable || !honeypotDirty || !honeypotValid || saving.honeypot;

  $: browserBlockNormalized = normalizeBrowserRulesForCompare(browserBlockRules);
  $: browserWhitelistNormalized = normalizeBrowserRulesForCompare(browserWhitelistRules);
  $: browserPolicyValid = browserBlockNormalized !== '__invalid__' && browserWhitelistNormalized !== '__invalid__';
  $: browserPolicyDirty = (
    browserBlockNormalized !== baseline.browserPolicy.block ||
    browserWhitelistNormalized !== baseline.browserPolicy.whitelist
  );
  $: saveBrowserPolicyDisabled = !writable || !browserPolicyDirty || !browserPolicyValid || saving.browserPolicy;

  $: whitelistNetworkNormalized = normalizeListTextareaForCompare(networkWhitelist);
  $: whitelistPathNormalized = normalizeListTextareaForCompare(pathWhitelist);
  $: whitelistDirty = (
    whitelistNetworkNormalized !== baseline.whitelist.network ||
    whitelistPathNormalized !== baseline.whitelist.path
  );
  $: saveWhitelistDisabled = !writable || !whitelistDirty || saving.whitelist;

  $: mazeValid = inRange(mazeThreshold, 5, 500);
  $: mazeDirty = (
    readBool(mazeEnabled) !== baseline.maze.enabled ||
    readBool(mazeAutoBan) !== baseline.maze.autoBan ||
    Number(mazeThreshold) !== baseline.maze.threshold
  );
  $: saveMazeDisabled = !writable || !mazeDirty || !mazeValid || saving.maze;

  $: cdpValid = inRange(cdpThreshold, 0.3, 1.0);
  $: cdpDirty = (
    readBool(cdpEnabled) !== baseline.cdp.enabled ||
    readBool(cdpAutoBan) !== baseline.cdp.autoBan ||
    Number(cdpThreshold) !== baseline.cdp.threshold
  );
  $: saveCdpDisabled = !writable || !cdpDirty || !cdpValid || saving.cdp;

  $: edgeModeDirty = normalizeEdgeMode(edgeIntegrationMode) !== baseline.edgeMode.mode;
  $: saveEdgeModeDisabled = !writable || !edgeModeDirty || saving.edgeMode;

  $: geoRiskNormalized = normalizeCountryCodesForCompare(geoRiskList);
  $: geoAllowNormalized = normalizeCountryCodesForCompare(geoAllowList);
  $: geoChallengeNormalized = normalizeCountryCodesForCompare(geoChallengeList);
  $: geoMazeNormalized = normalizeCountryCodesForCompare(geoMazeList);
  $: geoBlockNormalized = normalizeCountryCodesForCompare(geoBlockList);

  $: geoScoringValid = (() => {
    try {
      parseCountryCodesStrict(geoRiskList);
      return true;
    } catch (_error) {
      return false;
    }
  })();

  $: geoRoutingValid = (() => {
    try {
      parseCountryCodesStrict(geoAllowList);
      parseCountryCodesStrict(geoChallengeList);
      parseCountryCodesStrict(geoMazeList);
      parseCountryCodesStrict(geoBlockList);
      return true;
    } catch (_error) {
      return false;
    }
  })();

  $: geoScoringDirty = geoRiskNormalized !== baseline.geo.risk;
  $: geoRoutingDirty = (
    geoAllowNormalized !== baseline.geo.allow ||
    geoChallengeNormalized !== baseline.geo.challenge ||
    geoMazeNormalized !== baseline.geo.maze ||
    geoBlockNormalized !== baseline.geo.block
  );

  $: saveGeoScoringDisabled = !writable || !geoScoringDirty || !geoScoringValid || saving.geoScoring;
  $: saveGeoRoutingDisabled = !writable || !geoRoutingDirty || !geoRoutingValid || saving.geoRouting;

  $: honeypotDurationSeconds = durationSeconds(durHoneypotDays, durHoneypotHours, durHoneypotMinutes);
  $: rateDurationSeconds = durationSeconds(durRateLimitDays, durRateLimitHours, durRateLimitMinutes);
  $: browserDurationSeconds = durationSeconds(durBrowserDays, durBrowserHours, durBrowserMinutes);
  $: cdpDurationSeconds = durationSeconds(durCdpDays, durCdpHours, durCdpMinutes);
  $: adminDurationSeconds = durationSeconds(durAdminDays, durAdminHours, durAdminMinutes);

  $: durationsValid = (
    isDurationTupleValid(durHoneypotDays, durHoneypotHours, durHoneypotMinutes) &&
    isDurationTupleValid(durRateLimitDays, durRateLimitHours, durRateLimitMinutes) &&
    isDurationTupleValid(durBrowserDays, durBrowserHours, durBrowserMinutes) &&
    isDurationTupleValid(durCdpDays, durCdpHours, durCdpMinutes) &&
    isDurationTupleValid(durAdminDays, durAdminHours, durAdminMinutes)
  );

  $: durationsDirty = (
    honeypotDurationSeconds !== baseline.durations.honeypot ||
    rateDurationSeconds !== baseline.durations.rateLimit ||
    browserDurationSeconds !== baseline.durations.browser ||
    cdpDurationSeconds !== baseline.durations.cdp ||
    adminDurationSeconds !== baseline.durations.admin
  );

  $: saveDurationsDisabled = !writable || !durationsDirty || !durationsValid || saving.durations;

  $: robotsValid = inRange(robotsCrawlDelay, 0, 60);
  $: robotsDirty = (
    readBool(robotsEnabled) !== baseline.robots.enabled ||
    Number(robotsCrawlDelay) !== baseline.robots.crawlDelay
  );
  $: saveRobotsDisabled = !writable || !robotsDirty || !robotsValid || saving.robots;

  $: aiPolicyDirty = (
    readBool(robotsBlockTraining) !== baseline.aiPolicy.blockTraining ||
    readBool(robotsBlockSearch) !== baseline.aiPolicy.blockSearch ||
    readBool(robotsAllowSearch) !== baseline.aiPolicy.allowSearch
  );
  $: saveAiPolicyDisabled = !writable || !aiPolicyDirty || saving.aiPolicy;

  $: advancedNormalized = normalizeJsonObjectForCompare(advancedConfigJson);
  $: advancedValid = advancedNormalized !== null;
  $: advancedDirty = advancedValid && advancedNormalized !== baseline.advanced.normalized;
  $: saveAdvancedDisabled = !writable || !advancedDirty || !advancedValid || saving.advanced;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      applyConfig(configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {});
    }
  }
</script>

<section
  id="dashboard-panel-config"
  class="admin-group"
  data-dashboard-tab-panel="config"
  aria-labelledby="dashboard-tab-config"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="config" status={tabStatus} />
  <p id="config-mode-subtitle" class="admin-group-subtitle text-muted">{configModeText}</p>
  <div class="controls-grid controls-grid--config">
    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Test Mode</h3>
      <p class="control-desc text-muted">Use for safe tuning. Enabled logs detections without blocking; disable to enforce defenses.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="test-mode-toggle">Enable Test Mode</label>
          <label class="toggle-switch" for="test-mode-toggle">
            <input type="checkbox" id="test-mode-toggle" aria-label="Enable Test Mode" checked={testModeEnabled === true} disabled>
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
      <span id="test-mode-status" class={testModeStatusClass}>{testModeStatusText}</span>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>JS Required</h3>
      <p class="control-desc text-muted">Toggle whether normal requests require JS verification. Disable only for non-JS clients; this weakens bot defense and bypasses PoW on normal paths.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="js-required-enforced-toggle">Enforce JS Required</label>
          <label class="toggle-switch" for="js-required-enforced-toggle">
            <input type="checkbox" id="js-required-enforced-toggle" aria-label="Enforce JS required" bind:checked={jsRequiredEnforced}>
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>
      <button id="save-js-required-config" class="btn btn-submit" disabled={saveJsRequiredDisabled} on:click={saveJsRequiredConfig}>Save JS Required</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Proof-of-Work (PoW)</h3>
      <p class="control-desc text-muted">Set verification work cost. Higher values increase scraper cost but can add friction on slower devices.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="pow-enabled-toggle">Enable PoW</label>
          <label class="toggle-switch" for="pow-enabled-toggle">
            <input type="checkbox" id="pow-enabled-toggle" aria-label="Enable PoW challenge verification" bind:checked={powEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label" for="pow-difficulty">Difficulty (bits)</label>
          <input class="input-field" type="number" id="pow-difficulty" min="12" max="20" step="1" inputmode="numeric" aria-label="PoW difficulty in leading zero bits" bind:value={powDifficulty}>
        </div>
        <div class="input-row">
          <label class="control-label" for="pow-ttl">Seed TTL (seconds)</label>
          <input class="input-field" type="number" id="pow-ttl" min="30" max="300" step="1" inputmode="numeric" aria-label="PoW seed TTL in seconds" bind:value={powTtl}>
        </div>
      </div>
      <button id="save-pow-config" class="btn btn-submit" disabled={savePowDisabled} on:click={savePowConfig}>Save PoW Settings</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Challenge Puzzle</h3>
      <p class="control-desc text-muted">Set how many transform options are shown in puzzle challenges (higher values can increase solve time).</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="challenge-puzzle-enabled-toggle">Enable Challenge Puzzle</label>
          <label class="toggle-switch" for="challenge-puzzle-enabled-toggle">
            <input type="checkbox" id="challenge-puzzle-enabled-toggle" aria-label="Enable challenge puzzle routing" bind:checked={challengePuzzleEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label" for="challenge-puzzle-transform-count">Transform Options</label>
          <input class="input-field" type="number" id="challenge-puzzle-transform-count" min="4" max="8" step="1" inputmode="numeric" aria-label="Challenge transform option count" bind:value={challengePuzzleTransformCount}>
        </div>
      </div>
      <button id="save-challenge-puzzle-config" class="btn btn-submit" disabled={saveChallengePuzzleDisabled} on:click={saveChallengePuzzleConfig}>Save Challenge Puzzle</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Rate Limiting</h3>
      <p class="control-desc text-muted">Set allowed requests per minute. Lower values are stricter but can affect legitimate burst traffic.</p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label control-label--wide" for="rate-limit-threshold">Requests Per Minute</label>
          <input class="input-field" type="number" id="rate-limit-threshold" min="1" max="1000000" step="1" inputmode="numeric" aria-label="Rate limit requests per minute" bind:value={rateLimitThreshold}>
        </div>
      </div>
      <button id="save-rate-limit-config" class="btn btn-submit" disabled={saveRateLimitDisabled} on:click={saveRateLimitConfig}>Save Rate Limit</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Honeypot Paths</h3>
      <p class="control-desc text-muted">One path per line. Requests that hit these paths are treated as high-confidence bot behavior. Paths must start with <code>/</code>.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="honeypot-enabled-toggle">Enable Honeypot</label>
          <label class="toggle-switch" for="honeypot-enabled-toggle">
            <input type="checkbox" id="honeypot-enabled-toggle" aria-label="Enable honeypot" bind:checked={honeypotEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="geo-field">
          <label class="control-label" for="honeypot-paths">Paths</label>
          <textarea class="input-field geo-textarea" id="honeypot-paths" rows="3" aria-label="Honeypot paths" spellcheck="false" bind:value={honeypotPaths}></textarea>
        </div>
      </div>
      <button id="save-honeypot-config" class="btn btn-submit" disabled={saveHoneypotDisabled} on:click={saveHoneypotConfig}>Save Honeypots</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Browser Policy</h3>
      <p class="control-desc text-muted">Use one rule per line in <code>BrowserName,min_major</code> format (for example <code>Chrome,120</code>).</p>
      <div class="admin-controls">
        <div class="geo-field">
          <label class="control-label" for="browser-block-rules">Minimum Versions (Block)</label>
          <textarea class="input-field geo-textarea" id="browser-block-rules" rows="3" aria-label="Browser block minimum versions" spellcheck="false" bind:value={browserBlockRules}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="browser-whitelist-rules">Allowlist Exceptions</label>
          <textarea class="input-field geo-textarea" id="browser-whitelist-rules" rows="2" aria-label="Browser allowlist exceptions" spellcheck="false" bind:value={browserWhitelistRules}></textarea>
        </div>
      </div>
      <button id="save-browser-policy-config" class="btn btn-submit" disabled={saveBrowserPolicyDisabled} on:click={saveBrowserPolicyConfig}>Save Browser Policy</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Bypass Allowlists</h3>
      <p class="control-desc text-muted">Define trusted bypass entries. Use one entry per line.</p>
      <div class="admin-controls">
        <div class="geo-field">
          <label class="control-label" for="network-whitelist">IP/CIDR Allowlist</label>
          <textarea class="input-field geo-textarea" id="network-whitelist" rows="3" aria-label="IP and CIDR allowlist" spellcheck="false" bind:value={networkWhitelist}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="path-whitelist">Path Allowlist</label>
          <textarea class="input-field geo-textarea" id="path-whitelist" rows="3" aria-label="Path allowlist" spellcheck="false" bind:value={pathWhitelist}></textarea>
        </div>
      </div>
      <button id="save-whitelist-config" class="btn btn-submit" disabled={saveWhitelistDisabled} on:click={saveWhitelistConfig}>Save Allowlists</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Maze</h3>
      <p class="control-desc text-muted">
        Control trap-page routing and optional auto-ban. Lower thresholds ban faster but may increase false positives.
        <a id="preview-maze-link" href="/admin/maze/preview" target="_blank" rel="noopener noreferrer">Preview Maze</a>
        in a non-operational view (admin session required).
      </p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label" for="maze-enabled-toggle">Maze Enabled</label>
          <label class="toggle-switch">
            <input type="checkbox" id="maze-enabled-toggle" aria-label="Enable maze" bind:checked={mazeEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="toggle-row">
          <label class="control-label" for="maze-auto-ban-toggle">Ban on entry</label>
          <label class="toggle-switch">
            <input type="checkbox" id="maze-auto-ban-toggle" aria-label="Enable maze ban on entry" bind:checked={mazeAutoBan}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label" for="maze-threshold">Ban Threshold (pages)</label>
          <input class="input-field" type="number" id="maze-threshold" min="5" max="500" step="1" inputmode="numeric" aria-label="Maze ban threshold in pages" bind:value={mazeThreshold}>
        </div>
      </div>
      <button id="save-maze-config" class="btn btn-submit" disabled={saveMazeDisabled} on:click={saveMazeConfig}>Save Maze Settings</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>CDP (Detect Browser Automation)</h3>
      <p class="control-desc text-muted">Control automation-signal detection and optional auto-ban. Stricter thresholds catch more bots but may increase false positives.</p>
      <div class="admin-controls">
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="cdp-enabled-toggle">Enable Detection</label>
          <label class="toggle-switch">
            <input type="checkbox" id="cdp-enabled-toggle" aria-label="Enable CDP detection" bind:checked={cdpEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="cdp-auto-ban-toggle">Auto-ban on Detection</label>
          <label class="toggle-switch">
            <input type="checkbox" id="cdp-auto-ban-toggle" aria-label="Enable CDP auto-ban" bind:checked={cdpAutoBan}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="slider-control">
          <div class="slider-header">
            <label class="control-label control-label--wide" for="cdp-threshold-slider">Detection Threshold</label>
            <span id="cdp-threshold-value" class="slider-badge">{Number(cdpThreshold).toFixed(1)}</span>
          </div>
          <input type="range" id="cdp-threshold-slider" min="0.3" max="1.0" step="0.1" aria-label="CDP detection threshold" bind:value={cdpThreshold}>
          <div class="slider-labels">
            <span>Strict</span>
            <span>Permissive</span>
          </div>
        </div>
      </div>
      <button id="save-cdp-config" class="btn btn-submit" disabled={saveCdpDisabled} on:click={saveCdpConfig}>Save CDP Settings</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Edge Integration Mode</h3>
      <p class="control-desc text-muted">Control how external edge bot outcomes influence local policy: off ignores edge outcomes, advisory records them without direct enforcement, authoritative allows strong edge outcomes to short-circuit.</p>
      <div class="admin-controls">
        <div class="input-row">
          <label class="control-label control-label--wide" for="edge-integration-mode-select">Mode</label>
          <select class="input-field" id="edge-integration-mode-select" aria-label="Edge integration mode" bind:value={edgeIntegrationMode}>
            <option value="off">off</option>
            <option value="advisory">advisory</option>
            <option value="authoritative">authoritative</option>
          </select>
        </div>
      </div>
      <button id="save-edge-integration-mode-config" class="btn btn-submit" disabled={saveEdgeModeDisabled} on:click={saveEdgeIntegrationModeConfig}>Save Edge Integration Mode</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>GEO Risk Based Scoring</h3>
      <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be receive added botness risk to contribute to the combined score.</p>
      <div class="admin-controls geo-controls">
        <div class="geo-field">
          <label class="control-label" for="geo-risk-list">Scoring Countries</label>
          <textarea class="input-field geo-textarea" id="geo-risk-list" rows="1" aria-label="GEO scoring countries" spellcheck="false" bind:value={geoRiskList}></textarea>
        </div>
      </div>
      <button id="save-geo-scoring-config" class="btn btn-submit" disabled={saveGeoScoringDisabled} on:click={saveGeoScoringConfig}>Save GEO Scoring</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>GEO Risk Based Routing</h3>
      <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be automatically routed. Precedence is Block &gt; Maze &gt; Challenge &gt; Allow.</p>
      <div class="admin-controls geo-controls">
        <div class="geo-field">
          <label class="control-label" for="geo-block-list">Block Countries</label>
          <textarea class="input-field geo-textarea" id="geo-block-list" rows="1" aria-label="GEO block countries" spellcheck="false" bind:value={geoBlockList}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="geo-maze-list">Maze Countries</label>
          <textarea class="input-field geo-textarea" id="geo-maze-list" rows="1" aria-label="GEO maze countries" spellcheck="false" bind:value={geoMazeList}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="geo-challenge-list">Challenge Countries</label>
          <textarea class="input-field geo-textarea" id="geo-challenge-list" rows="1" aria-label="GEO challenge countries" spellcheck="false" bind:value={geoChallengeList}></textarea>
        </div>
        <div class="geo-field">
          <label class="control-label" for="geo-allow-list">Allow Countries</label>
          <textarea class="input-field geo-textarea" id="geo-allow-list" rows="1" aria-label="GEO allow countries" spellcheck="false" bind:value={geoAllowList}></textarea>
        </div>
      </div>
      <button id="save-geo-routing-config" class="btn btn-submit" disabled={saveGeoRoutingDisabled} on:click={saveGeoRoutingConfig}>Save GEO Routing</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Ban Durations</h3>
      <p class="control-desc text-muted">Set ban length in days, hours, and minutes per trigger type. Longer bans increase deterrence but slow recovery from false positives.</p>
      <div class="duration-grid">
        <div class="duration-row">
          <label class="control-label" for="dur-honeypot-days">Maze Threshold Exceeded</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-honeypot-days">
              <input id="dur-honeypot-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durHoneypotDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-honeypot-hours">
              <input id="dur-honeypot-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durHoneypotHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-honeypot-minutes">
              <input id="dur-honeypot-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durHoneypotMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-rate-limit-days">Rate Limit Exceeded</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-rate-limit-days">
              <input id="dur-rate-limit-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durRateLimitDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-rate-limit-hours">
              <input id="dur-rate-limit-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durRateLimitHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-rate-limit-minutes">
              <input id="dur-rate-limit-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durRateLimitMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-browser-days">Browser Automation Detected</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-browser-days">
              <input id="dur-browser-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durBrowserDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-browser-hours">
              <input id="dur-browser-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durBrowserHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-browser-minutes">
              <input id="dur-browser-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durBrowserMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-cdp-days">CDP Automation Detected</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-cdp-days">
              <input id="dur-cdp-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durCdpDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-cdp-hours">
              <input id="dur-cdp-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durCdpHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-cdp-minutes">
              <input id="dur-cdp-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durCdpMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
        <div class="duration-row">
          <label class="control-label" for="dur-admin-days">Admin Manual Ban Default</label>
          <div class="duration-inputs">
            <label class="duration-input" for="dur-admin-days">
              <input id="dur-admin-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" bind:value={durAdminDays} />
              <span class="input-unit">days</span>
            </label>
            <label class="duration-input" for="dur-admin-hours">
              <input id="dur-admin-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" bind:value={durAdminHours} />
              <span class="input-unit">hrs</span>
            </label>
            <label class="duration-input" for="dur-admin-minutes">
              <input id="dur-admin-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" bind:value={durAdminMinutes} />
              <span class="input-unit">mins</span>
            </label>
          </div>
        </div>
      </div>
      <button id="save-durations-btn" class="btn btn-submit" disabled={saveDurationsDisabled} on:click={saveDurationsConfig}>Save Durations</button>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Robots and AI Bot Policy</h3>
      <p class="control-desc text-muted">Keep robots.txt serving controls separate from AI bot policy controls.</p>
      <div class="admin-controls">
        <h4 class="control-subtitle">robots.txt Serving</h4>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-enabled-toggle">Serve robots.txt</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-enabled-toggle" aria-label="Serve robots.txt" bind:checked={robotsEnabled}>
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="input-row">
          <label class="control-label control-label--wide" for="robots-crawl-delay">Crawl Delay (seconds)</label>
          <input class="input-field" type="number" id="robots-crawl-delay" min="0" max="60" step="1" inputmode="numeric" aria-label="Robots crawl delay in seconds" bind:value={robotsCrawlDelay}>
        </div>
        <button id="save-robots-config" class="btn btn-submit" disabled={saveRobotsDisabled} on:click={saveRobotsConfig}>Save robots serving</button>
        <h4 class="control-subtitle">AI Bot Policy</h4>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-block-training-toggle">Opt-out AI Training</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-block-training-toggle" aria-label="Opt-out AI training" bind:checked={robotsBlockTraining}>
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-hint">GPTBot, CCBot, ClaudeBot</span>
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-block-search-toggle">Opt-out AI Search</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-block-search-toggle" aria-label="Opt-out AI search" bind:checked={robotsBlockSearch}>
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-hint">PerplexityBot, etc.</span>
        </div>
        <div class="toggle-row">
          <label class="control-label control-label--wide" for="robots-allow-search-toggle">Restrict Search Engines</label>
          <label class="toggle-switch">
            <input type="checkbox" id="robots-allow-search-toggle" aria-label="Restrict search engines" bind:checked={robotsAllowSearch}>
            <span class="toggle-slider"></span>
          </label>
          <span class="toggle-hint">Google, Bing, etc.</span>
        </div>
      </div>
      <button id="save-ai-policy-config" class="btn btn-submit" disabled={saveAiPolicyDisabled} on:click={saveAiPolicyConfig}>Save AI bot policy</button>
      <button id="preview-robots" class="btn btn-subtle" on:click={toggleRobotsPreview}>{robotsPreviewOpen ? 'Hide robots.txt' : 'Show robots.txt'}</button>
      <div id="robots-preview" class="robots-preview panel pad-sm" class:hidden={!robotsPreviewOpen}>
        <h4>robots.txt Preview</h4>
        <pre id="robots-preview-content">{robotsPreviewLoading ? 'Loading...' : robotsPreviewContent}</pre>
      </div>
    </div>

    <div class="control-group panel-soft pad-md config-edit-pane" class:hidden={!writable}>
      <h3>Advanced Config JSON</h3>
      <p class="control-desc text-muted">Directly edit writable config keys as a JSON object. This exposes advanced keys that do not yet have dedicated pane controls.</p>
      <div class="admin-controls">
        <div class="geo-field">
          <label class="control-label" for="advanced-config-json">JSON Patch</label>
          <textarea class="input-field geo-textarea" id="advanced-config-json" rows="8" aria-label="Advanced config JSON patch" spellcheck="false" bind:value={advancedConfigJson}></textarea>
        </div>
      </div>
      <button id="save-advanced-config" class="btn btn-submit" disabled={saveAdvancedDisabled} on:click={saveAdvancedConfig}>Save Advanced Config</button>
    </div>
  </div>
</section>
