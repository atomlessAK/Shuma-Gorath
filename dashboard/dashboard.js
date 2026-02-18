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
import * as featureControllersModule from './modules/feature-controllers.js';
import { createRuntimeEffects } from './modules/services/runtime-effects.js';

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
let autoRefreshTimer = null;
let pageVisible = document.visibilityState !== 'hidden';
let dashboardMounted = false;
let dashboardEventAbortController = null;
let dashboardMountVersion = 0;
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

function runDomWriteBatch(task) {
  return new Promise((resolve) => {
    domWriteScheduler.schedule(() => {
      task();
      resolve();
    });
  });
}

function setDashboardEventController() {
  dashboardEventAbortController = new AbortController();
}

function clearDashboardEventController() {
  if (!dashboardEventAbortController) return;
  dashboardEventAbortController.abort();
  dashboardEventAbortController = null;
}

function dashboardEventOptions() {
  if (!dashboardEventAbortController) return undefined;
  return { signal: dashboardEventAbortController.signal };
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

// Check if robots config has changed from saved state
function checkRobotsConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.robots);
}

function checkAiPolicyConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.aiPolicy);
}

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

function runDirtySaveCheck(spec) {
  if (!spec || typeof spec.compute !== 'function') return;
  const apiValid = hasValidApiContext();
  const result = spec.compute();
  const fieldsValid = result && result.fieldsValid !== false;
  const changed = Boolean(result && result.changed);
  setDirtySaveButtonState(spec.buttonId, changed, apiValid, fieldsValid);
  if (changed && typeof spec.onChanged === 'function') {
    spec.onChanged();
  }
}

const DIRTY_CHECK_REGISTRY = Object.freeze({
  robots: {
    buttonId: 'save-robots-config',
    onChanged: () => {
      const btn = getById('save-robots-config');
      if (btn) btn.textContent = 'Save robots serving';
    },
    compute: () => {
      const delayValid = validateIntegerFieldById('robots-crawl-delay');
      const current = {
        enabled: getById('robots-enabled-toggle').checked,
        crawlDelay: parseInt(getById('robots-crawl-delay').value, 10) || 2
      };
      return {
        fieldsValid: delayValid,
        changed: delayValid && isDraftDirty('robots', current)
      };
    }
  },
  aiPolicy: {
    buttonId: 'save-ai-policy-config',
    onChanged: () => {
      const btn = getById('save-ai-policy-config');
      if (btn) btn.textContent = 'Save AI bot policy';
    },
    compute: () => {
      const current = {
        blockTraining: getById('robots-block-training-toggle').checked,
        blockSearch: getById('robots-block-search-toggle').checked,
        allowSearch: getById('robots-allow-search-toggle').checked
      };
      return {
        fieldsValid: true,
        changed: isDraftDirty('aiPolicy', current)
      };
    }
  },
  maze: {
    buttonId: 'save-maze-config',
    compute: () => {
      const currentThreshold = parseIntegerLoose('maze-threshold');
      const fieldsValid = validateIntegerFieldById('maze-threshold');
      return {
        fieldsValid,
        changed: fieldsValid && isDraftDirty('maze', {
          enabled: getById('maze-enabled-toggle').checked,
          autoBan: getById('maze-auto-ban-toggle').checked,
          threshold: currentThreshold
        })
      };
    }
  },
  banDurations: {
    buttonId: 'save-durations-btn',
    compute: () => {
      const honeypot = readBanDurationFromInputs('honeypot');
      const rateLimit = readBanDurationFromInputs('rateLimit');
      const browser = readBanDurationFromInputs('browser');
      const cdp = readBanDurationFromInputs('cdp');
      const admin = readBanDurationFromInputs('admin');
      const fieldsValid = Boolean(honeypot && rateLimit && browser && cdp && admin);
      const current = fieldsValid ? {
        honeypot: honeypot.totalSeconds,
        rateLimit: rateLimit.totalSeconds,
        browser: browser.totalSeconds,
        cdp: cdp.totalSeconds,
        admin: admin.totalSeconds
      } : getDraft('banDurations');
      return {
        fieldsValid,
        changed: fieldsValid && isDraftDirty('banDurations', current)
      };
    }
  },
  honeypot: {
    buttonId: 'save-honeypot-config',
    compute: () => {
      const fieldsValid = validateHoneypotPathsField();
      const currentEnabled = getById('honeypot-enabled-toggle').checked;
      const saved = getDraft('honeypot');
      const current = fieldsValid
        ? normalizeListTextareaForCompare(getById('honeypot-paths').value)
        : saved.values;
      return {
        fieldsValid,
        changed: fieldsValid && (
          currentEnabled !== saved.enabled ||
          current !== saved.values
        )
      };
    }
  },
  browserPolicy: {
    buttonId: 'save-browser-policy-config',
    compute: () => {
      const blockValid = validateBrowserRulesField('browser-block-rules');
      const whitelistValid = validateBrowserRulesField('browser-whitelist-rules');
      const fieldsValid = blockValid && whitelistValid;
      const currentBlock = normalizeBrowserRulesForCompare(getById('browser-block-rules').value);
      const currentWhitelist = normalizeBrowserRulesForCompare(getById('browser-whitelist-rules').value);
      return {
        fieldsValid,
        changed: fieldsValid && isDraftDirty('browserPolicy', {
          block: currentBlock,
          whitelist: currentWhitelist
        })
      };
    }
  },
  bypassAllowlists: {
    buttonId: 'save-whitelist-config',
    compute: () => {
      const current = {
        network: normalizeListTextareaForCompare(getById('network-whitelist').value),
        path: normalizeListTextareaForCompare(getById('path-whitelist').value)
      };
      return {
        fieldsValid: true,
        changed: isDraftDirty('bypassAllowlists', current)
      };
    }
  },
  challengePuzzle: {
    buttonId: 'save-challenge-puzzle-config',
    compute: () => {
      const fieldsValid = validateIntegerFieldById('challenge-puzzle-transform-count');
      const toggle = getById('challenge-puzzle-enabled-toggle');
      const current = parseIntegerLoose('challenge-puzzle-transform-count');
      const saved = getDraft('challengePuzzle');
      const enabledChanged = Boolean(toggle && (toggle.checked !== saved.enabled));
      const countChanged = current !== null && current !== saved.count;
      return {
        fieldsValid,
        changed: fieldsValid && (enabledChanged || countChanged)
      };
    }
  },
  pow: {
    buttonId: 'save-pow-config',
    compute: () => {
      const fieldsValid =
        validateIntegerFieldById('pow-difficulty') && validateIntegerFieldById('pow-ttl');
      const current = {
        enabled: getById('pow-enabled-toggle').checked,
        difficulty: parseInt(getById('pow-difficulty').value, 10) || 15,
        ttl: parseInt(getById('pow-ttl').value, 10) || 90
      };
      return {
        fieldsValid,
        changed: isDraftDirty('pow', current)
      };
    }
  },
  botness: {
    buttonId: 'save-botness-config',
    compute: () => {
      const fieldsValid =
        validateIntegerFieldById('challenge-puzzle-threshold') &&
        validateIntegerFieldById('maze-threshold-score') &&
        validateIntegerFieldById('weight-js-required') &&
        validateIntegerFieldById('weight-geo-risk') &&
        validateIntegerFieldById('weight-rate-medium') &&
        validateIntegerFieldById('weight-rate-high');
      const current = {
        challengeThreshold: parseInt(getById('challenge-puzzle-threshold').value, 10) || 3,
        mazeThreshold: parseInt(getById('maze-threshold-score').value, 10) || 6,
        weightJsRequired: parseInt(getById('weight-js-required').value, 10) || 1,
        weightGeoRisk: parseInt(getById('weight-geo-risk').value, 10) || 2,
        weightRateMedium: parseInt(getById('weight-rate-medium').value, 10) || 1,
        weightRateHigh: parseInt(getById('weight-rate-high').value, 10) || 2
      };
      return {
        fieldsValid,
        changed: isDraftDirty('botness', current)
      };
    }
  },
  cdp: {
    buttonId: 'save-cdp-config',
    compute: () => {
      const current = {
        enabled: getById('cdp-enabled-toggle').checked,
        autoBan: getById('cdp-auto-ban-toggle').checked,
        threshold: parseFloat(getById('cdp-threshold-slider').value)
      };
      return {
        fieldsValid: true,
        changed: isDraftDirty('cdp', current)
      };
    }
  },
  edgeMode: {
    buttonId: 'save-edge-integration-mode-config',
    compute: () => {
      const select = getById('edge-integration-mode-select');
      if (!select) {
        return { fieldsValid: false, changed: false };
      }
      const current = normalizeEdgeIntegrationMode(select.value);
      return {
        fieldsValid: true,
        changed: isDraftDirty('edgeMode', { mode: current })
      };
    }
  },
  rateLimit: {
    buttonId: 'save-rate-limit-config',
    compute: () => {
      const valueValid = validateIntegerFieldById('rate-limit-threshold');
      const current = parseIntegerLoose('rate-limit-threshold');
      return {
        fieldsValid: valueValid,
        changed: current !== null && isDraftDirty('rateLimit', { value: current })
      };
    }
  },
  jsRequired: {
    buttonId: 'save-js-required-config',
    compute: () => {
      const current = getById('js-required-enforced-toggle').checked;
      return {
        fieldsValid: true,
        changed: isDraftDirty('jsRequired', { enforced: current })
      };
    }
  }
});

function checkMazeConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.maze);
}

function checkBanDurationsChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.banDurations);
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
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.honeypot);
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
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.browserPolicy);
}

function checkBypassAllowlistsConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.bypassAllowlists);
}

function checkChallengePuzzleConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.challengePuzzle);
}

const bindFieldEvent = (id, eventName, handler) => {
  const field = getById(id);
  if (!field || typeof handler !== 'function') return;
  field.addEventListener(eventName, handler, dashboardEventOptions());
};

const bindInputAndBlur = (id, handler) => {
  bindFieldEvent(id, 'input', handler);
  bindFieldEvent(id, 'blur', handler);
};

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
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.pow);
}

function checkBotnessConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.botness);
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

// Check if CDP config has changed from saved state
function checkCdpConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.cdp);
}

function checkEdgeIntegrationModeChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.edgeMode);
}

function checkRateLimitConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.rateLimit);
}

function checkJsRequiredConfigChanged() {
  runDirtySaveCheck(DIRTY_CHECK_REGISTRY.jsRequired);
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

const DIRTY_SECTIONS_BY_TAB = Object.freeze({
  monitoring: [],
  'ip-bans': [],
  status: [],
  config: [
    'maze',
    'robots',
    'aiPolicy',
    'geo',
    'honeypot',
    'browserPolicy',
    'bypassAllowlists',
    'edgeMode',
    'advancedConfig'
  ],
  tuning: [
    'banDurations',
    'pow',
    'challengePuzzle',
    'botness',
    'cdp',
    'rateLimit',
    'jsRequired'
  ]
});

const CORE_ACTION_FIELD_IDS = new Set([
  'ban-ip',
  'unban-ip',
  'ban-duration-days',
  'ban-duration-hours',
  'ban-duration-minutes'
]);

const DIRTY_SECTIONS_BY_FIELD_ID = Object.freeze({
  'robots-crawl-delay': ['robots'],
  'maze-threshold': ['maze'],
  'dur-honeypot-days': ['banDurations'],
  'dur-honeypot-hours': ['banDurations'],
  'dur-honeypot-minutes': ['banDurations'],
  'dur-rate-limit-days': ['banDurations'],
  'dur-rate-limit-hours': ['banDurations'],
  'dur-rate-limit-minutes': ['banDurations'],
  'dur-browser-days': ['banDurations'],
  'dur-browser-hours': ['banDurations'],
  'dur-browser-minutes': ['banDurations'],
  'dur-cdp-days': ['banDurations'],
  'dur-cdp-hours': ['banDurations'],
  'dur-cdp-minutes': ['banDurations'],
  'dur-admin-days': ['banDurations'],
  'dur-admin-hours': ['banDurations'],
  'dur-admin-minutes': ['banDurations'],
  'pow-difficulty': ['pow'],
  'pow-ttl': ['pow'],
  'challenge-puzzle-transform-count': ['challengePuzzle'],
  'challenge-puzzle-threshold': ['botness'],
  'maze-threshold-score': ['botness'],
  'weight-js-required': ['botness'],
  'weight-geo-risk': ['botness'],
  'weight-rate-medium': ['botness'],
  'weight-rate-high': ['botness'],
  'rate-limit-threshold': ['rateLimit']
});

function refreshDirtySections(sectionKeys = []) {
  sectionKeys.forEach((sectionKey) => {
    if (sectionKey === 'geo') {
      checkGeoConfigChanged();
      return;
    }
    if (sectionKey === 'advancedConfig') {
      checkAdvancedConfigChanged();
      return;
    }
    const spec = DIRTY_CHECK_REGISTRY[sectionKey];
    if (spec) {
      runDirtySaveCheck(spec);
    }
  });
}

function refreshAllDirtySections() {
  const allKeys = Object.keys(DIRTY_CHECK_REGISTRY);
  refreshDirtySections([...allKeys, 'geo', 'advancedConfig']);
}

function refreshDirtySectionsForField(fieldId) {
  if (!fieldId) return;
  const sections = DIRTY_SECTIONS_BY_FIELD_ID[fieldId];
  if (!Array.isArray(sections) || sections.length === 0) return;
  refreshDirtySections(sections);
}

function bindDashboardFieldEvents() {
  [
    { ids: ['robots-enabled-toggle'], event: 'change', handler: checkRobotsConfigChanged },
    { ids: ['robots-crawl-delay'], event: 'input', handler: checkRobotsConfigChanged },
    {
      ids: ['robots-block-training-toggle', 'robots-block-search-toggle', 'robots-allow-search-toggle'],
      event: 'change',
      handler: checkAiPolicyConfigChanged
    },
    { ids: ['maze-enabled-toggle', 'maze-auto-ban-toggle'], event: 'change', handler: checkMazeConfigChanged },
    { ids: ['maze-threshold'], event: 'input', handler: checkMazeConfigChanged },
    { ids: ['maze-threshold'], event: 'blur', handler: checkMazeConfigChanged },
    { ids: ['honeypot-enabled-toggle'], event: 'change', handler: checkHoneypotConfigChanged },
    { ids: ['challenge-puzzle-transform-count'], event: 'input', handler: checkChallengePuzzleConfigChanged },
    { ids: ['challenge-puzzle-enabled-toggle'], event: 'change', handler: checkChallengePuzzleConfigChanged }
  ].forEach(({ ids, event, handler }) => {
    ids.forEach((id) => bindFieldEvent(id, event, handler));
  });

  bindInputAndBlur('honeypot-paths', () => {
    validateHoneypotPathsField(true);
    checkHoneypotConfigChanged();
  });

  ['browser-block-rules', 'browser-whitelist-rules'].forEach((id) => {
    bindInputAndBlur(id, () => {
      validateBrowserRulesField(id, true);
      checkBrowserPolicyConfigChanged();
    });
  });

  ['network-whitelist', 'path-whitelist'].forEach((id) => {
    bindInputAndBlur(id, () => {
      checkBypassAllowlistsConfigChanged();
    });
  });

  const previewRobotsButton = getById('preview-robots');
  if (previewRobotsButton) {
    previewRobotsButton.addEventListener('click', async function toggleRobotsPreview() {
      const preview = getById('robots-preview');
      if (!preview) return;
      const btn = this;
      if (preview.classList.contains('hidden')) {
        btn.textContent = 'Loading...';
        btn.disabled = true;
        await refreshRobotsPreview();
        preview.classList.remove('hidden');
        btn.textContent = 'Hide robots.txt';
        btn.disabled = false;
      } else {
        preview.classList.add('hidden');
        btn.textContent = 'Show robots.txt';
      }
    }, dashboardEventOptions());
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
    bindFieldEvent(id, 'input', () => {
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
    });
    bindFieldEvent(id, 'blur', () => {
      validateGeoFieldById(id, true);
      checkGeoConfigChanged();
    });
  });

  bindFieldEvent('rate-limit-threshold', 'input', checkRateLimitConfigChanged);
  bindFieldEvent('js-required-enforced-toggle', 'change', checkJsRequiredConfigChanged);

  bindInputAndBlur('advanced-config-json', () => {
    checkAdvancedConfigChanged();
  });

  bindFieldEvent('cdp-threshold-slider', 'input', function onCdpSliderInput() {
    getById('cdp-threshold-value').textContent = parseFloat(this.value).toFixed(1);
    checkCdpConfigChanged();
  });

  ['cdp-enabled-toggle', 'cdp-auto-ban-toggle'].forEach((id) => {
    bindFieldEvent(id, 'change', checkCdpConfigChanged);
  });

  bindFieldEvent('edge-integration-mode-select', 'change', checkEdgeIntegrationModeChanged);

  const banButton = getById('ban-btn');
  if (banButton) {
    banButton.addEventListener('click', async function banIpAction() {
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
        msg.textContent = `Error: ${e.message}`;
        msg.className = 'message error';
      }
    }, dashboardEventOptions());
  }

  const unbanButton = getById('unban-btn');
  if (unbanButton) {
    unbanButton.addEventListener('click', async function unbanIpAction() {
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
        msg.textContent = `Error: ${e.message}`;
        msg.className = 'message error';
      }
    }, dashboardEventOptions());
  }

  document.addEventListener('visibilitychange', () => {
    pageVisible = document.visibilityState !== 'hidden';
    if (pageVisible) {
      scheduleAutoRefresh();
    } else {
      clearAutoRefreshTimer();
    }
  }, dashboardEventOptions());
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

async function refreshSharedConfig(reason = 'manual') {
  if (!dashboardApiClient) {
    return dashboardState ? dashboardState.getSnapshot('config') : null;
  }
  if (dashboardState && reason === 'auto-refresh' && !dashboardState.isTabStale('config')) {
    return dashboardState.getSnapshot('config');
  }
  const config = await dashboardApiClient.getConfig();
  if (dashboardState) dashboardState.setSnapshot('config', config);
  await runDomWriteBatch(() => {
    updateConfigModeUi(config, { configSnapshot: config });
    CONFIG_UI_REFRESH_METHODS.forEach((methodName) => invokeConfigUiState(methodName, config));
    invokeConfigUiState('setAdvancedConfigEditorFromConfig', config, true);
    refreshAllDirtySections();
  });
  return config;
}

async function refreshMonitoringTab(reason = 'manual') {
  if (!dashboardApiClient) return;
  if (reason !== 'auto-refresh') {
    showTabLoading('monitoring', 'Loading monitoring data...');
  }

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

  const [analytics, events, bansData, mazeData, cdpData, cdpEventsData, monitoringData] = await Promise.all([
    dashboardApiClient.getAnalytics(),
    dashboardApiClient.getEvents(24),
    dashboardApiClient.getBans(),
    dashboardApiClient.getMaze(),
    dashboardApiClient.getCdp(),
    dashboardApiClient.getCdpEvents({ hours: 24, limit: 500 }),
    dashboardApiClient.getMonitoring({ hours: 24, limit: 10 })
  ]);

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

async function refreshIpBansTab(reason = 'manual') {
  if (!dashboardApiClient) return;
  if (reason !== 'auto-refresh') {
    showTabLoading('ip-bans', 'Loading ban list...');
  }
  const bansData = await dashboardApiClient.getBans();
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

async function refreshConfigBackedTab(tab, reason = 'manual', loadingMessage, emptyMessage) {
  if (reason !== 'auto-refresh') {
    showTabLoading(tab, loadingMessage);
  }
  const config = await refreshSharedConfig(reason);
  if (isConfigSnapshotEmpty(config)) {
    showTabEmpty(tab, emptyMessage);
  } else {
    clearTabStateMessage(tab);
  }
}

const refreshStatusTab = (reason = 'manual') =>
  refreshConfigBackedTab(
    'status',
    reason,
    'Loading status signals...',
    'No status config snapshot available yet.'
  );

const refreshConfigTab = (reason = 'manual') =>
  refreshConfigBackedTab('config', reason, 'Loading config...', 'No config snapshot available yet.');

const refreshTuningTab = (reason = 'manual') =>
  refreshConfigBackedTab(
    'tuning',
    reason,
    'Loading tuning values...',
    'No tuning config snapshot available yet.'
  );

const TAB_REFRESH_HANDLERS = Object.freeze({
  monitoring: async (reason = 'manual') => {
    await refreshMonitoringTab(reason);
    if (reason !== 'auto-refresh') {
      await refreshSharedConfig(reason);
    }
  },
  'ip-bans': refreshIpBansTab,
  status: refreshStatusTab,
  config: refreshConfigTab,
  tuning: refreshTuningTab
});

async function refreshDashboardForTab(tab, reason = 'manual') {
  const activeTab = tabLifecycleModule.normalizeTab(tab);
  try {
    const handler = TAB_REFRESH_HANDLERS[activeTab] || TAB_REFRESH_HANDLERS.monitoring;
    await handler(reason);
    if (dashboardState) dashboardState.markTabUpdated(activeTab);
    refreshCoreActionButtonsState();
    refreshDirtySections(DIRTY_SECTIONS_BY_TAB[activeTab] || []);
    updateLastUpdatedTimestamp();
  } catch (error) {
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

function createDashboardFeatureControllers() {
  return featureControllersModule.createDashboardFeatureControllers({
    notifyTabMounted: (tab) => {
      document.body.dataset.activeDashboardTab = tab;
      if (dashboardState) {
        dashboardState.setActiveTab(tab);
      }
      refreshCoreActionButtonsState();
    },
    notifyTabUnmounted: () => {},
    refreshTab: (tab, reason) => refreshDashboardForTab(tab, reason),
    hasValidApiContext
  });
}

function refreshActiveTab(reason = 'manual') {
  if (dashboardTabCoordinator) {
    return dashboardTabCoordinator.refreshActive({ reason });
  }
  return refreshDashboardForTab('monitoring', reason);
}

function clearAutoRefreshTimer() {
  if (autoRefreshTimer) {
    runtimeEffects.clearTimer(autoRefreshTimer);
    autoRefreshTimer = null;
  }
}

function scheduleAutoRefresh() {
  clearAutoRefreshTimer();
  if (!hasValidApiContext() || !pageVisible) return;
  const activeTab = dashboardTabCoordinator
    ? dashboardTabCoordinator.getActiveTab()
    : (dashboardState ? dashboardState.getActiveTab() : 'monitoring');
  const interval = TAB_REFRESH_INTERVAL_MS[activeTab] || TAB_REFRESH_INTERVAL_MS.monitoring;
  autoRefreshTimer = runtimeEffects.setTimer(async () => {
    autoRefreshTimer = null;
    if (hasValidApiContext() && pageVisible) {
      await refreshDashboardForTab(activeTab, 'auto-refresh');
    }
    scheduleAutoRefresh();
  }, interval);
}

function createConfigControlsContext() {
  return {
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
}

function initializeDashboardRuntime() {
  configDraftStore = configDraftStoreModule.create(CONFIG_DRAFT_DEFAULTS);
  runtimeEffects = createRuntimeEffects();
  statusPanel = statusModule.create({ document });

  dashboardState = dashboardStateModule.create({
    initialTab: tabLifecycleModule.DEFAULT_DASHBOARD_TAB
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
  adminSessionController.bindLogoutButton('logout-btn', 'admin-msg');

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
        msg.textContent = `Error: ${e.message}`;
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
    onFieldInteraction: (fieldId) => {
      if (fieldId && CORE_ACTION_FIELD_IDS.has(fieldId)) {
        refreshCoreActionButtonsState();
      }
      refreshDirtySectionsForField(fieldId);
    }
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

  dashboardTabCoordinator = tabLifecycleModule.createTabLifecycleCoordinator({
    controllers: createDashboardFeatureControllers(),
    onActiveTabChange: (nextTab) => {
      if (dashboardState) dashboardState.setActiveTab(nextTab);
      scheduleAutoRefresh();
    },
    effects: runtimeEffects
  });
  dashboardTabCoordinator.init();
  initInputValidation();
  bindDashboardFieldEvents();

  if (monitoringView) {
    monitoringView.bindPrometheusCopyButtons();
  }
  dashboardCharts.init({
    getAdminContext,
    apiClient: dashboardApiClient
  });
  statusPanel.render();

  configControls.bind({
    context: createConfigControlsContext()
  });
}

function restoreSessionAndRefresh(version) {
  adminSessionController.restoreAdminSession().then((authenticated) => {
    if (!dashboardMounted || version !== dashboardMountVersion) return;
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
    scheduleAutoRefresh();
  });
}

export function mountDashboard() {
  if (dashboardMounted) {
    return unmountDashboard;
  }
  dashboardMounted = true;
  dashboardMountVersion += 1;
  pageVisible = document.visibilityState !== 'hidden';
  setDashboardEventController();
  initializeDashboardRuntime();
  restoreSessionAndRefresh(dashboardMountVersion);
  return unmountDashboard;
}

export function unmountDashboard() {
  if (!dashboardMounted) return;
  dashboardMounted = false;
  dashboardMountVersion += 1;
  clearAutoRefreshTimer();
  if (dashboardTabCoordinator && typeof dashboardTabCoordinator.destroy === 'function') {
    dashboardTabCoordinator.destroy();
  }
  clearDashboardEventController();
}
