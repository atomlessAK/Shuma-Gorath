// @ts-check

import * as formUtils from './config-form-utils.js';
import * as domModule from './core/dom.js';

const domCache = domModule.createCache({ document });
const getById = domCache.byId;

const OPTION_GROUP_KEYS = Object.freeze([
  'readers',
  'parsers',
  'updaters',
  'checks',
  'state',
  'actions',
  'callbacks'
]);

const DRAFT_SETTER_ALIAS = Object.freeze({
  setMazeSavedState: 'maze',
  setHoneypotSavedState: 'honeypot',
  setBrowserPolicySavedState: 'browserPolicy',
  setBypassAllowlistSavedState: 'bypassAllowlists',
  setRobotsSavedState: 'robots',
  setAiPolicySavedState: 'aiPolicy',
  setPowSavedState: 'pow',
  setChallengePuzzleSavedState: 'challengePuzzle',
  setBotnessSavedState: 'botness',
  setCdpSavedState: 'cdp',
  setEdgeIntegrationModeSavedState: 'edgeMode',
  setRateLimitSavedState: 'rateLimit',
  setJsRequiredSavedState: 'jsRequired'
});

const GEO_DRAFT_FALLBACK = Object.freeze({
  risk: '',
  allow: '',
  challenge: '',
  maze: '',
  block: '',
  mutable: false
});

function flattenBindOptions(rawOptions = {}) {
  // Accept grouped option buckets to keep the bind callsite compact while
  // retaining stable flat option names in this module.
  const flattened = { ...rawOptions };
  OPTION_GROUP_KEYS.forEach((groupKey) => {
    const group = rawOptions[groupKey];
    if (!group || typeof group !== 'object') return;
    Object.entries(group).forEach(([key, value]) => {
      if (flattened[key] === undefined) {
        flattened[key] = value;
      }
    });
  });
  return flattened;
}

function normalizeContextOptions(rawOptions = {}) {
  if (rawOptions.domainApi && typeof rawOptions.domainApi === 'object') {
    const merged = { ...rawOptions, ...rawOptions.domainApi };
    delete merged.domainApi;
    return merged;
  }

  if (!rawOptions.context || typeof rawOptions.context !== 'object') return rawOptions;
  const context = rawOptions.context;

  const normalized = {
    statusPanel: context.statusPanel || null,
    apiClient: context.apiClient || null,
    effects:
      context.effects && typeof context.effects === 'object'
        ? context.effects
        : (rawOptions.effects && typeof rawOptions.effects === 'object' ? rawOptions.effects : null)
  };

  const auth = context.auth || {};
  const callbacks = context.callbacks || {};
  const readers = context.readers || {};
  const parsers = context.parsers || {};
  const updaters = context.updaters || {};
  const checks = context.checks || {};
  const actions = context.actions || {};

  if (typeof auth.getAdminContext === 'function') {
    normalized.getAdminContext = auth.getAdminContext;
  }
  if (typeof callbacks.onConfigSaved === 'function') {
    normalized.onConfigSaved = callbacks.onConfigSaved;
  }

  Object.assign(normalized, readers, parsers, updaters, checks, actions);

  const draft = context.draft || {};
  if (typeof draft.get === 'function') {
    normalized.getGeoSavedState = () => draft.get('geo', GEO_DRAFT_FALLBACK);
  }
  if (typeof draft.set === 'function') {
    Object.entries(DRAFT_SETTER_ALIAS).forEach(([setterName, sectionKey]) => {
      normalized[setterName] = (next) => draft.set(sectionKey, next);
    });
    normalized.setGeoSavedState = (next) => draft.set('geo', next);
  }

  return normalized;
}

function bind(rawOptions = {}) {
  const options = flattenBindOptions(normalizeContextOptions(rawOptions));
  const statusPanel = options.statusPanel || null;

  const applyStatusPatch =
    statusPanel && typeof statusPanel.applyPatch === 'function'
      ? statusPanel.applyPatch.bind(statusPanel)
      : (patch) => {
        if (!statusPanel) return;
        statusPanel.update(patch);
        statusPanel.render();
      };

  const apiClient = options.apiClient || null;
  const timerSetTimeout =
    options.effects && typeof options.effects.setTimer === 'function'
      ? options.effects.setTimer
      : window.setTimeout.bind(window);
  const requestImpl =
    options.effects && typeof options.effects.request === 'function'
      ? options.effects.request
      : fetch.bind(globalThis);
  const parseCountryCodesStrict =
    typeof options.parseCountryCodesStrict === 'function'
      ? options.parseCountryCodesStrict
      : formUtils.parseCountryCodesStrict;

  const setAdminMessage = (messageTarget, text, variant = 'info') => {
    if (!messageTarget) return;
    messageTarget.textContent = String(text || '');
    messageTarget.className = `message ${variant}`;
  };

  const beginSaveButton = (button, label = 'Saving...') => {
    if (!button) return;
    button.textContent = label;
    button.dataset.saving = 'true';
    button.disabled = true;
  };

  const endSaveButton = (button, label) => {
    if (!button) return;
    button.dataset.saving = 'false';
    if (label !== undefined) {
      button.textContent = String(label);
    }
  };

  const endSaveButtonAfter = (
    button,
    interimLabel,
    nextLabel,
    delayMs = 1500,
    onReset = null
  ) => {
    if (!button) return;
    if (interimLabel !== undefined) {
      button.textContent = String(interimLabel);
    }
    timerSetTimeout(() => {
      endSaveButton(button, nextLabel);
      if (typeof onReset === 'function') onReset();
    }, delayMs);
  };

  function parseList(raw) {
    if (typeof options.parseListTextarea === 'function') {
      return options.parseListTextarea(raw);
    }
    return formUtils.parseListTextarea(raw);
  }

  function normalizeList(raw) {
    if (typeof options.normalizeListTextareaForCompare === 'function') {
      return options.normalizeListTextareaForCompare(raw);
    }
    return formUtils.normalizeListTextareaForCompare(raw);
  }

  function parseHoneypotPaths(raw) {
    if (typeof options.parseHoneypotPathsTextarea === 'function') {
      return options.parseHoneypotPathsTextarea(raw);
    }
    return formUtils.parseHoneypotPathsTextarea(raw);
  }

  function parseBrowserRules(raw) {
    if (typeof options.parseBrowserRulesTextarea === 'function') {
      return options.parseBrowserRulesTextarea(raw);
    }
    return formUtils.parseBrowserRulesTextarea(raw);
  }

  function normalizeBrowserRules(raw) {
    if (typeof options.normalizeBrowserRulesForCompare === 'function') {
      return options.normalizeBrowserRulesForCompare(raw);
    }
    return formUtils.normalizeBrowserRulesForCompare(raw);
  }

  async function saveConfigPatch(messageTarget, patch) {
    let result;
    if (apiClient && typeof apiClient.updateConfig === 'function') {
      result = await apiClient.updateConfig(patch);
    } else {
      const ctx = options.getAdminContext(messageTarget || null);
      if (!ctx) {
        throw new Error('Missing admin API context');
      }
      const { endpoint, apikey, sessionAuth, csrfToken } = ctx;
      const headers = new Headers({
        'Content-Type': 'application/json',
        Accept: 'application/json'
      });
      const apiKeyValue = String(apikey || '').trim();
      if (apiKeyValue) {
        headers.set('Authorization', `Bearer ${apiKeyValue}`);
      }
      if (sessionAuth === true && String(csrfToken || '').trim()) {
        headers.set('X-Shuma-CSRF', String(csrfToken).trim());
      }
      const resp = await requestImpl(`${endpoint}/admin/config`, {
        method: 'POST',
        headers,
        credentials: sessionAuth === true ? 'same-origin' : undefined,
        body: JSON.stringify(patch)
      });
      if (!resp.ok) {
        const text = await resp.text();
        throw new Error(text || 'Failed to save config');
      }
      result = await resp.json();
    }

    if (result && result.config && typeof result.config === 'object') {
      applyStatusPatch({ configSnapshot: result.config });
    }
    if (typeof options.onConfigSaved === 'function') {
      options.onConfigSaved(patch, result);
    }
    return result;
  }

  const saveRegistry = Object.freeze([
    {
      buttonId: 'save-maze-config',
      prepare(messageTarget) {
        const threshold = options.readIntegerFieldValue('maze-threshold', messageTarget);
        if (threshold === null) return null;
        return {
          enabled: getById('maze-enabled-toggle').checked,
          autoBan: getById('maze-auto-ban-toggle').checked,
          threshold
        };
      },
      buildPatch: ({ enabled, autoBan, threshold }) => ({
        maze_enabled: enabled,
        maze_auto_ban: autoBan,
        maze_auto_ban_threshold: threshold
      }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateMazeConfig === 'function') {
          options.updateMazeConfig(data.config);
        } else if (typeof options.setMazeSavedState === 'function') {
          options.setMazeSavedState(prepared);
        }
        setAdminMessage(messageTarget, 'Maze settings saved', 'success');
        endSaveButtonAfter(button, 'Saved!', 'Save Maze Settings', 1500, () => {
          if (typeof options.checkMazeConfigChanged === 'function') {
            options.checkMazeConfigChanged();
          }
        });
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Maze Settings');
        if (typeof options.checkMazeConfigChanged === 'function') {
          options.checkMazeConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-robots-config',
      prepare(messageTarget) {
        const crawlDelay = options.readIntegerFieldValue('robots-crawl-delay', messageTarget);
        if (crawlDelay === null) return null;
        return {
          enabled: getById('robots-enabled-toggle').checked,
          crawlDelay
        };
      },
      buildPatch: ({ enabled, crawlDelay }) => ({
        robots_enabled: enabled,
        robots_crawl_delay: crawlDelay
      }),
      async onSuccess({ button, data, prepared }) {
        if (data && data.config && typeof options.updateRobotsConfig === 'function') {
          options.updateRobotsConfig(data.config);
        } else if (typeof options.setRobotsSavedState === 'function') {
          options.setRobotsSavedState(prepared);
        }

        const preview = getById('robots-preview');
        if (
          preview &&
          !preview.classList.contains('hidden') &&
          typeof options.refreshRobotsPreview === 'function'
        ) {
          await options.refreshRobotsPreview();
        }

        endSaveButtonAfter(button, 'Updated!', 'Save robots serving', 1500, () => {
          if (typeof options.checkRobotsConfigChanged === 'function') {
            options.checkRobotsConfigChanged();
          }
        });
      },
      onError({ button }) {
        endSaveButton(button, 'Error');
        endSaveButtonAfter(button, undefined, 'Save robots serving', 2000, () => {
          if (typeof options.checkRobotsConfigChanged === 'function') {
            options.checkRobotsConfigChanged();
          }
        });
      }
    },
    {
      buttonId: 'save-ai-policy-config',
      prepare() {
        const allowSearchToggle = getById('robots-allow-search-toggle');
        return {
          blockTraining: getById('robots-block-training-toggle').checked,
          blockSearch: getById('robots-block-search-toggle').checked,
          allowSearch: allowSearchToggle ? allowSearchToggle.checked : false
        };
      },
      buildPatch: ({ blockTraining, blockSearch, allowSearch }) => ({
        ai_policy_block_training: blockTraining,
        ai_policy_block_search: blockSearch,
        ai_policy_allow_search_engines: !allowSearch
      }),
      async onSuccess({ button, data, prepared }) {
        if (data && data.config && typeof options.updateRobotsConfig === 'function') {
          options.updateRobotsConfig(data.config);
        } else if (typeof options.setAiPolicySavedState === 'function') {
          options.setAiPolicySavedState(prepared);
        }

        const preview = getById('robots-preview');
        if (
          preview &&
          !preview.classList.contains('hidden') &&
          typeof options.refreshRobotsPreview === 'function'
        ) {
          await options.refreshRobotsPreview();
        }

        endSaveButtonAfter(button, 'Saved!', 'Save AI bot policy', 1500, () => {
          if (typeof options.checkAiPolicyConfigChanged === 'function') {
            options.checkAiPolicyConfigChanged();
          }
        });
      },
      onError({ button }) {
        endSaveButton(button, 'Error');
        endSaveButtonAfter(button, undefined, 'Save AI bot policy', 2000, () => {
          if (typeof options.checkAiPolicyConfigChanged === 'function') {
            options.checkAiPolicyConfigChanged();
          }
        });
      }
    },
    {
      buttonId: 'save-geo-scoring-config',
      prepare(messageTarget, button) {
        const geoState =
          typeof options.getGeoSavedState === 'function'
            ? options.getGeoSavedState()
            : GEO_DRAFT_FALLBACK;
        if (!geoState.mutable) {
          setAdminMessage(
            messageTarget,
            'GEO settings are read-only while SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false.',
            'warning'
          );
          if (button) button.disabled = true;
          return null;
        }

        try {
          const risk = parseCountryCodesStrict(getById('geo-risk-list').value);
          return { risk };
        } catch (error) {
          setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
          return null;
        }
      },
      buildPatch: ({ risk }) => ({ geo_risk: risk }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateGeoConfig === 'function') {
          options.updateGeoConfig(data.config);
        } else if (typeof options.setGeoSavedState === 'function') {
          const next = {
            ...(typeof options.getGeoSavedState === 'function'
              ? options.getGeoSavedState()
              : GEO_DRAFT_FALLBACK),
            risk: prepared.risk.join(','),
            mutable: true
          };
          options.setGeoSavedState(next);
        }
        setAdminMessage(messageTarget, 'GEO scoring saved', 'success');
        endSaveButton(button, 'Save GEO Scoring');
        if (typeof options.checkGeoConfigChanged === 'function') {
          options.checkGeoConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save GEO Scoring');
        if (typeof options.checkGeoConfigChanged === 'function') {
          options.checkGeoConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-geo-routing-config',
      prepare(messageTarget, button) {
        const geoState =
          typeof options.getGeoSavedState === 'function'
            ? options.getGeoSavedState()
            : GEO_DRAFT_FALLBACK;
        if (!geoState.mutable) {
          setAdminMessage(
            messageTarget,
            'GEO settings are read-only while SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false.',
            'warning'
          );
          if (button) button.disabled = true;
          return null;
        }

        try {
          return {
            allow: parseCountryCodesStrict(getById('geo-allow-list').value),
            challenge: parseCountryCodesStrict(getById('geo-challenge-list').value),
            maze: parseCountryCodesStrict(getById('geo-maze-list').value),
            block: parseCountryCodesStrict(getById('geo-block-list').value)
          };
        } catch (error) {
          setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
          return null;
        }
      },
      buildPatch: ({ allow, challenge, maze, block }) => ({
        geo_allow: allow,
        geo_challenge: challenge,
        geo_maze: maze,
        geo_block: block
      }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateGeoConfig === 'function') {
          options.updateGeoConfig(data.config);
        } else if (typeof options.setGeoSavedState === 'function') {
          const next = {
            ...(typeof options.getGeoSavedState === 'function'
              ? options.getGeoSavedState()
              : GEO_DRAFT_FALLBACK),
            allow: prepared.allow.join(','),
            challenge: prepared.challenge.join(','),
            maze: prepared.maze.join(','),
            block: prepared.block.join(','),
            mutable: true
          };
          options.setGeoSavedState(next);
        }
        setAdminMessage(messageTarget, 'GEO routing saved', 'success');
        endSaveButton(button, 'Save GEO Routing');
        if (typeof options.checkGeoConfigChanged === 'function') {
          options.checkGeoConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save GEO Routing');
        if (typeof options.checkGeoConfigChanged === 'function') {
          options.checkGeoConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-honeypot-config',
      prepare(messageTarget) {
        const enabledToggle = getById('honeypot-enabled-toggle');
        const honeypotEnabled = enabledToggle ? enabledToggle.checked : true;
        const field = getById('honeypot-paths');
        const raw = field ? field.value : '';
        try {
          const honeypots = parseHoneypotPaths(raw);
          return {
            honeypotEnabled,
            honeypots,
            normalizedValues: normalizeList(raw)
          };
        } catch (error) {
          setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
          return null;
        }
      },
      buildPatch: ({ honeypotEnabled, honeypots }) => ({
        honeypot_enabled: honeypotEnabled,
        honeypots
      }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateHoneypotConfig === 'function') {
          options.updateHoneypotConfig(data.config);
        } else if (typeof options.setHoneypotSavedState === 'function') {
          options.setHoneypotSavedState({
            enabled: prepared.honeypotEnabled,
            values: prepared.normalizedValues
          });
        }
        setAdminMessage(messageTarget, 'Honeypot paths saved', 'success');
        endSaveButton(button, 'Save Honeypots');
        if (typeof options.checkHoneypotConfigChanged === 'function') {
          options.checkHoneypotConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Honeypots');
        if (typeof options.checkHoneypotConfigChanged === 'function') {
          options.checkHoneypotConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-browser-policy-config',
      prepare(messageTarget) {
        const blockField = getById('browser-block-rules');
        const whitelistField = getById('browser-whitelist-rules');
        const blockRaw = blockField ? blockField.value : '';
        const whitelistRaw = whitelistField ? whitelistField.value : '';
        try {
          const browserBlock = parseBrowserRules(blockRaw);
          const browserWhitelist = parseBrowserRules(whitelistRaw);
          return {
            browserBlock,
            browserWhitelist,
            normalizedBlock: normalizeBrowserRules(blockRaw),
            normalizedWhitelist: normalizeBrowserRules(whitelistRaw)
          };
        } catch (error) {
          setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
          return null;
        }
      },
      buildPatch: ({ browserBlock, browserWhitelist }) => ({
        browser_block: browserBlock,
        browser_whitelist: browserWhitelist
      }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateBrowserPolicyConfig === 'function') {
          options.updateBrowserPolicyConfig(data.config);
        } else if (typeof options.setBrowserPolicySavedState === 'function') {
          options.setBrowserPolicySavedState({
            block: prepared.normalizedBlock,
            whitelist: prepared.normalizedWhitelist
          });
        }
        setAdminMessage(messageTarget, 'Browser policy saved', 'success');
        endSaveButton(button, 'Save Browser Policy');
        if (typeof options.checkBrowserPolicyConfigChanged === 'function') {
          options.checkBrowserPolicyConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Browser Policy');
        if (typeof options.checkBrowserPolicyConfigChanged === 'function') {
          options.checkBrowserPolicyConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-whitelist-config',
      prepare() {
        const networkField = getById('network-whitelist');
        const pathField = getById('path-whitelist');
        const networkRaw = networkField ? networkField.value : '';
        const pathRaw = pathField ? pathField.value : '';
        return {
          whitelist: parseList(networkRaw),
          pathWhitelist: parseList(pathRaw),
          normalizedNetwork: normalizeList(networkRaw),
          normalizedPath: normalizeList(pathRaw)
        };
      },
      buildPatch: ({ whitelist, pathWhitelist }) => ({
        whitelist,
        path_whitelist: pathWhitelist
      }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateBypassAllowlistConfig === 'function') {
          options.updateBypassAllowlistConfig(data.config);
        } else if (typeof options.setBypassAllowlistSavedState === 'function') {
          options.setBypassAllowlistSavedState({
            network: prepared.normalizedNetwork,
            path: prepared.normalizedPath
          });
        }
        setAdminMessage(messageTarget, 'Bypass allowlists saved', 'success');
        endSaveButton(button, 'Save Allowlists');
        if (typeof options.checkBypassAllowlistsConfigChanged === 'function') {
          options.checkBypassAllowlistsConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Allowlists');
        if (typeof options.checkBypassAllowlistsConfigChanged === 'function') {
          options.checkBypassAllowlistsConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-pow-config',
      prepare(messageTarget) {
        const difficulty = options.readIntegerFieldValue('pow-difficulty', messageTarget);
        const ttl = options.readIntegerFieldValue('pow-ttl', messageTarget);
        if (difficulty === null || ttl === null) return null;
        return {
          enabled: getById('pow-enabled-toggle').checked,
          difficulty,
          ttl
        };
      },
      buildPatch: ({ enabled, difficulty, ttl }) => ({
        pow_enabled: enabled,
        pow_difficulty: difficulty,
        pow_ttl_seconds: ttl
      }),
      onSuccess({ button, messageTarget, prepared }) {
        if (typeof options.setPowSavedState === 'function') {
          options.setPowSavedState({
            enabled: prepared.enabled,
            difficulty: prepared.difficulty,
            ttl: prepared.ttl,
            mutable: true
          });
        }
        setAdminMessage(messageTarget, 'PoW settings saved', 'success');
        endSaveButton(button, 'Save PoW Settings');
        if (typeof options.checkPowConfigChanged === 'function') {
          options.checkPowConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save PoW Settings');
        if (typeof options.checkPowConfigChanged === 'function') {
          options.checkPowConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-challenge-puzzle-config',
      prepare(messageTarget) {
        const count = options.readIntegerFieldValue('challenge-puzzle-transform-count', messageTarget);
        if (count === null) return null;
        return {
          enabled: getById('challenge-puzzle-enabled-toggle').checked,
          count
        };
      },
      buildPatch: ({ enabled, count }) => ({
        challenge_puzzle_enabled: enabled,
        challenge_puzzle_transform_count: count
      }),
      onSuccess({ button, messageTarget, prepared }) {
        if (typeof options.setChallengePuzzleSavedState === 'function') {
          options.setChallengePuzzleSavedState({
            enabled: prepared.enabled,
            count: prepared.count,
            mutable: true
          });
        }
        setAdminMessage(messageTarget, 'Challenge puzzle settings saved', 'success');
        endSaveButton(button, 'Save Challenge Puzzle');
        if (typeof options.checkChallengePuzzleConfigChanged === 'function') {
          options.checkChallengePuzzleConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Challenge Puzzle');
        if (typeof options.checkChallengePuzzleConfigChanged === 'function') {
          options.checkChallengePuzzleConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-botness-config',
      prepare(messageTarget) {
        const challengeThreshold = options.readIntegerFieldValue('challenge-puzzle-threshold', messageTarget);
        const mazeThreshold = options.readIntegerFieldValue('maze-threshold-score', messageTarget);
        const weightJsRequired = options.readIntegerFieldValue('weight-js-required', messageTarget);
        const weightGeoRisk = options.readIntegerFieldValue('weight-geo-risk', messageTarget);
        const weightRateMedium = options.readIntegerFieldValue('weight-rate-medium', messageTarget);
        const weightRateHigh = options.readIntegerFieldValue('weight-rate-high', messageTarget);

        if (
          challengeThreshold === null ||
          mazeThreshold === null ||
          weightJsRequired === null ||
          weightGeoRisk === null ||
          weightRateMedium === null ||
          weightRateHigh === null
        ) {
          return null;
        }

        return {
          challengeThreshold,
          mazeThreshold,
          weightJsRequired,
          weightGeoRisk,
          weightRateMedium,
          weightRateHigh
        };
      },
      buildPatch: ({
        challengeThreshold,
        mazeThreshold,
        weightJsRequired,
        weightGeoRisk,
        weightRateMedium,
        weightRateHigh
      }) => ({
        challenge_puzzle_risk_threshold: challengeThreshold,
        botness_maze_threshold: mazeThreshold,
        botness_weights: {
          js_required: weightJsRequired,
          geo_risk: weightGeoRisk,
          rate_medium: weightRateMedium,
          rate_high: weightRateHigh
        }
      }),
      onSuccess({ button, messageTarget, prepared }) {
        if (typeof options.setBotnessSavedState === 'function') {
          options.setBotnessSavedState({
            challengeThreshold: prepared.challengeThreshold,
            mazeThreshold: prepared.mazeThreshold,
            weightJsRequired: prepared.weightJsRequired,
            weightGeoRisk: prepared.weightGeoRisk,
            weightRateMedium: prepared.weightRateMedium,
            weightRateHigh: prepared.weightRateHigh,
            mutable: true
          });
        }
        setAdminMessage(messageTarget, 'Botness scoring saved', 'success');
        endSaveButton(button, 'Save Botness Settings');
        if (typeof options.checkBotnessConfigChanged === 'function') {
          options.checkBotnessConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Botness Settings');
        if (typeof options.checkBotnessConfigChanged === 'function') {
          options.checkBotnessConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-cdp-config',
      prepare() {
        const thresholdRaw = getById('cdp-threshold-slider').value;
        return {
          enabled: getById('cdp-enabled-toggle').checked,
          autoBan: getById('cdp-auto-ban-toggle').checked,
          threshold: Number.parseFloat(thresholdRaw)
        };
      },
      buildPatch: ({ enabled, autoBan, threshold }) => ({
        cdp_detection_enabled: enabled,
        cdp_auto_ban: autoBan,
        cdp_detection_threshold: threshold
      }),
      onSuccess({ button, data, prepared }) {
        if (data && data.config && typeof options.updateCdpConfig === 'function') {
          options.updateCdpConfig(data.config);
        } else if (typeof options.setCdpSavedState === 'function') {
          options.setCdpSavedState(prepared);
          if (typeof options.checkCdpConfigChanged === 'function') {
            options.checkCdpConfigChanged();
          }
        }
        endSaveButtonAfter(button, 'Saved!', 'Save CDP Settings', 1500, () => {
          if (typeof options.checkCdpConfigChanged === 'function') {
            options.checkCdpConfigChanged();
          }
        });
      },
      onError({ button }) {
        endSaveButton(button, 'Error');
        endSaveButtonAfter(button, undefined, 'Save CDP Settings', 2000, () => {
          if (typeof options.checkCdpConfigChanged === 'function') {
            options.checkCdpConfigChanged();
          }
        });
      }
    },
    {
      buttonId: 'save-edge-integration-mode-config',
      prepare() {
        const modeSelect = getById('edge-integration-mode-select');
        const mode = String(modeSelect ? modeSelect.value : '').trim().toLowerCase();
        return { mode };
      },
      buildPatch: ({ mode }) => ({ edge_integration_mode: mode }),
      onSuccess({ button, messageTarget, data, prepared }) {
        if (data && data.config && typeof options.updateEdgeIntegrationModeConfig === 'function') {
          options.updateEdgeIntegrationModeConfig(data.config);
        } else if (typeof options.setEdgeIntegrationModeSavedState === 'function') {
          options.setEdgeIntegrationModeSavedState({ mode: prepared.mode });
          if (typeof options.checkEdgeIntegrationModeChanged === 'function') {
            options.checkEdgeIntegrationModeChanged();
          }
        }
        setAdminMessage(messageTarget, 'Edge integration mode saved', 'success');
        endSaveButton(button, 'Save Edge Integration Mode');
        if (typeof options.checkEdgeIntegrationModeChanged === 'function') {
          options.checkEdgeIntegrationModeChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Edge Integration Mode');
        if (typeof options.checkEdgeIntegrationModeChanged === 'function') {
          options.checkEdgeIntegrationModeChanged();
        }
      }
    },
    {
      buttonId: 'save-rate-limit-config',
      prepare(messageTarget) {
        const value = options.readIntegerFieldValue('rate-limit-threshold', messageTarget);
        if (value === null) return null;
        return { value };
      },
      buildPatch: ({ value }) => ({ rate_limit: value }),
      onSuccess({ button, messageTarget, prepared }) {
        if (typeof options.setRateLimitSavedState === 'function') {
          options.setRateLimitSavedState({ value: prepared.value });
        }
        applyStatusPatch({ rateLimit: prepared.value });
        setAdminMessage(messageTarget, 'Rate limit saved', 'success');
        endSaveButton(button, 'Save Rate Limit');
        if (typeof options.checkRateLimitConfigChanged === 'function') {
          options.checkRateLimitConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Rate Limit');
        if (typeof options.checkRateLimitConfigChanged === 'function') {
          options.checkRateLimitConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-js-required-config',
      prepare() {
        return { enforced: getById('js-required-enforced-toggle').checked };
      },
      buildPatch: ({ enforced }) => ({ js_required_enforced: enforced }),
      onSuccess({ button, messageTarget, prepared }) {
        if (typeof options.setJsRequiredSavedState === 'function') {
          options.setJsRequiredSavedState({ enforced: prepared.enforced });
        }
        applyStatusPatch({ jsRequiredEnforced: prepared.enforced });
        setAdminMessage(messageTarget, 'JS Required setting saved', 'success');
        endSaveButton(button, 'Save JS Required');
        if (typeof options.checkJsRequiredConfigChanged === 'function') {
          options.checkJsRequiredConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save JS Required');
        if (typeof options.checkJsRequiredConfigChanged === 'function') {
          options.checkJsRequiredConfigChanged();
        }
      }
    },
    {
      buttonId: 'save-durations-btn',
      prepare() {
        const durations = {
          honeypot: options.readBanDurationSeconds('honeypot'),
          rate_limit: options.readBanDurationSeconds('rateLimit'),
          browser: options.readBanDurationSeconds('browser'),
          cdp: options.readBanDurationSeconds('cdp'),
          admin: options.readBanDurationSeconds('admin')
        };
        const values = Object.values(durations);
        if (values.some((value) => value === null)) return null;
        return { banDurations: durations };
      },
      buildPatch: ({ banDurations }) => ({ ban_durations: banDurations }),
      onSuccess({ messageTarget, data, prepared }) {
        const saved =
          data && data.config && data.config.ban_durations
            ? data.config.ban_durations
            : prepared.banDurations;
        if (typeof options.updateBanDurations === 'function') {
          options.updateBanDurations({ ban_durations: saved });
        }
        setAdminMessage(messageTarget, 'Ban durations saved', 'success');
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Durations');
        if (typeof options.checkBanDurationsChanged === 'function') {
          options.checkBanDurationsChanged();
        }
      }
    },
    {
      buttonId: 'save-advanced-config',
      prepare(messageTarget) {
        if (typeof options.readAdvancedConfigPatch !== 'function') return null;
        const patch = options.readAdvancedConfigPatch(messageTarget);
        if (!patch) return null;
        return { patch };
      },
      buildPatch: ({ patch }) => patch,
      onSuccess({ button, messageTarget, data }) {
        if (data && data.config && typeof options.setAdvancedConfigFromConfig === 'function') {
          options.setAdvancedConfigFromConfig(data.config, false);
        }
        setAdminMessage(messageTarget, 'Advanced config patch saved', 'success');
        endSaveButton(button, 'Save Advanced Config');
        if (typeof options.checkAdvancedConfigChanged === 'function') {
          options.checkAdvancedConfigChanged();
        }
      },
      onError({ button, messageTarget, error }) {
        setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
        endSaveButton(button, 'Save Advanced Config');
        if (typeof options.checkAdvancedConfigChanged === 'function') {
          options.checkAdvancedConfigChanged();
        }
      }
    }
  ]);

  const bindSaveHandler = (spec) => {
    const button = getById(spec.buttonId);
    if (!button) return;
    button.onclick = async function onConfigSaveClick() {
      const messageTarget = getById('admin-msg');
      const prepared = spec.prepare ? spec.prepare(messageTarget, button) : {};
      if (prepared === null) return;

      beginSaveButton(button, 'Saving...');
      try {
        const patch = spec.buildPatch ? spec.buildPatch(prepared) : prepared;
        const data = await saveConfigPatch(messageTarget, patch);
        if (typeof spec.onSuccess === 'function') {
          await spec.onSuccess({
            prepared,
            data,
            button,
            messageTarget,
            setAdminMessage,
            endSaveButton,
            endSaveButtonAfter
          });
        } else {
          endSaveButton(button);
        }
      } catch (error) {
        if (typeof spec.onError === 'function') {
          await spec.onError({
            prepared,
            error,
            button,
            messageTarget,
            setAdminMessage,
            endSaveButton,
            endSaveButtonAfter
          });
        } else {
          setAdminMessage(messageTarget, `Error: ${error.message}`, 'error');
          endSaveButton(button);
        }
      }
    };
  };

  saveRegistry.forEach(bindSaveHandler);

  const testModeToggle = getById('test-mode-toggle');
  if (testModeToggle) {
    testModeToggle.addEventListener('change', async function onTestModeChange() {
      const msg = getById('admin-msg');
      if (!options.getAdminContext(msg)) {
        this.checked = !this.checked;
        return;
      }

      const testMode = this.checked;
      setAdminMessage(msg, `${testMode ? 'Enabling' : 'Disabling'} test mode...`, 'info');

      try {
        const data = await saveConfigPatch(msg, { test_mode: testMode });
        setAdminMessage(msg, `Test mode ${data.config.test_mode ? 'enabled' : 'disabled'}`, 'success');
        if (typeof options.refreshDashboard === 'function') {
          timerSetTimeout(() => options.refreshDashboard(), 500);
        }
      } catch (error) {
        setAdminMessage(msg, `Error: ${error.message}`, 'error');
        this.checked = !testMode;
      }
    });
  }
}

export {
  bind,
  flattenBindOptions as _flattenBindOptions,
  normalizeContextOptions as _normalizeContextOptions
};
