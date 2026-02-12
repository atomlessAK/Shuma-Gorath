(function (global) {
  const state = {
    failMode: 'unknown',
    httpsEnforced: false,
    forwardedHeaderTrustConfigured: false,
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

  function envVar(name) {
    return `<code class="env-var">${name}</code>`;
  }

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

  function cumulativeBotnessRoutingText(snapshot) {
    return (
      `This contributes to the cumulative <strong>botness</strong> score used for defense routing decisions ` +
      `(challenge at <strong>${snapshot.challengeThreshold}</strong>, maze at <strong>${snapshot.mazeThreshold}</strong>, ` +
      `and higher-severity controls such as tar pit or immediate IP ban where configured).`
    );
  }

  const STATUS_DEFINITIONS = [
    {
      title: 'Fail Mode Policy',
      description: () => (
        `Controls request handling when the KV store is unavailable. ${envVar('SHUMA_KV_STORE_FAIL_OPEN')}=<strong>true</strong> allows requests to continue (fail-open); ` +
        `${envVar('SHUMA_KV_STORE_FAIL_OPEN')}=<strong>false</strong> blocks requests that require KV-backed decisions (fail-closed).`
      ),
      status: snapshot => normalizeFailMode(snapshot.failMode).toUpperCase()
    },
    {
      title: 'HTTPS Enforcement',
      description: snapshot => (
        `When ${envVar('SHUMA_ENFORCE_HTTPS')} is true, the app rejects non-HTTPS requests with <strong>403 HTTPS required</strong>. ` +
        `Forwarded proto headers are trusted only when ${envVar('SHUMA_FORWARDED_IP_SECRET')} validation succeeds. ` +
        `Current forwarded-header trust configuration is <strong>${boolStatus(snapshot.forwardedHeaderTrustConfigured)}</strong>.`
      ),
      status: snapshot => boolStatus(snapshot.httpsEnforced)
    },
    {
      title: 'Test Mode',
      description: () => (
        `${envVar('SHUMA_TEST_MODE')} controls whether defenses are enforce-only or log-only. When enabled, detections and ban actions are logged but traffic is not blocked. ` +
        'Use this for safe tuning before turning enforcement on.'
      ),
      status: snapshot => boolStatus(snapshot.testMode)
    },
    {
      title: 'Proof-of-Work (PoW)',
      description: snapshot => (
        `PoW is applied in the JS verification flow and increases bot cost before <code>js_verified</code> is issued. ` +
        `Primary controls are ${envVar('SHUMA_POW_ENABLED')}, ${envVar('SHUMA_POW_DIFFICULTY')}, and ${envVar('SHUMA_POW_TTL_SECONDS')}. ` +
        `Runtime editability is controlled by ${envVar('SHUMA_POW_CONFIG_MUTABLE')} and is currently ${formatMutability(snapshot.powMutable)}. ` +
        `If ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} is disabled, normal visitor requests bypass this flow.`
      ),
      status: snapshot => boolStatus(snapshot.powEnabled)
    },
    {
      title: 'Challenge',
      description: snapshot => (
        `Step-up routing sends suspicious traffic to the puzzle challenge when cumulative botness reaches ${envVar('SHUMA_CHALLENGE_RISK_THRESHOLD')} ` +
        `(current: <strong>${snapshot.challengeThreshold}</strong>). ` +
        `Puzzle complexity is controlled by ${envVar('SHUMA_CHALLENGE_TRANSFORM_COUNT')}. ` +
        `Runtime threshold mutability is controlled by ${envVar('SHUMA_CHALLENGE_CONFIG_MUTABLE')} / ${envVar('SHUMA_BOTNESS_CONFIG_MUTABLE')} and is currently ${formatMutability(snapshot.challengeMutable || snapshot.botnessMutable)}.`
      ),
      status: snapshot => boolStatus(snapshot.challengeThreshold >= 1)
    },
    {
      title: 'CDP Detection',
      description: () => (
        `Detects browser automation from client CDP reports. Primary controls are ${envVar('SHUMA_CDP_DETECTION_ENABLED')}, ${envVar('SHUMA_CDP_AUTO_BAN')}, and ${envVar('SHUMA_CDP_DETECTION_THRESHOLD')}. ` +
        `Hard checks (for example <code>webdriver</code> or <code>automation_props</code>) are treated as <strong>strong</strong>. ` +
        `Without hard checks, detections are tiered by score and soft signals using ${envVar('SHUMA_CDP_DETECTION_THRESHOLD')}. ` +
        `If ${envVar('SHUMA_CDP_AUTO_BAN')} is enabled, only final <strong>strong</strong> CDP detections trigger automatic IP bans.`
      ),
      status: snapshot => boolStatus(snapshot.cdpEnabled)
    },
    {
      title: 'Maze',
      description: () => (
        `Maze routes suspicious traffic into trap pages when ${envVar('SHUMA_MAZE_ENABLED')} is enabled. ` +
        `If ${envVar('SHUMA_MAZE_AUTO_BAN')} is enabled, automatic bans trigger when maze hits exceed ${envVar('SHUMA_MAZE_AUTO_BAN_THRESHOLD')}.`
      ),
      status: snapshot => boolStatus(snapshot.mazeEnabled)
    },
    {
      title: 'JS Required',
      description: snapshot => (
        `When ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} is true, requests without a valid <code>js_verified</code> cookie are sent to the JS verification page. ` +
        `That flow writes <code>js_verified</code>, reloads the original path, and re-evaluates access. ` +
        `If ${envVar('SHUMA_POW_ENABLED')} is true, this step includes PoW before the cookie is issued. ` +
        `Disabling ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} allows non-JS clients but removes PoW from the normal request path. ` +
        `Its botness contribution is weighted separately by ${envVar('SHUMA_BOTNESS_WEIGHT_JS_REQUIRED')} ` +
        `(current weight: <strong>${snapshot.botnessWeights.js_required || 0}</strong>). ` +
        cumulativeBotnessRoutingText(snapshot)
      ),
      status: snapshot => boolStatus(snapshot.jsRequiredEnforced)
    },
    {
      title: 'GEO Fencing',
      description: snapshot => (
        `Uses trusted upstream GEO headers only (headers are trusted when ${envVar('SHUMA_FORWARDED_IP_SECRET')} validation succeeds). ` +
        `Scoring countries are configured by ${envVar('SHUMA_GEO_RISK_COUNTRIES')} ` +
        `(current count: <strong>${snapshot.geoRiskCount}</strong>). ` +
        `Routing precedence uses ${envVar('SHUMA_GEO_BLOCK_COUNTRIES')} (<strong>${snapshot.geoBlockCount}</strong>), ` +
        `${envVar('SHUMA_GEO_MAZE_COUNTRIES')} (<strong>${snapshot.geoMazeCount}</strong>), ` +
        `${envVar('SHUMA_GEO_CHALLENGE_COUNTRIES')} (<strong>${snapshot.geoChallengeCount}</strong>), ` +
        `and ${envVar('SHUMA_GEO_ALLOW_COUNTRIES')} (<strong>${snapshot.geoAllowCount}</strong>). ` +
        `Scoring matches contribute via ${envVar('SHUMA_BOTNESS_WEIGHT_GEO_RISK')} ` +
        `(current weight: <strong>${snapshot.botnessWeights.geo_risk || 0}</strong>). ` +
        cumulativeBotnessRoutingText(snapshot)
      ),
      status: snapshot => boolStatus((snapshot.botnessWeights.geo_risk || 0) > 0)
    },
    {
      title: 'Rate Limiting',
      description: snapshot => (
        `Rate pressure is measured against ${envVar('SHUMA_RATE_LIMIT')} (current limit: <strong>${snapshot.rateLimit}</strong> requests/min). ` +
        `Crossing the hard limit triggers immediate rate-limit enforcement. ` +
        `Medium pressure (>=50%) contributes ${envVar('SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM')} and high pressure (>=80%) ` +
        `contributes ${envVar('SHUMA_BOTNESS_WEIGHT_RATE_HIGH')} (current weights: <strong>${snapshot.botnessWeights.rate_medium || 0}</strong> / ` +
        `<strong>${snapshot.botnessWeights.rate_high || 0}</strong>). ` +
        cumulativeBotnessRoutingText(snapshot)
      ),
      status: snapshot => boolStatus(
        (snapshot.botnessWeights.rate_medium || 0) > 0 || (snapshot.botnessWeights.rate_high || 0) > 0
      )
    }
  ];

  function update(patch = {}) {
    if (!patch || typeof patch !== 'object') return getState();
    if (patch.botnessWeights && typeof patch.botnessWeights === 'object') {
      state.botnessWeights = {
        ...state.botnessWeights,
        ...patch.botnessWeights
      };
    }
    Object.keys(patch).forEach((key) => {
      if (key === 'botnessWeights') return;
      if (Object.prototype.hasOwnProperty.call(state, key)) {
        state[key] = patch[key];
      }
    });
    return getState();
  }

  function getState() {
    return {
      ...state,
      botnessWeights: { ...state.botnessWeights }
    };
  }

  function render() {
    const container = document.getElementById('status-items');
    if (!container) return;
    const snapshot = getState();
    container.innerHTML = STATUS_DEFINITIONS.map(definition => `
      <div class="status-item">
        <h3>${definition.title}</h3>
        <p class="control-desc text-muted">${definition.description(snapshot)}</p>
        <div class="status-rows">
          <div class="info-row">
            <span class="info-label text-muted">Status:</span>
            <span class="status-value">${definition.status(snapshot)}</span>
          </div>
        </div>
      </div>
    `).join('');
  }

  global.ShumaDashboardStatus = {
    update,
    getState,
    render,
    normalizeFailMode
  };
})(window);
