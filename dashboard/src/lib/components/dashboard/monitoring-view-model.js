import { formatCompactNumber } from '../../domain/core/format.js';
import {
  classifyIpRangeFallback,
  formatIpRangeReasonLabel,
  isIpRangeReason,
  parseIpRangeOutcome
} from '../../domain/ip-range-policy.js';

const CHALLENGE_REASON_LABELS = Object.freeze({
  incorrect: 'Incorrect',
  expired_replay: 'Expired/Replay',
  sequence_violation: 'Sequence Violation',
  invalid_output: 'Invalid Output',
  forbidden: 'Forbidden'
});

const POW_REASON_LABELS = Object.freeze({
  invalid_proof: 'Invalid Proof',
  missing_seed_nonce: 'Missing Seed/Nonce',
  sequence_violation: 'Sequence Violation',
  expired_replay: 'Expired/Replay',
  binding_timing_mismatch: 'Binding/Timing Mismatch'
});

const RATE_OUTCOME_LABELS = Object.freeze({
  limited: 'Limited',
  banned: 'Banned',
  fallback_allow: 'Fallback Allow',
  fallback_deny: 'Fallback Deny'
});

const NOT_A_BOT_OUTCOME_LABELS = Object.freeze({
  pass: 'Pass',
  escalate: 'Escalate',
  fail: 'Fail',
  replay: 'Replay'
});

const NOT_A_BOT_LATENCY_LABELS = Object.freeze({
  lt_1s: '<1s',
  '1_3s': '1-3s',
  '3_10s': '3-10s',
  '10s_plus': '10s+'
});

const IP_RANGE_SOURCE_LABELS = Object.freeze({
  custom: 'Custom Rule',
  managed: 'Managed Set',
  unknown: 'Unknown'
});

const IP_RANGE_ACTION_LABELS = Object.freeze({
  forbidden_403: '403 Forbidden',
  custom_message: 'Custom Message',
  drop_connection: 'Drop Connection',
  redirect_308: '308 Redirect',
  rate_limit: 'Rate Limit',
  honeypot: 'Honeypot',
  maze: 'Maze',
  tarpit: 'Tarpit',
  unknown: 'Unknown'
});

const IP_RANGE_FALLBACK_LABELS = Object.freeze({
  none: 'Direct',
  challenge: 'Fallback Challenge',
  maze: 'Fallback Maze',
  block: 'Fallback Block',
  block_missing_redirect: 'Block (Missing Redirect URL)'
});

const toNonNegativeNumber = (value) => {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric < 0) return 0;
  return numeric;
};

const normalizeOffenderBucketLabel = (rawLabel) => {
  const label = String(rawLabel || '').trim();
  if (!label) return 'untrusted/unknown';
  if (label.toLowerCase() === 'unknown') return 'untrusted/unknown';
  if (/^h\d+$/i.test(label)) return 'untrusted/unknown';
  return label;
};

const formatUnitLabel = (count, singular, plural) => (count === 1 ? singular : plural);

const deriveTopOffenderViewModel = (rawLabel, rawCount, singularUnit, pluralUnit) => {
  const label = String(rawLabel || '').trim();
  const count = Number(rawCount || 0);
  if (!label || !Number.isFinite(count) || count <= 0) {
    return {
      value: 'None',
      label: 'Top Offender'
    };
  }
  const normalizedLabel = normalizeOffenderBucketLabel(label);
  const unit = formatUnitLabel(count, singularUnit, pluralUnit);
  return {
    value: normalizedLabel,
    label: `Top Offender (${formatCompactNumber(count, '0')} ${unit})`
  };
};

const formatTrendTimestamp = (ts) => {
  if (!Number.isFinite(ts)) return '-';
  return new Date(ts * 1000).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric'
  });
};

const sortCountEntries = (source) =>
  Object.entries(source && typeof source === 'object' ? source : {})
    .sort((left, right) => Number(right[1] || 0) - Number(left[1] || 0));

const deriveTrendSeries = (trend = []) => {
  const points = Array.isArray(trend) ? trend : [];
  return {
    labels: points.map((point) => formatTrendTimestamp(Number(point.ts || 0))),
    data: points.map((point) => Number(point.total || 0))
  };
};

const incrementCount = (target, key, amount = 1) => {
  const normalizedKey = String(key || '').trim() || 'unknown';
  target[normalizedKey] = Number(target[normalizedKey] || 0) + Number(amount || 0);
};

const toSortedCountEntries = (target) =>
  Object.entries(target && typeof target === 'object' ? target : {})
    .sort((left, right) => Number(right[1] || 0) - Number(left[1] || 0));

const normalizeMode = (value) => {
  const mode = String(value || '').trim().toLowerCase();
  if (mode === 'advisory' || mode === 'enforce' || mode === 'off') return mode;
  return 'off';
};

const TREND_POINT_LIMIT = 24;

export const deriveIpRangeMonitoringViewModel = (
  events = [],
  configSnapshot = {},
  nowUnix = Math.floor(Date.now() / 1000)
) => {
  const rows = Array.isArray(events) ? events : [];
  const config = configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {};

  const reasonCounts = {};
  const sourceCounts = {};
  const actionCounts = {};
  const detectionCounts = {};
  const sourceIdCounts = {};
  const fallbackCounts = {};
  const trendBuckets = {};
  let totalMatches = 0;

  rows.forEach((entry) => {
    const reason = String(entry?.reason || '').trim().toLowerCase();
    if (!isIpRangeReason(reason)) return;
    totalMatches += 1;

    const parsed = parseIpRangeOutcome(entry?.outcome);
    const source = String(parsed.source || 'unknown').toLowerCase();
    const action = String(parsed.action || 'unknown').toLowerCase();
    const detection = String(parsed.detection || 'unknown').toUpperCase();
    const sourceId = String(parsed.sourceId || 'unknown').toLowerCase();
    const fallback = classifyIpRangeFallback(reason, parsed);

    incrementCount(reasonCounts, reason);
    incrementCount(sourceCounts, source);
    incrementCount(actionCounts, action);
    incrementCount(detectionCounts, detection);
    incrementCount(sourceIdCounts, sourceId);
    incrementCount(fallbackCounts, fallback);

    const ts = Number(entry?.ts || 0);
    if (Number.isFinite(ts) && ts > 0) {
      const hour = Math.floor(ts / 3600) * 3600;
      trendBuckets[hour] = Number(trendBuckets[hour] || 0) + 1;
    }
  });

  const sortedHours = Object.keys(trendBuckets)
    .map((rawHour) => Number(rawHour))
    .filter((hour) => Number.isFinite(hour))
    .sort((left, right) => left - right);
  const trendStartIndex = Math.max(0, sortedHours.length - TREND_POINT_LIMIT);
  const trendHours = sortedHours.slice(trendStartIndex);
  const trend = {
    labels: trendHours.map((hour) => formatTrendTimestamp(hour)),
    data: trendHours.map((hour) => Number(trendBuckets[hour] || 0))
  };

  const managedSets = Array.isArray(config.ip_range_managed_sets)
    ? config.ip_range_managed_sets
    : [];
  const managedPolicies = Array.isArray(config.ip_range_managed_policies)
    ? config.ip_range_managed_policies
    : [];
  const customRules = Array.isArray(config.ip_range_custom_rules)
    ? config.ip_range_custom_rules
    : [];
  const emergencyAllowlist = Array.isArray(config.ip_range_emergency_allowlist)
    ? config.ip_range_emergency_allowlist
    : [];
  const managedStaleCount = managedSets.filter((set) => set?.stale === true).length;
  const managedMaxStalenessHours = toNonNegativeNumber(config.ip_range_managed_max_staleness_hours);
  const generatedAtUnix = toNonNegativeNumber(config.ip_range_managed_catalog_generated_at_unix);
  const catalogAgeHours = generatedAtUnix > 0 && nowUnix >= generatedAtUnix
    ? Math.floor((nowUnix - generatedAtUnix) / 3600)
    : null;
  const staleByAge = Number.isFinite(catalogAgeHours) && managedMaxStalenessHours > 0
    ? Number(catalogAgeHours) > managedMaxStalenessHours
    : false;
  const catalogStale = managedStaleCount > 0 || staleByAge;

  return {
    mode: normalizeMode(config.ip_range_policy_mode),
    totalMatches,
    totalFallbacks: toNonNegativeNumber(totalMatches - Number(fallbackCounts.none || 0)),
    uniqueSourceIds: Object.keys(sourceIdCounts).filter((key) => key !== 'unknown').length,
    reasons: toSortedCountEntries(reasonCounts),
    sources: toSortedCountEntries(sourceCounts),
    actions: toSortedCountEntries(actionCounts),
    detections: toSortedCountEntries(detectionCounts),
    sourceIds: toSortedCountEntries(sourceIdCounts),
    fallbacks: toSortedCountEntries(fallbackCounts),
    trend,
    catalog: {
      version: String(config.ip_range_managed_catalog_version || '-'),
      generatedAt: String(config.ip_range_managed_catalog_generated_at || '-'),
      ageHours: Number.isFinite(catalogAgeHours) ? Number(catalogAgeHours) : null,
      stale: catalogStale,
      managedSets: managedSets.map((set) => ({
        setId: String(set?.set_id || '-'),
        provider: String(set?.provider || '-'),
        version: String(set?.version || '-'),
        generatedAt: String(set?.generated_at || '-'),
        entryCount: toNonNegativeNumber(set?.entry_count),
        stale: set?.stale === true
      })),
      managedSetCount: managedSets.length,
      managedSetStaleCount: managedStaleCount,
      managedPolicyCount: managedPolicies.length,
      managedPolicyEnabledCount: managedPolicies.filter((policy) => policy?.enabled === true).length,
      customRuleCount: customRules.length,
      customRuleEnabledCount: customRules.filter((rule) => rule?.enabled === true).length,
      emergencyAllowlistCount: emergencyAllowlist.length,
      managedMaxStalenessHours,
      allowStaleManagedEnforce: config.ip_range_allow_stale_managed_enforce === true
    }
  };
};

export const formatMetricLabel = (key, fallbackMap) => {
  if (fallbackMap && fallbackMap[key]) return fallbackMap[key];
  return String(key || '-')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase());
};

export const deriveMazeStatsViewModel = (data = {}) => {
  const topCrawler =
    Array.isArray(data.top_crawlers) && data.top_crawlers.length ? data.top_crawlers[0] : null;
  return {
    totalHits: formatCompactNumber(data.total_hits, '0'),
    uniqueCrawlers: formatCompactNumber(data.unique_crawlers, '0'),
    mazeAutoBans: formatCompactNumber(data.maze_auto_bans, '0'),
    topOffender: deriveTopOffenderViewModel(
      topCrawler?.ip,
      topCrawler?.hits,
      'page',
      'pages'
    )
  };
};

export const deriveMonitoringSummaryViewModel = (summary = {}) => {
  const honeypot = summary.honeypot || {};
  const challenge = summary.challenge || {};
  const notABot = summary.not_a_bot || {};
  const pow = summary.pow || {};
  const rate = summary.rate || {};
  const geo = summary.geo || {};
  const honeypotTopPaths = Array.isArray(honeypot.top_paths)
    ? honeypot.top_paths.map((entry) => ({
      path: String(
        Array.isArray(entry)
          ? (entry[0] ?? '')
          : (entry?.path ?? entry?.label ?? '')
      ),
      count: toNonNegativeNumber(Array.isArray(entry) ? entry[1] : entry?.count)
    }))
    : [];
  const geoTopCountries = Array.isArray(geo.top_countries)
    ? geo.top_countries.map((entry) => ({
      country: String(
        Array.isArray(entry)
          ? (entry[0] ?? '')
          : (entry?.country ?? entry?.label ?? '')
      ),
      count: toNonNegativeNumber(Array.isArray(entry) ? entry[1] : entry?.count)
    }))
    : [];

  const topHoneypotCrawler =
    Array.isArray(honeypot.top_crawlers) && honeypot.top_crawlers.length
      ? honeypot.top_crawlers[0]
      : null;
  const topChallengeOffender =
    Array.isArray(challenge.top_offenders) && challenge.top_offenders.length
      ? challenge.top_offenders[0]
      : null;
  const topPowOffender =
    Array.isArray(pow.top_offenders) && pow.top_offenders.length
      ? pow.top_offenders[0]
      : null;
  const topRateOffender =
    Array.isArray(rate.top_offenders) && rate.top_offenders.length
      ? rate.top_offenders[0]
      : null;
  const powFailureTotal = toNonNegativeNumber(pow.total_failures);
  const powSuccessTotal = toNonNegativeNumber(pow.total_successes);
  const powAttemptFallback = powFailureTotal + powSuccessTotal;
  const powAttemptsTotal = Math.max(powAttemptFallback, toNonNegativeNumber(pow.total_attempts));
  const powRatioRaw = Number(pow.success_ratio);
  const powSuccessRatio = Number.isFinite(powRatioRaw)
    ? Math.min(1, Math.max(0, powRatioRaw))
    : (powAttemptsTotal > 0 ? Math.min(1, Math.max(0, powSuccessTotal / powAttemptsTotal)) : 0);
  const notABotServed = toNonNegativeNumber(notABot.served);
  const notABotSubmitted = toNonNegativeNumber(notABot.submitted);
  const notABotPass = toNonNegativeNumber(notABot.pass);
  const notABotEscalate = toNonNegativeNumber(notABot.escalate);
  const notABotFail = toNonNegativeNumber(notABot.fail);
  const notABotReplay = toNonNegativeNumber(notABot.replay);
  const notABotAbandonments = toNonNegativeNumber(notABot.abandonments_estimated);
  const notABotAbandonmentRatioRaw = Number(notABot.abandonment_ratio);
  const notABotAbandonmentRatio = Number.isFinite(notABotAbandonmentRatioRaw)
    ? Math.min(1, Math.max(0, notABotAbandonmentRatioRaw))
    : (notABotServed > 0
      ? Math.min(1, Math.max(0, notABotAbandonments / notABotServed))
      : 0);

  return {
    honeypot: {
      totalHits: formatCompactNumber(honeypot.total_hits, '0'),
      uniqueCrawlers: formatCompactNumber(honeypot.unique_crawlers, '0'),
      topOffender: deriveTopOffenderViewModel(
        topHoneypotCrawler?.label,
        topHoneypotCrawler?.count,
        'hit',
        'hits'
      ),
      topPaths: honeypotTopPaths
    },
    challenge: {
      totalFailures: formatCompactNumber(challenge.total_failures, '0'),
      uniqueOffenders: formatCompactNumber(challenge.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topChallengeOffender?.label,
        topChallengeOffender?.count,
        'hit',
        'hits'
      ),
      reasons: sortCountEntries(challenge.reasons),
      trend: deriveTrendSeries(challenge.trend)
    },
    notABot: {
      served: formatCompactNumber(notABotServed, '0'),
      submitted: formatCompactNumber(notABotSubmitted, '0'),
      pass: formatCompactNumber(notABotPass, '0'),
      escalate: formatCompactNumber(notABotEscalate, '0'),
      fail: formatCompactNumber(notABotFail, '0'),
      replay: formatCompactNumber(notABotReplay, '0'),
      abandonmentsEstimated: formatCompactNumber(notABotAbandonments, '0'),
      abandonmentRate: `${(notABotAbandonmentRatio * 100).toFixed(1)}%`,
      outcomes: sortCountEntries(notABot.outcomes),
      latencyBuckets: sortCountEntries(notABot.solve_latency_buckets)
    },
    pow: {
      totalFailures: formatCompactNumber(powFailureTotal, '0'),
      totalSuccesses: formatCompactNumber(powSuccessTotal, '0'),
      totalAttempts: formatCompactNumber(powAttemptsTotal, '0'),
      successRatio: powSuccessRatio,
      successRate: `${(powSuccessRatio * 100).toFixed(1)}%`,
      uniqueOffenders: formatCompactNumber(pow.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topPowOffender?.label,
        topPowOffender?.count,
        'hit',
        'hits'
      ),
      reasons: sortCountEntries(pow.reasons),
      outcomes: sortCountEntries(pow.outcomes),
      trend: deriveTrendSeries(pow.trend)
    },
    rate: {
      totalViolations: formatCompactNumber(rate.total_violations, '0'),
      uniqueOffenders: formatCompactNumber(rate.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topRateOffender?.label,
        topRateOffender?.count,
        'hit',
        'hits'
      ),
      outcomes: sortCountEntries(rate.outcomes)
    },
    geo: {
      totalViolations: formatCompactNumber(geo.total_violations, '0'),
      actionMix: {
        block: formatCompactNumber(geo.actions?.block || 0, '0'),
        challenge: formatCompactNumber(geo.actions?.challenge || 0, '0'),
        maze: formatCompactNumber(geo.actions?.maze || 0, '0')
      },
      topCountries: geoTopCountries
    }
  };
};

export const derivePrometheusHelperViewModel = (prometheusData = {}, origin = '') => {
  const readString = (value) => (typeof value === 'string' ? value.trim() : '');
  const sanitizeExternalUrl = (value) => {
    const raw = readString(value);
    if (!/^https?:\/\//i.test(raw)) return '';
    try {
      const parsed = new URL(raw);
      if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return '';
      return parsed.href;
    } catch (_error) {
      return '';
    }
  };
  const endpoint =
    typeof prometheusData.endpoint === 'string' ? prometheusData.endpoint : '/metrics';
  const docs =
    prometheusData && typeof prometheusData.docs === 'object' ? prometheusData.docs : {};
  const notes = Array.isArray(prometheusData?.notes)
    ? prometheusData.notes.map(readString).filter((entry) => entry.length > 0)
    : [];
  const fallbackFacts = ['Monitoring guidance unavailable; see docs links below.'];
  const siteOrigin = origin || 'http://127.0.0.1:3000';

  return {
    exampleJs: readString(prometheusData?.example_js) || '// Example unavailable.',
    copyCurlText: `curl -sS '${siteOrigin}${endpoint}'`,
    facts: notes.length ? notes : fallbackFacts,
    exampleOutput: readString(prometheusData?.example_output) || '# Example unavailable.',
    exampleStats: readString(prometheusData?.example_stats) || '// Example unavailable.',
    exampleWindowed: readString(prometheusData?.example_windowed) || '// Example unavailable.',
    exampleSummaryStats:
      readString(prometheusData?.example_summary_stats) || '// Example unavailable.',
    observabilityLink: sanitizeExternalUrl(docs.observability),
    apiLink: sanitizeExternalUrl(docs.api)
  };
};

export {
  CHALLENGE_REASON_LABELS,
  IP_RANGE_ACTION_LABELS,
  IP_RANGE_FALLBACK_LABELS,
  IP_RANGE_SOURCE_LABELS,
  NOT_A_BOT_OUTCOME_LABELS,
  NOT_A_BOT_LATENCY_LABELS,
  POW_REASON_LABELS,
  RATE_OUTCOME_LABELS,
  normalizeOffenderBucketLabel
};
