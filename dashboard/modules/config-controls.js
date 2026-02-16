// @ts-check

(function (global) {
  function bind(options = {}) {
    const statusPanel = options.statusPanel || null;
    const apiClient = options.apiClient || null;

    async function saveConfigPatch(messageTarget, patch) {
      let result;
      if (apiClient && typeof apiClient.updateConfig === 'function') {
        result = await apiClient.updateConfig(patch);
      } else {
        const ctx = options.getAdminContext(messageTarget || null);
        if (!ctx) {
          throw new Error('Missing admin API context');
        }
        const { endpoint, apikey } = ctx;
        const resp = await fetch(`${endpoint}/admin/config`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${apikey}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(patch)
        });
        if (!resp.ok) {
          const text = await resp.text();
          throw new Error(text || 'Failed to save config');
        }
        result = await resp.json();
      }
      if (statusPanel && result && result.config && typeof result.config === 'object') {
        statusPanel.update({ configSnapshot: result.config });
        statusPanel.render();
      }
      if (typeof options.onConfigSaved === 'function') {
        options.onConfigSaved(patch, result);
      }
      return result;
    }

    function parseList(raw) {
      if (typeof options.parseListTextarea === 'function') {
        return options.parseListTextarea(raw);
      }
      return String(raw || '')
        .split(/[\n,]/)
        .map((part) => part.trim())
        .filter(Boolean);
    }

    function normalizeList(raw) {
      if (typeof options.normalizeListTextareaForCompare === 'function') {
        return options.normalizeListTextareaForCompare(raw);
      }
      return parseList(raw).join('\n');
    }

    function parseHoneypotPaths(raw) {
      if (typeof options.parseHoneypotPathsTextarea === 'function') {
        return options.parseHoneypotPathsTextarea(raw);
      }
      const values = parseList(raw);
      values.forEach((path) => {
        if (!String(path).startsWith('/')) {
          throw new Error(`Invalid honeypot path '${path}'. Paths must start with '/'.`);
        }
      });
      return values;
    }

    function parseBrowserRules(raw) {
      if (typeof options.parseBrowserRulesTextarea === 'function') {
        return options.parseBrowserRulesTextarea(raw);
      }
      return [];
    }

    function normalizeBrowserRules(raw) {
      if (typeof options.normalizeBrowserRulesForCompare === 'function') {
        return options.normalizeBrowserRulesForCompare(raw);
      }
      return String(raw || '').trim();
    }

    const saveMazeButton = document.getElementById('save-maze-config');
    if (saveMazeButton) {
      saveMazeButton.onclick = async function saveMazeConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;

        const mazeEnabled = document.getElementById('maze-enabled-toggle').checked;
        const mazeAutoBan = document.getElementById('maze-auto-ban-toggle').checked;
        const mazeThreshold = options.readIntegerFieldValue('maze-threshold', msg);
        if (mazeThreshold === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          await saveConfigPatch(msg, {
            maze_enabled: mazeEnabled,
            maze_auto_ban: mazeAutoBan,
            maze_auto_ban_threshold: mazeThreshold
          });

          options.setMazeSavedState({
            enabled: mazeEnabled,
            autoBan: mazeAutoBan,
            threshold: mazeThreshold
          });
          btn.textContent = 'Saved!';
          setTimeout(() => {
            btn.dataset.saving = 'false';
            btn.textContent = 'Save Maze Settings';
            options.checkMazeConfigChanged();
          }, 1500);
          msg.textContent = 'Maze settings saved';
          msg.className = 'message success';
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          console.error('Failed to save maze config:', e);
          btn.dataset.saving = 'false';
          btn.textContent = 'Save Maze Settings';
          options.checkMazeConfigChanged();
        }
      };
    }

    const saveRobotsButton = document.getElementById('save-robots-config');
    if (saveRobotsButton) {
      saveRobotsButton.onclick = async function saveRobotsConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;

        const robotsEnabled = document.getElementById('robots-enabled-toggle').checked;
        const crawlDelay = options.readIntegerFieldValue('robots-crawl-delay', msg);
        if (crawlDelay === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          await saveConfigPatch(msg, {
            robots_enabled: robotsEnabled,
            robots_crawl_delay: crawlDelay
          });

          btn.textContent = 'Updated!';
          options.setRobotsSavedState({
            enabled: robotsEnabled,
            crawlDelay: crawlDelay
          });
          const preview = document.getElementById('robots-preview');
          if (preview && !preview.classList.contains('hidden')) {
            await options.refreshRobotsPreview();
          }
          setTimeout(() => {
            btn.dataset.saving = 'false';
            btn.textContent = 'Save robots serving';
            options.checkRobotsConfigChanged();
          }, 1500);
        } catch (e) {
          btn.dataset.saving = 'false';
          btn.textContent = 'Error';
          console.error('Failed to save robots config:', e);
          setTimeout(() => {
            btn.textContent = 'Save robots serving';
            options.checkRobotsConfigChanged();
          }, 2000);
        }
      };
    }

    const saveAiPolicyButton = document.getElementById('save-ai-policy-config');
    if (saveAiPolicyButton) {
      saveAiPolicyButton.onclick = async function saveAiPolicyConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;

        const blockTraining = document.getElementById('robots-block-training-toggle').checked;
        const blockSearch = document.getElementById('robots-block-search-toggle').checked;
        const allowSearchEngines = !document.getElementById('robots-allow-search-toggle').checked;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          await saveConfigPatch(msg, {
            ai_policy_block_training: blockTraining,
            ai_policy_block_search: blockSearch,
            ai_policy_allow_search_engines: allowSearchEngines
          });

          btn.textContent = 'Saved!';
          options.setAiPolicySavedState({
            blockTraining: blockTraining,
            blockSearch: blockSearch,
            allowSearch: document.getElementById('robots-allow-search-toggle').checked
          });
          const preview = document.getElementById('robots-preview');
          if (preview && !preview.classList.contains('hidden')) {
            await options.refreshRobotsPreview();
          }
          setTimeout(() => {
            btn.dataset.saving = 'false';
            btn.textContent = 'Save AI bot policy';
            options.checkAiPolicyConfigChanged();
          }, 1500);
        } catch (e) {
          btn.dataset.saving = 'false';
          btn.textContent = 'Error';
          console.error('Failed to save AI bot policy:', e);
          setTimeout(() => {
            btn.textContent = 'Save AI bot policy';
            options.checkAiPolicyConfigChanged();
          }, 2000);
        }
      };
    }

    const saveGeoScoringButton = document.getElementById('save-geo-scoring-config');
    if (saveGeoScoringButton) {
      saveGeoScoringButton.onclick = async function saveGeoScoringConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;
        const geoState = options.getGeoSavedState();

        if (!geoState.mutable) {
          msg.textContent = 'GEO settings are read-only while SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false.';
          msg.className = 'message warning';
          btn.disabled = true;
          return;
        }

        let geoRisk;
        try {
          geoRisk = options.parseCountryCodesStrict(document.getElementById('geo-risk-list').value);
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          return;
        }

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const data = await saveConfigPatch(msg, { geo_risk: geoRisk });
          if (data && data.config) {
            options.updateGeoConfig(data.config);
          } else {
            options.setGeoSavedState({
              ...options.getGeoSavedState(),
              risk: geoRisk.join(','),
              mutable: true
            });
          }
          msg.textContent = 'GEO scoring saved';
          msg.className = 'message success';
          btn.textContent = 'Save GEO Scoring';
          btn.dataset.saving = 'false';
          options.checkGeoConfigChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save GEO Scoring';
          btn.dataset.saving = 'false';
          options.checkGeoConfigChanged();
        }
      };
    }

    const saveGeoRoutingButton = document.getElementById('save-geo-routing-config');
    if (saveGeoRoutingButton) {
      saveGeoRoutingButton.onclick = async function saveGeoRoutingConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;
        const geoState = options.getGeoSavedState();

        if (!geoState.mutable) {
          msg.textContent = 'GEO settings are read-only while SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false.';
          msg.className = 'message warning';
          btn.disabled = true;
          return;
        }

        let geoAllow;
        let geoChallenge;
        let geoMaze;
        let geoBlock;
        try {
          geoAllow = options.parseCountryCodesStrict(document.getElementById('geo-allow-list').value);
          geoChallenge = options.parseCountryCodesStrict(document.getElementById('geo-challenge-list').value);
          geoMaze = options.parseCountryCodesStrict(document.getElementById('geo-maze-list').value);
          geoBlock = options.parseCountryCodesStrict(document.getElementById('geo-block-list').value);
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          return;
        }

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const data = await saveConfigPatch(msg, {
            geo_allow: geoAllow,
            geo_challenge: geoChallenge,
            geo_maze: geoMaze,
            geo_block: geoBlock
          });
          if (data && data.config) {
            options.updateGeoConfig(data.config);
          } else {
            options.setGeoSavedState({
              ...options.getGeoSavedState(),
              allow: geoAllow.join(','),
              challenge: geoChallenge.join(','),
              maze: geoMaze.join(','),
              block: geoBlock.join(','),
              mutable: true
            });
          }
          msg.textContent = 'GEO routing saved';
          msg.className = 'message success';
          btn.textContent = 'Save GEO Routing';
          btn.dataset.saving = 'false';
          options.checkGeoConfigChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save GEO Routing';
          btn.dataset.saving = 'false';
          options.checkGeoConfigChanged();
        }
      };
    }

    const saveHoneypotButton = document.getElementById('save-honeypot-config');
    if (saveHoneypotButton) {
      saveHoneypotButton.onclick = async function saveHoneypotConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;
        const field = document.getElementById('honeypot-paths');
        let honeypots;

        try {
          honeypots = parseHoneypotPaths(field ? field.value : '');
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          return;
        }

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const data = await saveConfigPatch(msg, { honeypots });
          if (data && data.config && typeof options.updateHoneypotConfig === 'function') {
            options.updateHoneypotConfig(data.config);
          } else if (typeof options.setHoneypotSavedState === 'function') {
            options.setHoneypotSavedState({
              values: normalizeList(field ? field.value : '')
            });
            if (typeof options.checkHoneypotConfigChanged === 'function') {
              options.checkHoneypotConfigChanged();
            }
          }
          msg.textContent = 'Honeypot paths saved';
          msg.className = 'message success';
          btn.textContent = 'Save Honeypots';
          btn.dataset.saving = 'false';
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Honeypots';
          btn.dataset.saving = 'false';
          if (typeof options.checkHoneypotConfigChanged === 'function') {
            options.checkHoneypotConfigChanged();
          }
        }
      };
    }

    const saveBrowserPolicyButton = document.getElementById('save-browser-policy-config');
    if (saveBrowserPolicyButton) {
      saveBrowserPolicyButton.onclick = async function saveBrowserPolicyConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;
        const blockField = document.getElementById('browser-block-rules');
        const whitelistField = document.getElementById('browser-whitelist-rules');
        let browserBlock;
        let browserWhitelist;

        try {
          browserBlock = parseBrowserRules(blockField ? blockField.value : '');
          browserWhitelist = parseBrowserRules(whitelistField ? whitelistField.value : '');
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          return;
        }

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const data = await saveConfigPatch(msg, {
            browser_block: browserBlock,
            browser_whitelist: browserWhitelist
          });
          if (data && data.config && typeof options.updateBrowserPolicyConfig === 'function') {
            options.updateBrowserPolicyConfig(data.config);
          } else if (typeof options.setBrowserPolicySavedState === 'function') {
            options.setBrowserPolicySavedState({
              block: normalizeBrowserRules(blockField ? blockField.value : ''),
              whitelist: normalizeBrowserRules(whitelistField ? whitelistField.value : '')
            });
            if (typeof options.checkBrowserPolicyConfigChanged === 'function') {
              options.checkBrowserPolicyConfigChanged();
            }
          }
          msg.textContent = 'Browser policy saved';
          msg.className = 'message success';
          btn.textContent = 'Save Browser Policy';
          btn.dataset.saving = 'false';
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Browser Policy';
          btn.dataset.saving = 'false';
          if (typeof options.checkBrowserPolicyConfigChanged === 'function') {
            options.checkBrowserPolicyConfigChanged();
          }
        }
      };
    }

    const saveWhitelistButton = document.getElementById('save-whitelist-config');
    if (saveWhitelistButton) {
      saveWhitelistButton.onclick = async function saveWhitelistConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;
        const networkField = document.getElementById('network-whitelist');
        const pathField = document.getElementById('path-whitelist');
        const whitelist = parseList(networkField ? networkField.value : '');
        const pathWhitelist = parseList(pathField ? pathField.value : '');

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const data = await saveConfigPatch(msg, {
            whitelist,
            path_whitelist: pathWhitelist
          });
          if (data && data.config && typeof options.updateBypassAllowlistConfig === 'function') {
            options.updateBypassAllowlistConfig(data.config);
          } else if (typeof options.setBypassAllowlistSavedState === 'function') {
            options.setBypassAllowlistSavedState({
              network: normalizeList(networkField ? networkField.value : ''),
              path: normalizeList(pathField ? pathField.value : '')
            });
            if (typeof options.checkBypassAllowlistsConfigChanged === 'function') {
              options.checkBypassAllowlistsConfigChanged();
            }
          }
          msg.textContent = 'Bypass allowlists saved';
          msg.className = 'message success';
          btn.textContent = 'Save Allowlists';
          btn.dataset.saving = 'false';
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Allowlists';
          btn.dataset.saving = 'false';
          if (typeof options.checkBypassAllowlistsConfigChanged === 'function') {
            options.checkBypassAllowlistsConfigChanged();
          }
        }
      };
    }

    const savePowButton = document.getElementById('save-pow-config');
    if (savePowButton) {
      savePowButton.onclick = async function savePowConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');

        const powEnabled = document.getElementById('pow-enabled-toggle').checked;
        const powDifficulty = options.readIntegerFieldValue('pow-difficulty', msg);
        const powTtl = options.readIntegerFieldValue('pow-ttl', msg);
        if (powDifficulty === null || powTtl === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          await saveConfigPatch(msg, {
            pow_enabled: powEnabled,
            pow_difficulty: powDifficulty,
            pow_ttl_seconds: powTtl
          });

          options.setPowSavedState({
            enabled: powEnabled,
            difficulty: powDifficulty,
            ttl: powTtl,
            mutable: true
          });
          msg.textContent = 'PoW settings saved';
          msg.className = 'message success';
          btn.textContent = 'Save PoW Settings';
          btn.dataset.saving = 'false';
          options.checkPowConfigChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save PoW Settings';
          btn.dataset.saving = 'false';
          options.checkPowConfigChanged();
        }
      };
    }

    const saveChallengeTransformButton = document.getElementById('save-challenge-transform-config');
    if (saveChallengeTransformButton) {
      saveChallengeTransformButton.onclick = async function saveChallengeTransformConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const challengeEnabled = document.getElementById('challenge-enabled-toggle').checked;
        const transformCount = options.readIntegerFieldValue('challenge-transform-count', msg);
        if (transformCount === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          await saveConfigPatch(msg, {
            challenge_enabled: challengeEnabled,
            challenge_transform_count: transformCount
          });
          if (typeof options.setChallengeTransformSavedState === 'function') {
            options.setChallengeTransformSavedState({
              enabled: challengeEnabled,
              count: transformCount,
              mutable: true
            });
          }
          msg.textContent = 'Challenge puzzle settings saved';
          msg.className = 'message success';
          btn.textContent = 'Save Challenge Puzzle';
          btn.dataset.saving = 'false';
          if (typeof options.checkChallengeTransformConfigChanged === 'function') {
            options.checkChallengeTransformConfigChanged();
          }
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Challenge Puzzle';
          btn.dataset.saving = 'false';
          if (typeof options.checkChallengeTransformConfigChanged === 'function') {
            options.checkChallengeTransformConfigChanged();
          }
        }
      };
    }

    const saveBotnessButton = document.getElementById('save-botness-config');
    if (saveBotnessButton) {
      saveBotnessButton.onclick = async function saveBotnessConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');

        const challengeThreshold = options.readIntegerFieldValue('challenge-threshold', msg);
        const mazeThreshold = options.readIntegerFieldValue('maze-threshold-score', msg);
        const weightJsRequired = options.readIntegerFieldValue('weight-js-required', msg);
        const weightGeoRisk = options.readIntegerFieldValue('weight-geo-risk', msg);
        const weightRateMedium = options.readIntegerFieldValue('weight-rate-medium', msg);
        const weightRateHigh = options.readIntegerFieldValue('weight-rate-high', msg);

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
          await saveConfigPatch(msg, {
            challenge_risk_threshold: challengeThreshold,
            botness_maze_threshold: mazeThreshold,
            botness_weights: {
              js_required: weightJsRequired,
              geo_risk: weightGeoRisk,
              rate_medium: weightRateMedium,
              rate_high: weightRateHigh
            }
          });

          options.setBotnessSavedState({
            challengeThreshold: challengeThreshold,
            mazeThreshold: mazeThreshold,
            weightJsRequired: weightJsRequired,
            weightGeoRisk: weightGeoRisk,
            weightRateMedium: weightRateMedium,
            weightRateHigh: weightRateHigh,
            mutable: true
          });
          msg.textContent = 'Botness scoring saved';
          msg.className = 'message success';
          btn.textContent = 'Save Botness Settings';
          btn.dataset.saving = 'false';
          options.checkBotnessConfigChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Botness Settings';
          btn.dataset.saving = 'false';
          options.checkBotnessConfigChanged();
        }
      };
    }

    const saveCdpButton = document.getElementById('save-cdp-config');
    if (saveCdpButton) {
      saveCdpButton.onclick = async function saveCdpConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;

        const cdpEnabled = document.getElementById('cdp-enabled-toggle').checked;
        const cdpAutoBan = document.getElementById('cdp-auto-ban-toggle').checked;
        const cdpThreshold = parseFloat(document.getElementById('cdp-threshold-slider').value);

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          await saveConfigPatch(msg, {
            cdp_detection_enabled: cdpEnabled,
            cdp_auto_ban: cdpAutoBan,
            cdp_detection_threshold: cdpThreshold
          });

          btn.textContent = 'Saved!';
          options.setCdpSavedState({
            enabled: cdpEnabled,
            autoBan: cdpAutoBan,
            threshold: cdpThreshold
          });
          setTimeout(() => {
            btn.dataset.saving = 'false';
            btn.textContent = 'Save CDP Settings';
            options.checkCdpConfigChanged();
          }, 1500);
        } catch (e) {
          btn.dataset.saving = 'false';
          btn.textContent = 'Error';
          console.error('Failed to save CDP config:', e);
          setTimeout(() => {
            btn.textContent = 'Save CDP Settings';
            options.checkCdpConfigChanged();
          }, 2000);
        }
      };
    }

    const saveEdgeModeButton = document.getElementById('save-edge-integration-mode-config');
    if (saveEdgeModeButton) {
      saveEdgeModeButton.onclick = async function saveEdgeIntegrationModeConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const modeSelect = document.getElementById('edge-integration-mode-select');
        const mode = String(modeSelect ? modeSelect.value : '').trim().toLowerCase();

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const data = await saveConfigPatch(msg, { edge_integration_mode: mode });
          if (data && data.config && typeof options.updateEdgeIntegrationModeConfig === 'function') {
            options.updateEdgeIntegrationModeConfig(data.config);
          } else {
            options.setEdgeIntegrationModeSavedState({ mode });
            options.checkEdgeIntegrationModeChanged();
          }

          msg.textContent = 'Edge integration mode saved';
          msg.className = 'message success';
          btn.textContent = 'Save Edge Integration Mode';
          btn.dataset.saving = 'false';
          options.checkEdgeIntegrationModeChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Edge Integration Mode';
          btn.dataset.saving = 'false';
          options.checkEdgeIntegrationModeChanged();
        }
      };
    }

    const saveRateLimitButton = document.getElementById('save-rate-limit-config');
    if (saveRateLimitButton) {
      saveRateLimitButton.onclick = async function saveRateLimitConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const rateLimit = options.readIntegerFieldValue('rate-limit-threshold', msg);
        if (rateLimit === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          await saveConfigPatch(msg, { rate_limit: rateLimit });
          options.setRateLimitSavedState({ value: rateLimit });
          if (statusPanel) {
            statusPanel.update({ rateLimit });
            statusPanel.render();
          }
          msg.textContent = 'Rate limit saved';
          msg.className = 'message success';
          btn.textContent = 'Save Rate Limit';
          btn.dataset.saving = 'false';
          options.checkRateLimitConfigChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Rate Limit';
          btn.dataset.saving = 'false';
          options.checkRateLimitConfigChanged();
        }
      };
    }

    const saveJsRequiredButton = document.getElementById('save-js-required-config');
    if (saveJsRequiredButton) {
      saveJsRequiredButton.onclick = async function saveJsRequiredConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const enforced = document.getElementById('js-required-enforced-toggle').checked;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          await saveConfigPatch(msg, { js_required_enforced: enforced });
          options.setJsRequiredSavedState({ enforced });
          if (statusPanel) {
            statusPanel.update({ jsRequiredEnforced: enforced });
            statusPanel.render();
          }
          msg.textContent = 'JS Required setting saved';
          msg.className = 'message success';
          btn.textContent = 'Save JS Required';
          btn.dataset.saving = 'false';
          options.checkJsRequiredConfigChanged();
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save JS Required';
          btn.dataset.saving = 'false';
          options.checkJsRequiredConfigChanged();
        }
      };
    }

    const saveDurationsButton = document.getElementById('save-durations-btn');
    if (saveDurationsButton) {
      saveDurationsButton.onclick = async function saveDurations() {
        const msg = document.getElementById('admin-msg');
        const btn = this;

        const banDurations = {
          honeypot: options.readBanDurationSeconds('honeypot'),
          rate_limit: options.readBanDurationSeconds('rateLimit'),
          browser: options.readBanDurationSeconds('browser'),
          cdp: options.readBanDurationSeconds('cdp'),
          admin: options.readBanDurationSeconds('admin')
        };

        if (
          banDurations.honeypot === null ||
          banDurations.rate_limit === null ||
          banDurations.browser === null ||
          banDurations.cdp === null ||
          banDurations.admin === null
        ) {
          return;
        }

        msg.textContent = 'Saving ban durations...';
        msg.className = 'message info';
        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          const data = await saveConfigPatch(msg, { ban_durations: banDurations });
          const saved = data && data.config && data.config.ban_durations
            ? data.config.ban_durations
            : banDurations;
          options.updateBanDurations({ ban_durations: saved });
          msg.textContent = 'Ban durations saved';
          msg.className = 'message success';
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.dataset.saving = 'false';
          btn.textContent = 'Save Durations';
          options.checkBanDurationsChanged();
        }
      };
    }

    const saveAdvancedConfigButton = document.getElementById('save-advanced-config');
    if (saveAdvancedConfigButton) {
      saveAdvancedConfigButton.onclick = async function saveAdvancedConfig() {
        const msg = document.getElementById('admin-msg');
        const btn = this;
        const patch = typeof options.readAdvancedConfigPatch === 'function'
          ? options.readAdvancedConfigPatch(msg)
          : null;
        if (!patch) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          const data = await saveConfigPatch(msg, patch);
          if (data && data.config && typeof options.setAdvancedConfigFromConfig === 'function') {
            options.setAdvancedConfigFromConfig(data.config, false);
          } else if (typeof options.checkAdvancedConfigChanged === 'function') {
            options.checkAdvancedConfigChanged();
          }
          msg.textContent = 'Advanced config patch saved';
          msg.className = 'message success';
          btn.textContent = 'Save Advanced Config';
          btn.dataset.saving = 'false';
          if (typeof options.checkAdvancedConfigChanged === 'function') {
            options.checkAdvancedConfigChanged();
          }
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          btn.textContent = 'Save Advanced Config';
          btn.dataset.saving = 'false';
          if (typeof options.checkAdvancedConfigChanged === 'function') {
            options.checkAdvancedConfigChanged();
          }
        }
      };
    }

    const testModeToggle = document.getElementById('test-mode-toggle');
    if (testModeToggle) {
      testModeToggle.addEventListener('change', async function onTestModeChange() {
        const msg = document.getElementById('admin-msg');
        if (!options.getAdminContext(msg)) {
          this.checked = !this.checked;
          return;
        }
        const testMode = this.checked;

        msg.textContent = `${testMode ? 'Enabling' : 'Disabling'} test mode...`;
        msg.className = 'message info';

        try {
          const data = await saveConfigPatch(msg, { test_mode: testMode });
          msg.textContent = `Test mode ${data.config.test_mode ? 'enabled' : 'disabled'}`;
          msg.className = 'message success';
          setTimeout(() => options.refreshDashboard(), 500);
        } catch (e) {
          msg.textContent = `Error: ${e.message}`;
          msg.className = 'message error';
          this.checked = !testMode;
        }
      });
    }
  }

  global.ShumaDashboardConfigControls = {
    bind
  };
})(window);
