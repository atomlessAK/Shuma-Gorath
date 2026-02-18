// @ts-check

import { createDashboardCharts } from '../../../modules/charts.js';
import * as statusModule from '../../../modules/status.js';
import * as configControls from '../../../modules/config-controls.js';
import * as adminSessionModule from '../../../modules/admin-session.js';
import * as tabLifecycleModule from '../../../modules/tab-lifecycle.js';
import * as dashboardApiClientModule from '../../../modules/api-client.js';
import * as tablesViewModule from '../../../modules/tables-view.js';
import * as configUiStateModule from '../../../modules/config-ui-state.js';
import * as formatModule from '../../../modules/core/format.js';
import * as domModule from '../../../modules/core/dom.js';
import * as jsonObjectModule from '../../../modules/core/json-object.js';
import * as configSchemaModule from '../../../modules/config-schema.js';
import * as configDraftStoreModule from '../../../modules/config-draft-store.js';
import * as configFormUtilsModule from '../../../modules/config-form-utils.js';
import * as inputValidationModule from '../../../modules/input-validation.js';
import * as adminEndpointModule from '../../../modules/services/admin-endpoint.js';
import {
  acquireChartRuntime,
  getChartConstructor,
  releaseChartRuntime
} from '../../../modules/services/chart-runtime-adapter.js';
import { createRuntimeEffects } from '../../../modules/services/runtime-effects.js';
import {
  createDashboardSessionRuntime,
  createDashboardTabRuntime
} from './dashboard-runtime-orchestration.js';
import { createConfigDirtyRuntime } from './dashboard-runtime-config-dirty.js';
import { createDashboardRefreshRuntime } from './dashboard-runtime-refresh.js';
import { createDashboardTabStateRuntime } from './dashboard-runtime-tab-state.js';
import {
  buildDashboardLoginPath,
  normalizeDashboardBasePath,
  resolveDashboardBasePathFromLocation
} from './dashboard-paths.js';

const {
  parseCountryCodesStrict,
  normalizeCountryCodesForCompare,
  parseListTextarea,
  formatListTextarea,
  normalizeListTextareaForCompare,
  parseHoneypotPathsTextarea,
  formatBrowserRulesTextarea,
  parseBrowserRulesTextarea,
  normalizeBrowserRulesForCompare
} = configFormUtilsModule;
const escapeHtml = formatModule.escapeHtml;

const INTEGER_FIELD_RULES = {
  'ban-duration-days': { min: 0, max: 365, fallback: 0, label: 'Manual ban duration days' },
  'ban-duration-hours': { min: 0, max: 23, fallback: 1, label: 'Manual ban duration hours' },
  'ban-duration-minutes': { min: 0, max: 59, fallback: 0, label: 'Manual ban duration minutes' },
  'robots-crawl-delay': { min: 0, max: 60, fallback: 2, label: 'Crawl delay' },
  'maze-threshold': { min: 5, max: 500, fallback: 50, label: 'Maze threshold' },
  'rate-limit-threshold': { min: 1, max: 1000000, fallback: 80, label: 'Rate limit' },
  'challenge-puzzle-transform-count': { min: 4, max: 8, fallback: 6, label: 'Challenge transform count' },
  'pow-difficulty': { min: 12, max: 20, fallback: 15, label: 'PoW difficulty' },
  'pow-ttl': { min: 30, max: 300, fallback: 90, label: 'PoW seed TTL' },
  'dur-honeypot-days': { min: 0, max: 365, fallback: 1, label: 'Maze Threshold Exceeded days' },
  'dur-honeypot-hours': { min: 0, max: 23, fallback: 0, label: 'Maze Threshold Exceeded hours' },
  'dur-honeypot-minutes': { min: 0, max: 59, fallback: 0, label: 'Maze Threshold Exceeded minutes' },
  'dur-rate-limit-days': { min: 0, max: 365, fallback: 0, label: 'Rate Limit Exceeded days' },
  'dur-rate-limit-hours': { min: 0, max: 23, fallback: 1, label: 'Rate Limit Exceeded hours' },
  'dur-rate-limit-minutes': { min: 0, max: 59, fallback: 0, label: 'Rate Limit Exceeded minutes' },
  'dur-browser-days': { min: 0, max: 365, fallback: 0, label: 'Browser Automation Detected days' },
  'dur-browser-hours': { min: 0, max: 23, fallback: 6, label: 'Browser Automation Detected hours' },
  'dur-browser-minutes': { min: 0, max: 59, fallback: 0, label: 'Browser Automation Detected minutes' },
  'dur-cdp-days': { min: 0, max: 365, fallback: 0, label: 'CDP Automation Detected days' },
  'dur-cdp-hours': { min: 0, max: 23, fallback: 12, label: 'CDP Automation Detected hours' },
  'dur-cdp-minutes': { min: 0, max: 59, fallback: 0, label: 'CDP Automation Detected minutes' },
  'dur-admin-days': { min: 0, max: 365, fallback: 0, label: 'Admin Manual Ban days' },
  'dur-admin-hours': { min: 0, max: 23, fallback: 6, label: 'Admin Manual Ban hours' },
  'dur-admin-minutes': { min: 0, max: 59, fallback: 0, label: 'Admin Manual Ban minutes' },
  'challenge-puzzle-threshold': { min: 1, max: 10, fallback: 3, label: 'Challenge threshold' },
  'maze-threshold-score': { min: 1, max: 10, fallback: 6, label: 'Maze threshold' },
  'weight-js-required': { min: 0, max: 10, fallback: 1, label: 'JS weight' },
  'weight-geo-risk': { min: 0, max: 10, fallback: 2, label: 'GEO weight' },
  'weight-rate-medium': { min: 0, max: 10, fallback: 1, label: 'Rate 50% weight' },
  'weight-rate-high': { min: 0, max: 10, fallback: 2, label: 'Rate 80% weight' }
};

const BAN_DURATION_BOUNDS_SECONDS = { min: 60, max: 31536000 };

const BAN_DURATION_FIELDS = {
  honeypot: {
    label: 'Maze Threshold Exceeded duration',
    fallback: 86400,
    daysId: 'dur-honeypot-days',
    hoursId: 'dur-honeypot-hours',
    minutesId: 'dur-honeypot-minutes'
  },
  rateLimit: {
    label: 'Rate Limit Exceeded duration',
    fallback: 3600,
    daysId: 'dur-rate-limit-days',
    hoursId: 'dur-rate-limit-hours',
    minutesId: 'dur-rate-limit-minutes'
  },
  browser: {
    label: 'Browser Automation Detected duration',
    fallback: 21600,
    daysId: 'dur-browser-days',
    hoursId: 'dur-browser-hours',
    minutesId: 'dur-browser-minutes'
  },
  cdp: {
    label: 'CDP Automation Detected duration',
    fallback: 43200,
    daysId: 'dur-cdp-days',
    hoursId: 'dur-cdp-hours',
    minutesId: 'dur-cdp-minutes'
  },
  admin: {
    label: 'Admin Manual Ban duration',
    fallback: 21600,
    daysId: 'dur-admin-days',
    hoursId: 'dur-admin-hours',
    minutesId: 'dur-admin-minutes'
  }
};

const MANUAL_BAN_DURATION_FIELD = {
  label: 'Manual ban duration',
  fallback: 3600,
  daysId: 'ban-duration-days',
  hoursId: 'ban-duration-hours',
  minutesId: 'ban-duration-minutes'
};

const EDGE_INTEGRATION_MODES = new Set(['off', 'advisory', 'authoritative']);
const ADVANCED_CONFIG_TEMPLATE_PATHS = Object.freeze(
  configSchemaModule.advancedConfigTemplatePaths || []
);

let adminSessionController = null;
let dashboardApiClient = null;
let dashboardState = null;
let tablesView = null;
let configUiState = null;
let inputValidation = null;
let configDraftStore = null;
let runtimeEffects = null;
let statusPanel = null;
let configDirtyRuntime = null;
let tabRuntime = null;
let sessionRuntime = null;
let dashboardChartsRuntime = null;
let dashboardRefreshRuntime = null;
let tabStateRuntime = null;
let teardownControlBindings = null;
let runtimeMounted = false;
let runtimeMountOptions = {
  chartRuntimeSrc: '',
  basePath: normalizeDashboardBasePath(),
  initialTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB
};
const domCache = domModule.createCache({ document });
const getById = domCache.byId;
const query = domCache.query;
const queryAll = domCache.queryAll;
const domWriteScheduler = domModule.createWriteScheduler();
const resolveAdminApiEndpoint = adminEndpointModule.createAdminEndpointResolver({ window });

const DASHBOARD_STATE_REQUIRED_METHODS = Object.freeze([
  'getState',
  'setActiveTab',
  'getActiveTab',
  'setSession',
  'getSession',
  'setSnapshot',
  'getSnapshot',
  'setTabLoading',
  'setTabError',
  'clearTabError',
  'setTabEmpty',
  'markTabUpdated',
  'invalidate',
  'isTabStale',
  'getDerivedState'
]);

function hasDashboardStateContract(candidate) {
  if (!candidate || typeof candidate !== 'object') return false;
  return DASHBOARD_STATE_REQUIRED_METHODS.every(
    (methodName) => typeof candidate[methodName] === 'function'
  );
}

function resolveDashboardStateStore(options = {}) {
  const providedStore = options.store;
  if (hasDashboardStateContract(providedStore)) {
    return providedStore;
  }
  throw new Error(
    'mountDashboardApp requires an injected dashboard store contract.'
  );
}

function normalizeRuntimeMountOptions(options = {}) {
  const source = options || {};
  const chartRuntimeSrc = typeof source.chartRuntimeSrc === 'string'
    ? source.chartRuntimeSrc.trim()
    : '';
  const basePath = normalizeDashboardBasePath(
    source.basePath || resolveDashboardBasePathFromLocation(window.location)
  );
  return {
    chartRuntimeSrc,
    basePath,
    initialTab: tabLifecycleModule.normalizeTab(
      source.initialTab || tabLifecycleModule.DEFAULT_DASHBOARD_TAB
    )
  };
}

function runDomWriteBatch(task) {
  return new Promise((resolve) => {
    domWriteScheduler.schedule(() => {
      task();
      resolve();
    });
  });
}

const cloneJsonValue = jsonObjectModule.cloneJsonValue;
const buildAdvancedConfigTemplate = (config) =>
  jsonObjectModule.buildTemplateFromPaths(config, ADVANCED_CONFIG_TEMPLATE_PATHS);
const normalizeJsonObjectForCompare = jsonObjectModule.normalizeJsonObjectForCompare;

function fieldErrorIdFor(input) {
  const raw = input.id || input.name || 'field';
  return `field-error-${raw.replace(/[^a-zA-Z0-9_-]/g, '-')}`;
}

function getOrCreateFieldErrorElement(input) {
  if (!input || !input.parentElement) return null;
  const id = fieldErrorIdFor(input);
  let errorEl = getById(id);
  if (!errorEl || errorEl.dataset.fieldFor !== input.id) {
    errorEl = document.createElement('div');
    errorEl.id = id;
    errorEl.className = 'field-error';
    errorEl.dataset.fieldFor = input.id || '';
    errorEl.setAttribute('aria-live', 'polite');
    input.insertAdjacentElement('afterend', errorEl);
  }
  if (input.getAttribute('aria-describedby') !== id) {
    input.setAttribute('aria-describedby', id);
  }
  return errorEl;
}

function setFieldError(input, message, showInline = true) {
  if (!input) return;
  input.setCustomValidity(message || '');
  if (!showInline) return;

  const errorEl = getOrCreateFieldErrorElement(input);
  if (!errorEl) return;
  if (message) {
    input.setAttribute('aria-invalid', 'true');
    errorEl.textContent = message;
    errorEl.classList.add('visible');
    return;
  }

  input.removeAttribute('aria-invalid');
  errorEl.textContent = '';
  errorEl.classList.remove('visible');
}

const parseIntegerLoose = (id) => inputValidation.parseIntegerLoose(id);
const validateIntegerFieldById = (id, showInline = false) =>
  inputValidation.validateIntegerFieldById(id, showInline);
const readIntegerFieldValue = (id, messageTarget) =>
  inputValidation.readIntegerFieldValue(id, messageTarget);
const validateIpFieldById = (id, required, label, showInline = false) =>
  inputValidation.validateIpFieldById(id, required, label, showInline);
const readIpFieldValue = (id, required, messageTarget, label) =>
  inputValidation.readIpFieldValue(id, required, messageTarget, label);
const setBanDurationInputFromSeconds = (durationKey, totalSeconds) =>
  inputValidation.setBanDurationInputFromSeconds(durationKey, totalSeconds);
const readBanDurationFromInputs = (durationKey, showInline = false) =>
  inputValidation.readBanDurationFromInputs(durationKey, showInline);
const readBanDurationSeconds = (durationKey) =>
  inputValidation.readBanDurationSeconds(durationKey);
const readManualBanDurationSeconds = (showInline = false) =>
  inputValidation.readManualBanDurationSeconds(showInline);

function hasValidApiContext() {
  return adminSessionController ? adminSessionController.hasValidApiContext() : false;
}

function refreshMazePreviewLink() {
  const link = getById('preview-maze-link');
  if (!link) return;
  const resolved = resolveAdminApiEndpoint();
  const endpoint = resolved && resolved.endpoint ? resolved.endpoint : '';
  link.href = `${endpoint}/admin/maze/preview`;
}

function redirectToLogin() {
  const nextPath = `${window.location.pathname}${window.location.search}${window.location.hash || ''}`;
  window.location.replace(
    buildDashboardLoginPath({
      basePath: runtimeMountOptions.basePath,
      nextPath
    })
  );
}

function validateGeoFieldById(id, showInline = false) {
  const field = getById(id);
  if (!field) return false;
  try {
    parseCountryCodesStrict(field.value);
    setFieldError(field, '', showInline);
    return true;
  } catch (e) {
    setFieldError(field, e.message || 'Invalid country list.', showInline);
    return false;
  }
}

function refreshCoreActionButtonsState() {
  const apiValid = hasValidApiContext();
  refreshMazePreviewLink();
  const logoutBtn = getById('logout-btn');
  if (logoutBtn) {
    logoutBtn.disabled = !apiValid;
  }
  setValidActionButtonState(
    'ban-btn',
    apiValid,
    validateIpFieldById('ban-ip', true, 'Ban IP') &&
      Boolean(inputValidation && inputValidation.readManualBanDurationFromInputs())
  );
  setValidActionButtonState(
    'unban-btn',
    apiValid,
    validateIpFieldById('unban-ip', true, 'Unban IP')
  );

  if (configDirtyRuntime && typeof configDirtyRuntime.runCoreChecks === 'function') {
    configDirtyRuntime.runCoreChecks();
  }
  checkGeoConfigChanged();
  checkAdvancedConfigChanged();
}

function getAdminContext(messageTarget) {
  if (!adminSessionController) return null;
  return adminSessionController.getAdminContext(messageTarget);
}

function initInputValidation() {
  if (!inputValidation) return;
  Object.keys(INTEGER_FIELD_RULES).forEach((id) => inputValidation.bindIntegerFieldValidation(id));
  inputValidation.bindIpFieldValidation('ban-ip', true, 'Ban IP');
  inputValidation.bindIpFieldValidation('unban-ip', true, 'Unban IP');
  refreshCoreActionButtonsState();
  refreshAllDirtySections();
}

function envVar(name) {
  return `<code class="env-var">${name}</code>`;
}

function parseBoolLike(value, fallback = false) {
  if (typeof value === 'boolean') return value;
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') return true;
  if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') return false;
  return fallback;
}

function normalizeEdgeIntegrationMode(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (EDGE_INTEGRATION_MODES.has(normalized)) {
    return normalized;
  }
  return 'off';
}

function adminConfigWriteEnabled(config) {
  return parseBoolLike(config && config.admin_config_write_enabled, false);
}

function applyStatusPanelPatch(patch) {
  if (!statusPanel) return;
  if (typeof statusPanel.applyPatch === 'function') {
    statusPanel.applyPatch(patch);
    return;
  }
  statusPanel.update(patch);
  statusPanel.render();
}

function updateConfigModeUi(config, baseStatusPatch = {}) {
  const writeEnabled = adminConfigWriteEnabled(config);
  const failModeFromConfig = parseBoolLike(config && config.kv_store_fail_open, true)
    ? 'open'
    : 'closed';
  applyStatusPanelPatch({
    ...baseStatusPatch,
    testMode: parseBoolLike(config && config.test_mode, false),
    failMode: statusModule.normalizeFailMode(failModeFromConfig),
    httpsEnforced: parseBoolLike(config && config.https_enforced, false),
    forwardedHeaderTrustConfigured: parseBoolLike(
      config && config.forwarded_header_trust_configured,
      false
    )
  });
  const subtitle = getById('config-mode-subtitle');
  if (subtitle) {
    if (writeEnabled) {
      subtitle.innerHTML =
        `Admin page configuration enabled. Saved changes persist across builds. Set ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} to <strong>false</strong> in deployment env to disable.`;
    } else {
      subtitle.innerHTML =
        `Admin page configuration disabled. Set ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} to <strong>true</strong> to enable.`;
    }
  }

  queryAll('.config-edit-pane').forEach(el => {
    el.classList.toggle('hidden', !writeEnabled);
  });
}

function deriveMonitoringAnalytics(configSnapshot = {}, analyticsSnapshot = {}) {
  const config = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};
  const analytics = analyticsSnapshot && typeof analyticsSnapshot === 'object' ? analyticsSnapshot : {};
  return {
    ban_count: Number(analytics.ban_count || 0),
    test_mode: parseBoolLike(config.test_mode, analytics.test_mode === true),
    fail_mode: parseBoolLike(config.kv_store_fail_open, true) ? 'open' : 'closed'
  };
}

// Update ban duration fields from config
const invokeConfigUiState = (methodName, ...args) => {
  if (!configUiState) return;
  const method = configUiState[methodName];
  if (typeof method === 'function') {
    method(...args);
  }
};

const updateBanDurations = (config) => invokeConfigUiState('updateBanDurations', config);

// Update maze config controls from loaded config
const updateMazeConfig = (config) => invokeConfigUiState('updateMazeConfig', config);

const updateGeoConfig = (config) => invokeConfigUiState('updateGeoConfig', config);

const updateHoneypotConfig = (config) => invokeConfigUiState('updateHoneypotConfig', config);

const updateBrowserPolicyConfig = (config) => invokeConfigUiState('updateBrowserPolicyConfig', config);

const updateBypassAllowlistConfig = (config) => invokeConfigUiState('updateBypassAllowlistConfig', config);

const CONFIG_DRAFT_DEFAULTS = configUiStateModule.CONFIG_DRAFT_DEFAULTS;

function getDraft(sectionKey) {
  const fallback = CONFIG_DRAFT_DEFAULTS[sectionKey] || null;
  return configDraftStore ? configDraftStore.get(sectionKey, fallback) : cloneJsonValue(fallback);
}

function setDraft(sectionKey, value) {
  if (configDraftStore) {
    configDraftStore.set(sectionKey, value);
  }
}

function isDraftDirty(sectionKey, currentValue) {
  if (!configDraftStore) return true;
  return configDraftStore.isDirty(sectionKey, currentValue);
}

const GEO_SCORING_FIELD_IDS = configUiStateModule.GEO_SCORING_FIELD_IDS;
const GEO_ROUTING_FIELD_IDS = configUiStateModule.GEO_ROUTING_FIELD_IDS;
const GEO_FIELD_IDS = configUiStateModule.GEO_FIELD_IDS;
const sanitizeGeoTextareaValue = configUiStateModule.sanitizeGeoTextareaValue;

const updateRobotsConfig = (config) => invokeConfigUiState('updateRobotsConfig', config);

function setButtonState(buttonId, apiValid, fieldsValid, changed, requireChange) {
  const btn = getById(buttonId);
  if (!btn) return;
  if (btn.dataset.saving === 'true') return;
  const canSubmit = apiValid && fieldsValid && (!requireChange || changed);
  btn.disabled = !canSubmit;
}

function setDirtySaveButtonState(buttonId, changed, apiValid, fieldsValid = true) {
  setButtonState(buttonId, apiValid, fieldsValid, changed, true);
}

function setValidActionButtonState(buttonId, apiValid, fieldsValid = true) {
  setButtonState(buttonId, apiValid, fieldsValid, true, false);
}

function runConfigDirtyCheck(methodName) {
  if (!configDirtyRuntime) return;
  const handler = configDirtyRuntime[methodName];
  if (typeof handler === 'function') {
    handler();
  }
}

function checkRobotsConfigChanged() {
  runConfigDirtyCheck('checkRobots');
}

function checkAiPolicyConfigChanged() {
  runConfigDirtyCheck('checkAiPolicy');
}

function checkMazeConfigChanged() {
  runConfigDirtyCheck('checkMaze');
}

function checkBanDurationsChanged() {
  runConfigDirtyCheck('checkBanDurations');
}

function validateHoneypotPathsField(showInline = false) {
  const field = getById('honeypot-paths');
  if (!field) return false;
  try {
    parseHoneypotPathsTextarea(field.value);
    setFieldError(field, '', showInline);
    return true;
  } catch (e) {
    setFieldError(field, e.message || 'Invalid honeypot paths.', showInline);
    return false;
  }
}

function checkHoneypotConfigChanged() {
  runConfigDirtyCheck('checkHoneypot');
}

function validateBrowserRulesField(id, showInline = false) {
  const field = getById(id);
  if (!field) return false;
  try {
    parseBrowserRulesTextarea(field.value);
    setFieldError(field, '', showInline);
    return true;
  } catch (e) {
    setFieldError(field, e.message || 'Invalid browser rules.', showInline);
    return false;
  }
}

function checkBrowserPolicyConfigChanged() {
  runConfigDirtyCheck('checkBrowserPolicy');
}

function checkBypassAllowlistsConfigChanged() {
  runConfigDirtyCheck('checkBypassAllowlists');
}

function checkChallengePuzzleConfigChanged() {
  runConfigDirtyCheck('checkChallengePuzzle');
}

// Fetch and update robots.txt preview content
async function refreshRobotsPreview() {
  if (!getAdminContext(getById('admin-msg'))) return;
  const previewContent = getById('robots-preview-content');
  if (!previewContent) return;
  
  try {
    const data = await dashboardApiClient.getRobotsPreview();
    previewContent.textContent = data.content || '# No preview available';
  } catch (e) {
    previewContent.textContent = '# Error loading preview: ' + e.message;
    console.error('Failed to load robots preview:', e);
  }
}

async function handleToggleRobotsPreviewClick() {
  const preview = getById('robots-preview');
  if (!preview) return;
  const btn = this;

  if (preview.classList.contains('hidden')) {
    // Show preview
    btn.textContent = 'Loading...';
    btn.disabled = true;
    await refreshRobotsPreview();
    preview.classList.remove('hidden');
    btn.textContent = 'Hide robots.txt';
    btn.disabled = false;
    return;
  }

  // Hide preview
  preview.classList.add('hidden');
  btn.textContent = 'Show robots.txt';
}

// Update CDP detection config controls from loaded config
const updateCdpConfig = (config) => invokeConfigUiState('updateCdpConfig', config);

const updateEdgeIntegrationModeConfig = (config) =>
  invokeConfigUiState('updateEdgeIntegrationModeConfig', config);

const updateRateLimitConfig = (config) => invokeConfigUiState('updateRateLimitConfig', config);

const updateJsRequiredConfig = (config) => invokeConfigUiState('updateJsRequiredConfig', config);

// Update PoW config controls from loaded config
const updatePowConfig = (config) => invokeConfigUiState('updatePowConfig', config);

const updateChallengeConfig = (config) => invokeConfigUiState('updateChallengeConfig', config);

function checkPowConfigChanged() {
  runConfigDirtyCheck('checkPow');
}

function checkBotnessConfigChanged() {
  runConfigDirtyCheck('checkBotness');
}

function checkGeoConfigChanged() {
  const apiValid = hasValidApiContext();
  const scoringValid = GEO_SCORING_FIELD_IDS.every(validateGeoFieldById);
  const routingValid = GEO_ROUTING_FIELD_IDS.every(validateGeoFieldById);
  const savedGeo = getDraft('geo');
  if (!savedGeo.mutable) {
    const scoringBtn = getById('save-geo-scoring-config');
    if (scoringBtn) scoringBtn.disabled = true;
    const routingBtn = getById('save-geo-routing-config');
    if (routingBtn) routingBtn.disabled = true;
    return;
  }

  const current = {
    risk: normalizeCountryCodesForCompare(getById('geo-risk-list').value),
    allow: normalizeCountryCodesForCompare(getById('geo-allow-list').value),
    challenge: normalizeCountryCodesForCompare(getById('geo-challenge-list').value),
    maze: normalizeCountryCodesForCompare(getById('geo-maze-list').value),
    block: normalizeCountryCodesForCompare(getById('geo-block-list').value)
  };
  const scoringChanged = current.risk !== savedGeo.risk;
  const routingChanged =
    current.allow !== savedGeo.allow ||
    current.challenge !== savedGeo.challenge ||
    current.maze !== savedGeo.maze ||
    current.block !== savedGeo.block;

  setDirtySaveButtonState('save-geo-scoring-config', scoringChanged, apiValid, scoringValid);
  setDirtySaveButtonState('save-geo-routing-config', routingChanged, apiValid, routingValid);
}

function handleGeoFieldInput(id, field) {
  const sanitized = sanitizeGeoTextareaValue(field.value);
  if (field.value !== sanitized) {
    const cursor = field.selectionStart;
    const delta = field.value.length - sanitized.length;
    field.value = sanitized;
    if (typeof cursor === 'number') {
      const next = Math.max(0, cursor - Math.max(0, delta));
      field.setSelectionRange(next, next);
    }
  }
  validateGeoFieldById(id, true);
  checkGeoConfigChanged();
  refreshCoreActionButtonsState();
}

function handleGeoFieldBlur(id) {
  validateGeoFieldById(id, true);
  checkGeoConfigChanged();
  refreshCoreActionButtonsState();
}

// Check if CDP config has changed from saved state
function checkCdpConfigChanged() {
  runConfigDirtyCheck('checkCdp');
}

function checkEdgeIntegrationModeChanged() {
  runConfigDirtyCheck('checkEdgeMode');
}

function checkRateLimitConfigChanged() {
  runConfigDirtyCheck('checkRateLimit');
}

function checkJsRequiredConfigChanged() {
  runConfigDirtyCheck('checkJsRequired');
}

function setAdvancedConfigEditorFromConfig(config, preserveDirty = true) {
  if (!configUiState) return;
  configUiState.setAdvancedConfigEditorFromConfig(config, preserveDirty);
  checkAdvancedConfigChanged();
}

function readAdvancedConfigPatch(messageTarget) {
  const field = getById('advanced-config-json');
  if (!field) return null;
  const raw = String(field.value || '').trim();
  const parsedText = raw.length > 0 ? raw : '{}';
  let patch;
  try {
    patch = JSON.parse(parsedText);
  } catch (e) {
    const message = `Advanced config JSON parse error: ${e.message}`;
    setFieldError(field, message, true);
    if (messageTarget) {
      messageTarget.textContent = message;
      messageTarget.className = 'message error';
    }
    return null;
  }
  if (!patch || Array.isArray(patch) || typeof patch !== 'object') {
    const message = 'Advanced config patch must be a JSON object.';
    setFieldError(field, message, true);
    if (messageTarget) {
      messageTarget.textContent = message;
      messageTarget.className = 'message error';
    }
    return null;
  }
  setFieldError(field, '', true);
  return patch;
}

function checkAdvancedConfigChanged() {
  const field = getById('advanced-config-json');
  const btn = getById('save-advanced-config');
  if (!field || !btn) return;
  const apiValid = hasValidApiContext();
  const normalized = normalizeJsonObjectForCompare(field.value);
  const valid = normalized !== null;
  const baseline = getDraft('advancedConfig').normalized || '{}';
  const changed = valid && normalized !== baseline;
  field.dataset.dirty = changed ? 'true' : 'false';
  setFieldError(field, valid ? '' : 'Advanced config patch must be valid JSON object syntax.', true);
  setDirtySaveButtonState('save-advanced-config', changed, apiValid, valid);
}

const DIRTY_SECTION_CHECKS = Object.freeze({
  maze: checkMazeConfigChanged,
  banDurations: checkBanDurationsChanged,
  honeypot: checkHoneypotConfigChanged,
  browserPolicy: checkBrowserPolicyConfigChanged,
  bypassAllowlists: checkBypassAllowlistsConfigChanged,
  challengePuzzle: checkChallengePuzzleConfigChanged,
  pow: checkPowConfigChanged,
  botness: checkBotnessConfigChanged,
  geo: checkGeoConfigChanged,
  cdp: checkCdpConfigChanged,
  edgeMode: checkEdgeIntegrationModeChanged,
  rateLimit: checkRateLimitConfigChanged,
  jsRequired: checkJsRequiredConfigChanged,
  robots: checkRobotsConfigChanged,
  aiPolicy: checkAiPolicyConfigChanged,
  advancedConfig: checkAdvancedConfigChanged
});

const DIRTY_SECTIONS_BY_TAB = Object.freeze({
  monitoring: [],
  'ip-bans': [],
  status: [],
  config: Object.keys(DIRTY_SECTION_CHECKS),
  tuning: [
    'pow',
    'challengePuzzle',
    'botness',
    'cdp',
    'edgeMode',
    'rateLimit',
    'jsRequired'
  ]
});

function refreshDirtySections(sectionKeys = []) {
  sectionKeys.forEach((sectionKey) => {
    const handler = DIRTY_SECTION_CHECKS[sectionKey];
    if (typeof handler === 'function') {
      handler();
    }
  });
}

function refreshAllDirtySections() {
  refreshDirtySections(DIRTY_SECTIONS_BY_TAB.config);
}

async function handleBanIpAction() {
  const msg = getById('admin-msg');
  if (!msg || !getAdminContext(msg)) return;
  const ip = readIpFieldValue('ban-ip', true, msg, 'Ban IP');
  if (ip === null) return;
  const duration = readManualBanDurationSeconds(true);
  if (duration === null) return;

  msg.textContent = `Banning ${ip}...`;
  msg.className = 'message info';

  try {
    await dashboardApiClient.banIp(ip, duration);
    msg.textContent = `Banned ${ip} for ${duration}s`;
    msg.className = 'message success';
    const banIpField = getById('ban-ip');
    if (banIpField) banIpField.value = '';
    if (dashboardState) dashboardState.invalidate('ip-bans');
    runtimeEffects.setTimer(() => refreshActiveTab('ban-save'), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
}

async function handleUnbanIpAction() {
  const msg = getById('admin-msg');
  if (!msg || !getAdminContext(msg)) return;
  const ip = readIpFieldValue('unban-ip', true, msg, 'Unban IP');
  if (ip === null) return;

  msg.textContent = `Unbanning ${ip}...`;
  msg.className = 'message info';

  try {
    await dashboardApiClient.unbanIp(ip);
    msg.textContent = `Unbanned ${ip}`;
    msg.className = 'message success';
    const unbanIpField = getById('unban-ip');
    if (unbanIpField) unbanIpField.value = '';
    if (dashboardState) dashboardState.invalidate('ip-bans');
    runtimeEffects.setTimer(() => refreshActiveTab('unban-save'), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
}

function bindMountScopedDomEvents() {
  const cleanupTasks = [];

  const bindEvent = (node, eventName, handler) => {
    if (!node || typeof handler !== 'function') return;
    node.addEventListener(eventName, handler);
    cleanupTasks.push(() => {
      node.removeEventListener(eventName, handler);
    });
  };

  const bindFieldEvent = (id, eventName, handler) => {
    const field = getById(id);
    bindEvent(field, eventName, handler);
  };

  const bindInputAndBlur = (id, handler) => {
    bindFieldEvent(id, 'input', handler);
    bindFieldEvent(id, 'blur', handler);
  };

  // Add change listeners for robots serving and AI-policy controls.
  [
    { ids: ['robots-enabled-toggle'], event: 'change', handler: checkRobotsConfigChanged },
    { ids: ['robots-crawl-delay'], event: 'input', handler: checkRobotsConfigChanged },
    {
      ids: ['robots-block-training-toggle', 'robots-block-search-toggle', 'robots-allow-search-toggle'],
      event: 'change',
      handler: checkAiPolicyConfigChanged
    },
    { ids: ['maze-enabled-toggle', 'maze-auto-ban-toggle'], event: 'change', handler: checkMazeConfigChanged },
    { ids: ['honeypot-enabled-toggle'], event: 'change', handler: checkHoneypotConfigChanged },
    { ids: ['challenge-puzzle-transform-count'], event: 'input', handler: checkChallengePuzzleConfigChanged },
    { ids: ['challenge-puzzle-enabled-toggle'], event: 'change', handler: checkChallengePuzzleConfigChanged }
  ].forEach(({ ids, event, handler }) => {
    ids.forEach((id) => bindFieldEvent(id, event, handler));
  });

  bindInputAndBlur('honeypot-paths', () => {
    validateHoneypotPathsField(true);
    checkHoneypotConfigChanged();
    refreshCoreActionButtonsState();
  });

  ['browser-block-rules', 'browser-whitelist-rules'].forEach((id) => {
    bindInputAndBlur(id, () => {
      validateBrowserRulesField(id, true);
      checkBrowserPolicyConfigChanged();
      refreshCoreActionButtonsState();
    });
  });

  ['network-whitelist', 'path-whitelist'].forEach((id) => {
    bindInputAndBlur(id, () => {
      checkBypassAllowlistsConfigChanged();
      refreshCoreActionButtonsState();
    });
  });

  const previewRobotsButton = getById('preview-robots');
  if (previewRobotsButton) {
    bindEvent(previewRobotsButton, 'click', handleToggleRobotsPreviewClick);
  }

  bindFieldEvent('pow-enabled-toggle', 'change', checkPowConfigChanged);
  bindFieldEvent('pow-difficulty', 'input', checkPowConfigChanged);
  bindFieldEvent('pow-ttl', 'input', checkPowConfigChanged);

  [
    'challenge-puzzle-threshold',
    'maze-threshold-score',
    'weight-js-required',
    'weight-geo-risk',
    'weight-rate-medium',
    'weight-rate-high'
  ].forEach((id) => {
    bindFieldEvent(id, 'input', checkBotnessConfigChanged);
  });

  GEO_FIELD_IDS.forEach((id) => {
    const field = getById(id);
    if (!field) return;
    const inputHandler = () => handleGeoFieldInput(id, field);
    const blurHandler = () => handleGeoFieldBlur(id);
    bindEvent(field, 'input', inputHandler);
    bindEvent(field, 'blur', blurHandler);
  });

  bindFieldEvent('rate-limit-threshold', 'input', checkRateLimitConfigChanged);
  bindFieldEvent('js-required-enforced-toggle', 'change', checkJsRequiredConfigChanged);

  const advancedConfigField = getById('advanced-config-json');
  if (advancedConfigField) {
    const inputHandler = () => {
      checkAdvancedConfigChanged();
      refreshCoreActionButtonsState();
    };
    const blurHandler = () => {
      checkAdvancedConfigChanged();
      refreshCoreActionButtonsState();
    };
    bindEvent(advancedConfigField, 'input', inputHandler);
    bindEvent(advancedConfigField, 'blur', blurHandler);
  }

  // Update threshold display when slider moves
  bindFieldEvent('cdp-threshold-slider', 'input', function onCdpThresholdSliderInput() {
    const value = getById('cdp-threshold-value');
    if (value) {
      value.textContent = parseFloat(this.value).toFixed(1);
    }
    checkCdpConfigChanged();
  });

  // Add change listeners for CDP config controls
  ['cdp-enabled-toggle', 'cdp-auto-ban-toggle'].forEach((id) => {
    bindFieldEvent(id, 'change', checkCdpConfigChanged);
  });

  bindFieldEvent('edge-integration-mode-select', 'change', checkEdgeIntegrationModeChanged);

  const banButton = getById('ban-btn');
  if (banButton) {
    bindEvent(banButton, 'click', handleBanIpAction);
  }

  const unbanButton = getById('unban-btn');
  if (unbanButton) {
    bindEvent(unbanButton, 'click', handleUnbanIpAction);
  }

  return () => {
    cleanupTasks.reverse().forEach((cleanup) => cleanup());
  };
}

const CONFIG_UI_REFRESH_METHODS = Object.freeze([
  'updateBanDurations',
  'updateRateLimitConfig',
  'updateJsRequiredConfig',
  'updateMazeConfig',
  'updateGeoConfig',
  'updateHoneypotConfig',
  'updateBrowserPolicyConfig',
  'updateBypassAllowlistConfig',
  'updateRobotsConfig',
  'updateCdpConfig',
  'updateEdgeIntegrationModeConfig',
  'updatePowConfig',
  'updateChallengeConfig'
]);

function refreshActiveTab(reason = 'manual') {
  if (!dashboardRefreshRuntime || typeof dashboardRefreshRuntime.refreshActiveTab !== 'function') {
    return Promise.resolve();
  }
  return dashboardRefreshRuntime.refreshActiveTab(reason);
}

function refreshDashboardForTab(tab, reason = 'manual', options = {}) {
  if (
    !dashboardRefreshRuntime ||
    typeof dashboardRefreshRuntime.refreshDashboardForTab !== 'function'
  ) {
    return Promise.resolve();
  }
  return dashboardRefreshRuntime.refreshDashboardForTab(tab, reason, options || {});
}

export async function mountDashboardApp(options = {}) {
  if (runtimeMounted) return;
  runtimeMountOptions = normalizeRuntimeMountOptions(options);
  await acquireChartRuntime({
    window,
    document,
    src: runtimeMountOptions.chartRuntimeSrc || undefined
  });
  runtimeMounted = true;

  configDraftStore = configDraftStoreModule.create(CONFIG_DRAFT_DEFAULTS);
  runtimeEffects = createRuntimeEffects();
  statusPanel = statusModule.create({ document });

  dashboardState = resolveDashboardStateStore(options);
  dashboardState.setActiveTab(runtimeMountOptions.initialTab);
  tabStateRuntime = createDashboardTabStateRuntime({
    getStateStore: () => dashboardState
  });

  adminSessionController = adminSessionModule.create({
    resolveAdminApiEndpoint,
    refreshCoreActionButtonsState,
    redirectToLogin,
    request: (input, init) => runtimeEffects.request(input, init)
  });

  tabRuntime = createDashboardTabRuntime({
    document,
    normalizeTab: tabLifecycleModule.normalizeTab,
    defaultTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB,
    getStateStore: () => dashboardState,
    refreshCoreActionButtonsState,
    refreshDashboardForTab
  });

  sessionRuntime = createDashboardSessionRuntime({
    getAdminSessionController: () => adminSessionController,
    getStateStore: () => dashboardState,
    refreshCoreActionButtonsState,
    resolveAdminApiEndpoint,
    getRuntimeEffects: () => runtimeEffects,
    getMessageNode: () => getById('admin-msg')
  });

  dashboardApiClient = dashboardApiClientModule.create({
    getAdminContext,
    onUnauthorized: redirectToLogin,
    request: (input, init) => runtimeEffects.request(input, init)
  });
  const chartConstructor = getChartConstructor({ window });

  tablesView = tablesViewModule.create({
    escapeHtml,
    onQuickUnban: async (ip) => {
      const msg = getById('admin-msg');
      if (!getAdminContext(msg)) return;

      msg.textContent = `Unbanning ${ip}...`;
      msg.className = 'message info';

      try {
        await dashboardApiClient.unbanIp(ip);
        msg.textContent = `Unbanned ${ip}`;
        msg.className = 'message success';
        if (dashboardState) dashboardState.invalidate('ip-bans');
        runtimeEffects.setTimer(() => refreshActiveTab('quick-unban'), 500);
      } catch (e) {
        msg.textContent = 'Error: ' + e.message;
        msg.className = 'message error';
      }
    }
  });

  inputValidation = inputValidationModule.create({
    getById,
    setFieldError,
    integerFieldRules: INTEGER_FIELD_RULES,
    banDurationBoundsSeconds: BAN_DURATION_BOUNDS_SECONDS,
    banDurationFields: BAN_DURATION_FIELDS,
    manualBanDurationField: MANUAL_BAN_DURATION_FIELD,
    onFieldInteraction: refreshCoreActionButtonsState
  });

  configUiState = configUiStateModule.create({
    getById,
    setDraft,
    getDraft,
    statusPanel,
    adminConfigWriteEnabled,
    parseBoolLike,
    normalizeEdgeIntegrationMode,
    normalizeCountryCodesForCompare,
    formatListTextarea,
    normalizeListTextareaForCompare,
    formatBrowserRulesTextarea,
    normalizeBrowserRulesForCompare,
    setBanDurationInputFromSeconds,
    banDurationFields: BAN_DURATION_FIELDS,
    buildAdvancedConfigTemplate,
    normalizeJsonObjectForCompare
  });

  configDirtyRuntime = createConfigDirtyRuntime({
    getById,
    getDraft,
    isDraftDirty,
    hasValidApiContext,
    validateIntegerFieldById,
    parseIntegerLoose,
    readBanDurationFromInputs,
    validateHoneypotPathsField,
    validateBrowserRulesField,
    normalizeListTextareaForCompare,
    normalizeBrowserRulesForCompare,
    normalizeEdgeIntegrationMode,
    setDirtySaveButtonState
  });

  dashboardRefreshRuntime = createDashboardRefreshRuntime({
    normalizeTab: tabLifecycleModule.normalizeTab,
    getApiClient: () => dashboardApiClient,
    getStateStore: () => dashboardState,
    getTablesView: () => tablesView,
    getChartsRuntime: () => dashboardChartsRuntime,
    getMessageNode: () => getById('admin-msg'),
    runDomWriteBatch,
    updateConfigModeUi,
    invokeConfigUiState,
    refreshAllDirtySections,
    refreshDirtySections,
    refreshCoreActionButtonsState,
    tabState: tabStateRuntime,
    deriveMonitoringAnalytics,
    configUiRefreshMethods: CONFIG_UI_REFRESH_METHODS,
    dirtySectionsByTab: DIRTY_SECTIONS_BY_TAB
  });

  initInputValidation();
  teardownControlBindings = bindMountScopedDomEvents();
  if (!dashboardChartsRuntime) {
    dashboardChartsRuntime = createDashboardCharts({ document, window });
  }
  dashboardChartsRuntime.init({
    getAdminContext,
    apiClient: dashboardApiClient,
    chartConstructor
  });
  statusPanel.render();

  const configControlsContext = {
    statusPanel,
    apiClient: dashboardApiClient,
    effects: runtimeEffects,
    auth: {
      getAdminContext
    },
    callbacks: {
      onConfigSaved: (_patch, result) => {
        if (result && result.config) {
          applyStatusPanelPatch({ configSnapshot: result.config });
          setAdvancedConfigEditorFromConfig(result.config, true);
        }
        if (dashboardState) {
          dashboardState.invalidate('securityConfig');
          dashboardState.invalidate('monitoring');
          dashboardState.invalidate('ip-bans');
        }
      }
    },
    readers: {
      readIntegerFieldValue,
      readBanDurationSeconds,
      readAdvancedConfigPatch
    },
    parsers: {
      parseCountryCodesStrict,
      parseListTextarea,
      normalizeListTextareaForCompare,
      parseHoneypotPathsTextarea,
      parseBrowserRulesTextarea,
      normalizeBrowserRulesForCompare
    },
    updaters: {
      updateBanDurations,
      updateGeoConfig,
      updateHoneypotConfig,
      updateBrowserPolicyConfig,
      updateBypassAllowlistConfig,
      updateEdgeIntegrationModeConfig,
      refreshRobotsPreview,
      setAdvancedConfigFromConfig: setAdvancedConfigEditorFromConfig
    },
    checks: {
      checkMazeConfigChanged,
      checkRobotsConfigChanged,
      checkAiPolicyConfigChanged,
      checkGeoConfigChanged,
      checkHoneypotConfigChanged,
      checkBrowserPolicyConfigChanged,
      checkBypassAllowlistsConfigChanged,
      checkPowConfigChanged,
      checkChallengePuzzleConfigChanged,
      checkBotnessConfigChanged,
      checkCdpConfigChanged,
      checkEdgeIntegrationModeChanged,
      checkRateLimitConfigChanged,
      checkJsRequiredConfigChanged,
      checkAdvancedConfigChanged,
      checkBanDurationsChanged
    },
    actions: {
      refreshDashboard: () => refreshActiveTab('config-controls')
    },
    draft: {
      get: getDraft,
      set: setDraft
    }
  };

  configControls.bind({
    context: configControlsContext
  });
}

export function getDashboardActiveTab() {
  if (!tabRuntime || typeof tabRuntime.getActiveTab !== 'function') {
    return tabLifecycleModule.DEFAULT_DASHBOARD_TAB;
  }
  return tabRuntime.getActiveTab();
}

export function setDashboardActiveTab(tab, reason = 'external') {
  const normalized = tabLifecycleModule.normalizeTab(
    tab || tabLifecycleModule.DEFAULT_DASHBOARD_TAB
  );
  if (!tabRuntime || typeof tabRuntime.setActiveTab !== 'function') {
    return normalized;
  }
  return tabRuntime.setActiveTab(normalized, reason);
}

export async function refreshDashboardTab(tab, reason = 'manual', options = {}) {
  if (!tabRuntime || typeof tabRuntime.refreshTab !== 'function') return;
  return tabRuntime.refreshTab(tab, reason, options || {});
}

export async function restoreDashboardSession() {
  if (!sessionRuntime || typeof sessionRuntime.restoreSession !== 'function') {
    return false;
  }
  return sessionRuntime.restoreSession();
}

export function getDashboardSessionState() {
  if (!sessionRuntime || typeof sessionRuntime.getSessionState !== 'function') {
    return { authenticated: false, csrfToken: '' };
  }
  return sessionRuntime.getSessionState();
}

export async function logoutDashboardSession() {
  if (!sessionRuntime || typeof sessionRuntime.logoutSession !== 'function') return;
  await sessionRuntime.logoutSession();
}

export function unmountDashboardApp() {
  if (!runtimeMounted) return;
  runtimeMounted = false;
  runtimeMountOptions = normalizeRuntimeMountOptions({});
  configDirtyRuntime = null;
  dashboardRefreshRuntime = null;
  tabStateRuntime = null;
  tabRuntime = null;
  sessionRuntime = null;
  if (teardownControlBindings) {
    teardownControlBindings();
    teardownControlBindings = null;
  }
  if (dashboardChartsRuntime && typeof dashboardChartsRuntime.destroy === 'function') {
    dashboardChartsRuntime.destroy();
  }
  dashboardChartsRuntime = null;
  if (document.body && document.body.dataset) {
    delete document.body.dataset.activeDashboardTab;
  }
  dashboardApiClient = null;
  dashboardState = null;
  tablesView = null;
  configUiState = null;
  inputValidation = null;
  configDraftStore = null;
  runtimeEffects = null;
  statusPanel = null;
  adminSessionController = null;
  releaseChartRuntime({ window });
}
