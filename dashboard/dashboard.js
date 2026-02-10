// Global chart instances
let eventTypesChart = null;
let topIpsChart = null;
let timeSeriesChart = null;
let currentTimeRange = 'hour';

const CHART_PALETTE = [
  "rgb(255,205,235)",
  "rgb(225,175,205)",
  "rgb(205, 155, 185)",
  "rgb(190, 140, 170)",
  "rgb(175, 125, 155)",
  "rgb(160, 110, 140)",
  "rgb(147, 97, 127)",
  "rgb(135, 85, 115)"
];

const statusPanelState = {
  failMode: 'unknown',
  testMode: false,
  powEnabled: false,
  powMutable: false,
  mazeEnabled: false,
  mazeAutoBan: false,
  cdpEnabled: false,
  cdpAutoBan: false,
  jsRequiredEnforced: true,
  challengeThreshold: 3,
  challengeMutable: false,
  mazeThreshold: 6,
  rateLimit: 80,
  geoRiskCount: 0,
  geoAllowCount: 0,
  geoChallengeCount: 0,
  geoMazeCount: 0,
  geoBlockCount: 0,
  botnessMutable: false,
  botnessWeights: {
    js_required: 1,
    geo_risk: 2,
    rate_medium: 1,
    rate_high: 2
  }
};

const INTEGER_FIELD_RULES = {
  'ban-duration': { min: 60, max: 31536000, fallback: 3600, label: 'Ban duration' },
  'robots-crawl-delay': { min: 0, max: 60, fallback: 2, label: 'Crawl delay' },
  'maze-threshold': { min: 5, max: 500, fallback: 50, label: 'Maze threshold' },
  'rate-limit-threshold': { min: 1, max: 1000000, fallback: 80, label: 'Rate limit' },
  'pow-difficulty': { min: 12, max: 20, fallback: 15, label: 'PoW difficulty' },
  'pow-ttl': { min: 30, max: 300, fallback: 90, label: 'PoW seed TTL' },
  'dur-honeypot-hours': { min: 0, max: 8760, fallback: 24, label: 'Link Maze Threshold Exceeded hours' },
  'dur-honeypot-minutes': { min: 0, max: 59, fallback: 0, label: 'Link Maze Threshold Exceeded minutes' },
  'dur-rate-limit-hours': { min: 0, max: 8760, fallback: 1, label: 'Rate Limit Exceeded hours' },
  'dur-rate-limit-minutes': { min: 0, max: 59, fallback: 0, label: 'Rate Limit Exceeded minutes' },
  'dur-browser-hours': { min: 0, max: 8760, fallback: 6, label: 'Outdated Browser Detected hours' },
  'dur-browser-minutes': { min: 0, max: 59, fallback: 0, label: 'Outdated Browser Detected minutes' },
  'dur-admin-hours': { min: 0, max: 8760, fallback: 6, label: 'Admin Manual Ban hours' },
  'dur-admin-minutes': { min: 0, max: 59, fallback: 0, label: 'Admin Manual Ban minutes' },
  'challenge-threshold': { min: 1, max: 10, fallback: 3, label: 'Challenge threshold' },
  'maze-threshold-score': { min: 1, max: 10, fallback: 6, label: 'Maze threshold' },
  'weight-js-required': { min: 0, max: 10, fallback: 1, label: 'JS weight' },
  'weight-geo-risk': { min: 0, max: 10, fallback: 2, label: 'GEO weight' },
  'weight-rate-medium': { min: 0, max: 10, fallback: 1, label: 'Rate 50% weight' },
  'weight-rate-high': { min: 0, max: 10, fallback: 2, label: 'Rate 80% weight' }
};

const BAN_DURATION_BOUNDS_SECONDS = { min: 60, max: 31536000 };

const BAN_DURATION_FIELDS = {
  honeypot: {
    label: 'Link Maze Threshold Exceeded duration',
    fallback: 86400,
    hoursId: 'dur-honeypot-hours',
    minutesId: 'dur-honeypot-minutes'
  },
  rateLimit: {
    label: 'Rate Limit Exceeded duration',
    fallback: 3600,
    hoursId: 'dur-rate-limit-hours',
    minutesId: 'dur-rate-limit-minutes'
  },
  browser: {
    label: 'Outdated Browser Detected duration',
    fallback: 21600,
    hoursId: 'dur-browser-hours',
    minutesId: 'dur-browser-minutes'
  },
  admin: {
    label: 'Admin Manual Ban duration',
    fallback: 21600,
    hoursId: 'dur-admin-hours',
    minutesId: 'dur-admin-minutes'
  }
};

const IPV4_SEGMENT_PATTERN = /^\d{1,3}$/;
const IPV6_INPUT_PATTERN = /^[0-9a-fA-F:.]+$/;

function sanitizeIntegerText(value) {
  return (value || '').replace(/[^\d]/g, '');
}

function sanitizeIpText(value) {
  return (value || '').replace(/[^0-9a-fA-F:.]/g, '');
}

function sanitizeEndpointText(value) {
  return (value || '').replace(/\s+/g, '').trim();
}

function fieldErrorIdFor(input) {
  const raw = input.id || input.name || 'field';
  return `field-error-${raw.replace(/[^a-zA-Z0-9_-]/g, '-')}`;
}

function getOrCreateFieldErrorElement(input) {
  if (!input || !input.parentElement) return null;
  const id = fieldErrorIdFor(input);
  let errorEl = document.getElementById(id);
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

function parseIntegerLoose(id) {
  const input = document.getElementById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return null;
  const sanitized = sanitizeIntegerText(input.value);
  if (input.value !== sanitized) input.value = sanitized;
  if (sanitized.length === 0) return null;
  const parsed = Number.parseInt(sanitized, 10);
  if (!Number.isInteger(parsed)) return null;
  return parsed;
}

function durationPartsToSeconds(hours, minutes) {
  return (hours * 3600) + (minutes * 60);
}

function secondsToDurationParts(totalSeconds, fallbackSeconds) {
  const fallback = Number.parseInt(fallbackSeconds, 10) || 0;
  let seconds = Number.parseInt(totalSeconds, 10);
  if (!Number.isFinite(seconds) || seconds <= 0) seconds = fallback;
  if (seconds < BAN_DURATION_BOUNDS_SECONDS.min) seconds = BAN_DURATION_BOUNDS_SECONDS.min;
  if (seconds > BAN_DURATION_BOUNDS_SECONDS.max) seconds = BAN_DURATION_BOUNDS_SECONDS.max;
  return {
    hours: Math.floor(seconds / 3600),
    minutes: Math.floor((seconds % 3600) / 60)
  };
}

function setBanDurationInputFromSeconds(durationKey, totalSeconds) {
  const group = BAN_DURATION_FIELDS[durationKey];
  if (!group) return;
  const hoursInput = document.getElementById(group.hoursId);
  const minutesInput = document.getElementById(group.minutesId);
  if (!hoursInput || !minutesInput) return;

  const parts = secondsToDurationParts(totalSeconds, group.fallback);
  hoursInput.value = String(parts.hours);
  minutesInput.value = String(parts.minutes);
}

function readBanDurationFromInputs(durationKey, showInline = false) {
  const group = BAN_DURATION_FIELDS[durationKey];
  if (!group) return null;

  const hoursInput = document.getElementById(group.hoursId);
  const minutesInput = document.getElementById(group.minutesId);
  if (!hoursInput || !minutesInput) return null;

  const hoursValid = validateIntegerFieldById(group.hoursId, showInline);
  const minutesValid = validateIntegerFieldById(group.minutesId, showInline);
  const hours = parseIntegerLoose(group.hoursId);
  const minutes = parseIntegerLoose(group.minutesId);

  if (!hoursValid || !minutesValid || hours === null || minutes === null) return null;

  const totalSeconds = durationPartsToSeconds(hours, minutes);
  if (totalSeconds < BAN_DURATION_BOUNDS_SECONDS.min || totalSeconds > BAN_DURATION_BOUNDS_SECONDS.max) {
    const message = `${group.label} must be between 1 minute and 8760 hours.`;
    setFieldError(hoursInput, message, showInline);
    setFieldError(minutesInput, message, showInline);
    return null;
  }

  setFieldError(hoursInput, '', showInline);
  setFieldError(minutesInput, '', showInline);
  return { hours, minutes, totalSeconds };
}

function readBanDurationSeconds(durationKey) {
  const group = BAN_DURATION_FIELDS[durationKey];
  if (!group) return null;
  const result = readBanDurationFromInputs(durationKey, true);
  if (result) return result.totalSeconds;

  const hoursInput = document.getElementById(group.hoursId);
  const minutesInput = document.getElementById(group.minutesId);
  if (hoursInput && !hoursInput.checkValidity()) {
    hoursInput.reportValidity();
    hoursInput.focus();
    return null;
  }
  if (minutesInput && !minutesInput.checkValidity()) {
    minutesInput.reportValidity();
    minutesInput.focus();
    return null;
  }
  if (hoursInput) {
    hoursInput.reportValidity();
    hoursInput.focus();
  }
  return null;
}

function isValidIpv4(value) {
  const parts = value.split('.');
  if (parts.length !== 4) return false;
  return parts.every(part => {
    if (!IPV4_SEGMENT_PATTERN.test(part)) return false;
    if (part.length > 1 && part.startsWith('0')) return false;
    const num = Number.parseInt(part, 10);
    return num >= 0 && num <= 255;
  });
}

function isValidIpv6(value) {
  if (!IPV6_INPUT_PATTERN.test(value)) return false;
  try {
    new URL(`http://[${value}]/`);
    return true;
  } catch (e) {
    return false;
  }
}

function isValidIpAddress(value) {
  if (!value) return false;
  if (value.includes(':')) return isValidIpv6(value);
  if (value.includes('.')) return isValidIpv4(value);
  return false;
}

function parseEndpointUrl(value) {
  const sanitized = sanitizeEndpointText(value);
  if (!sanitized) return null;
  try {
    const url = new URL(sanitized);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
    if (!url.hostname) return null;
    const pathname = url.pathname === '/' ? '' : url.pathname.replace(/\/+$/, '');
    return `${url.protocol}//${url.host}${pathname}`;
  } catch (e) {
    return null;
  }
}

function validateIntegerFieldById(id, showInline = false) {
  const input = document.getElementById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return false;
  const parsed = parseIntegerLoose(id);
  if (parsed === null) {
    setFieldError(input, `${rules.label} is required.`, showInline);
    return false;
  }
  if (parsed < rules.min || parsed > rules.max) {
    setFieldError(input, `${rules.label} must be between ${rules.min} and ${rules.max}.`, showInline);
    return false;
  }
  setFieldError(input, '', showInline);
  return true;
}

function readIntegerFieldValue(id, messageTarget) {
  const input = document.getElementById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return null;
  if (!validateIntegerFieldById(id, true)) {
    const parsed = parseIntegerLoose(id);
    const message = parsed === null
      ? `${rules.label} is required.`
      : `${rules.label} must be between ${rules.min} and ${rules.max}.`;
    input.reportValidity();
    input.focus();
    return null;
  }
  const value = parseIntegerLoose(id);
  input.value = String(value);
  setFieldError(input, '', true);
  return value;
}

function validateIpFieldById(id, required, label, showInline = false) {
  const input = document.getElementById(id);
  if (!input) return false;
  const sanitized = sanitizeIpText(input.value.trim());
  if (input.value !== sanitized) input.value = sanitized;

  if (!sanitized) {
    if (!required) {
      setFieldError(input, '', showInline);
      return true;
    }
    setFieldError(input, `${label} is required.`, showInline);
    return false;
  }

  if (!isValidIpAddress(sanitized)) {
    setFieldError(input, `${label} must be a valid IPv4 or IPv6 address.`, showInline);
    return false;
  }
  setFieldError(input, '', showInline);
  return true;
}

function readIpFieldValue(id, required, messageTarget, label) {
  const input = document.getElementById(id);
  if (!input) return null;
  if (!validateIpFieldById(id, required, label, true)) {
    const sanitized = sanitizeIpText(input.value.trim());
    const message = sanitized.length === 0
      ? `${label} is required.`
      : `${label} must be a valid IPv4 or IPv6 address.`;
    input.reportValidity();
    input.focus();
    return null;
  }
  const sanitized = sanitizeIpText(input.value.trim());
  input.value = sanitized;
  setFieldError(input, '', true);
  return sanitized;
}

function validateEndpointInput(showInline = false) {
  const endpointInput = document.getElementById('endpoint');
  if (!endpointInput) return null;
  const sanitized = sanitizeEndpointText(endpointInput.value);
  if (endpointInput.value !== sanitized) endpointInput.value = sanitized;
  const endpoint = parseEndpointUrl(sanitized);
  setFieldError(endpointInput, endpoint ? '' : 'Enter a valid http(s) API endpoint URL.', showInline);
  return endpoint;
}

function validateApiKeyInput(showInline = false) {
  const apikeyInput = document.getElementById('apikey');
  if (!apikeyInput) return false;
  const apikey = (apikeyInput.value || '').trim();
  setFieldError(apikeyInput, apikey ? '' : 'API key is required.', showInline);
  return Boolean(apikey);
}

function hasValidApiContext() {
  const endpoint = validateEndpointInput();
  const hasKey = validateApiKeyInput();
  return Boolean(endpoint && hasKey);
}

function validateGeoFieldById(id, showInline = false) {
  const field = document.getElementById(id);
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
  setValidActionButtonState('refresh', apiValid, true);
  setValidActionButtonState(
    'ban-btn',
    apiValid,
    validateIpFieldById('ban-ip', true, 'Ban IP') && validateIntegerFieldById('ban-duration')
  );
  setValidActionButtonState(
    'unban-btn',
    apiValid,
    validateIpFieldById('unban-ip', true, 'Unban IP')
  );

  if (typeof checkBanDurationsChanged === 'function') {
    checkBanDurationsChanged();
  }
  if (typeof checkMazeConfigChanged === 'function') {
    checkMazeConfigChanged();
  }

  if (typeof checkRobotsConfigChanged === 'function') {
    checkRobotsConfigChanged();
  }
  if (typeof checkGeoConfigChanged === 'function') {
    checkGeoConfigChanged();
  }
  if (typeof checkPowConfigChanged === 'function') {
    checkPowConfigChanged();
  }
  if (typeof checkBotnessConfigChanged === 'function') {
    checkBotnessConfigChanged();
  }
  if (typeof checkCdpConfigChanged === 'function') {
    checkCdpConfigChanged();
  }
  if (typeof checkRateLimitConfigChanged === 'function') {
    checkRateLimitConfigChanged();
  }
  if (typeof checkJsRequiredConfigChanged === 'function') {
    checkJsRequiredConfigChanged();
  }
}

function getAdminContext(messageTarget) {
  const endpointInput = document.getElementById('endpoint');
  const apikeyInput = document.getElementById('apikey');
  const endpoint = validateEndpointInput(true);
  if (!endpoint) {
    endpointInput.reportValidity();
    endpointInput.focus();
    refreshCoreActionButtonsState();
    return null;
  }

  const apikey = (apikeyInput.value || '').trim();
  if (!validateApiKeyInput(true)) {
    apikeyInput.reportValidity();
    apikeyInput.focus();
    refreshCoreActionButtonsState();
    return null;
  }

  endpointInput.value = endpoint;
  setFieldError(endpointInput, '', true);
  setFieldError(apikeyInput, '', true);
  refreshCoreActionButtonsState();
  return { endpoint, apikey };
}

function bindIntegerFieldValidation(id) {
  const input = document.getElementById(id);
  const rules = INTEGER_FIELD_RULES[id];
  if (!input || !rules) return;

  const apply = (showInline = false) => {
    const sanitized = sanitizeIntegerText(input.value);
    if (input.value !== sanitized) input.value = sanitized;
    if (!sanitized) {
      setFieldError(input, `${rules.label} is required.`, showInline);
      return;
    }
    const parsed = Number.parseInt(sanitized, 10);
    if (!Number.isInteger(parsed)) {
      setFieldError(input, `${rules.label} must be a whole number.`, showInline);
      return;
    }
    if (parsed < rules.min || parsed > rules.max) {
      setFieldError(input, `${rules.label} must be between ${rules.min} and ${rules.max}.`, showInline);
      return;
    }
    setFieldError(input, '', showInline);
  };

  input.addEventListener('input', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  input.addEventListener('blur', () => {
    if (!input.value) {
      input.value = String(rules.fallback);
    }
    const parsed = parseIntegerLoose(id);
    if (parsed !== null && parsed < rules.min) input.value = String(rules.min);
    if (parsed !== null && parsed > rules.max) input.value = String(rules.max);
    apply(true);
    refreshCoreActionButtonsState();
  });
  apply(false);
}

function bindIpFieldValidation(id, required, label) {
  const input = document.getElementById(id);
  if (!input) return;
  const apply = (showInline = false) => {
    validateIpFieldById(id, required, label, showInline);
  };
  input.addEventListener('input', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  input.addEventListener('blur', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  apply(false);
}

function bindEndpointFieldValidation() {
  const input = document.getElementById('endpoint');
  if (!input) return;
  const apply = (showInline = false) => validateEndpointInput(showInline);
  input.addEventListener('input', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  input.addEventListener('blur', () => {
    const endpoint = validateEndpointInput(true);
    if (endpoint) input.value = endpoint;
    refreshCoreActionButtonsState();
  });
  apply(false);
}

function bindApiKeyFieldValidation() {
  const input = document.getElementById('apikey');
  if (!input) return;
  const apply = (showInline = false) => validateApiKeyInput(showInline);
  input.addEventListener('input', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  input.addEventListener('blur', () => {
    apply(true);
    refreshCoreActionButtonsState();
  });
  apply(false);
}

function initInputValidation() {
  Object.keys(INTEGER_FIELD_RULES).forEach(bindIntegerFieldValidation);
  bindIpFieldValidation('ban-ip', true, 'Ban IP');
  bindIpFieldValidation('unban-ip', true, 'Unban IP');
  bindEndpointFieldValidation();
  bindApiKeyFieldValidation();
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

function adminPageConfigEnabled(config) {
  return parseBoolLike(config && config.admin_page_config_enabled, false);
}

function updateConfigModeUi(config) {
  const enabled = adminPageConfigEnabled(config);
  const subtitle = document.getElementById('config-mode-subtitle');
  if (subtitle) {
    subtitle.innerHTML = enabled
      ? `Admin page configuration enabled. Set the ${envVar('SHUMA_ADMIN_PAGE_CONFIG')} ENV var to <strong>false</strong> to disable.`
      : `Admin page configuration disabled. Set the ${envVar('SHUMA_ADMIN_PAGE_CONFIG')} ENV var to <strong>true</strong> to enable.`;
  }

  document.querySelectorAll('.config-edit-pane').forEach(el => {
    el.classList.toggle('hidden', !enabled);
  });
}

const STATUS_DEFINITIONS = [
  {
    title: 'Fail Mode Policy',
    description: () => (
      `Controlled by ${envVar('SHUMA_KV_STORE_FAIL_OPEN')}. <strong>true</strong> uses fail-open (allow requests when KV is unavailable); ` +
      '<strong>false</strong> uses fail-closed (block requests when KV is unavailable).'
    ),
    status: state => normalizeFailMode(state.failMode).toUpperCase()
  },
  {
    title: 'Test Mode',
    description: () => (
      `When enabled, actions are logged without blocking traffic. It can be set at build time with the ENV var: ${envVar('SHUMA_TEST_MODE')} ` +
      'and it can be toggled on and off in admin.'
    ),
    status: state => boolStatus(state.testMode)
  },
  {
    title: 'Proof-of-Work (PoW)',
    description: state => (
      'PoW adds a lightweight computational puzzle before JS verification to increase bot cost. ' +
      'For a human visitor the work happens invisibly, once, in a couple of milliseconds. For a scraper or botnet, having to repeatedly perform the work accumulates a prohibitive cost. ' +
      `Primary controls are ${envVar('SHUMA_POW_ENABLED')}, ${envVar('SHUMA_POW_DIFFICULTY')}, and ${envVar('SHUMA_POW_TTL_SECONDS')}. ` +
      `Runtime editability is controlled by ${envVar('SHUMA_POW_CONFIG_MUTABLE')} and is currently ${formatMutability(state.powMutable)}.`
    ),
    status: state => boolStatus(state.powEnabled)
  },
  {
    title: 'Challenge',
    description: state => (
      `Step-up to the puzzle challenge is gated by ${envVar('SHUMA_CHALLENGE_RISK_THRESHOLD')} (current: <strong>${state.challengeThreshold}</strong>) ` +
      `and uses ${envVar('SHUMA_CHALLENGE_TRANSFORM_COUNT')} to enable adjustment of the complexity of puzzle served. ` +
      `Runtime threshold mutability is controlled by ${envVar('SHUMA_CHALLENGE_CONFIG_MUTABLE')} / ${envVar('SHUMA_BOTNESS_CONFIG_MUTABLE')} and is currently ${formatMutability(state.challengeMutable || state.botnessMutable)}.`
    ),
    status: state => boolStatus(state.challengeThreshold >= 1)
  },
  {
    title: 'CDP Detection',
    description: () => (
      `Detects browser automation fingerprints. Primary controls: ${envVar('SHUMA_CDP_DETECTION_ENABLED')}, ` +
      `${envVar('SHUMA_CDP_AUTO_BAN')}, and ${envVar('SHUMA_CDP_DETECTION_THRESHOLD')}. ` +
      `Hard checks like <code>webdriver</code> or <code>automation_props</code> are classified as <strong>strong</strong> CDP detections. ` +
      `If hard checks are absent, reports are <strong>medium</strong> when score >= ${envVar('SHUMA_CDP_DETECTION_THRESHOLD')} (or when <code>cdp_timing</code> is present). ` +
      `Without hard checks, <strong>strong</strong> requires score >= threshold + 0.4 and at least two soft checks (<code>cdp_timing</code>, <code>chrome_obj</code>, <code>plugins</code>). ` +
      `Automatic ban applies only when ${envVar('SHUMA_CDP_AUTO_BAN')} is enabled and the final CDP tier is <strong>strong</strong>.`
    ),
    status: state => boolStatus(state.cdpEnabled)
  },
  {
    title: 'Link Maze',
    description: () => (
      'Link Maze serves trap pages to suspicious traffic. ' +
      `Primary controls are ${envVar('SHUMA_MAZE_ENABLED')}, ${envVar('SHUMA_MAZE_AUTO_BAN')}, and ${envVar('SHUMA_MAZE_AUTO_BAN_THRESHOLD')}. ` +
      `Automatic bans will be triggered when Link Maze hits exceed the configured ${envVar('SHUMA_MAZE_AUTO_BAN_THRESHOLD')}.`
    ),
    status: state => boolStatus(state.mazeEnabled)
  },  
  {
    title: 'JS Required',
    description: state => (
      `When ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} is true, requests without a valid <code>js_verified</code> cookie are served a JS verification page ` +
      `to write the cookie. The originally requested page is then reloaded and access is re-evaluated. ` +
      `If ${envVar('SHUMA_POW_ENABLED')} is true, this verification step includes PoW before <code>js_verified</code> is issued. ` +
      `Disable ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} only if you must allow non-JS clients; doing so weakens bot defense and removes PoW from the normal request path. ` +
      `Its botness contribution is weighted separately by ${envVar('SHUMA_BOTNESS_WEIGHT_JS_REQUIRED')} ` +
      `(current weight: <strong>${state.botnessWeights.js_required || 0}</strong>). ` +
      cumulativeBotnessRoutingText(state)
    ),
    status: state => boolStatus(state.jsRequiredEnforced)
  },
  {
    title: 'GEO Fencing',
    description: state => (
      `Uses trusted edge GEO headers only (trusted when ${envVar('SHUMA_FORWARDED_IP_SECRET')} validation succeeds). ` +
      `Scoring countries are set by ${envVar('SHUMA_GEO_RISK_COUNTRIES')} ` +
      `(current count: <strong>${state.geoRiskCount}</strong>). ` +
      `Policy tiers use ${envVar('SHUMA_GEO_ALLOW_COUNTRIES')} (<strong>${state.geoAllowCount}</strong>), ` +
      `${envVar('SHUMA_GEO_CHALLENGE_COUNTRIES')} (<strong>${state.geoChallengeCount}</strong>), ` +
      `${envVar('SHUMA_GEO_MAZE_COUNTRIES')} (<strong>${state.geoMazeCount}</strong>), and ` +
      `${envVar('SHUMA_GEO_BLOCK_COUNTRIES')} (<strong>${state.geoBlockCount}</strong>). ` +
      `Scoring matches contribute via ${envVar('SHUMA_BOTNESS_WEIGHT_GEO_RISK')} ` +
      `(current weight: <strong>${state.botnessWeights.geo_risk || 0}</strong>). ` +
      cumulativeBotnessRoutingText(state)
    ),
    status: state => boolStatus((state.botnessWeights.geo_risk || 0) > 0)
  },
  {
    title: 'Rate Limiting',
    description: state => (
      `Rate pressure is measured against ${envVar('SHUMA_RATE_LIMIT')} (current limit: <strong>${state.rateLimit}</strong>). ` +
      `Medium pressure (>=50%) contributes ${envVar('SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM')} and high pressure (>=80%) ` +
      `contributes ${envVar('SHUMA_BOTNESS_WEIGHT_RATE_HIGH')} (current weights: <strong>${state.botnessWeights.rate_medium || 0}</strong> / ` +
      `<strong>${state.botnessWeights.rate_high || 0}</strong>). ` +
      cumulativeBotnessRoutingText(state)
    ),
    status: state => boolStatus(
      (state.botnessWeights.rate_medium || 0) > 0 || (state.botnessWeights.rate_high || 0) > 0
    )
  }
];

function normalizeFailMode(value) {
  const mode = (value || 'unknown').toString().toLowerCase();
  if (mode === 'open' || mode === 'closed') return mode;
  return 'unknown';
}

function formatMutability(isMutable) {
  return isMutable ? 'EDITABLE' : 'READ_ONLY';
}

function boolStatus(enabled) {
  return enabled ? 'ENABLED' : 'DISABLED';
}

function cumulativeBotnessRoutingText(state) {
  return (
    `This contributes to the cumulative <strong>botness</strong> score used for defense routing decisions ` +
    `(challenge at <strong>${state.challengeThreshold}</strong>, maze at <strong>${state.mazeThreshold}</strong>, ` +
    `and higher-severity controls such as tar pit or immediate IP ban where configured).`
  );
}

function renderStatusItems() {
  const container = document.getElementById('status-items');
  if (!container) return;

  container.innerHTML = STATUS_DEFINITIONS.map(definition => `
    <div class="status-item">
      <h3>${definition.title}</h3>
      <p class="control-desc text-muted">${definition.description(statusPanelState)}</p>
      <div class="status-rows">
        <div class="info-row">
          <span class="info-label text-muted">Status:</span>
          <span class="status-value">${definition.status(statusPanelState)}</span>
        </div>
      </div>
    </div>
  `).join('');
}

// Initialize charts
function initCharts() {
  const ctx1 = document.getElementById('eventTypesChart').getContext('2d');
  eventTypesChart = new Chart(ctx1, {
    type: 'doughnut',
    data: {
      labels: [],
      datasets: [{
        data: []
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      plugins: {
        legend: {
          position: 'bottom'
        }
      }
    }
  });

  const ctx2 = document.getElementById('topIpsChart').getContext('2d');
  topIpsChart = new Chart(ctx2, {
    type: 'bar',
    data: {
      labels: [],
      datasets: [{
        label: 'Events',
        data: []
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      scales: {
        y: {
          beginAtZero: true,
          ticks: {
            stepSize: 1
          }
        }
      },
      plugins: {
        legend: {
          display: false
        }
      }
    }
  });

  const ctx3 = document.getElementById('timeSeriesChart').getContext('2d');
  timeSeriesChart = new Chart(ctx3, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: 'Events',
        data: [],
        fill: true,
        tension: 0.4,
        borderWidth: 0,
        pointRadius: 0,
        pointHoverRadius: 0
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      scales: {
        y: {
          beginAtZero: true,
          ticks: {
            stepSize: 1
          }
        }
      },
      plugins: {
        legend: {
          display: false
        }
      }
    }
  });

  // Setup time range button handlers
  document.querySelectorAll('.time-btn').forEach(btn => {
    btn.addEventListener('click', function() {
      document.querySelectorAll('.time-btn').forEach(b => b.classList.remove('active'));
      this.classList.add('active');
      currentTimeRange = this.dataset.range;
      updateTimeSeriesChart();
    });
  });
}

// Update stat cards
function updateStatCards(analytics, events, bans) {
  document.getElementById('total-bans').textContent = analytics.ban_count || 0;
  document.getElementById('active-bans').textContent = bans.length || 0;
  document.getElementById('total-events').textContent = (events.recent_events || []).length;
  const uniqueIps = typeof events.unique_ips === 'number' ? events.unique_ips : (events.top_ips || []).length;
  document.getElementById('unique-ips').textContent = uniqueIps;
  
  // Update test mode banner and toggle
  const testMode = analytics.test_mode === true;
  const banner = document.getElementById('test-mode-banner');
  const toggle = document.getElementById('test-mode-toggle');
  const status = document.getElementById('test-mode-status');
  
  if (testMode) {
    banner.classList.remove('hidden');
    status.textContent = 'ENABLED (LOGGING ONLY)';
    status.style.color = '#d97706';
  } else {
    banner.classList.add('hidden');
    status.textContent = 'DISABLED (BLOCKING ACTIVE)';
    status.style.color = '#10b981';
  }
  toggle.checked = testMode;

  statusPanelState.testMode = testMode;
  statusPanelState.failMode = normalizeFailMode(analytics.fail_mode);
  renderStatusItems();
}

// Update ban duration fields from config
function updateBanDurations(config) {
  if (config.ban_durations) {
    setBanDurationInputFromSeconds('honeypot', config.ban_durations.honeypot);
    setBanDurationInputFromSeconds('rateLimit', config.ban_durations.rate_limit);
    setBanDurationInputFromSeconds('browser', config.ban_durations.browser);
    setBanDurationInputFromSeconds('admin', config.ban_durations.admin);
    banDurationsSavedState = {
      honeypot: Number.parseInt(config.ban_durations.honeypot, 10) || BAN_DURATION_FIELDS.honeypot.fallback,
      rateLimit: Number.parseInt(config.ban_durations.rate_limit, 10) || BAN_DURATION_FIELDS.rateLimit.fallback,
      browser: Number.parseInt(config.ban_durations.browser, 10) || BAN_DURATION_FIELDS.browser.fallback,
      admin: Number.parseInt(config.ban_durations.admin, 10) || BAN_DURATION_FIELDS.admin.fallback
    };
    const btn = document.getElementById('save-durations-btn');
    if (btn) {
      btn.dataset.saving = 'false';
      btn.disabled = true;
      btn.textContent = 'Save Durations';
    }
  }
}

// Update event types chart
function updateEventTypesChart(eventCounts) {
  const labels = Object.keys(eventCounts);
  const data = Object.values(eventCounts);
  
  eventTypesChart.data.labels = labels;
  eventTypesChart.data.datasets[0].data = data;
  const bg = data.map((_, i) => CHART_PALETTE[i % CHART_PALETTE.length]);
  eventTypesChart.data.datasets[0].backgroundColor = bg;
  eventTypesChart.data.datasets[0].borderColor = bg;
  eventTypesChart.update();
}

// Update top IPs chart
function updateTopIpsChart(topIps) {
  const labels = topIps.map(([ip, _]) => ip);
  const data = topIps.map(([_, count]) => count);
  const barColors = data.map((_, i) => CHART_PALETTE[i % CHART_PALETTE.length]);

  topIpsChart.data.labels = labels;
  topIpsChart.data.datasets[0].data = data;
  topIpsChart.data.datasets[0].backgroundColor = barColors;
  topIpsChart.data.datasets[0].borderColor = barColors;
  topIpsChart.update();
}

// Update time series chart
function updateTimeSeriesChart() {
  const ctx = getAdminContext(document.getElementById('last-updated'));
  if (!ctx) return;
  const { endpoint, apikey } = ctx;

  const hours = currentTimeRange === 'hour' ? 1 :
                currentTimeRange === 'day' ? 24 :
                currentTimeRange === 'week' ? 168 : 720;

  fetch(`${endpoint}/admin/events?hours=${hours}`, {
    headers: { 'Authorization': 'Bearer ' + apikey }
  })
  .then(r => {
    if (!r.ok) throw new Error('Failed to fetch events');
    return r.json();
  })
  .then(data => {
    const now = Date.now();
    let cutoffTime;
    
    switch(currentTimeRange) {
      case 'hour':
        cutoffTime = now - (60 * 60 * 1000);
        break;
      case 'day':
        cutoffTime = now - (24 * 60 * 60 * 1000);
        break;
      case 'week':
        cutoffTime = now - (7 * 24 * 60 * 60 * 1000);
        break;
      case 'month':
        cutoffTime = now - (30 * 24 * 60 * 60 * 1000);
        break;
    }

    // Filter events by time range
    const events = data.recent_events || [];
    const filteredEvents = events.filter(e => {
      const eventTime = e.ts * 1000; // ts is in seconds, convert to milliseconds
      return eventTime >= cutoffTime;
    });

    // Group events by time bucket
    const buckets = {};
    const bucketSize = currentTimeRange === 'hour' ? 300000 : // 5 mins for hour
                       currentTimeRange === 'day' ? 3600000 : // 1 hour for day
                       currentTimeRange === 'week' ? 86400000 : // 1 day for week
                       86400000; // 1 day for month

    // Pre-fill buckets to ensure full time range is shown
    for (let time = cutoffTime; time <= now; time += bucketSize) {
      const bucketKey = Math.floor(time / bucketSize) * bucketSize;
      buckets[bucketKey] = 0;
    }

    // Count events in buckets
    filteredEvents.forEach(event => {
      const eventTime = event.ts * 1000; // ts is in seconds, convert to milliseconds
      const bucketKey = Math.floor(eventTime / bucketSize) * bucketSize;
      buckets[bucketKey] = (buckets[bucketKey] || 0) + 1;
    });

    // Sort by time and prepare chart data
    const sortedBuckets = Object.keys(buckets)
      .map(k => parseInt(k))
      .sort((a, b) => a - b);

    const labels = sortedBuckets.map(time => {
      const date = new Date(time);
      if (currentTimeRange === 'hour') {
        // Hour view: just time
        return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
      } else if (currentTimeRange === 'day') {
        // Day view: date + time
        return date.toLocaleString('en-US', { 
          month: 'short', 
          day: 'numeric', 
          hour: 'numeric', 
          minute: '2-digit' 
        });
      } else {
        // Week/month view: just date
        return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
      }
    });

    const counts = sortedBuckets.map(time => buckets[time]);

    timeSeriesChart.data.labels = labels;
    timeSeriesChart.data.datasets[0].data = counts;
    timeSeriesChart.data.datasets[0].borderColor = 'rgba(0, 0, 0, 0)';
    timeSeriesChart.data.datasets[0].borderWidth = 0;
    timeSeriesChart.data.datasets[0].backgroundColor = CHART_PALETTE[0];
    timeSeriesChart.update();
  })
  .catch(err => console.error('Failed to update time series:', err));
}

// Update bans table
function updateBansTable(bans) {
  const tbody = document.querySelector('#bans-table tbody');
  tbody.innerHTML = '';
  
  if (bans.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No active bans</td></tr>';
    return;
  }
  
  for (const ban of bans) {
    const tr = document.createElement('tr');
    const now = Math.floor(Date.now() / 1000);
    const isExpired = ban.expires < now;
    const bannedAt = ban.banned_at ? new Date(ban.banned_at * 1000).toLocaleString() : '-';
    const expiresAt = new Date(ban.expires * 1000).toLocaleString();
    const signals = (ban.fingerprint && Array.isArray(ban.fingerprint.signals)) ? ban.fingerprint.signals : [];
    const signalBadges = signals.length
      ? signals.map(signal => `<span class="ban-signal-badge">${signal}</span>`).join('')
      : '<span class="text-muted">none</span>';
    const detailsId = `ban-detail-${ban.ip.replace(/[^a-zA-Z0-9]/g, '-')}`;
    
    tr.innerHTML = `
      <td><code>${ban.ip}</code></td>
      <td>${ban.reason || 'unknown'}</td>
      <td>${bannedAt}</td>
      <td class="${isExpired ? 'expired' : ''}">${isExpired ? 'Expired' : expiresAt}</td>
      <td>${signalBadges}</td>
      <td class="ban-action-cell">
        <button class="ban-details-toggle" data-target="${detailsId}">Details</button>
        <button class="unban-quick" data-ip="${ban.ip}">Unban</button>
      </td>
    `;
    tbody.appendChild(tr);

    const detailRow = document.createElement('tr');
    detailRow.id = detailsId;
    detailRow.className = 'ban-detail-row hidden';
    const score = ban.fingerprint && typeof ban.fingerprint.score === 'number' ? ban.fingerprint.score : null;
    const summary = ban.fingerprint && ban.fingerprint.summary ? ban.fingerprint.summary : 'No additional fingerprint details.';
    detailRow.innerHTML = `
      <td colspan="6">
        <div class="ban-detail-content">
          <div><strong>Score:</strong> ${score === null ? 'n/a' : score}</div>
          <div><strong>Summary:</strong> ${summary}</div>
        </div>
      </td>
    `;
    tbody.appendChild(detailRow);
  }

  document.querySelectorAll('.ban-details-toggle').forEach(btn => {
    btn.onclick = function() {
      const target = document.getElementById(this.dataset.target);
      if (!target) return;
      target.classList.toggle('hidden');
      this.textContent = target.classList.contains('hidden') ? 'Details' : 'Hide';
    };
  });
  
  // Add click handlers for quick unban buttons
  document.querySelectorAll('.unban-quick').forEach(btn => {
    btn.onclick = async function() {
      const ip = this.dataset.ip;
      const msg = document.getElementById('admin-msg');
      const ctx = getAdminContext(msg);
      if (!ctx) return;
      const { endpoint, apikey } = ctx;
      
      msg.textContent = `Unbanning ${ip}...`;
      msg.className = 'message info';
      
      try {
        await window.unbanIp(endpoint, apikey, ip);
        msg.textContent = `Unbanned ${ip}`;
        msg.className = 'message success';
        setTimeout(() => document.getElementById('refresh').click(), 500);
      } catch (e) {
        msg.textContent = 'Error: ' + e.message;
        msg.className = 'message error';
      }
    };
  });
}

// Update events table
function updateEventsTable(events) {
  const tbody = document.querySelector('#events tbody');
  tbody.innerHTML = '';
  
  if (!events || events.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No recent events</td></tr>';
    return;
  }
  
  for (const ev of events) {
    const tr = document.createElement('tr');
    const eventClass = ev.event.toLowerCase();
    tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><span class="badge ${eventClass}">${ev.event}</span></td>
      <td><code>${ev.ip || '-'}</code></td>
      <td>${ev.reason || '-'}</td>
      <td>${ev.outcome || '-'}</td>
      <td>${ev.admin || '-'}</td>
    `;
    tbody.appendChild(tr);
  }
}

function extractCdpField(text, key) {
  const match = new RegExp(`${key}=([^\\s]+)`, 'i').exec(text || '');
  return match ? match[1] : '-';
}

function updateCdpEventsTable(events) {
  const tbody = document.querySelector('#cdp-events tbody');
  if (!tbody) return;
  tbody.innerHTML = '';

  const cdpEvents = events || [];

  if (cdpEvents.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No CDP detections or auto-bans in the selected window</td></tr>';
    return;
  }

  for (const ev of cdpEvents) {
    const reason = ev.reason || '';
    const reasonLower = reason.toLowerCase();
    const outcome = ev.outcome || '-';
    const isBan = reasonLower === 'cdp_automation';
    const tierSource = isBan ? outcome : reason;
    const tier = extractCdpField(tierSource, 'tier').toUpperCase();
    const score = extractCdpField(tierSource, 'score');
    const details = isBan
      ? `Auto-ban: ${outcome}`
      : (outcome.toLowerCase().startsWith('checks:') ? outcome.replace(/^checks:/i, 'Checks: ') : outcome);

    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><code>${ev.ip || '-'}</code></td>
      <td><span class="badge ${isBan ? 'ban' : 'challenge'}">${isBan ? 'BAN' : 'DETECTION'}</span></td>
      <td>${tier}</td>
      <td>${score}</td>
      <td>${details}</td>
    `;
    tbody.appendChild(tr);
  }
}

function updateCdpTotals(cdpData) {
  const detections = cdpData?.stats?.total_detections ?? 0;
  const autoBans = cdpData?.stats?.auto_bans ?? 0;

  const detectionsEl = document.getElementById('cdp-total-detections');
  const autoBansEl = document.getElementById('cdp-total-auto-bans');

  if (detectionsEl) {
    detectionsEl.textContent = Number(detections).toLocaleString();
  }
  if (autoBansEl) {
    autoBansEl.textContent = Number(autoBans).toLocaleString();
  }
}

// Update maze stats section
function updateMazeStats(data) {
  document.getElementById('maze-total-hits').textContent = 
    data.total_hits?.toLocaleString() || '0';
  document.getElementById('maze-unique-crawlers').textContent = 
    data.unique_crawlers?.toLocaleString() || '0';
  document.getElementById('maze-auto-bans').textContent = 
    data.maze_auto_bans?.toLocaleString() || '0';
  
  // Update crawler list
  const crawlerList = document.getElementById('maze-crawler-list');
  const crawlers = data.top_crawlers || [];
  
  if (crawlers.length === 0) {
    crawlerList.innerHTML = '<p class="no-data">No crawlers in maze yet</p>';
    return;
  }
  
  crawlerList.innerHTML = crawlers.map(crawler => {
    const isHigh = crawler.hits >= 30;
    return `
      <div class="crawler-item panel panel-border">
        <span class="crawler-ip">${crawler.ip}</span>
        <span class="crawler-hits ${isHigh ? 'high' : ''}">${crawler.hits} pages</span>
      </div>
    `;
  }).join('');
}

// Update maze config controls from loaded config
function updateMazeConfig(config) {
  if (config.maze_enabled !== undefined) {
    document.getElementById('maze-enabled-toggle').checked = config.maze_enabled;
    statusPanelState.mazeEnabled = config.maze_enabled === true;
  }
  if (config.maze_auto_ban !== undefined) {
    document.getElementById('maze-auto-ban-toggle').checked = config.maze_auto_ban;
    statusPanelState.mazeAutoBan = config.maze_auto_ban === true;
  }
  if (config.maze_auto_ban_threshold !== undefined) {
    document.getElementById('maze-threshold').value = config.maze_auto_ban_threshold;
  }
  mazeSavedState = {
    enabled: document.getElementById('maze-enabled-toggle').checked,
    autoBan: document.getElementById('maze-auto-ban-toggle').checked,
    threshold: parseInt(document.getElementById('maze-threshold').value, 10) || 50
  };
  const btn = document.getElementById('save-maze-config');
  if (btn) {
    btn.dataset.saving = 'false';
    btn.disabled = true;
    btn.textContent = 'Save Maze Settings';
  }
  renderStatusItems();
}

function updateGeoConfig(config) {
  const mutable = adminPageConfigEnabled(config);
  const risk = formatCountryCodes(config.geo_risk);
  const allow = formatCountryCodes(config.geo_allow);
  const challenge = formatCountryCodes(config.geo_challenge);
  const maze = formatCountryCodes(config.geo_maze);
  const block = formatCountryCodes(config.geo_block);

  document.getElementById('geo-risk-list').value = risk;
  document.getElementById('geo-allow-list').value = allow;
  document.getElementById('geo-challenge-list').value = challenge;
  document.getElementById('geo-maze-list').value = maze;
  document.getElementById('geo-block-list').value = block;

  geoSavedState = {
    risk: normalizeCountryCodesForCompare(risk),
    allow: normalizeCountryCodesForCompare(allow),
    challenge: normalizeCountryCodesForCompare(challenge),
    maze: normalizeCountryCodesForCompare(maze),
    block: normalizeCountryCodesForCompare(block),
    mutable
  };

  statusPanelState.geoRiskCount = Array.isArray(config.geo_risk) ? config.geo_risk.length : 0;
  statusPanelState.geoAllowCount = Array.isArray(config.geo_allow) ? config.geo_allow.length : 0;
  statusPanelState.geoChallengeCount = Array.isArray(config.geo_challenge) ? config.geo_challenge.length : 0;
  statusPanelState.geoMazeCount = Array.isArray(config.geo_maze) ? config.geo_maze.length : 0;
  statusPanelState.geoBlockCount = Array.isArray(config.geo_block) ? config.geo_block.length : 0;
  renderStatusItems();

  setGeoConfigEditable(mutable);

  const btn = document.getElementById('save-geo-config');
  btn.disabled = true;
  btn.textContent = 'Save GEO Settings';
}

// Update robots.txt config controls from loaded config
// Track saved state for change detection
let robotsSavedState = {
  enabled: true,
  blockTraining: true,
  blockSearch: false,
  allowSearch: false,  // This is the toggle state (inverted from allow_search_engines)
  crawlDelay: 2
};

// Track CDP detection saved state for change detection
let cdpSavedState = {
  enabled: true,
  autoBan: true,
  threshold: 0.6
};

let rateLimitSavedState = {
  value: 80
};

let jsRequiredSavedState = {
  enforced: true
};

let mazeSavedState = {
  enabled: false,
  autoBan: false,
  threshold: 50
};

let banDurationsSavedState = {
  honeypot: 86400,
  rateLimit: 3600,
  browser: 21600,
  admin: 21600
};

// Track PoW saved state for change detection
let powSavedState = {
  difficulty: 15,
  ttl: 90,
  mutable: false
};

// Track botness scoring saved state for change detection
let botnessSavedState = {
  challengeThreshold: 3,
  mazeThreshold: 6,
  weightJsRequired: 1,
  weightGeoRisk: 2,
  weightRateMedium: 1,
  weightRateHigh: 2,
  mutable: false
};

let geoSavedState = {
  risk: '',
  allow: '',
  challenge: '',
  maze: '',
  block: '',
  mutable: false
};

const GEO_FIELD_IDS = [
  'geo-risk-list',
  'geo-allow-list',
  'geo-challenge-list',
  'geo-maze-list',
  'geo-block-list'
];

const ISO_ALPHA2_CODES = new Set([
  'AD', 'AE', 'AF', 'AG', 'AI', 'AL', 'AM', 'AO', 'AQ', 'AR', 'AS', 'AT', 'AU', 'AW', 'AX', 'AZ', 'BA', 'BB', 'BD', 'BE', 'BF', 'BG', 'BH', 'BI', 'BJ', 'BL', 'BM', 'BN', 'BO', 'BQ', 'BR', 'BS', 'BT', 'BV', 'BW', 'BY', 'BZ', 'CA', 'CC', 'CD', 'CF', 'CG', 'CH', 'CI', 'CK', 'CL', 'CM', 'CN', 'CO', 'CR', 'CU', 'CV', 'CW', 'CX', 'CY', 'CZ', 'DE', 'DJ', 'DK', 'DM', 'DO', 'DZ', 'EC', 'EE', 'EG', 'EH', 'ER', 'ES', 'ET', 'FI', 'FJ', 'FK', 'FM', 'FO', 'FR', 'GA', 'GB', 'GD', 'GE', 'GF', 'GG', 'GH', 'GI', 'GL', 'GM', 'GN', 'GP', 'GQ', 'GR', 'GS', 'GT', 'GU', 'GW', 'GY', 'HK', 'HM', 'HN', 'HR', 'HT', 'HU', 'ID', 'IE', 'IL', 'IM', 'IN', 'IO', 'IQ', 'IR', 'IS', 'IT', 'JE', 'JM', 'JO', 'JP', 'KE', 'KG', 'KH', 'KI', 'KM', 'KN', 'KP', 'KR', 'KW', 'KY', 'KZ', 'LA', 'LB', 'LC', 'LI', 'LK', 'LR', 'LS', 'LT', 'LU', 'LV', 'LY', 'MA', 'MC', 'MD', 'ME', 'MF', 'MG', 'MH', 'MK', 'ML', 'MM', 'MN', 'MO', 'MP', 'MQ', 'MR', 'MS', 'MT', 'MU', 'MV', 'MW', 'MX', 'MY', 'MZ', 'NA', 'NC', 'NE', 'NF', 'NG', 'NI', 'NL', 'NO', 'NP', 'NR', 'NU', 'NZ', 'OM', 'PA', 'PE', 'PF', 'PG', 'PH', 'PK', 'PL', 'PM', 'PN', 'PR', 'PS', 'PT', 'PW', 'PY', 'QA', 'RE', 'RO', 'RS', 'RU', 'RW', 'SA', 'SB', 'SC', 'SD', 'SE', 'SG', 'SH', 'SI', 'SJ', 'SK', 'SL', 'SM', 'SN', 'SO', 'SR', 'SS', 'ST', 'SV', 'SX', 'SY', 'SZ', 'TC', 'TD', 'TF', 'TG', 'TH', 'TJ', 'TK', 'TL', 'TM', 'TN', 'TO', 'TR', 'TT', 'TV', 'TW', 'TZ', 'UA', 'UG', 'UM', 'US', 'UY', 'UZ', 'VA', 'VC', 'VE', 'VG', 'VI', 'VN', 'VU', 'WF', 'WS', 'YE', 'YT', 'ZA', 'ZM', 'ZW'
]);

function setGeoConfigEditable(editable) {
  GEO_FIELD_IDS.forEach(id => {
    const field = document.getElementById(id);
    field.disabled = !editable;
    if (!editable) {
      field.blur();
    }
  });
}

function sanitizeGeoTextareaValue(value) {
  return (value || '')
    .replace(/[^a-zA-Z,]/g, '')
    .toUpperCase();
}

function parseCountryCodesStrict(raw) {
  const sanitized = sanitizeGeoTextareaValue(raw);
  if (!sanitized) return [];
  if (!/^[A-Z]{2}(,[A-Z]{2})*$/.test(sanitized)) {
    throw new Error('Use comma-separated 2-letter country codes only (example: GB,US,RU).');
  }

  const values = sanitized.split(',');
  const seen = new Set();
  const parsed = [];
  for (const value of values) {
    if (!ISO_ALPHA2_CODES.has(value)) {
      throw new Error(`Invalid country code: ${value}. Use valid ISO 3166-1 alpha-2 codes.`);
    }
    if (!seen.has(value)) {
      seen.add(value);
      parsed.push(value);
    }
  }
  return parsed;
}

function normalizeCountryCodesForCompare(raw) {
  return (raw || '')
    .split(',')
    .map(value => value.trim())
    .filter(value => value.length > 0)
    .map(value => value.toUpperCase())
    .join(',');
}

function formatCountryCodes(list) {
  if (!Array.isArray(list) || list.length === 0) return '';
  return list.join(',');
}

function updateRobotsConfig(config) {
  // Update toggles from server config
  if (config.robots_enabled !== undefined) {
    document.getElementById('robots-enabled-toggle').checked = config.robots_enabled;
  }
  if (config.robots_block_ai_training !== undefined) {
    document.getElementById('robots-block-training-toggle').checked = config.robots_block_ai_training;
  }
  if (config.robots_block_ai_search !== undefined) {
    document.getElementById('robots-block-search-toggle').checked = config.robots_block_ai_search;
  }
  if (config.robots_allow_search_engines !== undefined) {
    // Invert: toggle ON = restrict (allow=false), toggle OFF = allow (allow=true)
    document.getElementById('robots-allow-search-toggle').checked = !config.robots_allow_search_engines;
  }
  if (config.robots_crawl_delay !== undefined) {
    document.getElementById('robots-crawl-delay').value = config.robots_crawl_delay;
  }
  // Store saved state for change detection (read from DOM after updates)
  robotsSavedState = {
    enabled: document.getElementById('robots-enabled-toggle').checked,
    blockTraining: document.getElementById('robots-block-training-toggle').checked,
    blockSearch: document.getElementById('robots-block-search-toggle').checked,
    allowSearch: document.getElementById('robots-allow-search-toggle').checked,
    crawlDelay: parseInt(document.getElementById('robots-crawl-delay').value) || 2
  };
  // Reset button state
  const btn = document.getElementById('save-robots-config');
  btn.disabled = true;
  btn.textContent = 'Update Policy';
}

// Check if robots config has changed from saved state
function checkRobotsConfigChanged() {
  const apiValid = hasValidApiContext();
  const delayValid = validateIntegerFieldById('robots-crawl-delay');
  const current = {
    enabled: document.getElementById('robots-enabled-toggle').checked,
    blockTraining: document.getElementById('robots-block-training-toggle').checked,
    blockSearch: document.getElementById('robots-block-search-toggle').checked,
    allowSearch: document.getElementById('robots-allow-search-toggle').checked,
    crawlDelay: parseInt(document.getElementById('robots-crawl-delay').value) || 2
  };
  const changed = (
    current.enabled !== robotsSavedState.enabled ||
    current.blockTraining !== robotsSavedState.blockTraining ||
    current.blockSearch !== robotsSavedState.blockSearch ||
    current.allowSearch !== robotsSavedState.allowSearch ||
    current.crawlDelay !== robotsSavedState.crawlDelay
  );
  setDirtySaveButtonState('save-robots-config', changed, apiValid, delayValid);
  const btn = document.getElementById('save-robots-config');
  if (changed) {
    btn.textContent = 'Update Policy';
  }
}

function setButtonState(buttonId, apiValid, fieldsValid, changed, requireChange) {
  const btn = document.getElementById(buttonId);
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

function checkMazeConfigChanged() {
  const currentThreshold = parseIntegerLoose('maze-threshold');
  const fieldsValid = validateIntegerFieldById('maze-threshold');
  const apiValid = hasValidApiContext();
  const changed = fieldsValid && (
    document.getElementById('maze-enabled-toggle').checked !== mazeSavedState.enabled ||
    document.getElementById('maze-auto-ban-toggle').checked !== mazeSavedState.autoBan ||
    currentThreshold !== mazeSavedState.threshold
  );
  setDirtySaveButtonState('save-maze-config', changed, apiValid, fieldsValid);
}

function checkBanDurationsChanged() {
  const honeypot = readBanDurationFromInputs('honeypot');
  const rateLimit = readBanDurationFromInputs('rateLimit');
  const browser = readBanDurationFromInputs('browser');
  const admin = readBanDurationFromInputs('admin');
  const fieldsValid = Boolean(honeypot && rateLimit && browser && admin);
  const apiValid = hasValidApiContext();
  const current = fieldsValid ? {
    honeypot: honeypot.totalSeconds,
    rateLimit: rateLimit.totalSeconds,
    browser: browser.totalSeconds,
    admin: admin.totalSeconds
  } : banDurationsSavedState;
  const changed = fieldsValid && (
    current.honeypot !== banDurationsSavedState.honeypot ||
    current.rateLimit !== banDurationsSavedState.rateLimit ||
    current.browser !== banDurationsSavedState.browser ||
    current.admin !== banDurationsSavedState.admin
  );
  setDirtySaveButtonState('save-durations-btn', changed, apiValid, fieldsValid);
}

// Add change listeners for robots config controls
['robots-enabled-toggle', 'robots-block-training-toggle', 'robots-block-search-toggle', 'robots-allow-search-toggle'].forEach(id => {
  document.getElementById(id).addEventListener('change', checkRobotsConfigChanged);
});
document.getElementById('robots-crawl-delay').addEventListener('input', checkRobotsConfigChanged);
['maze-enabled-toggle', 'maze-auto-ban-toggle'].forEach(id => {
  document.getElementById(id).addEventListener('change', checkMazeConfigChanged);
});

// Save maze configuration
document.getElementById('save-maze-config').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const btn = this;
  
  const mazeEnabled = document.getElementById('maze-enabled-toggle').checked;
  const mazeAutoBan = document.getElementById('maze-auto-ban-toggle').checked;
  const mazeThreshold = readIntegerFieldValue('maze-threshold', msg);
  if (mazeThreshold === null) return;
  
  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        maze_enabled: mazeEnabled,
        maze_auto_ban: mazeAutoBan,
        maze_auto_ban_threshold: mazeThreshold
      })
    });
    
    if (!resp.ok) throw new Error('Failed to save config');

    mazeSavedState = {
      enabled: mazeEnabled,
      autoBan: mazeAutoBan,
      threshold: mazeThreshold
    };
    btn.textContent = 'Saved!';
    setTimeout(() => {
      btn.dataset.saving = 'false';
      btn.textContent = 'Save Maze Settings';
      checkMazeConfigChanged();
    }, 1500);
    msg.textContent = 'Maze settings saved';
    msg.className = 'message success';
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    console.error('Failed to save maze config:', e);
    btn.dataset.saving = 'false';
    btn.textContent = 'Save Maze Settings';
    checkMazeConfigChanged();
  }
};

// Save robots.txt configuration
document.getElementById('save-robots-config').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const btn = this;
  
  const robotsEnabled = document.getElementById('robots-enabled-toggle').checked;
  const blockTraining = document.getElementById('robots-block-training-toggle').checked;
  const blockSearch = document.getElementById('robots-block-search-toggle').checked;
  // Invert: toggle ON = restrict (allow=false), toggle OFF = allow (allow=true)
  const allowSearchEngines = !document.getElementById('robots-allow-search-toggle').checked;
  const crawlDelay = readIntegerFieldValue('robots-crawl-delay', msg);
  if (crawlDelay === null) return;
  
  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        robots_enabled: robotsEnabled,
        robots_block_ai_training: blockTraining,
        robots_block_ai_search: blockSearch,
        robots_allow_search_engines: allowSearchEngines,
        robots_crawl_delay: crawlDelay
      })
    });
    
    if (!resp.ok) throw new Error('Failed to save config');

    btn.textContent = 'Updated!';
    // Update saved state to current values (store toggle states, not server values)
    robotsSavedState = {
      enabled: robotsEnabled,
      blockTraining: blockTraining,
      blockSearch: blockSearch,
      allowSearch: document.getElementById('robots-allow-search-toggle').checked,  // Store toggle state
      crawlDelay: crawlDelay
    };
    // Refresh preview if it's visible
    const preview = document.getElementById('robots-preview');
    if (!preview.classList.contains('hidden')) {
      refreshRobotsPreview();
    }
    setTimeout(() => {
      btn.dataset.saving = 'false';
      btn.textContent = 'Update Policy';
      checkRobotsConfigChanged();
    }, 1500);
  } catch (e) {
    btn.dataset.saving = 'false';
    btn.textContent = 'Error';
    console.error('Failed to save robots config:', e);
    setTimeout(() => {
      btn.textContent = 'Update Policy';
      checkRobotsConfigChanged();
    }, 2000);
  }
};

// Fetch and update robots.txt preview content
async function refreshRobotsPreview() {
  const ctx = getAdminContext(document.getElementById('admin-msg'));
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const previewContent = document.getElementById('robots-preview-content');
  
  try {
    const resp = await fetch(endpoint + '/admin/robots', {
      headers: { 'Authorization': 'Bearer ' + apikey }
    });
    
    if (!resp.ok) throw new Error('Failed to fetch robots preview');
    
    const data = await resp.json();
    previewContent.textContent = data.preview || '# No preview available';
  } catch (e) {
    previewContent.textContent = '# Error loading preview: ' + e.message;
    console.error('Failed to load robots preview:', e);
  }
}

// Toggle robots.txt preview visibility
document.getElementById('preview-robots').onclick = async function() {
  const preview = document.getElementById('robots-preview');
  const btn = this;
  
  if (preview.classList.contains('hidden')) {
    // Show preview
    btn.textContent = 'Loading...';
    btn.disabled = true;
    await refreshRobotsPreview();
    preview.classList.remove('hidden');
    btn.textContent = 'Hide robots.txt';
    btn.disabled = false;
  } else {
    // Hide preview
    preview.classList.add('hidden');
    btn.textContent = 'Show robots.txt';
  }
};

// Update CDP detection config controls from loaded config
function updateCdpConfig(config) {
  if (config.cdp_detection_enabled !== undefined) {
    document.getElementById('cdp-enabled-toggle').checked = config.cdp_detection_enabled;
    statusPanelState.cdpEnabled = config.cdp_detection_enabled === true;
  }
  if (config.cdp_auto_ban !== undefined) {
    document.getElementById('cdp-auto-ban-toggle').checked = config.cdp_auto_ban;
    statusPanelState.cdpAutoBan = config.cdp_auto_ban === true;
  }
  if (config.cdp_detection_threshold !== undefined) {
    document.getElementById('cdp-threshold-slider').value = config.cdp_detection_threshold;
     document.getElementById('cdp-threshold-value').textContent = parseFloat(config.cdp_detection_threshold).toFixed(1);
  }
  // Store saved state for change detection
  cdpSavedState = {
    enabled: document.getElementById('cdp-enabled-toggle').checked,
    autoBan: document.getElementById('cdp-auto-ban-toggle').checked,
    threshold: parseFloat(document.getElementById('cdp-threshold-slider').value)
  };
  // Reset button state
  const btn = document.getElementById('save-cdp-config');
  btn.disabled = true;
  btn.textContent = 'Save CDP Settings';
  renderStatusItems();
}

function updateRateLimitConfig(config) {
  const rateLimit = parseInt(config.rate_limit, 10) || 80;
  const field = document.getElementById('rate-limit-threshold');
  field.value = rateLimit;
  rateLimitSavedState = { value: rateLimit };
  statusPanelState.rateLimit = rateLimit;
  renderStatusItems();

  const btn = document.getElementById('save-rate-limit-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save Rate Limit';
  }
}

function updateJsRequiredConfig(config) {
  const enforced = parseBoolLike(config.js_required_enforced, true);
  const toggle = document.getElementById('js-required-enforced-toggle');
  toggle.checked = enforced;
  jsRequiredSavedState = { enforced };
  statusPanelState.jsRequiredEnforced = enforced;
  renderStatusItems();

  const btn = document.getElementById('save-js-required-config');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Save JS Required';
  }
}

// Update PoW config controls from loaded config
function updatePowConfig(config) {
  const powEnabled = config.pow_enabled === true;
  const powMutable = config.pow_config_mutable === true;
  const difficulty = parseInt(config.pow_difficulty, 10);
  const ttl = parseInt(config.pow_ttl_seconds, 10);

  statusPanelState.powEnabled = powEnabled;
  statusPanelState.powMutable = powMutable;
  renderStatusItems();

  if (!Number.isNaN(difficulty)) {
    document.getElementById('pow-difficulty').value = difficulty;
  }
  if (!Number.isNaN(ttl)) {
    document.getElementById('pow-ttl').value = ttl;
  }

  // Disable inputs when config is immutable
  document.getElementById('pow-difficulty').disabled = !powMutable;
  document.getElementById('pow-ttl').disabled = !powMutable;

  powSavedState = {
    difficulty: parseInt(document.getElementById('pow-difficulty').value, 10) || 15,
    ttl: parseInt(document.getElementById('pow-ttl').value, 10) || 90,
    mutable: powMutable
  };

  const btn = document.getElementById('save-pow-config');
  btn.disabled = !powMutable;
  btn.textContent = 'Save PoW Settings';
}

function updateBotnessSignalDefinitions(signalDefinitions) {
  const scoredSignals = (signalDefinitions && Array.isArray(signalDefinitions.scored_signals))
    ? signalDefinitions.scored_signals
    : [];
  const terminalSignals = (signalDefinitions && Array.isArray(signalDefinitions.terminal_signals))
    ? signalDefinitions.terminal_signals
    : [];

  const scoredTarget = document.getElementById('botness-signal-list');
  const terminalTarget = document.getElementById('botness-terminal-list');

  scoredTarget.innerHTML = scoredSignals.length
    ? scoredSignals.map(signal => `
      <div class="info-row">
        <span class="info-label">${signal.label}</span>
        <span>${signal.weight}</span>
      </div>
    `).join('')
    : '<p class="text-muted">No scored signals</p>';

  terminalTarget.innerHTML = terminalSignals.length
    ? terminalSignals.map(signal => `
      <div class="info-row">
        <span class="info-label">${signal.label}</span>
        <span>${signal.action}</span>
      </div>
    `).join('')
    : '<p class="text-muted">No terminal signals</p>';
}

function updateChallengeConfig(config) {
  const mutable = config.botness_config_mutable === true;
  const challengeMutable = config.challenge_config_mutable === true;
  const challengeThreshold = parseInt(config.challenge_risk_threshold, 10);
  const challengeDefault = parseInt(config.challenge_risk_threshold_default, 10);
  const mazeThreshold = parseInt(config.botness_maze_threshold, 10);
  const mazeDefault = parseInt(config.botness_maze_threshold_default, 10);
  const weights = config.botness_weights || {};

  if (!Number.isNaN(challengeThreshold)) {
    document.getElementById('challenge-threshold').value = challengeThreshold;
  }
  if (!Number.isNaN(mazeThreshold)) {
    document.getElementById('maze-threshold-score').value = mazeThreshold;
  }
  document.getElementById('weight-js-required').value = parseInt(weights.js_required, 10) || 1;
  document.getElementById('weight-geo-risk').value = parseInt(weights.geo_risk, 10) || 2;
  document.getElementById('weight-rate-medium').value = parseInt(weights.rate_medium, 10) || 1;
  document.getElementById('weight-rate-high').value = parseInt(weights.rate_high, 10) || 2;

  document.getElementById('botness-config-status').textContent = mutable ? 'EDITABLE' : 'READ ONLY';
  document.getElementById('challenge-default').textContent = Number.isNaN(challengeDefault) ? '--' : challengeDefault;
  document.getElementById('maze-threshold-default').textContent = Number.isNaN(mazeDefault) ? '--' : mazeDefault;

  statusPanelState.challengeThreshold = Number.isNaN(challengeThreshold) ? 3 : challengeThreshold;
  statusPanelState.mazeThreshold = Number.isNaN(mazeThreshold) ? 6 : mazeThreshold;
  statusPanelState.challengeMutable = challengeMutable;
  statusPanelState.botnessMutable = mutable;
  statusPanelState.botnessWeights = {
    js_required: parseInt(weights.js_required, 10) || 0,
    geo_risk: parseInt(weights.geo_risk, 10) || 0,
    rate_medium: parseInt(weights.rate_medium, 10) || 0,
    rate_high: parseInt(weights.rate_high, 10) || 0
  };

  const editableFields = [
    'challenge-threshold',
    'maze-threshold-score',
    'weight-js-required',
    'weight-geo-risk',
    'weight-rate-medium',
    'weight-rate-high'
  ];
  editableFields.forEach(id => {
    document.getElementById(id).disabled = !mutable;
  });

  botnessSavedState = {
    challengeThreshold: parseInt(document.getElementById('challenge-threshold').value, 10) || 3,
    mazeThreshold: parseInt(document.getElementById('maze-threshold-score').value, 10) || 6,
    weightJsRequired: parseInt(document.getElementById('weight-js-required').value, 10) || 1,
    weightGeoRisk: parseInt(document.getElementById('weight-geo-risk').value, 10) || 2,
    weightRateMedium: parseInt(document.getElementById('weight-rate-medium').value, 10) || 1,
    weightRateHigh: parseInt(document.getElementById('weight-rate-high').value, 10) || 2,
    mutable: mutable
  };

  updateBotnessSignalDefinitions(config.botness_signal_definitions);

  const btn = document.getElementById('save-botness-config');
  btn.disabled = !mutable;
  btn.textContent = 'Save Botness Settings';
  renderStatusItems();
}

function checkPowConfigChanged() {
  const apiValid = hasValidApiContext();
  const powFieldsValid = validateIntegerFieldById('pow-difficulty') && validateIntegerFieldById('pow-ttl');
  if (!powSavedState.mutable) {
    const btn = document.getElementById('save-pow-config');
    btn.disabled = true;
    return;
  }
  const current = {
    difficulty: parseInt(document.getElementById('pow-difficulty').value, 10) || 15,
    ttl: parseInt(document.getElementById('pow-ttl').value, 10) || 90
  };
  const changed = current.difficulty !== powSavedState.difficulty || current.ttl !== powSavedState.ttl;
  setDirtySaveButtonState('save-pow-config', changed, apiValid, powFieldsValid);
}

document.getElementById('pow-difficulty').addEventListener('input', checkPowConfigChanged);
document.getElementById('pow-ttl').addEventListener('input', checkPowConfigChanged);

function checkBotnessConfigChanged() {
  const apiValid = hasValidApiContext();
  const fieldsValid =
    validateIntegerFieldById('challenge-threshold') &&
    validateIntegerFieldById('maze-threshold-score') &&
    validateIntegerFieldById('weight-js-required') &&
    validateIntegerFieldById('weight-geo-risk') &&
    validateIntegerFieldById('weight-rate-medium') &&
    validateIntegerFieldById('weight-rate-high');
  if (!botnessSavedState.mutable) {
    const btn = document.getElementById('save-botness-config');
    btn.disabled = true;
    return;
  }
  const current = {
    challengeThreshold: parseInt(document.getElementById('challenge-threshold').value, 10) || 3,
    mazeThreshold: parseInt(document.getElementById('maze-threshold-score').value, 10) || 6,
    weightJsRequired: parseInt(document.getElementById('weight-js-required').value, 10) || 1,
    weightGeoRisk: parseInt(document.getElementById('weight-geo-risk').value, 10) || 2,
    weightRateMedium: parseInt(document.getElementById('weight-rate-medium').value, 10) || 1,
    weightRateHigh: parseInt(document.getElementById('weight-rate-high').value, 10) || 2
  };
  const changed =
    current.challengeThreshold !== botnessSavedState.challengeThreshold ||
    current.mazeThreshold !== botnessSavedState.mazeThreshold ||
    current.weightJsRequired !== botnessSavedState.weightJsRequired ||
    current.weightGeoRisk !== botnessSavedState.weightGeoRisk ||
    current.weightRateMedium !== botnessSavedState.weightRateMedium ||
    current.weightRateHigh !== botnessSavedState.weightRateHigh;
  setDirtySaveButtonState('save-botness-config', changed, apiValid, fieldsValid);
}

[
  'challenge-threshold',
  'maze-threshold-score',
  'weight-js-required',
  'weight-geo-risk',
  'weight-rate-medium',
  'weight-rate-high'
].forEach(id => {
  document.getElementById(id).addEventListener('input', checkBotnessConfigChanged);
});

function checkGeoConfigChanged() {
  const apiValid = hasValidApiContext();
  const geoValid = GEO_FIELD_IDS.every(validateGeoFieldById);
  if (!geoSavedState.mutable) {
    const btn = document.getElementById('save-geo-config');
    btn.disabled = true;
    return;
  }

  const current = {
    risk: normalizeCountryCodesForCompare(document.getElementById('geo-risk-list').value),
    allow: normalizeCountryCodesForCompare(document.getElementById('geo-allow-list').value),
    challenge: normalizeCountryCodesForCompare(document.getElementById('geo-challenge-list').value),
    maze: normalizeCountryCodesForCompare(document.getElementById('geo-maze-list').value),
    block: normalizeCountryCodesForCompare(document.getElementById('geo-block-list').value)
  };
  const changed =
    current.risk !== geoSavedState.risk ||
    current.allow !== geoSavedState.allow ||
    current.challenge !== geoSavedState.challenge ||
    current.maze !== geoSavedState.maze ||
    current.block !== geoSavedState.block;
  setDirtySaveButtonState('save-geo-config', changed, apiValid, geoValid);
}

GEO_FIELD_IDS.forEach(id => {
  const field = document.getElementById(id);
  field.addEventListener('input', () => {
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
  });
  field.addEventListener('blur', () => {
    validateGeoFieldById(id, true);
    checkGeoConfigChanged();
    refreshCoreActionButtonsState();
  });
});

document.getElementById('save-geo-config').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const btn = this;

  if (!geoSavedState.mutable) {
    msg.textContent = 'GEO settings are read-only while SHUMA_ADMIN_PAGE_CONFIG=false.';
    msg.className = 'message warning';
    btn.disabled = true;
    return;
  }

  let geoRisk;
  let geoAllow;
  let geoChallenge;
  let geoMaze;
  let geoBlock;
  try {
    geoRisk = parseCountryCodesStrict(document.getElementById('geo-risk-list').value);
    geoAllow = parseCountryCodesStrict(document.getElementById('geo-allow-list').value);
    geoChallenge = parseCountryCodesStrict(document.getElementById('geo-challenge-list').value);
    geoMaze = parseCountryCodesStrict(document.getElementById('geo-maze-list').value);
    geoBlock = parseCountryCodesStrict(document.getElementById('geo-block-list').value);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    return;
  }

  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        geo_risk: geoRisk,
        geo_allow: geoAllow,
        geo_challenge: geoChallenge,
        geo_maze: geoMaze,
        geo_block: geoBlock
      })
    });
    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(text || 'Failed to save GEO config');
    }
    const data = await resp.json();
    if (data && data.config) {
      updateGeoConfig(data.config);
    } else {
      geoSavedState = {
        risk: geoRisk.join(','),
        allow: geoAllow.join(','),
        challenge: geoChallenge.join(','),
        maze: geoMaze.join(','),
        block: geoBlock.join(','),
        mutable: true
      };
      btn.disabled = true;
    }
    msg.textContent = 'GEO settings saved';
    msg.className = 'message success';
    btn.textContent = 'Save GEO Settings';
    btn.dataset.saving = 'false';
    checkGeoConfigChanged();
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    btn.textContent = 'Save GEO Settings';
    btn.dataset.saving = 'false';
    checkGeoConfigChanged();
  }
};

// Save PoW configuration
document.getElementById('save-pow-config').onclick = async function() {
  const btn = this;
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;

  const powDifficulty = readIntegerFieldValue('pow-difficulty', msg);
  const powTtl = readIntegerFieldValue('pow-ttl', msg);
  if (powDifficulty === null || powTtl === null) return;

  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;

  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        pow_difficulty: powDifficulty,
        pow_ttl_seconds: powTtl
      })
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(text || 'Failed to save PoW config');
    }

    powSavedState = {
      difficulty: powDifficulty,
      ttl: powTtl,
      mutable: true
    };
    msg.textContent = 'PoW settings saved';
    msg.className = 'message success';
    btn.textContent = 'Save PoW Settings';
    btn.dataset.saving = 'false';
    checkPowConfigChanged();
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    btn.textContent = 'Save PoW Settings';
    btn.dataset.saving = 'false';
    checkPowConfigChanged();
  }
};

// Save botness scoring configuration
document.getElementById('save-botness-config').onclick = async function() {
  const btn = this;
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;

  const challengeThreshold = readIntegerFieldValue('challenge-threshold', msg);
  const mazeThreshold = readIntegerFieldValue('maze-threshold-score', msg);
  const weightJsRequired = readIntegerFieldValue('weight-js-required', msg);
  const weightGeoRisk = readIntegerFieldValue('weight-geo-risk', msg);
  const weightRateMedium = readIntegerFieldValue('weight-rate-medium', msg);
  const weightRateHigh = readIntegerFieldValue('weight-rate-high', msg);

  if (
    challengeThreshold === null ||
    mazeThreshold === null ||
    weightJsRequired === null ||
    weightGeoRisk === null ||
    weightRateMedium === null ||
    weightRateHigh === null
  ) {
    return;
  }

  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;

  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        challenge_risk_threshold: challengeThreshold,
        botness_maze_threshold: mazeThreshold,
        botness_weights: {
          js_required: weightJsRequired,
          geo_risk: weightGeoRisk,
          rate_medium: weightRateMedium,
          rate_high: weightRateHigh
        }
      })
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(text || 'Failed to save botness config');
    }

    botnessSavedState = {
      challengeThreshold: challengeThreshold,
      mazeThreshold: mazeThreshold,
      weightJsRequired: weightJsRequired,
      weightGeoRisk: weightGeoRisk,
      weightRateMedium: weightRateMedium,
      weightRateHigh: weightRateHigh,
      mutable: true
    };
    msg.textContent = 'Botness scoring saved';
    msg.className = 'message success';
    btn.textContent = 'Save Botness Settings';
    btn.dataset.saving = 'false';
    checkBotnessConfigChanged();
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    btn.textContent = 'Save Botness Settings';
    btn.dataset.saving = 'false';
    checkBotnessConfigChanged();
  }
};

// Check if CDP config has changed from saved state
function checkCdpConfigChanged() {
  const apiValid = hasValidApiContext();
  const current = {
    enabled: document.getElementById('cdp-enabled-toggle').checked,
    autoBan: document.getElementById('cdp-auto-ban-toggle').checked,
    threshold: parseFloat(document.getElementById('cdp-threshold-slider').value)
  };
  const changed = (
    current.enabled !== cdpSavedState.enabled ||
    current.autoBan !== cdpSavedState.autoBan ||
    current.threshold !== cdpSavedState.threshold
  );
  setDirtySaveButtonState('save-cdp-config', changed, apiValid);
}

function checkRateLimitConfigChanged() {
  const apiValid = hasValidApiContext();
  const valueValid = validateIntegerFieldById('rate-limit-threshold');
  const current = parseIntegerLoose('rate-limit-threshold');
  const changed = current !== null && current !== rateLimitSavedState.value;
  setDirtySaveButtonState('save-rate-limit-config', changed, apiValid, valueValid);
}

document.getElementById('rate-limit-threshold').addEventListener('input', checkRateLimitConfigChanged);

function checkJsRequiredConfigChanged() {
  const apiValid = hasValidApiContext();
  const current = document.getElementById('js-required-enforced-toggle').checked;
  const changed = current !== jsRequiredSavedState.enforced;
  setDirtySaveButtonState('save-js-required-config', changed, apiValid);
}

document.getElementById('js-required-enforced-toggle').addEventListener('change', checkJsRequiredConfigChanged);

// Update threshold display when slider moves
document.getElementById('cdp-threshold-slider').addEventListener('input', function() {
  document.getElementById('cdp-threshold-value').textContent = this.value;
    document.getElementById('cdp-threshold-value').textContent = parseFloat(this.value).toFixed(1);
    checkCdpConfigChanged();
});

// Add change listeners for CDP config controls
['cdp-enabled-toggle', 'cdp-auto-ban-toggle'].forEach(id => {
  document.getElementById(id).addEventListener('change', checkCdpConfigChanged);
});

// Save CDP detection configuration
document.getElementById('save-cdp-config').onclick = async function() {
  const ctx = getAdminContext(document.getElementById('admin-msg'));
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const btn = this;
  
  const cdpEnabled = document.getElementById('cdp-enabled-toggle').checked;
  const cdpAutoBan = document.getElementById('cdp-auto-ban-toggle').checked;
  const cdpThreshold = parseFloat(document.getElementById('cdp-threshold-slider').value);
  
  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        cdp_detection_enabled: cdpEnabled,
        cdp_auto_ban: cdpAutoBan,
        cdp_detection_threshold: cdpThreshold
      })
    });
    
    if (!resp.ok) throw new Error('Failed to save config');
    
    btn.textContent = 'Saved!';
    // Update saved state to current values
    cdpSavedState = {
      enabled: cdpEnabled,
      autoBan: cdpAutoBan,
      threshold: cdpThreshold
    };
    setTimeout(() => {
      btn.dataset.saving = 'false';
      btn.textContent = 'Save CDP Settings';
      checkCdpConfigChanged();
    }, 1500);
  } catch (e) {
    btn.dataset.saving = 'false';
    btn.textContent = 'Error';
    console.error('Failed to save CDP config:', e);
    setTimeout(() => {
      btn.textContent = 'Save CDP Settings';
      checkCdpConfigChanged();
    }, 2000);
  }
};

document.getElementById('save-rate-limit-config').onclick = async function() {
  const btn = this;
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const rateLimit = readIntegerFieldValue('rate-limit-threshold', msg);
  if (rateLimit === null) return;

  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ rate_limit: rateLimit })
    });
    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(text || 'Failed to save rate limit');
    }
    rateLimitSavedState = { value: rateLimit };
    statusPanelState.rateLimit = rateLimit;
    renderStatusItems();
    msg.textContent = 'Rate limit saved';
    msg.className = 'message success';
    btn.textContent = 'Save Rate Limit';
    btn.dataset.saving = 'false';
    checkRateLimitConfigChanged();
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    btn.textContent = 'Save Rate Limit';
    btn.dataset.saving = 'false';
    checkRateLimitConfigChanged();
  }
};

document.getElementById('save-js-required-config').onclick = async function() {
  const btn = this;
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const enforced = document.getElementById('js-required-enforced-toggle').checked;

  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  try {
    const resp = await fetch(endpoint + '/admin/config', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ js_required_enforced: enforced })
    });
    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(text || 'Failed to save JS Required setting');
    }
    jsRequiredSavedState = { enforced };
    statusPanelState.jsRequiredEnforced = enforced;
    renderStatusItems();
    msg.textContent = 'JS Required setting saved';
    msg.className = 'message success';
    btn.textContent = 'Save JS Required';
    btn.dataset.saving = 'false';
    checkJsRequiredConfigChanged();
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    btn.textContent = 'Save JS Required';
    btn.dataset.saving = 'false';
    checkJsRequiredConfigChanged();
  }
};

// Main refresh function
document.getElementById('refresh').onclick = async function() {
  const ctx = getAdminContext(document.getElementById('last-updated'));
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const cdpWindowHours = 24;
  const cdpWindowLimit = 500;
  
  // Show loading state
  document.getElementById('total-bans').textContent = '...';
  document.getElementById('active-bans').textContent = '...';
  document.getElementById('total-events').textContent = '...';
  document.getElementById('unique-ips').textContent = '...';
  const cdpTotalDetections = document.getElementById('cdp-total-detections');
  const cdpTotalAutoBans = document.getElementById('cdp-total-auto-bans');
  if (cdpTotalDetections) cdpTotalDetections.textContent = '...';
  if (cdpTotalAutoBans) cdpTotalAutoBans.textContent = '...';

  try {
    // Fetch all data in parallel
    const [analyticsResp, eventsResp, bansResp, mazeResp, cdpResp, cdpEventsResp] = await Promise.all([
      fetch(endpoint + '/admin/analytics', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/events?hours=24', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/ban', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/maze', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + '/admin/cdp', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      }),
      fetch(endpoint + `/admin/cdp/events?hours=${cdpWindowHours}&limit=${cdpWindowLimit}`, {
        headers: { 'Authorization': 'Bearer ' + apikey }
      })
    ]);

    if (!analyticsResp.ok || !eventsResp.ok || !bansResp.ok || !cdpResp.ok || !cdpEventsResp.ok) {
      throw new Error('Failed to fetch data. Check API key and endpoint.');
    }

    const analytics = await analyticsResp.json();
    const events = await eventsResp.json();
    const bansData = await bansResp.json();
    const mazeData = mazeResp.ok ? await mazeResp.json() : null;
    const cdpData = await cdpResp.json();
    const cdpEventsData = await cdpEventsResp.json();

    // Update all sections
    updateStatCards(analytics, events, bansData.bans || []);
    updateEventTypesChart(events.event_counts || {});
    updateTopIpsChart(events.top_ips || []);
    updateTimeSeriesChart();
    updateBansTable(bansData.bans || []);
    updateEventsTable(events.recent_events || []);
    updateCdpTotals(cdpData);
    updateCdpEventsTable(cdpEventsData.events || []);
    
    // Update maze stats
    if (mazeData) {
      updateMazeStats(mazeData);
    }
    
    // Fetch and update ban durations from config
    try {
      const configResp = await fetch(endpoint + '/admin/config', {
        headers: { 'Authorization': 'Bearer ' + apikey }
      });
      if (configResp.ok) {
        const config = await configResp.json();
        updateConfigModeUi(config);
        updateBanDurations(config);
        updateRateLimitConfig(config);
        updateJsRequiredConfig(config);
        updateMazeConfig(config);
        updateGeoConfig(config);
        updateRobotsConfig(config);
        updateCdpConfig(config);
        updatePowConfig(config);
        updateChallengeConfig(config);
      }
    } catch (e) {
      console.error('Failed to load config:', e);
    }

    refreshCoreActionButtonsState();
    
    // Update last updated time
    document.getElementById('last-updated').textContent = 
      'Last updated: ' + new Date().toLocaleTimeString();
    document.getElementById('last-updated').style.color = '#10b981';
    
  } catch (e) {
    console.error('Dashboard refresh error:', e);
    document.getElementById('last-updated').textContent = 'Error: ' + e.message;
    document.getElementById('last-updated').style.color = '#ef4444';
  }
};

// Admin controls - Ban IP
document.getElementById('ban-btn').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const ip = readIpFieldValue('ban-ip', true, msg, 'Ban IP');
  if (ip === null) return;
  const reason = 'manual_ban';
  const duration = readIntegerFieldValue('ban-duration', msg);
  if (duration === null) return;
  
  msg.textContent = `Banning ${ip}...`;
  msg.className = 'message info';
  
  try {
    await window.banIp(endpoint, apikey, ip, duration);
    msg.textContent = `Banned ${ip} for ${duration}s`;
    msg.className = 'message success';
    document.getElementById('ban-ip').value = '';
    setTimeout(() => document.getElementById('refresh').click(), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Admin controls - Unban IP
document.getElementById('unban-btn').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const ip = readIpFieldValue('unban-ip', true, msg, 'Unban IP');
  if (ip === null) return;
  
  msg.textContent = `Unbanning ${ip}...`;
  msg.className = 'message info';
  
  try {
    await window.unbanIp(endpoint, apikey, ip);
    msg.textContent = `Unbanned ${ip}`;
    msg.className = 'message success';
    document.getElementById('unban-ip').value = '';
    setTimeout(() => document.getElementById('refresh').click(), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Save Ban Durations Handler
document.getElementById('save-durations-btn').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const btn = this;
  
  const ban_durations = {
    honeypot: readBanDurationSeconds('honeypot'),
    rate_limit: readBanDurationSeconds('rateLimit'),
    browser: readBanDurationSeconds('browser'),
    admin: readBanDurationSeconds('admin')
  };

  if (
    ban_durations.honeypot === null ||
    ban_durations.rate_limit === null ||
    ban_durations.browser === null ||
    ban_durations.admin === null
  ) {
    return;
  }
  
  msg.textContent = 'Saving ban durations...';
  msg.className = 'message info';
  btn.textContent = 'Saving...';
  btn.dataset.saving = 'true';
  btn.disabled = true;
  
  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ ban_durations })
    });
    
    if (!resp.ok) {
      throw new Error('Failed to save config');
    }
    
    const data = await resp.json();
    const saved = data && data.config && data.config.ban_durations ? data.config.ban_durations : ban_durations;
    updateBanDurations({ ban_durations: saved });
    msg.textContent = 'Ban durations saved';
    msg.className = 'message success';
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    btn.dataset.saving = 'false';
    btn.textContent = 'Save Durations';
    checkBanDurationsChanged();
  }
};

// Initialize charts and load data on page load
initInputValidation();
initCharts();
renderStatusItems();
document.getElementById('refresh').click();

// Test Mode Toggle Handler
document.getElementById('test-mode-toggle').addEventListener('change', async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) {
    this.checked = !this.checked;
    return;
  }
  const { endpoint, apikey } = ctx;
  const testMode = this.checked;
  
  msg.textContent = `${testMode ? 'Enabling' : 'Disabling'} test mode...`;
  msg.className = 'message info';
  
  try {
    const resp = await fetch(`${endpoint}/admin/config`, {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + apikey,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ test_mode: testMode })
    });
    
    if (!resp.ok) {
      throw new Error('Failed to update config');
    }
    
    const data = await resp.json();
    msg.textContent = `Test mode ${data.config.test_mode ? 'enabled' : 'disabled'}`;
    msg.className = 'message success';
    
    // Refresh dashboard to update banner
    setTimeout(() => document.getElementById('refresh').click(), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
    // Revert toggle on error
    this.checked = !testMode;
  }
});

// Auto-refresh every 30 seconds
setInterval(() => {
  document.getElementById('refresh').click();
}, 30000);
