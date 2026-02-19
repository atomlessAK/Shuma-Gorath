// @ts-check

import { writableStatusVarPaths } from './config-schema.js';

const INITIAL_STATE = Object.freeze({
    failMode: 'unknown',
    httpsEnforced: false,
    forwardedHeaderTrustConfigured: false,
    testMode: false,
    powEnabled: false,
    mazeEnabled: false,
    mazeAutoBan: false,
    cdpEnabled: false,
    cdpAutoBan: false,
    jsRequiredEnforced: true,
    challengeEnabled: true,
    challengeThreshold: 3,
    mazeThreshold: 6,
    rateLimit: 80,
    geoRiskCount: 0,
    geoAllowCount: 0,
    geoChallengeCount: 0,
    geoMazeCount: 0,
    geoBlockCount: 0,
    botnessWeights: {
      js_required: 1,
      geo_risk: 2,
      rate_medium: 1,
      rate_high: 2
    },
    configSnapshot: {}
  });

const createInitialState = () => ({
    ...INITIAL_STATE,
    botnessWeights: { ...INITIAL_STATE.botnessWeights },
    configSnapshot: cloneConfigSnapshot(INITIAL_STATE.configSnapshot)
  });

const WRITABLE_VAR_PATHS = new Set(writableStatusVarPaths || []);

const VAR_MEANINGS = Object.freeze({
    test_mode: 'Logs detections/actions without enforcing blocks.',
    ban_duration: 'Legacy fallback ban duration (seconds) when no specific trigger duration applies.',
    'ban_durations.honeypot': 'Ban duration (seconds) for honeypot/instaban trigger.',
    'ban_durations.rate_limit': 'Ban duration (seconds) for rate-limit enforcement.',
    'ban_durations.browser': 'Ban duration (seconds) for browser-policy automation detections.',
    'ban_durations.admin': 'Default ban duration (seconds) for manual admin bans.',
    'ban_durations.cdp': 'Ban duration (seconds) for strong CDP automation detections.',
    rate_limit: 'Requests-per-minute threshold used by rate limiting.',
    honeypot_enabled: 'Enables/disables honeypot trap handling and enforcement for configured honeypot paths.',
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
    cdp_probe_family: 'Active CDP probe family (v1, v2, split) for detector-surface rotation.',
    cdp_probe_rollout_percent:
      'Percentage of traffic receiving probe family v2 when cdp_probe_family=split.',
    fingerprint_signal_enabled: 'Enables/disables internal fingerprint signal collection.',
    fingerprint_state_ttl_seconds: 'Retention window for per-identity fingerprint coherence state.',
    fingerprint_flow_window_seconds: 'Window size for flow-centric mismatch aggregation.',
    fingerprint_flow_violation_threshold:
      'Mismatch count threshold within the flow window before flow violation is raised.',
    fingerprint_pseudonymize:
      'When true, stores fingerprint state under pseudonymous identity keys rather than raw IP.',
    fingerprint_entropy_budget:
      'Total botness contribution cap for all fingerprint-family signals combined.',
    fingerprint_family_cap_header_runtime:
      'Per-family cap for UA/client-hint/runtime mismatch fingerprint contributions.',
    fingerprint_family_cap_transport:
      'Per-family cap for transport and trusted-header fingerprint contributions.',
    fingerprint_family_cap_temporal:
      'Per-family cap for temporal coherence and impossible-transition fingerprint contributions.',
    fingerprint_family_cap_persistence:
      'Per-family cap for persistence-abuse fingerprint contributions.',
    fingerprint_family_cap_behavior:
      'Per-family cap for flow/behavioral fingerprint contributions.',
    js_required_enforced: 'Requires valid js_verified cookie for normal request flow.',
    pow_enabled: 'Enables PoW in JS verification flow.',
    pow_difficulty: 'PoW difficulty (leading-zero bits).',
    pow_ttl_seconds: 'PoW seed lifetime in seconds.',
    challenge_puzzle_enabled: 'Enables/disables challenge puzzle routing at the challenge escalation step.',
    challenge_puzzle_transform_count: 'Challenge puzzle transform-option count.',
    challenge_puzzle_risk_threshold: 'Botness threshold for challenge step-up routing.',
    challenge_puzzle_risk_threshold_default: 'Default challenge threshold derived from environment seed.',
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
        path.startsWith('cdp_') ||
        path.startsWith('fingerprint_')
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

export function normalizeFailMode(value) {
    const mode = (value || 'unknown').toString().toLowerCase();
    if (mode === 'open' || mode === 'closed') return mode;
    return 'unknown';
  }

function boolStatus(enabled) {
    return enabled ? 'ENABLED' : 'DISABLED';
  }

function cloneConfigSnapshot(configSnapshot) {
    if (!configSnapshot || typeof configSnapshot !== 'object') return {};
    try {
      return JSON.parse(JSON.stringify(configSnapshot));
    } catch (_e) {
      return {};
    }
  }

function parseBoolLike(value, fallback) {
    if (typeof value === 'boolean') return value;
    const normalized = String(value || '').trim().toLowerCase();
    if (!normalized) return fallback;
    if (normalized === 'true' || normalized === '1' || normalized === 'yes' || normalized === 'on') return true;
    if (normalized === 'false' || normalized === '0' || normalized === 'no' || normalized === 'off') return false;
    return fallback;
  }

function parseIntegerLike(value, fallback) {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? parsed : fallback;
  }

function listCount(value) {
    return Array.isArray(value) ? value.length : 0;
  }

export function deriveStatusSnapshot(configSnapshot = {}) {
    const config = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};
    const base = createInitialState();
    const botnessWeights = config.botness_weights && typeof config.botness_weights === 'object'
      ? config.botness_weights
      : {};
    return {
      ...base,
      failMode: parseBoolLike(config.kv_store_fail_open, true) ? 'open' : 'closed',
      httpsEnforced: parseBoolLike(config.https_enforced, false),
      forwardedHeaderTrustConfigured: parseBoolLike(config.forwarded_header_trust_configured, false),
      testMode: parseBoolLike(config.test_mode, false),
      powEnabled: parseBoolLike(config.pow_enabled, true),
      mazeEnabled: parseBoolLike(config.maze_enabled, true),
      mazeAutoBan: parseBoolLike(config.maze_auto_ban, true),
      cdpEnabled: parseBoolLike(config.cdp_detection_enabled, true),
      cdpAutoBan: parseBoolLike(config.cdp_auto_ban, true),
      jsRequiredEnforced: parseBoolLike(config.js_required_enforced, true),
      challengeEnabled: parseBoolLike(config.challenge_puzzle_enabled, true),
      challengeThreshold: parseIntegerLike(
        config.challenge_puzzle_risk_threshold,
        base.challengeThreshold
      ),
      mazeThreshold: parseIntegerLike(config.botness_maze_threshold, base.mazeThreshold),
      rateLimit: parseIntegerLike(config.rate_limit, base.rateLimit),
      geoRiskCount: listCount(config.geo_risk),
      geoAllowCount: listCount(config.geo_allow),
      geoChallengeCount: listCount(config.geo_challenge),
      geoMazeCount: listCount(config.geo_maze),
      geoBlockCount: listCount(config.geo_block),
      botnessWeights: {
        js_required: parseIntegerLike(botnessWeights.js_required, base.botnessWeights.js_required),
        geo_risk: parseIntegerLike(botnessWeights.geo_risk, base.botnessWeights.geo_risk),
        rate_medium: parseIntegerLike(botnessWeights.rate_medium, base.botnessWeights.rate_medium),
        rate_high: parseIntegerLike(botnessWeights.rate_high, base.botnessWeights.rate_high)
      },
      configSnapshot: cloneConfigSnapshot(config)
    };
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
        `Runtime updates are available only when ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} is enabled. ` +
        `If ${envVar('SHUMA_JS_REQUIRED_ENFORCED')} is disabled, normal visitor requests bypass this flow.`
      ),
      status: snapshot => boolStatus(snapshot.powEnabled)
    },
    {
      title: 'Challenge',
      description: snapshot => (
        `Step-up routing sends suspicious traffic to the puzzle challenge when ${envVar('SHUMA_CHALLENGE_PUZZLE_ENABLED')} is true and cumulative botness reaches ${envVar('SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD')} ` +
        `(enabled: <strong>${boolStatus(snapshot.challengeEnabled)}</strong>, current: <strong>${snapshot.challengeThreshold}</strong>). ` +
        `Puzzle complexity is controlled by ${envVar('SHUMA_CHALLENGE_PUZZLE_TRANSFORM_COUNT')}. ` +
        `Runtime updates are available only when ${envVar('SHUMA_ADMIN_CONFIG_WRITE_ENABLED')} is enabled.`
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

export function buildFeatureStatusItems(snapshot) {
    return STATUS_DEFINITIONS.map((definition) => ({
      title: definition.title,
      description: definition.description(snapshot),
      status: definition.status(snapshot)
    }));
  }

export function buildVariableInventoryGroups(snapshot) {
    const flattened = flattenConfigEntries(snapshot.configSnapshot || {})
      .filter((entry) => entry.path && entry.path.length > 0)
      .map((entry) => {
        const valueClass = classifyVarPath(entry.path);
        return {
          path: entry.path,
          valueClass,
          group: classifyVarGroup(entry.path),
          valueText: formatVarValue(entry.value),
          meaning: meaningForVarPath(entry.path),
          isAdminWrite: valueClass === 'ADMIN_WRITE'
        };
      })
      .sort((a, b) => {
        if (a.group.key !== b.group.key) {
          const groupOrder = VAR_GROUP_DEFINITIONS.map((group) => group.key).concat(['other']);
          return groupOrder.indexOf(a.group.key) - groupOrder.indexOf(b.group.key);
        }
        if (a.valueClass !== b.valueClass) {
          return a.valueClass === 'ADMIN_WRITE' ? -1 : 1;
        }
        return a.path.localeCompare(b.path);
      });

    if (flattened.length === 0) {
      return [];
    }

    const grouped = new Map();
    flattened.forEach((entry) => {
      if (!grouped.has(entry.group.key)) {
        grouped.set(entry.group.key, {
          key: entry.group.key,
          title: entry.group.title,
          entries: []
        });
      }
      grouped.get(entry.group.key).entries.push(entry);
    });

    const orderedGroupKeys = VAR_GROUP_DEFINITIONS
      .map((group) => group.key)
      .concat(['other'])
      .filter((key) => grouped.has(key));
    return orderedGroupKeys.map((key) => grouped.get(key));
  }
