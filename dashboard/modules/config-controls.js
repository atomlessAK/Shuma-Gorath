(function (global) {
  function bind(options = {}) {
    const statusPanel = options.statusPanel || null;

    const saveMazeButton = document.getElementById('save-maze-config');
    if (saveMazeButton) {
      saveMazeButton.onclick = async function saveMazeConfig() {
        const msg = document.getElementById('admin-msg');
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
        const btn = this;

        const mazeEnabled = document.getElementById('maze-enabled-toggle').checked;
        const mazeAutoBan = document.getElementById('maze-auto-ban-toggle').checked;
        const mazeThreshold = options.readIntegerFieldValue('maze-threshold', msg);
        if (mazeThreshold === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({
              maze_enabled: mazeEnabled,
              maze_auto_ban: mazeAutoBan,
              maze_auto_ban_threshold: mazeThreshold
            })
          });

          if (!resp.ok) throw new Error('Failed to save config');

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
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
        const btn = this;

        const robotsEnabled = document.getElementById('robots-enabled-toggle').checked;
        const blockTraining = document.getElementById('robots-block-training-toggle').checked;
        const blockSearch = document.getElementById('robots-block-search-toggle').checked;
        const allowSearchEngines = !document.getElementById('robots-allow-search-toggle').checked;
        const crawlDelay = options.readIntegerFieldValue('robots-crawl-delay', msg);
        if (crawlDelay === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
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
          options.setRobotsSavedState({
            enabled: robotsEnabled,
            blockTraining: blockTraining,
            blockSearch: blockSearch,
            allowSearch: document.getElementById('robots-allow-search-toggle').checked,
            crawlDelay: crawlDelay
          });
          const preview = document.getElementById('robots-preview');
          if (preview && !preview.classList.contains('hidden')) {
            await options.refreshRobotsPreview();
          }
          setTimeout(() => {
            btn.dataset.saving = 'false';
            btn.textContent = 'Update Policy';
            options.checkRobotsConfigChanged();
          }, 1500);
        } catch (e) {
          btn.dataset.saving = 'false';
          btn.textContent = 'Error';
          console.error('Failed to save robots config:', e);
          setTimeout(() => {
            btn.textContent = 'Update Policy';
            options.checkRobotsConfigChanged();
          }, 2000);
        }
      };
    }

    const saveGeoScoringButton = document.getElementById('save-geo-scoring-config');
    if (saveGeoScoringButton) {
      saveGeoScoringButton.onclick = async function saveGeoScoringConfig() {
        const msg = document.getElementById('admin-msg');
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
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
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({ geo_risk: geoRisk })
          });
          if (!resp.ok) {
            const text = await resp.text();
            throw new Error(text || 'Failed to save GEO scoring config');
          }
          const data = await resp.json();
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
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
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
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({
              geo_allow: geoAllow,
              geo_challenge: geoChallenge,
              geo_maze: geoMaze,
              geo_block: geoBlock
            })
          });
          if (!resp.ok) {
            const text = await resp.text();
            throw new Error(text || 'Failed to save GEO routing config');
          }
          const data = await resp.json();
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

    const savePowButton = document.getElementById('save-pow-config');
    if (savePowButton) {
      savePowButton.onclick = async function savePowConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;

        const powDifficulty = options.readIntegerFieldValue('pow-difficulty', msg);
        const powTtl = options.readIntegerFieldValue('pow-ttl', msg);
        if (powDifficulty === null || powTtl === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;

        try {
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
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

          options.setPowSavedState({
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

    const saveBotnessButton = document.getElementById('save-botness-config');
    if (saveBotnessButton) {
      saveBotnessButton.onclick = async function saveBotnessConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;

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
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
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
        const ctx = options.getAdminContext(document.getElementById('admin-msg'));
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
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
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

    const saveRateLimitButton = document.getElementById('save-rate-limit-config');
    if (saveRateLimitButton) {
      saveRateLimitButton.onclick = async function saveRateLimitConfig() {
        const btn = this;
        const msg = document.getElementById('admin-msg');
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
        const rateLimit = options.readIntegerFieldValue('rate-limit-threshold', msg);
        if (rateLimit === null) return;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({ rate_limit: rateLimit })
          });
          if (!resp.ok) {
            const text = await resp.text();
            throw new Error(text || 'Failed to save rate limit');
          }
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
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
        const enforced = document.getElementById('js-required-enforced-toggle').checked;

        btn.textContent = 'Saving...';
        btn.dataset.saving = 'true';
        btn.disabled = true;
        try {
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({ js_required_enforced: enforced })
          });
          if (!resp.ok) {
            const text = await resp.text();
            throw new Error(text || 'Failed to save JS Required setting');
          }
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
        const ctx = options.getAdminContext(msg);
        if (!ctx) return;
        const { endpoint, apikey } = ctx;
        const btn = this;

        const banDurations = {
          honeypot: options.readBanDurationSeconds('honeypot'),
          rate_limit: options.readBanDurationSeconds('rateLimit'),
          browser: options.readBanDurationSeconds('browser'),
          admin: options.readBanDurationSeconds('admin')
        };

        if (
          banDurations.honeypot === null ||
          banDurations.rate_limit === null ||
          banDurations.browser === null ||
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
          const resp = await fetch(`${endpoint}/admin/config`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${apikey}`,
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({ ban_durations: banDurations })
          });

          if (!resp.ok) {
            throw new Error('Failed to save config');
          }

          const data = await resp.json();
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

    const testModeToggle = document.getElementById('test-mode-toggle');
    if (testModeToggle) {
      testModeToggle.addEventListener('change', async function onTestModeChange() {
        const msg = document.getElementById('admin-msg');
        const ctx = options.getAdminContext(msg);
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
              'Authorization': `Bearer ${apikey}`,
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
