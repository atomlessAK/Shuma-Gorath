// @ts-check

import * as dashboardCharts from './modules/charts.js';
import * as statusModule from './modules/status.js';
import * as configControls from './modules/config-controls.js';
import * as adminSessionModule from './modules/admin-session.js';
import * as tabLifecycleModule from './modules/tab-lifecycle.js';
import * as dashboardApiClientModule from './modules/api-client.js';
import * as dashboardStateModule from './modules/dashboard-state.js';
import * as monitoringViewModule from './modules/monitoring-view.js';
import * as tablesViewModule from './modules/tables-view.js';
import * as configUiStateModule from './modules/config-ui-state.js';
import * as tabStateViewModule from './modules/tab-state-view.js';
import * as formatModule from './modules/core/format.js';
import * as domModule from './modules/core/dom.js';
import * as jsonObjectModule from './modules/core/json-object.js';
import * as configSchemaModule from './modules/config-schema.js';
import * as configDraftStoreModule from './modules/config-draft-store.js';
import * as configFormUtilsModule from './modules/config-form-utils.js';
import * as inputValidationModule from './modules/input-validation.js';
import * as adminEndpointModule from './modules/services/admin-endpoint.js';
import { createRuntimeEffects } from './modules/services/runtime-effects.js';
import {
  createLegacyAutoRefreshRuntime,
  createLegacyDashboardSessionRuntime,
  createLegacyDashboardTabRuntime
} from './src/lib/runtime/dashboard-legacy-orchestration.js';
import { createLegacyConfigDirtyRuntime } from './src/lib/runtime/dashboard-legacy-config-dirty.js';
import {
  clearDashboardExternalAdapters,
  configureDashboardExternalAdapters,
  getDashboardActiveTab as getExternalDashboardActiveTab,
  getDashboardSessionState as getExternalDashboardSessionState,
  logoutDashboardSession as logoutExternalDashboardSession,
  refreshDashboardTab as refreshExternalDashboardTab,
  restoreDashboardSession as restoreExternalDashboardSession,
  setDashboardActiveTab as setExternalDashboardActiveTab
} from './src/lib/runtime/dashboard-external-adapters.js';

export {
  getExternalDashboardActiveTab as getDashboardActiveTab,
  getExternalDashboardSessionState as getDashboardSessionState,
  logoutExternalDashboardSession as logoutDashboardSession,
  refreshExternalDashboardTab as refreshDashboardTab,
  restoreExternalDashboardSession as restoreDashboardSession,
  setExternalDashboardActiveTab as setDashboardActiveTab
};

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
let dashboardTabCoordinator = null;
let dashboardApiClient = null;
let dashboardState = null;
let monitoringView = null;
let tablesView = null;
let tabStateView = null;
let configUiState = null;
let inputValidation = null;
let configDraftStore = null;
let runtimeEffects = null;
let statusPanel = null;
let legacyAutoRefreshRuntime = null;
let legacyConfigDirtyRuntime = null;
let legacyTabRuntime = null;
let legacySessionRuntime = null;
let teardownControlBindings = null;
let runtimeMounted = false;
let runtimeMountOptions = {
  useExternalTabPipeline: false,
  useExternalPollingPipeline: false,
  useExternalSessionPipeline: false,
  bindLogoutButton: true,
  initialTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB
};
const domCache = domModule.createCache({ document });
const getById = domCache.byId;
const query = domCache.query;
const queryAll = domCache.queryAll;
const domWriteScheduler = domModule.createWriteScheduler();
const resolveAdminApiEndpoint = adminEndpointModule.createAdminEndpointResolver({ window });

const TAB_REFRESH_INTERVAL_MS = Object.freeze({
  monitoring: 30000,
  'ip-bans': 45000,
  status: 60000,
  config: 60000,
  tuning: 60000
});

function normalizeRuntimeMountOptions(options = {}) {
  const source = options || {};
  return {
    useExternalTabPipeline: source.useExternalTabPipeline === true,
    useExternalPollingPipeline: source.useExternalPollingPipeline === true,
    useExternalSessionPipeline: source.useExternalSessionPipeline === true,
    bindLogoutButton: source.bindLogoutButton !== false,
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

const parseIntegerLoose = (id) => (inputValidation ? inputValidation.parseIntegerLoose(id) : null);
const validateIntegerFieldById = (id, showInline = false) =>
  (inputValidation ? inputValidation.validateIntegerFieldById(id, showInline) : false);
const readIntegerFieldValue = (id, messageTarget) =>
  (inputValidation ? inputValidation.readIntegerFieldValue(id, messageTarget) : null);
const validateIpFieldById = (id, required, label, showInline = false) =>
  (inputValidation ? inputValidation.validateIpFieldById(id, required, label, showInline) : false);
const readIpFieldValue = (id, required, messageTarget, label) =>
  (inputValidation ? inputValidation.readIpFieldValue(id, required, messageTarget, label) : null);
const setBanDurationInputFromSeconds = (durationKey, totalSeconds) => {
  if (!inputValidation) return;
  inputValidation.setBanDurationInputFromSeconds(durationKey, totalSeconds);
};
const readBanDurationFromInputs = (durationKey, showInline = false) =>
  (inputValidation ? inputValidation.readBanDurationFromInputs(durationKey, showInline) : null);
const readBanDurationSeconds = (durationKey) =>
  (inputValidation ? inputValidation.readBanDurationSeconds(durationKey) : null);
const readManualBanDurationSeconds = (showInline = false) =>
  (inputValidation ? inputValidation.readManualBanDurationSeconds(showInline) : null);

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
  const next = encodeURIComponent(window.location.pathname + window.location.search);
  window.location.replace(`/dashboard/login.html?next=${next}`);
}

function showTabLoading(tab, message = 'Loading...') {
  if (!tabStateView) return;
  tabStateView.showLoading(tab, message);
}

function showTabError(tab, message) {
  if (!tabStateView) return;
  tabStateView.showError(tab, message);
}

function showTabEmpty(tab, message) {
  if (!tabStateView) return;
  tabStateView.showEmpty(tab, message);
}

function clearTabStateMessage(tab) {
  if (!tabStateView) return;
  tabStateView.clear(tab);
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

  if (legacyConfigDirtyRuntime && typeof legacyConfigDirtyRuntime.runCoreChecks === 'function') {
    legacyConfigDirtyRuntime.runCoreChecks();
  }
  checkGeoConfigChanged();
  checkAdvancedConfigChanged();
}

function createDashboardTabControllers() {
  function makeController(tab) {
    return {
      init: function initTabController() {},
      mount: function mountTabController() {
        document.body.dataset.activeDashboardTab = tab;
        if (dashboardState) {
          dashboardState.setActiveTab(tab);
        }
        refreshCoreActionButtonsState();
        if (hasValidApiContext()) {
          refreshDashboardForTab(tab, 'tab-mount');
        }
      },
      unmount: function unmountTabController() {},
      refresh: function refreshTabController(context = {}) {
        return refreshDashboardForTab(tab, context.reason || 'manual');
      }
    };
  }

  return {
    monitoring: makeController('monitoring'),
    'ip-bans': makeController('ip-bans'),
    status: makeController('status'),
    config: makeController('config'),
    tuning: makeController('tuning')
  };
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

// Update stat cards
function updateStatCards(analytics, events, bans) {
  getById('total-bans').textContent = analytics.ban_count || 0;
  getById('active-bans').textContent = bans.length || 0;
  getById('total-events').textContent = (events.recent_events || []).length;
  const uniqueIps = typeof events.unique_ips === 'number' ? events.unique_ips : (events.top_ips || []).length;
  getById('unique-ips').textContent = uniqueIps;
  
  // Update test mode banner and toggle
  const testMode = analytics.test_mode === true;
  const banner = getById('test-mode-banner');
  const toggle = getById('test-mode-toggle');
  const status = getById('test-mode-status');
  
  if (testMode) {
    banner.classList.remove('hidden');
    status.textContent = 'ENABLED (LOGGING ONLY)';
    status.classList.add('test-mode-status--enabled');
    status.classList.remove('test-mode-status--disabled');
  } else {
    banner.classList.add('hidden');
    status.textContent = 'DISABLED (BLOCKING ACTIVE)';
    status.classList.add('test-mode-status--disabled');
    status.classList.remove('test-mode-status--enabled');
  }
  toggle.checked = testMode;

  applyStatusPanelPatch({
    testMode,
    failMode: statusModule.normalizeFailMode(analytics.fail_mode)
  });
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

function runLegacyConfigDirtyCheck(methodName) {
  if (!legacyConfigDirtyRuntime) return;
  const handler = legacyConfigDirtyRuntime[methodName];
  if (typeof handler === 'function') {
    handler();
  }
}

function checkRobotsConfigChanged() {
  runLegacyConfigDirtyCheck('checkRobots');
}

function checkAiPolicyConfigChanged() {
  runLegacyConfigDirtyCheck('checkAiPolicy');
}

function checkMazeConfigChanged() {
  runLegacyConfigDirtyCheck('checkMaze');
}

function checkBanDurationsChanged() {
  runLegacyConfigDirtyCheck('checkBanDurations');
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
  runLegacyConfigDirtyCheck('checkHoneypot');
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
  runLegacyConfigDirtyCheck('checkBrowserPolicy');
}

function checkBypassAllowlistsConfigChanged() {
  runLegacyConfigDirtyCheck('checkBypassAllowlists');
}

function checkChallengePuzzleConfigChanged() {
  runLegacyConfigDirtyCheck('checkChallengePuzzle');
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
  runLegacyConfigDirtyCheck('checkPow');
}

function checkBotnessConfigChanged() {
  runLegacyConfigDirtyCheck('checkBotness');
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
  runLegacyConfigDirtyCheck('checkCdp');
}

function checkEdgeIntegrationModeChanged() {
  runLegacyConfigDirtyCheck('checkEdgeMode');
}

function checkRateLimitConfigChanged() {
  runLegacyConfigDirtyCheck('checkRateLimit');
}

function checkJsRequiredConfigChanged() {
  runLegacyConfigDirtyCheck('checkJsRequired');
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

function updateLastUpdatedTimestamp() {
  const ts = new Date().toISOString();
  const label = getById('last-updated');
  if (label) label.textContent = `updated: ${ts}`;
}

function isConfigSnapshotEmpty(config) {
  return !config || typeof config !== 'object' || Object.keys(config).length === 0;
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

async function refreshSharedConfig(reason = 'manual', options = {}) {
  const requestOptions = options && options.signal ? { signal: options.signal } : {};
  if (!dashboardApiClient) {
    return dashboardState ? dashboardState.getSnapshot('config') : null;
  }
  if (dashboardState && reason === 'auto-refresh' && !dashboardState.isTabStale('config')) {
    return dashboardState.getSnapshot('config');
  }
  const config = await dashboardApiClient.getConfig(requestOptions);
  if (dashboardState) dashboardState.setSnapshot('config', config);
  await runDomWriteBatch(() => {
    updateConfigModeUi(config, { configSnapshot: config });
    CONFIG_UI_REFRESH_METHODS.forEach((methodName) => invokeConfigUiState(methodName, config));
    invokeConfigUiState('setAdvancedConfigEditorFromConfig', config, true);
    checkAdvancedConfigChanged();
  });
  return config;
}

async function refreshMonitoringTab(reason = 'manual', options = {}) {
  if (!dashboardApiClient) return;
  const isAutoRefresh = reason === 'auto-refresh';
  if (!isAutoRefresh) {
    showTabLoading('monitoring', 'Loading monitoring data...');
    getById('total-bans').textContent = '...';
    getById('active-bans').textContent = '...';
    getById('total-events').textContent = '...';
    getById('unique-ips').textContent = '...';
    if (tablesView && typeof tablesView.showMonitoringLoadingState === 'function') {
      tablesView.showMonitoringLoadingState();
    }
    if (monitoringView && typeof monitoringView.showLoadingState === 'function') {
      monitoringView.showLoadingState();
    }
  }

  const requestOptions = options && options.signal ? { signal: options.signal } : {};
  if (isAutoRefresh) {
    const monitoringData = await dashboardApiClient.getMonitoring({ hours: 24, limit: 10 }, requestOptions);
    if (dashboardState) {
      dashboardState.setSnapshot('monitoring', monitoringData);
    }
    await runDomWriteBatch(() => {
      if (monitoringView) {
        monitoringView.updateMonitoringSummary(monitoringData.summary || {});
        monitoringView.updatePrometheusHelper(monitoringData.prometheus || {});
      }
    });
    return;
  }

  const configSnapshot = dashboardState ? dashboardState.getSnapshot('config') : {};
  const [analyticsResponse, events, bansData, mazeData, cdpData, cdpEventsData, monitoringData] = await Promise.all([
    dashboardApiClient.getAnalytics(requestOptions),
    dashboardApiClient.getEvents(24, requestOptions),
    dashboardApiClient.getBans(requestOptions),
    dashboardApiClient.getMaze(requestOptions),
    dashboardApiClient.getCdp(requestOptions),
    dashboardApiClient.getCdpEvents({ hours: 24, limit: 500 }, requestOptions),
    dashboardApiClient.getMonitoring({ hours: 24, limit: 10 }, requestOptions)
  ]);
  const analytics = deriveMonitoringAnalytics(configSnapshot, analyticsResponse);
  if (Array.isArray(bansData.bans)) {
    analytics.ban_count = bansData.bans.length;
  }

  if (dashboardState) {
    dashboardState.setSnapshot('analytics', analytics);
    dashboardState.setSnapshot('events', events);
    dashboardState.setSnapshot('bans', bansData);
    dashboardState.setSnapshot('maze', mazeData);
    dashboardState.setSnapshot('cdp', cdpData);
    dashboardState.setSnapshot('cdpEvents', cdpEventsData);
    dashboardState.setSnapshot('monitoring', monitoringData);
  }

  await runDomWriteBatch(() => {
    updateStatCards(analytics, events, bansData.bans || []);
    dashboardCharts.updateEventTypesChart(events.event_counts || {});
    dashboardCharts.updateTopIpsChart(events.top_ips || []);
    dashboardCharts.updateTimeSeriesChart();
    if (tablesView) {
      tablesView.updateEventsTable(events.recent_events || []);
      tablesView.updateCdpTotals(cdpData);
      tablesView.updateCdpEventsTable(cdpEventsData.events || []);
    }
    if (monitoringView) {
      monitoringView.updateMazeStats(mazeData);
      monitoringView.updateMonitoringSummary(monitoringData.summary || {});
      monitoringView.updatePrometheusHelper(monitoringData.prometheus || {});
    }
  });

  if (dashboardState && dashboardState.getDerivedState().monitoringEmpty) {
    showTabEmpty('monitoring', 'No operational events yet. Monitoring will populate as traffic arrives.');
  } else {
    clearTabStateMessage('monitoring');
  }
}

async function refreshIpBansTab(reason = 'manual', options = {}) {
  if (!dashboardApiClient) return;
  if (reason !== 'auto-refresh') {
    showTabLoading('ip-bans', 'Loading ban list...');
  }
  const requestOptions = options && options.signal ? { signal: options.signal } : {};
  const bansData = await dashboardApiClient.getBans(requestOptions);
  if (dashboardState) dashboardState.setSnapshot('bans', bansData);
  await runDomWriteBatch(() => {
    if (tablesView) {
      tablesView.updateBansTable(bansData.bans || []);
    }
  });
  if (!Array.isArray(bansData.bans) || bansData.bans.length === 0) {
    showTabEmpty('ip-bans', 'No active bans.');
  } else {
    clearTabStateMessage('ip-bans');
  }
}

async function refreshConfigBackedTab(tab, reason = 'manual', loadingMessage, emptyMessage, options = {}) {
  if (reason !== 'auto-refresh') {
    showTabLoading(tab, loadingMessage);
  }
  const config = await refreshSharedConfig(reason, options);
  if (isConfigSnapshotEmpty(config)) {
    showTabEmpty(tab, emptyMessage);
  } else {
    clearTabStateMessage(tab);
  }
}

const refreshStatusTab = (reason = 'manual', options = {}) =>
  refreshConfigBackedTab(
    'status',
    reason,
    'Loading status signals...',
    'No status config snapshot available yet.',
    options
  );

const refreshConfigTab = (reason = 'manual', options = {}) =>
  refreshConfigBackedTab(
    'config',
    reason,
    'Loading config...',
    'No config snapshot available yet.',
    options
  );

const refreshTuningTab = (reason = 'manual', options = {}) =>
  refreshConfigBackedTab(
    'tuning',
    reason,
    'Loading tuning values...',
    'No tuning config snapshot available yet.',
    options
  );

const TAB_REFRESH_HANDLERS = Object.freeze({
  monitoring: async (reason = 'manual', options = {}) => {
    await refreshMonitoringTab(reason, options);
    if (reason !== 'auto-refresh') {
      await refreshSharedConfig(reason, options);
    }
  },
  'ip-bans': refreshIpBansTab,
  status: refreshStatusTab,
  config: refreshConfigTab,
  tuning: refreshTuningTab
});

async function refreshDashboardForTab(tab, reason = 'manual', options = {}) {
  const activeTab = tabLifecycleModule.normalizeTab(tab);
  try {
    const handler = TAB_REFRESH_HANDLERS[activeTab] || TAB_REFRESH_HANDLERS.monitoring;
    await handler(reason, options);
    if (dashboardState) dashboardState.markTabUpdated(activeTab);
    refreshCoreActionButtonsState();
    updateLastUpdatedTimestamp();
  } catch (error) {
    if (error && error.name === 'AbortError') {
      return;
    }
    const message = error && error.message ? error.message : 'Refresh failed';
    console.error(`Dashboard refresh error (${activeTab}):`, error);
    showTabError(activeTab, message);
    const msg = getById('admin-msg');
    if (msg) {
      msg.textContent = `Refresh failed: ${message}`;
      msg.className = 'message error';
    }
  }
}

function refreshActiveTab(reason = 'manual') {
  if (dashboardTabCoordinator) {
    return dashboardTabCoordinator.refreshActive({ reason });
  }
  const activeTab = dashboardState ? dashboardState.getActiveTab() : 'monitoring';
  return refreshDashboardForTab(activeTab, reason);
}

export function mountDashboardApp(options = {}) {
  if (runtimeMounted) return;
  runtimeMounted = true;
  runtimeMountOptions = normalizeRuntimeMountOptions(options);

  configDraftStore = configDraftStoreModule.create(CONFIG_DRAFT_DEFAULTS);
  runtimeEffects = createRuntimeEffects();
  statusPanel = statusModule.create({ document });

  dashboardState = dashboardStateModule.create({
    initialTab: runtimeMountOptions.initialTab
  });
  tabStateView = tabStateViewModule.create({
    query,
    getStateStore: () => dashboardState
  });

  adminSessionController = adminSessionModule.create({
    resolveAdminApiEndpoint,
    refreshCoreActionButtonsState,
    redirectToLogin,
    request: (input, init) => runtimeEffects.request(input, init)
  });
  if (runtimeMountOptions.bindLogoutButton) {
    adminSessionController.bindLogoutButton('logout-btn', 'admin-msg');
  }

  legacyTabRuntime = createLegacyDashboardTabRuntime({
    document,
    normalizeTab: tabLifecycleModule.normalizeTab,
    defaultTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB,
    getStateStore: () => dashboardState,
    getTabCoordinator: () => dashboardTabCoordinator,
    getRuntimeMountOptions: () => runtimeMountOptions,
    refreshCoreActionButtonsState,
    refreshDashboardForTab
  });

  legacySessionRuntime = createLegacyDashboardSessionRuntime({
    getAdminSessionController: () => adminSessionController,
    getStateStore: () => dashboardState,
    refreshCoreActionButtonsState,
    resolveAdminApiEndpoint,
    getRuntimeEffects: () => runtimeEffects,
    getMessageNode: () => getById('admin-msg')
  });
  configureDashboardExternalAdapters({
    tabRuntime: legacyTabRuntime,
    sessionRuntime: legacySessionRuntime,
    normalizeTab: tabLifecycleModule.normalizeTab,
    defaultTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB
  });

  legacyAutoRefreshRuntime = createLegacyAutoRefreshRuntime({
    effects: runtimeEffects,
    document,
    tabRefreshIntervals: TAB_REFRESH_INTERVAL_MS,
    defaultTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB,
    normalizeTab: tabLifecycleModule.normalizeTab,
    getActiveTab: () => getExternalDashboardActiveTab(),
    hasValidApiContext,
    refreshDashboardForTab
  });

  dashboardApiClient = dashboardApiClientModule.create({
    getAdminContext,
    onUnauthorized: redirectToLogin,
    request: (input, init) => runtimeEffects.request(input, init)
  });

  monitoringView = monitoringViewModule.create({
    escapeHtml,
    effects: runtimeEffects
  });

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

  legacyConfigDirtyRuntime = createLegacyConfigDirtyRuntime({
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

  if (!runtimeMountOptions.useExternalTabPipeline) {
    dashboardTabCoordinator = tabLifecycleModule.createTabLifecycleCoordinator({
      controllers: createDashboardTabControllers(),
      onActiveTabChange: (nextTab) => {
        if (dashboardState) dashboardState.setActiveTab(nextTab);
        if (!runtimeMountOptions.useExternalPollingPipeline && legacyAutoRefreshRuntime) {
          legacyAutoRefreshRuntime.schedule();
        }
      }
    });
    dashboardTabCoordinator.init();
  } else {
    setExternalDashboardActiveTab(runtimeMountOptions.initialTab, 'external-init');
  }
  initInputValidation();
  teardownControlBindings = bindMountScopedDomEvents();
  if (monitoringView) {
    monitoringView.bindPrometheusCopyButtons();
  }
  dashboardCharts.init({
    getAdminContext,
    apiClient: dashboardApiClient
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
  if (!runtimeMountOptions.useExternalSessionPipeline) {
    adminSessionController.restoreAdminSession().then((authenticated) => {
      const sessionState = adminSessionController.getState();
      if (dashboardState) {
        dashboardState.setSession({
          authenticated: sessionState.authenticated === true,
          csrfToken: sessionState.csrfToken || ''
        });
      }
      if (!authenticated) {
        redirectToLogin();
        return;
      }
      refreshActiveTab('session-restored');
      if (!runtimeMountOptions.useExternalPollingPipeline && legacyAutoRefreshRuntime) {
        legacyAutoRefreshRuntime.schedule();
      }
    });
  }

  if (!runtimeMountOptions.useExternalPollingPipeline && legacyAutoRefreshRuntime) {
    legacyAutoRefreshRuntime.bindVisibility();
  }
}

export function mountDashboardExternalRuntime(options = {}) {
  const source = options || {};
  mountDashboardApp({
    useExternalTabPipeline: true,
    useExternalPollingPipeline: true,
    useExternalSessionPipeline: true,
    bindLogoutButton: false,
    initialTab: tabLifecycleModule.normalizeTab(
      source.initialTab || tabLifecycleModule.DEFAULT_DASHBOARD_TAB
    )
  });
}

export function unmountDashboardApp() {
  if (!runtimeMounted) return;
  runtimeMounted = false;
  runtimeMountOptions = normalizeRuntimeMountOptions({});
  if (legacyAutoRefreshRuntime) {
    legacyAutoRefreshRuntime.destroy();
    legacyAutoRefreshRuntime = null;
  }
  legacyConfigDirtyRuntime = null;
  legacyTabRuntime = null;
  legacySessionRuntime = null;
  clearDashboardExternalAdapters();
  if (teardownControlBindings) {
    teardownControlBindings();
    teardownControlBindings = null;
  }
  if (dashboardTabCoordinator && typeof dashboardTabCoordinator.destroy === 'function') {
    dashboardTabCoordinator.destroy();
  }
  if (monitoringView && typeof monitoringView.destroy === 'function') {
    monitoringView.destroy();
  }
  if (dashboardCharts && typeof dashboardCharts.destroy === 'function') {
    dashboardCharts.destroy();
  }
  const logoutButton = getById('logout-btn');
  if (logoutButton) {
    logoutButton.onclick = null;
  }
  if (document.body && document.body.dataset) {
    delete document.body.dataset.activeDashboardTab;
  }
  dashboardTabCoordinator = null;
  dashboardApiClient = null;
  dashboardState = null;
  monitoringView = null;
  tablesView = null;
  tabStateView = null;
  configUiState = null;
  inputValidation = null;
  configDraftStore = null;
  runtimeEffects = null;
  statusPanel = null;
  adminSessionController = null;
}
