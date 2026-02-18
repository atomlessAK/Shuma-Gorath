/**
 * @typedef {Object} LegacyConfigDirtyRuntimeOptions
 * @property {(id: string) => any} getById
 * @property {(sectionKey: string) => any} getDraft
 * @property {(sectionKey: string, currentValue?: any) => boolean} isDraftDirty
 * @property {() => boolean} hasValidApiContext
 * @property {(id: string, showInline?: boolean) => boolean} validateIntegerFieldById
 * @property {(id: string) => number | null} parseIntegerLoose
 * @property {(durationKey: string) => { totalSeconds: number } | null} readBanDurationFromInputs
 * @property {(showInline?: boolean) => boolean} validateHoneypotPathsField
 * @property {(id: string, showInline?: boolean) => boolean} validateBrowserRulesField
 * @property {(value: string) => string} normalizeListTextareaForCompare
 * @property {(value: string) => string} normalizeBrowserRulesForCompare
 * @property {(value: string) => string} normalizeEdgeIntegrationMode
 * @property {(buttonId: string, changed: boolean, apiValid: boolean, fieldsValid?: boolean) => void} setDirtySaveButtonState
 */

function defaultGetById() {
  return null;
}

function readValue(getById, id, fallback = '') {
  const node = getById(id);
  if (!node) return fallback;
  return typeof node.value === 'string' ? node.value : fallback;
}

function readChecked(getById, id) {
  const node = getById(id);
  if (!node) return false;
  return node.checked === true;
}

function setButtonLabel(getById, id, label) {
  const button = getById(id);
  if (!button) return;
  button.textContent = label;
}

/**
 * @param {LegacyConfigDirtyRuntimeOptions} options
 */
export function createLegacyConfigDirtyRuntime(options = /** @type {LegacyConfigDirtyRuntimeOptions} */ ({})) {
  const getById = typeof options.getById === 'function' ? options.getById : defaultGetById;
  const getDraft = typeof options.getDraft === 'function' ? options.getDraft : () => null;
  const isDraftDirty = typeof options.isDraftDirty === 'function' ? options.isDraftDirty : () => false;
  const hasValidApiContext =
    typeof options.hasValidApiContext === 'function' ? options.hasValidApiContext : () => false;
  const validateIntegerFieldById =
    typeof options.validateIntegerFieldById === 'function'
      ? options.validateIntegerFieldById
      : () => false;
  const parseIntegerLoose =
    typeof options.parseIntegerLoose === 'function' ? options.parseIntegerLoose : () => null;
  const readBanDurationFromInputs =
    typeof options.readBanDurationFromInputs === 'function'
      ? options.readBanDurationFromInputs
      : () => null;
  const validateHoneypotPathsField =
    typeof options.validateHoneypotPathsField === 'function'
      ? options.validateHoneypotPathsField
      : () => false;
  const validateBrowserRulesField =
    typeof options.validateBrowserRulesField === 'function'
      ? options.validateBrowserRulesField
      : () => false;
  const normalizeListTextareaForCompare =
    typeof options.normalizeListTextareaForCompare === 'function'
      ? options.normalizeListTextareaForCompare
      : (value) => String(value || '').trim();
  const normalizeBrowserRulesForCompare =
    typeof options.normalizeBrowserRulesForCompare === 'function'
      ? options.normalizeBrowserRulesForCompare
      : (value) => String(value || '').trim();
  const normalizeEdgeIntegrationMode =
    typeof options.normalizeEdgeIntegrationMode === 'function'
      ? options.normalizeEdgeIntegrationMode
      : (value) => String(value || '').trim().toLowerCase();
  const setDirtySaveButtonState =
    typeof options.setDirtySaveButtonState === 'function'
      ? options.setDirtySaveButtonState
      : () => {};

  const runDirtySaveCheck = (spec) => {
    if (!spec || typeof spec.compute !== 'function') return;
    const apiValid = hasValidApiContext();
    const result = spec.compute();
    const fieldsValid = result && result.fieldsValid !== false;
    const changed = Boolean(result && result.changed);
    setDirtySaveButtonState(spec.buttonId, changed, apiValid, fieldsValid);
    if (changed && typeof spec.onChanged === 'function') {
      spec.onChanged();
    }
  };

  const dirtyCheckRegistry = Object.freeze({
    robots: {
      buttonId: 'save-robots-config',
      onChanged: () => {
        setButtonLabel(getById, 'save-robots-config', 'Save robots serving');
      },
      compute: () => {
        const delayValid = validateIntegerFieldById('robots-crawl-delay');
        const current = {
          enabled: readChecked(getById, 'robots-enabled-toggle'),
          crawlDelay: parseInt(readValue(getById, 'robots-crawl-delay', '2'), 10) || 2
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
        setButtonLabel(getById, 'save-ai-policy-config', 'Save AI bot policy');
      },
      compute: () => {
        const current = {
          blockTraining: readChecked(getById, 'robots-block-training-toggle'),
          blockSearch: readChecked(getById, 'robots-block-search-toggle'),
          allowSearch: readChecked(getById, 'robots-allow-search-toggle')
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
          changed:
            fieldsValid &&
            isDraftDirty('maze', {
              enabled: readChecked(getById, 'maze-enabled-toggle'),
              autoBan: readChecked(getById, 'maze-auto-ban-toggle'),
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
        const current = fieldsValid
          ? {
              honeypot: honeypot.totalSeconds,
              rateLimit: rateLimit.totalSeconds,
              browser: browser.totalSeconds,
              cdp: cdp.totalSeconds,
              admin: admin.totalSeconds
            }
          : getDraft('banDurations');
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
        const currentEnabled = readChecked(getById, 'honeypot-enabled-toggle');
        const saved = getDraft('honeypot') || { enabled: false, values: '' };
        const current = fieldsValid
          ? normalizeListTextareaForCompare(readValue(getById, 'honeypot-paths', ''))
          : saved.values;
        return {
          fieldsValid,
          changed:
            fieldsValid &&
            (currentEnabled !== Boolean(saved.enabled) || current !== String(saved.values || ''))
        };
      }
    },
    browserPolicy: {
      buttonId: 'save-browser-policy-config',
      compute: () => {
        const blockValid = validateBrowserRulesField('browser-block-rules');
        const whitelistValid = validateBrowserRulesField('browser-whitelist-rules');
        const fieldsValid = blockValid && whitelistValid;
        const currentBlock = normalizeBrowserRulesForCompare(readValue(getById, 'browser-block-rules', ''));
        const currentWhitelist = normalizeBrowserRulesForCompare(
          readValue(getById, 'browser-whitelist-rules', '')
        );
        return {
          fieldsValid,
          changed:
            fieldsValid &&
            isDraftDirty('browserPolicy', {
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
          network: normalizeListTextareaForCompare(readValue(getById, 'network-whitelist', '')),
          path: normalizeListTextareaForCompare(readValue(getById, 'path-whitelist', ''))
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
        const current = parseIntegerLoose('challenge-puzzle-transform-count');
        const saved = getDraft('challengePuzzle') || { enabled: false, count: 6 };
        const enabledChanged = readChecked(getById, 'challenge-puzzle-enabled-toggle') !== Boolean(saved.enabled);
        const countChanged = current !== null && current !== Number(saved.count || 0);
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
          enabled: readChecked(getById, 'pow-enabled-toggle'),
          difficulty: parseInt(readValue(getById, 'pow-difficulty', '15'), 10) || 15,
          ttl: parseInt(readValue(getById, 'pow-ttl', '90'), 10) || 90
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
          challengeThreshold: parseInt(readValue(getById, 'challenge-puzzle-threshold', '3'), 10) || 3,
          mazeThreshold: parseInt(readValue(getById, 'maze-threshold-score', '6'), 10) || 6,
          weightJsRequired: parseInt(readValue(getById, 'weight-js-required', '1'), 10) || 1,
          weightGeoRisk: parseInt(readValue(getById, 'weight-geo-risk', '2'), 10) || 2,
          weightRateMedium: parseInt(readValue(getById, 'weight-rate-medium', '1'), 10) || 1,
          weightRateHigh: parseInt(readValue(getById, 'weight-rate-high', '2'), 10) || 2
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
          enabled: readChecked(getById, 'cdp-enabled-toggle'),
          autoBan: readChecked(getById, 'cdp-auto-ban-toggle'),
          threshold: parseFloat(readValue(getById, 'cdp-threshold-slider', '0'))
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
        const current = normalizeEdgeIntegrationMode(readValue(getById, 'edge-integration-mode-select', 'off'));
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
        return {
          fieldsValid: true,
          changed: isDraftDirty('jsRequired', { enforced: readChecked(getById, 'js-required-enforced-toggle') })
        };
      }
    }
  });

  const checkByKey = (key) => {
    runDirtySaveCheck(dirtyCheckRegistry[key]);
  };

  const runCoreChecks = () => {
    checkByKey('banDurations');
    checkByKey('maze');
    checkByKey('robots');
    checkByKey('aiPolicy');
    checkByKey('honeypot');
    checkByKey('browserPolicy');
    checkByKey('bypassAllowlists');
    checkByKey('pow');
    checkByKey('challengePuzzle');
    checkByKey('botness');
    checkByKey('cdp');
    checkByKey('edgeMode');
    checkByKey('rateLimit');
    checkByKey('jsRequired');
  };

  return {
    checkByKey,
    runCoreChecks,
    checkRobots: () => checkByKey('robots'),
    checkAiPolicy: () => checkByKey('aiPolicy'),
    checkMaze: () => checkByKey('maze'),
    checkBanDurations: () => checkByKey('banDurations'),
    checkHoneypot: () => checkByKey('honeypot'),
    checkBrowserPolicy: () => checkByKey('browserPolicy'),
    checkBypassAllowlists: () => checkByKey('bypassAllowlists'),
    checkChallengePuzzle: () => checkByKey('challengePuzzle'),
    checkPow: () => checkByKey('pow'),
    checkBotness: () => checkByKey('botness'),
    checkCdp: () => checkByKey('cdp'),
    checkEdgeMode: () => checkByKey('edgeMode'),
    checkRateLimit: () => checkByKey('rateLimit'),
    checkJsRequired: () => checkByKey('jsRequired')
  };
}
