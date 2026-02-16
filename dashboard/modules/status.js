// @ts-check

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
    challengeEnabled: true,
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
    },
    configSnapshot: {}
  };

  const WRITABLE_VAR_PATHS = new Set([
    'test_mode',
    'ban_duration',
    'ban_durations.honeypot',
    'ban_durations.rate_limit',
    'ban_durations.browser',
    'ban_durations.admin',
    'ban_durations.cdp',
    'rate_limit',
    'honeypots',
    'browser_block',
    'browser_whitelist',
    'geo_risk',
    'geo_allow',
    'geo_challenge',
    'geo_maze',
    'geo_block',
    'whitelist',
    'path_whitelist',
    'maze_enabled',
    'maze_auto_ban',
    'maze_auto_ban_threshold',
    'maze_rollout_phase',
    'maze_token_ttl_seconds',
    'maze_token_max_depth',
    'maze_token_branch_budget',
    'maze_replay_ttl_seconds',
    'maze_entropy_window_seconds',
    'maze_client_expansion_enabled',
    'maze_checkpoint_every_nodes',
    'maze_checkpoint_every_ms',
    'maze_step_ahead_max',
    'maze_no_js_fallback_max_depth',
    'maze_micro_pow_enabled',
    'maze_micro_pow_depth_start',
    'maze_micro_pow_base_difficulty',
    'maze_max_concurrent_global',
    'maze_max_concurrent_per_ip_bucket',
    'maze_max_response_bytes',
    'maze_max_response_duration_ms',
    'maze_server_visible_links',
    'maze_max_links',
    'maze_max_paragraphs',
    'maze_path_entropy_segment_len',
    'maze_covert_decoys_enabled',
    'maze_seed_provider',
    'maze_seed_refresh_interval_seconds',
    'maze_seed_refresh_rate_limit_per_hour',
    'maze_seed_refresh_max_sources',
    'maze_seed_metadata_only',
    'robots_enabled',
    'robots_block_ai_training',
    'robots_block_ai_search',
    'robots_allow_search_engines',
    'ai_policy_block_training',
    'ai_policy_block_search',
    'ai_policy_allow_search_engines',
    'robots_crawl_delay',
    'cdp_detection_enabled',
    'cdp_auto_ban',
    'cdp_detection_threshold',
    'js_required_enforced',
    'pow_enabled',
    'pow_difficulty',
    'pow_ttl_seconds',
    'challenge_enabled',
    'challenge_transform_count',
    'challenge_risk_threshold',
    'botness_maze_threshold',
    'botness_weights.js_required',
    'botness_weights.geo_risk',
    'botness_weights.rate_medium',
    'botness_weights.rate_high',
    'botness_weights.maze_behavior',
    'defence_modes.rate',
    'defence_modes.geo',
    'defence_modes.js',
    'provider_backends.rate_limiter',
    'provider_backends.ban_store',
    'provider_backends.challenge_engine',
    'provider_backends.maze_tarpit',
    'provider_backends.fingerprint_signal',
    'edge_integration_mode'
  ]);

  const VAR_MEANINGS = Object.freeze({
    test_mode: 'Logs detections/actions without enforcing blocks.',
    ban_duration: 'Legacy fallback ban duration (seconds) when no specific trigger duration applies.',
    'ban_durations.honeypot': 'Ban duration (seconds) for honeypot/instaban trigger.',
    'ban_durations.rate_limit': 'Ban duration (seconds) for rate-limit enforcement.',
    'ban_durations.browser': 'Ban duration (seconds) for browser-policy automation detections.',
    'ban_durations.admin': 'Default ban duration (seconds) for manual admin bans.',
    'ban_durations.cdp': 'Ban duration (seconds) for strong CDP automation detections.',
    rate_limit: 'Requests-per-minute threshold used by rate limiting.',
    honeypots: 'Trap paths that are treated as high-confidence bot traffic.',
    browser_block: 'Minimum browser-version policy used for blocking suspicious automation stacks.',
    browser_whitelist: 'Browser-version exceptions allowed past browser policy blocks.',
    geo_risk: 'Country codes that add GEO botness score.',
    geo_allow: 'Country codes explicitly allowed by GEO routing precedence.',
    geo_challenge: 'Country codes forced to challenge routing.',
    geo_maze: 'Country codes forced to maze routing.',
    geo_block: 'Country codes forced to block routing.',
    whitelist: 'Trusted IP/CIDR allowlist that bypasses bot defenses.',
    path_whitelist: 'Trusted path allowlist that bypasses bot defenses.',
    maze_enabled: 'Turns maze routing on/off.',
    maze_auto_ban: 'Enables/disables maze-triggered auto-ban.',
    maze_auto_ban_threshold: 'Maze hit count threshold before auto-ban.',
    maze_rollout_phase: 'Maze enforcement phase: instrument, advisory, or enforce.',
    maze_token_ttl_seconds: 'Traversal token lifetime in seconds.',
    maze_token_max_depth: 'Maximum signed traversal depth.',
    maze_token_branch_budget: 'Signed branch budget attached to traversal tokens.',
    maze_replay_ttl_seconds: 'Replay-marker retention window in seconds.',
    maze_entropy_window_seconds: 'Entropy rotation window for maze variant selection.',
    maze_client_expansion_enabled: 'Enables checkpointed client-side link expansion.',
    maze_checkpoint_every_nodes: 'Checkpoint cadence by node count.',
    maze_checkpoint_every_ms: 'Checkpoint cadence by elapsed milliseconds.',
    maze_step_ahead_max: 'Maximum unverified step-ahead depth before checkpoint fallback.',
    maze_no_js_fallback_max_depth: 'Maximum tolerated maze depth when JS checkpointing is absent.',
    maze_micro_pow_enabled: 'Enables optional micro-PoW on deeper maze tiers.',
    maze_micro_pow_depth_start: 'Depth where maze micro-PoW begins.',
    maze_micro_pow_base_difficulty: 'Base micro-PoW difficulty for maze links.',
    maze_max_concurrent_global: 'Global concurrent maze-response budget.',
    maze_max_concurrent_per_ip_bucket: 'Per-IP-bucket concurrent maze-response budget.',
    maze_max_response_bytes: 'Maximum maze response size before budget fallback.',
    maze_max_response_duration_ms: 'Maximum maze render duration before budget fallback.',
    maze_server_visible_links: 'Count of server-visible links before client expansion.',
    maze_max_links: 'Hard cap on generated links per maze response.',
    maze_max_paragraphs: 'Hard cap on generated paragraphs per maze response.',
    maze_path_entropy_segment_len: 'Entropy segment length used in generated maze paths.',
    maze_covert_decoys_enabled: 'Enables covert decoy injection on eligible traffic.',
    maze_seed_provider: 'Seed corpus source: internal or operator.',
    maze_seed_refresh_interval_seconds: 'Scheduled seed refresh interval.',
    maze_seed_refresh_rate_limit_per_hour: 'Maximum seed refresh operations per hour.',
    maze_seed_refresh_max_sources: 'Maximum accepted operator seed sources.',
    maze_seed_metadata_only: 'Restricts operator seed extraction to metadata/keywords.',
    robots_enabled: 'Enables robots.txt serving.',
    robots_block_ai_training: 'Adds robots directives to disallow training crawlers.',
    robots_block_ai_search: 'Adds robots directives to disallow AI search crawlers.',
    robots_allow_search_engines: 'Allows mainstream search engines in robots policy.',
    ai_policy_block_training: 'First-class AI policy alias for training-bot blocking.',
    ai_policy_block_search: 'First-class AI policy alias for AI-search-bot blocking.',
    ai_policy_allow_search_engines: 'First-class AI policy alias for mainstream search allowance.',
    robots_crawl_delay: 'Crawl-delay value emitted in robots.txt.',
    cdp_detection_enabled: 'Enables client CDP automation-signal collection and scoring.',
    cdp_auto_ban: 'Auto-bans only on strong CDP automation outcomes.',
    cdp_detection_threshold: 'CDP score threshold when hard automation checks are absent.',
    js_required_enforced: 'Requires valid js_verified cookie for normal request flow.',
    pow_enabled: 'Enables PoW in JS verification flow.',
    pow_difficulty: 'PoW difficulty (leading-zero bits).',
    pow_ttl_seconds: 'PoW seed lifetime in seconds.',
    challenge_enabled: 'Enables/disables challenge puzzle routing at the challenge escalation step.',
    challenge_transform_count: 'Challenge puzzle transform-option count.',
    challenge_risk_threshold: 'Botness threshold for challenge step-up routing.',
    challenge_risk_threshold_default: 'Default challenge threshold derived from environment seed.',
    botness_maze_threshold: 'Botness threshold for maze routing.',
    botness_maze_threshold_default: 'Default maze threshold derived from environment seed.',
    'botness_weights.js_required': 'Botness points for missing JS verification.',
    'botness_weights.geo_risk': 'Botness points for GEO risk match.',
    'botness_weights.rate_medium': 'Botness points at medium rate pressure.',
    'botness_weights.rate_high': 'Botness points at high rate pressure.',
    'botness_weights.maze_behavior': 'Botness points for suspicious maze traversal behavior.',
    'defence_modes.rate': 'Configured composability mode for rate module.',
    'defence_modes.geo': 'Configured composability mode for GEO module.',
    'defence_modes.js': 'Configured composability mode for JS module.',
    'defence_modes_effective.rate': 'Effective runtime rate mode after guardrails.',
    'defence_modes_effective.geo': 'Effective runtime GEO mode after guardrails.',
    'defence_modes_effective.js': 'Effective runtime JS mode after guardrails.',
    defence_mode_warnings: 'Runtime warnings emitted for conflicting defence-mode combinations.',
    'provider_backends.rate_limiter': 'Selected provider backend for rate limiting.',
    'provider_backends.ban_store': 'Selected provider backend for ban state.',
    'provider_backends.challenge_engine': 'Selected provider backend for challenge generation.',
    'provider_backends.maze_tarpit': 'Selected provider backend for maze/tarpit capability.',
    'provider_backends.fingerprint_signal': 'Selected provider backend for fingerprint signals.',
    edge_integration_mode: 'How external edge outcomes affect local routing: off, advisory, authoritative.',
    admin_config_write_enabled: 'Enables/disables admin API config writes.',
    pow_config_mutable: 'Controls whether PoW fields are editable in runtime admin config.',
    challenge_config_mutable: 'Controls whether challenge enable and transform fields are editable in runtime admin config.',
    botness_config_mutable: 'Controls whether botness thresholds/weights are editable in runtime admin config.',
    kv_store_fail_open: 'KV outage posture. true=fail-open, false=fail-closed.',
    https_enforced: 'Reject non-HTTPS requests when enabled.',
    forwarded_header_trust_configured: 'Indicates forwarded header trust secret is configured.',
    enterprise_multi_instance: 'Indicates enterprise multi-instance rollout posture is enabled.',
    enterprise_unsynced_state_exception_confirmed:
      'Explicit temporary attestation for local-only state under enterprise rollout.',
    enterprise_state_guardrail_warnings: 'Advisory guardrail warnings for enterprise rollout posture.',
    enterprise_state_guardrail_error: 'Blocking guardrail error for invalid enterprise rollout posture.',
    botness_signal_definitions: 'Reference catalog of scored and terminal botness signals.',
    'botness_signal_definitions.scored_signals': 'Scored signals used in cumulative botness routing.',
    'botness_signal_definitions.terminal_signals': 'Terminal signals that immediately enforce actions.'
  });

  const VAR_GROUP_DEFINITIONS = Object.freeze([
    {
      key: 'policy_runtime',
      title: 'Policy and Runtime Controls',
      matches: path => (
        path === 'test_mode' ||
        path === 'ban_duration' ||
        path.startsWith('ban_durations.') ||
        path === 'rate_limit' ||
        path === 'admin_config_write_enabled' ||
        path === 'pow_config_mutable' ||
        path === 'challenge_config_mutable' ||
        path === 'botness_config_mutable' ||
        path === 'kv_store_fail_open' ||
        path === 'https_enforced' ||
        path === 'forwarded_header_trust_configured'
      )
    },
    {
      key: 'risk_challenge',
      title: 'Risk Scoring and Challenge',
      matches: path => (
        path === 'js_required_enforced' ||
        path.startsWith('pow_') ||
        path.startsWith('challenge_') ||
        path.startsWith('botness_')
      )
    },
    {
      key: 'signals_bypass',
      title: 'Signals and Bypass Lists',
      matches: path => (
        path === 'honeypots' ||
        path.startsWith('browser_') ||
        path === 'whitelist' ||
        path === 'path_whitelist' ||
        path.startsWith('geo_') ||
        path.startsWith('cdp_')
      )
    },
    {
      key: 'maze_runtime',
      title: 'Maze Runtime',
      matches: path => path.startsWith('maze_')
    },
    {
      key: 'crawler_policy',
      title: 'Crawler and AI Policy',
      matches: path => path.startsWith('robots_') || path.startsWith('ai_policy_')
    },
    {
      key: 'provider_edge',
      title: 'Provider and Edge Integration',
      matches: path => (
        path.startsWith('defence_modes') ||
        path.startsWith('provider_backends') ||
        path === 'edge_integration_mode' ||
        path === 'defence_mode_warnings'
      )
    },
    {
      key: 'enterprise_guardrails',
      title: 'Enterprise Guardrails',
      matches: path => path.startsWith('enterprise_')
    },
    {
      key: 'signal_taxonomy',
      title: 'Signal Taxonomy',
      matches: path => path.startsWith('botness_signal_definitions')
    }
  ]);

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

  function escapeHtml(value) {
    return String(value ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function cloneConfigSnapshot(configSnapshot) {
    if (!configSnapshot || typeof configSnapshot !== 'object') return {};
    try {
      return JSON.parse(JSON.stringify(configSnapshot));
    } catch (_e) {
      return {};
    }
  }

  function flattenConfigEntries(value, prefix = '') {
    if (value === null || value === undefined) {
      return [{ path: prefix, value: value === undefined ? null : value }];
    }
    if (Array.isArray(value)) {
      return [{ path: prefix, value }];
    }
    if (typeof value !== 'object') {
      return [{ path: prefix, value }];
    }
    const keys = Object.keys(value).sort();
    if (keys.length === 0) {
      return prefix ? [{ path: prefix, value: {} }] : [];
    }
    const entries = [];
    keys.forEach((key) => {
      const nextPath = prefix ? `${prefix}.${key}` : key;
      const child = value[key];
      if (child && typeof child === 'object' && !Array.isArray(child)) {
        entries.push(...flattenConfigEntries(child, nextPath));
        return;
      }
      entries.push({ path: nextPath, value: child });
    });
    return entries;
  }

  function classifyVarPath(path) {
    return WRITABLE_VAR_PATHS.has(path) ? 'ADMIN_WRITE' : 'READ_ONLY';
  }

  function classifyVarGroup(path) {
    const matched = VAR_GROUP_DEFINITIONS.find(group => group.matches(path));
    if (matched) return matched;
    return {
      key: 'other',
      title: 'Other Runtime Variables'
    };
  }

  function formatVarValue(value) {
    if (value === null) return 'null';
    if (Array.isArray(value)) return JSON.stringify(value);
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  }

  function humanizeVarPath(path) {
    return path
      .replace(/\./g, ' ')
      .replace(/_/g, ' ')
      .replace(/\b[a-z]/g, char => char.toUpperCase());
  }

  function meaningForVarPath(path) {
    if (Object.prototype.hasOwnProperty.call(VAR_MEANINGS, path)) {
      return VAR_MEANINGS[path];
    }
    return `${humanizeVarPath(path)} runtime value. See docs/configuration.md for canonical definition.`;
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
        `Step-up routing sends suspicious traffic to the puzzle challenge when ${envVar('SHUMA_CHALLENGE_ENABLED')} is true and cumulative botness reaches ${envVar('SHUMA_CHALLENGE_RISK_THRESHOLD')} ` +
        `(enabled: <strong>${boolStatus(snapshot.challengeEnabled)}</strong>, current: <strong>${snapshot.challengeThreshold}</strong>). ` +
        `Puzzle complexity is controlled by ${envVar('SHUMA_CHALLENGE_TRANSFORM_COUNT')}. ` +
        `Runtime threshold mutability is controlled by ${envVar('SHUMA_CHALLENGE_CONFIG_MUTABLE')} / ${envVar('SHUMA_BOTNESS_CONFIG_MUTABLE')} and is currently ${formatMutability(snapshot.challengeMutable || snapshot.botnessMutable)}.`
      ),
      status: snapshot => boolStatus(snapshot.challengeEnabled)
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
    if (Object.prototype.hasOwnProperty.call(patch, 'configSnapshot')) {
      state.configSnapshot = cloneConfigSnapshot(patch.configSnapshot);
    }
    if (patch.botnessWeights && typeof patch.botnessWeights === 'object') {
      state.botnessWeights = {
        ...state.botnessWeights,
        ...patch.botnessWeights
      };
    }
    Object.keys(patch).forEach((key) => {
      if (key === 'botnessWeights' || key === 'configSnapshot') return;
      if (Object.prototype.hasOwnProperty.call(state, key)) {
        state[key] = patch[key];
      }
    });
    return getState();
  }

  function getState() {
    return {
      ...state,
      botnessWeights: { ...state.botnessWeights },
      configSnapshot: cloneConfigSnapshot(state.configSnapshot)
    };
  }

  function renderFeatureStatus(snapshot) {
    const container = document.getElementById('status-items');
    if (!container) return;
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

  function renderVariableInventory(snapshot) {
    const groupsContainer = document.getElementById('status-vars-groups');
    if (!groupsContainer) return;
    const flattened = flattenConfigEntries(snapshot.configSnapshot || {})
      .filter(entry => entry.path && entry.path.length > 0)
      .map((entry) => {
        const valueClass = classifyVarPath(entry.path);
        return {
          ...entry,
          valueClass,
          group: classifyVarGroup(entry.path)
        };
      })
      .sort((a, b) => {
        if (a.group.key !== b.group.key) {
          const groupOrder = VAR_GROUP_DEFINITIONS
            .map(group => group.key)
            .concat(['other']);
          return groupOrder.indexOf(a.group.key) - groupOrder.indexOf(b.group.key);
        }
        if (a.valueClass !== b.valueClass) {
          return a.valueClass === 'ADMIN_WRITE' ? -1 : 1;
        }
        return a.path.localeCompare(b.path);
      });

    if (flattened.length === 0) {
      groupsContainer.innerHTML = `
        <p class="text-muted">No configuration snapshot loaded yet.</p>
      `;
      return;
    }

    const grouped = new Map();
    flattened.forEach((entry) => {
      if (!grouped.has(entry.group.key)) {
        grouped.set(entry.group.key, {
          title: entry.group.title,
          entries: []
        });
      }
      grouped.get(entry.group.key).entries.push(entry);
    });

    const orderedGroupKeys = VAR_GROUP_DEFINITIONS
      .map(group => group.key)
      .concat(['other'])
      .filter(key => grouped.has(key));

    groupsContainer.innerHTML = orderedGroupKeys.map((groupKey) => {
      const group = grouped.get(groupKey);
      const rows = group.entries.map(entry => `
        <tr class="status-var-row ${entry.valueClass === 'ADMIN_WRITE' ? 'status-var-row--admin-write' : ''}">
          <td><code>${escapeHtml(entry.path)}</code></td>
          <td><code>${escapeHtml(formatVarValue(entry.value))}</code></td>
          <td>${escapeHtml(meaningForVarPath(entry.path))}</td>
        </tr>
      `).join('');
      return `
        <section class="status-var-group">
          <h4 class="status-var-group-title">${escapeHtml(group.title)}</h4>
          <table class="status-vars-table">
            <colgroup>
              <col class="status-vars-col status-vars-col--variable" />
              <col class="status-vars-col status-vars-col--value" />
              <col class="status-vars-col status-vars-col--meaning" />
            </colgroup>
            <thead>
              <tr>
                <th scope="col">Variable</th>
                <th scope="col">Current Value</th>
                <th scope="col">Meaning</th>
              </tr>
            </thead>
            <tbody>
              ${rows}
            </tbody>
          </table>
        </section>
      `;
    }).join('');
  }

  function render() {
    const snapshot = getState();
    renderFeatureStatus(snapshot);
    renderVariableInventory(snapshot);
  }

  global.ShumaDashboardStatus = {
    update,
    getState,
    render,
    normalizeFailMode
  };
})(window);
