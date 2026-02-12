const dashboardCharts = window.ShumaDashboardCharts;
if (!dashboardCharts) {
  throw new Error('Missing dashboard charts module (window.ShumaDashboardCharts)');
}
const statusPanel = window.ShumaDashboardStatus;
if (!statusPanel) {
  throw new Error('Missing dashboard status module (window.ShumaDashboardStatus)');
}
const configControls = window.ShumaDashboardConfigControls;
if (!configControls) {
  throw new Error('Missing dashboard config-controls module (window.ShumaDashboardConfigControls)');
}
const adminSessionModule = window.ShumaDashboardAdminSession;
if (!adminSessionModule) {
  throw new Error('Missing dashboard admin-session module (window.ShumaDashboardAdminSession)');
}

const INTEGER_FIELD_RULES = {
  'ban-duration-days': { min: 0, max: 365, fallback: 0, label: 'Manual ban duration days' },
  'ban-duration-hours': { min: 0, max: 23, fallback: 1, label: 'Manual ban duration hours' },
  'ban-duration-minutes': { min: 0, max: 59, fallback: 0, label: 'Manual ban duration minutes' },
  'robots-crawl-delay': { min: 0, max: 60, fallback: 2, label: 'Crawl delay' },
  'maze-threshold': { min: 5, max: 500, fallback: 50, label: 'Maze threshold' },
  'rate-limit-threshold': { min: 1, max: 1000000, fallback: 80, label: 'Rate limit' },
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
  'dur-admin-days': { min: 0, max: 365, fallback: 0, label: 'Admin Manual Ban days' },
  'dur-admin-hours': { min: 0, max: 23, fallback: 6, label: 'Admin Manual Ban hours' },
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

const IPV4_SEGMENT_PATTERN = /^\d{1,3}$/;
const IPV6_INPUT_PATTERN = /^[0-9a-fA-F:.]+$/;
let adminEndpointContext = null;
let adminSessionController = null;

function sanitizeIntegerText(value) {
  return (value || '').replace(/[^\d]/g, '');
}

function sanitizeIpText(value) {
  return (value || '').replace(/[^0-9a-fA-F:.]/g, '');
}

function sanitizeEndpointText(value) {
  return (value || '').replace(/\s+/g, '').trim();
}

function escapeHtml(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function isLoopbackHostname(hostname) {
  const normalized = String(hostname || '').trim().toLowerCase();
  return (
    normalized === 'localhost' ||
    normalized === '127.0.0.1' ||
    normalized === '::1' ||
    normalized === '[::1]'
  );
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

function durationPartsToSeconds(days, hours, minutes) {
  return (days * 86400) + (hours * 3600) + (minutes * 60);
}

function secondsToDurationParts(totalSeconds, fallbackSeconds) {
  const fallback = Number.parseInt(fallbackSeconds, 10) || 0;
  let seconds = Number.parseInt(totalSeconds, 10);
  if (!Number.isFinite(seconds) || seconds <= 0) seconds = fallback;
  if (seconds < BAN_DURATION_BOUNDS_SECONDS.min) seconds = BAN_DURATION_BOUNDS_SECONDS.min;
  if (seconds > BAN_DURATION_BOUNDS_SECONDS.max) seconds = BAN_DURATION_BOUNDS_SECONDS.max;
  return {
    days: Math.floor(seconds / 86400),
    hours: Math.floor((seconds % 86400) / 3600),
    minutes: Math.floor((seconds % 3600) / 60)
  };
}

function setDurationInputsFromSeconds(group, totalSeconds) {
  if (!group) return;
  const daysInput = document.getElementById(group.daysId);
  const hoursInput = document.getElementById(group.hoursId);
  const minutesInput = document.getElementById(group.minutesId);
  if (!daysInput || !hoursInput || !minutesInput) return;

  const parts = secondsToDurationParts(totalSeconds, group.fallback);
  daysInput.value = String(parts.days);
  hoursInput.value = String(parts.hours);
  minutesInput.value = String(parts.minutes);
}

function setBanDurationInputFromSeconds(durationKey, totalSeconds) {
  const group = BAN_DURATION_FIELDS[durationKey];
  setDurationInputsFromSeconds(group, totalSeconds);
}

function readDurationFromInputs(group, showInline = false) {
  if (!group) return null;

  const daysInput = document.getElementById(group.daysId);
  const hoursInput = document.getElementById(group.hoursId);
  const minutesInput = document.getElementById(group.minutesId);
  if (!daysInput || !hoursInput || !minutesInput) return null;

  const daysValid = validateIntegerFieldById(group.daysId, showInline);
  const hoursValid = validateIntegerFieldById(group.hoursId, showInline);
  const minutesValid = validateIntegerFieldById(group.minutesId, showInline);
  const days = parseIntegerLoose(group.daysId);
  const hours = parseIntegerLoose(group.hoursId);
  const minutes = parseIntegerLoose(group.minutesId);

  if (!daysValid || !hoursValid || !minutesValid || days === null || hours === null || minutes === null) return null;

  const totalSeconds = durationPartsToSeconds(days, hours, minutes);
  if (totalSeconds < BAN_DURATION_BOUNDS_SECONDS.min || totalSeconds > BAN_DURATION_BOUNDS_SECONDS.max) {
    const message = `${group.label} must be between 1 minute and 365 days.`;
    setFieldError(daysInput, message, showInline);
    setFieldError(hoursInput, message, showInline);
    setFieldError(minutesInput, message, showInline);
    return null;
  }

  setFieldError(daysInput, '', showInline);
  setFieldError(hoursInput, '', showInline);
  setFieldError(minutesInput, '', showInline);
  return { days, hours, minutes, totalSeconds };
}

function readBanDurationFromInputs(durationKey, showInline = false) {
  const group = BAN_DURATION_FIELDS[durationKey];
  return readDurationFromInputs(group, showInline);
}

function readBanDurationSeconds(durationKey) {
  const group = BAN_DURATION_FIELDS[durationKey];
  if (!group) return null;
  const result = readDurationFromInputs(group, true);
  if (result) return result.totalSeconds;

  const daysInput = document.getElementById(group.daysId);
  const hoursInput = document.getElementById(group.hoursId);
  const minutesInput = document.getElementById(group.minutesId);
  if (daysInput && !daysInput.checkValidity()) {
    daysInput.reportValidity();
    daysInput.focus();
    return null;
  }
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
  if (daysInput) {
    daysInput.reportValidity();
    daysInput.focus();
  }
  return null;
}

function readManualBanDurationSeconds(showInline = false) {
  const result = readDurationFromInputs(MANUAL_BAN_DURATION_FIELD, showInline);
  if (result) return result.totalSeconds;

  const daysInput = document.getElementById(MANUAL_BAN_DURATION_FIELD.daysId);
  const hoursInput = document.getElementById(MANUAL_BAN_DURATION_FIELD.hoursId);
  const minutesInput = document.getElementById(MANUAL_BAN_DURATION_FIELD.minutesId);
  if (daysInput && !daysInput.checkValidity()) {
    daysInput.reportValidity();
    daysInput.focus();
    return null;
  }
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
  if (daysInput) {
    daysInput.reportValidity();
    daysInput.focus();
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

function resolveAdminApiEndpoint() {
  if (adminEndpointContext) return adminEndpointContext;

  const origin = window.location.origin || `${window.location.protocol}//${window.location.host}`;
  let endpoint = parseEndpointUrl(origin) || origin;

  // Local diagnostics only: allow ?api_endpoint=http://127.0.0.1:3000 override on loopback dashboards.
  if (isLoopbackHostname(window.location.hostname)) {
    const params = new URLSearchParams(window.location.search || '');
    const override = sanitizeEndpointText(params.get('api_endpoint') || '');
    if (override) {
      const parsed = parseEndpointUrl(override);
      if (parsed) {
        try {
          const parsedUrl = new URL(parsed);
          if (isLoopbackHostname(parsedUrl.hostname)) {
            endpoint = parsed;
          }
        } catch (_e) {}
      }
    }
  }

  adminEndpointContext = { endpoint };
  return adminEndpointContext;
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

function hasValidApiContext() {
  return adminSessionController ? adminSessionController.hasValidApiContext() : false;
}

function redirectToLogin() {
  const next = encodeURIComponent(window.location.pathname + window.location.search);
  window.location.replace(`/dashboard/login.html?next=${next}`);
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
  const logoutBtn = document.getElementById('logout-btn');
  if (logoutBtn) {
    logoutBtn.disabled = !apiValid;
  }
  setValidActionButtonState(
    'ban-btn',
    apiValid,
    validateIpFieldById('ban-ip', true, 'Ban IP') && Boolean(readDurationFromInputs(MANUAL_BAN_DURATION_FIELD))
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
  if (!adminSessionController) return null;
  return adminSessionController.getAdminContext(messageTarget);
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

function initInputValidation() {
  Object.keys(INTEGER_FIELD_RULES).forEach(bindIntegerFieldValidation);
  bindIpFieldValidation('ban-ip', true, 'Ban IP');
  bindIpFieldValidation('unban-ip', true, 'Unban IP');
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

function adminConfigWriteEnabled(config) {
  return parseBoolLike(config && config.admin_config_write_enabled, false);
}

function updateConfigModeUi(config) {
  const writeEnabled = adminConfigWriteEnabled(config);
  statusPanel.update({
    httpsEnforced: parseBoolLike(config && config.https_enforced, false),
    forwardedHeaderTrustConfigured: parseBoolLike(
      config && config.forwarded_header_trust_configured,
      false
    )
  });
  const subtitle = document.getElementById('config-mode-subtitle');
  if (subtitle) {
    if (writeEnabled) {
      subtitle.innerHTML =
        `Admin page configuration enabled. Saved changes persist across builds. Set ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} to <strong>false</strong> to disable.`;
    } else {
      subtitle.innerHTML =
        `Admin page configuration disabled. Set ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} to <strong>true</strong> to enable.`;
    }
  }

  document.querySelectorAll('.config-edit-pane').forEach(el => {
    el.classList.toggle('hidden', !writeEnabled);
  });
  statusPanel.render();
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

  statusPanel.update({
    testMode,
    failMode: statusPanel.normalizeFailMode(analytics.fail_mode)
  });
  statusPanel.render();
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
    const safeIp = escapeHtml(ban.ip || '-');
    const safeReason = escapeHtml(ban.reason || 'unknown');
    const signals = (ban.fingerprint && Array.isArray(ban.fingerprint.signals)) ? ban.fingerprint.signals : [];
    const signalBadges = signals.length
      ? signals.map(signal => `<span class="ban-signal-badge">${escapeHtml(signal)}</span>`).join('')
      : '<span class="text-muted">none</span>';
    const detailsId = `ban-detail-${String(ban.ip || 'unknown').replace(/[^a-zA-Z0-9]/g, '-')}`;
    
    tr.innerHTML = `
      <td><code>${safeIp}</code></td>
      <td>${safeReason}</td>
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
    const safeSummary = escapeHtml(summary);
    detailRow.innerHTML = `
      <td colspan="6">
        <div class="ban-detail-content">
          <div><strong>Score:</strong> ${score === null ? 'n/a' : score}</div>
          <div><strong>Summary:</strong> ${safeSummary}</div>
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
        setTimeout(() => refreshDashboard(), 500);
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
    const eventClass = String(ev.event || '').toLowerCase().replace(/[^a-z_]/g, '');
    const safeEvent = escapeHtml(ev.event || '-');
    const safeIp = escapeHtml(ev.ip || '-');
    const safeReason = escapeHtml(ev.reason || '-');
    const safeOutcome = escapeHtml(ev.outcome || '-');
    const safeAdmin = escapeHtml(ev.admin || '-');
    tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><span class="badge ${eventClass}">${safeEvent}</span></td>
      <td><code>${safeIp}</code></td>
      <td>${safeReason}</td>
      <td>${safeOutcome}</td>
      <td>${safeAdmin}</td>
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
    const safeIp = escapeHtml(ev.ip || '-');
    const safeTier = escapeHtml(tier);
    const safeScore = escapeHtml(score);
    const safeDetails = escapeHtml(details);
    tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><code>${safeIp}</code></td>
      <td><span class="badge ${isBan ? 'ban' : 'challenge'}">${isBan ? 'BAN' : 'DETECTION'}</span></td>
      <td>${safeTier}</td>
      <td>${safeScore}</td>
      <td>${safeDetails}</td>
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
  const statusPatch = {};
  if (config.maze_enabled !== undefined) {
    document.getElementById('maze-enabled-toggle').checked = config.maze_enabled;
    statusPatch.mazeEnabled = config.maze_enabled === true;
  }
  if (config.maze_auto_ban !== undefined) {
    document.getElementById('maze-auto-ban-toggle').checked = config.maze_auto_ban;
    statusPatch.mazeAutoBan = config.maze_auto_ban === true;
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
  statusPanel.update(statusPatch);
  statusPanel.render();
}

function updateGeoConfig(config) {
  const mutable = adminConfigWriteEnabled(config);
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

  statusPanel.update({
    geoRiskCount: Array.isArray(config.geo_risk) ? config.geo_risk.length : 0,
    geoAllowCount: Array.isArray(config.geo_allow) ? config.geo_allow.length : 0,
    geoChallengeCount: Array.isArray(config.geo_challenge) ? config.geo_challenge.length : 0,
    geoMazeCount: Array.isArray(config.geo_maze) ? config.geo_maze.length : 0,
    geoBlockCount: Array.isArray(config.geo_block) ? config.geo_block.length : 0
  });
  statusPanel.render();

  setGeoConfigEditable(mutable);

  const scoringBtn = document.getElementById('save-geo-scoring-config');
  if (scoringBtn) {
    scoringBtn.disabled = true;
    scoringBtn.textContent = 'Save GEO Scoring';
  }

  const routingBtn = document.getElementById('save-geo-routing-config');
  if (routingBtn) {
    routingBtn.disabled = true;
    routingBtn.textContent = 'Save GEO Routing';
  }
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

const GEO_SCORING_FIELD_IDS = ['geo-risk-list'];
const GEO_ROUTING_FIELD_IDS = [
  'geo-allow-list',
  'geo-challenge-list',
  'geo-maze-list',
  'geo-block-list'
];
const GEO_FIELD_IDS = [...GEO_SCORING_FIELD_IDS, ...GEO_ROUTING_FIELD_IDS];

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
  const statusPatch = {};
  if (config.cdp_detection_enabled !== undefined) {
    document.getElementById('cdp-enabled-toggle').checked = config.cdp_detection_enabled;
    statusPatch.cdpEnabled = config.cdp_detection_enabled === true;
  }
  if (config.cdp_auto_ban !== undefined) {
    document.getElementById('cdp-auto-ban-toggle').checked = config.cdp_auto_ban;
    statusPatch.cdpAutoBan = config.cdp_auto_ban === true;
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
  statusPanel.update(statusPatch);
  statusPanel.render();
}

function updateRateLimitConfig(config) {
  const rateLimit = parseInt(config.rate_limit, 10) || 80;
  const field = document.getElementById('rate-limit-threshold');
  field.value = rateLimit;
  rateLimitSavedState = { value: rateLimit };
  statusPanel.update({ rateLimit });
  statusPanel.render();

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
  statusPanel.update({ jsRequiredEnforced: enforced });
  statusPanel.render();

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

  statusPanel.update({
    powEnabled,
    powMutable
  });
  statusPanel.render();

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

  statusPanel.update({
    challengeThreshold: Number.isNaN(challengeThreshold) ? 3 : challengeThreshold,
    mazeThreshold: Number.isNaN(mazeThreshold) ? 6 : mazeThreshold,
    challengeMutable,
    botnessMutable: mutable,
    botnessWeights: {
      js_required: parseInt(weights.js_required, 10) || 0,
      geo_risk: parseInt(weights.geo_risk, 10) || 0,
      rate_medium: parseInt(weights.rate_medium, 10) || 0,
      rate_high: parseInt(weights.rate_high, 10) || 0
    }
  });

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
  statusPanel.render();
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
  const scoringValid = GEO_SCORING_FIELD_IDS.every(validateGeoFieldById);
  const routingValid = GEO_ROUTING_FIELD_IDS.every(validateGeoFieldById);
  if (!geoSavedState.mutable) {
    const scoringBtn = document.getElementById('save-geo-scoring-config');
    if (scoringBtn) scoringBtn.disabled = true;
    const routingBtn = document.getElementById('save-geo-routing-config');
    if (routingBtn) routingBtn.disabled = true;
    return;
  }

  const current = {
    risk: normalizeCountryCodesForCompare(document.getElementById('geo-risk-list').value),
    allow: normalizeCountryCodesForCompare(document.getElementById('geo-allow-list').value),
    challenge: normalizeCountryCodesForCompare(document.getElementById('geo-challenge-list').value),
    maze: normalizeCountryCodesForCompare(document.getElementById('geo-maze-list').value),
    block: normalizeCountryCodesForCompare(document.getElementById('geo-block-list').value)
  };
  const scoringChanged = current.risk !== geoSavedState.risk;
  const routingChanged =
    current.allow !== geoSavedState.allow ||
    current.challenge !== geoSavedState.challenge ||
    current.maze !== geoSavedState.maze ||
    current.block !== geoSavedState.block;

  setDirtySaveButtonState('save-geo-scoring-config', scoringChanged, apiValid, scoringValid);
  setDirtySaveButtonState('save-geo-routing-config', routingChanged, apiValid, routingValid);
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

// Main refresh function
async function refreshDashboard() {
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

    if (
      analyticsResp.status === 401 ||
      eventsResp.status === 401 ||
      bansResp.status === 401 ||
      mazeResp.status === 401 ||
      cdpResp.status === 401 ||
      cdpEventsResp.status === 401
    ) {
      redirectToLogin();
      return;
    }

    if (!analyticsResp.ok || !eventsResp.ok || !bansResp.ok || !cdpResp.ok || !cdpEventsResp.ok) {
      throw new Error('Failed to fetch dashboard data.');
    }

    const analytics = await analyticsResp.json();
    const events = await eventsResp.json();
    const bansData = await bansResp.json();
    const mazeData = mazeResp.ok ? await mazeResp.json() : null;
    const cdpData = await cdpResp.json();
    const cdpEventsData = await cdpEventsResp.json();

    // Update all sections
    updateStatCards(analytics, events, bansData.bans || []);
    dashboardCharts.updateEventTypesChart(events.event_counts || {});
    dashboardCharts.updateTopIpsChart(events.top_ips || []);
    dashboardCharts.updateTimeSeriesChart();
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
    
    // Update last updated time (full ISO timestamp)
    document.getElementById('last-updated').textContent =
      'updated: ' + new Date().toISOString();
    
  } catch (e) {
    console.error('Dashboard refresh error:', e);
    const msg = document.getElementById('admin-msg');
    if (msg) {
      msg.textContent = 'Refresh failed: ' + e.message;
      msg.className = 'message error';
    }
  }
}

// Admin controls - Ban IP
document.getElementById('ban-btn').onclick = async function() {
  const msg = document.getElementById('admin-msg');
  const ctx = getAdminContext(msg);
  if (!ctx) return;
  const { endpoint, apikey } = ctx;
  const ip = readIpFieldValue('ban-ip', true, msg, 'Ban IP');
  if (ip === null) return;
  const duration = readManualBanDurationSeconds(true);
  if (duration === null) return;
  
  msg.textContent = `Banning ${ip}...`;
  msg.className = 'message info';
  
  try {
    await window.banIp(endpoint, apikey, ip, duration);
    msg.textContent = `Banned ${ip} for ${duration}s`;
    msg.className = 'message success';
    document.getElementById('ban-ip').value = '';
    setTimeout(() => refreshDashboard(), 500);
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
    setTimeout(() => refreshDashboard(), 500);
  } catch (e) {
    msg.textContent = 'Error: ' + e.message;
    msg.className = 'message error';
  }
};

// Initialize charts and load data on page load
adminSessionController = adminSessionModule.create({
  resolveAdminApiEndpoint,
  refreshCoreActionButtonsState,
  redirectToLogin
});
adminSessionController.bindLogoutButton('logout-btn', 'admin-msg');

initInputValidation();
dashboardCharts.init({ getAdminContext });
statusPanel.render();
configControls.bind({
  statusPanel,
  getAdminContext,
  readIntegerFieldValue,
  readBanDurationSeconds,
  parseCountryCodesStrict,
  updateBanDurations,
  updateGeoConfig,
  refreshRobotsPreview,
  refreshDashboard,
  checkMazeConfigChanged,
  checkRobotsConfigChanged,
  checkGeoConfigChanged,
  checkPowConfigChanged,
  checkBotnessConfigChanged,
  checkCdpConfigChanged,
  checkRateLimitConfigChanged,
  checkJsRequiredConfigChanged,
  checkBanDurationsChanged,
  getGeoSavedState: () => geoSavedState,
  setGeoSavedState: (next) => {
    geoSavedState = next;
  },
  setMazeSavedState: (next) => {
    mazeSavedState = next;
  },
  setRobotsSavedState: (next) => {
    robotsSavedState = next;
  },
  setPowSavedState: (next) => {
    powSavedState = next;
  },
  setBotnessSavedState: (next) => {
    botnessSavedState = next;
  },
  setCdpSavedState: (next) => {
    cdpSavedState = next;
  },
  setRateLimitSavedState: (next) => {
    rateLimitSavedState = next;
  },
  setJsRequiredSavedState: (next) => {
    jsRequiredSavedState = next;
  }
});
adminSessionController.restoreAdminSession().then((authenticated) => {
  if (!authenticated) {
    redirectToLogin();
    return;
  }
  refreshDashboard();
});

// Auto-refresh every 30 seconds
setInterval(() => {
  if (hasValidApiContext()) {
    refreshDashboard();
  }
}, 30000);
