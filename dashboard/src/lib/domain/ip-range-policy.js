// @ts-check

const IP_RANGE_REASON_PREFIX = 'ip_range_';

const IP_RANGE_REASON_LABELS = Object.freeze({
  ip_range_emergency_allowlist: 'Emergency Allowlist',
  ip_range_policy_advisory: 'Advisory Match',
  ip_range_policy_forbidden: '403 Forbidden',
  ip_range_policy_custom_message: 'Custom Message',
  ip_range_policy_drop_connection: 'Drop Connection',
  ip_range_policy_redirect: '308 Redirect',
  ip_range_policy_redirect_missing_url: 'Redirect Missing URL',
  ip_range_policy_rate_limit: 'Rate Limited',
  ip_range_policy_honeypot: 'Honeypot Ban',
  ip_range_policy_maze: 'Maze',
  ip_range_policy_maze_fallback_challenge: 'Maze Fallback Challenge',
  ip_range_policy_maze_fallback_block: 'Maze Fallback Block',
  ip_range_policy_tarpit: 'Tarpit',
  ip_range_policy_tarpit_fallback_maze: 'Tarpit Fallback Maze',
  ip_range_policy_tarpit_fallback_block: 'Tarpit Fallback Block'
});

const sanitizeText = (value) => String(value || '').trim();

const parseKeyValuePairs = (text) => {
  const source = String(text || '');
  const pairs = {};
  const matcher = /([a-z_]+)=([^\s\]]+)/gi;
  let match = matcher.exec(source);
  while (match) {
    const key = String(match[1] || '').trim().toLowerCase();
    const value = sanitizeText(match[2]);
    if (key && value) {
      pairs[key] = value;
    }
    match = matcher.exec(source);
  }
  return pairs;
};

const normalizeSignals = (rawSignals) => {
  const source = sanitizeText(rawSignals);
  if (!source) return [];
  return source
    .split(',')
    .map((entry) => sanitizeText(entry))
    .filter((entry) => entry.length > 0);
};

export const isIpRangeReason = (reason) =>
  sanitizeText(reason).toLowerCase().startsWith(IP_RANGE_REASON_PREFIX);

export const formatIpRangeReasonLabel = (reason) => {
  const key = sanitizeText(reason).toLowerCase();
  if (Object.prototype.hasOwnProperty.call(IP_RANGE_REASON_LABELS, key)) {
    return IP_RANGE_REASON_LABELS[key];
  }
  if (!key) return '-';
  return key
    .replace(/_/g, ' ')
    .replace(/\b[a-z]/g, (char) => char.toUpperCase());
};

export const parseIpRangeOutcome = (outcome) => {
  const rawOutcome = sanitizeText(outcome);
  const taxonomyMatch = /taxonomy\[([^\]]+)\]/i.exec(rawOutcome);
  const taxonomyPairs = parseKeyValuePairs(taxonomyMatch ? taxonomyMatch[1] : '');
  const outcomeWithoutTaxonomy = taxonomyMatch
    ? rawOutcome.replace(taxonomyMatch[0], '').trim()
    : rawOutcome;
  const outcomePairs = parseKeyValuePairs(outcomeWithoutTaxonomy);
  return {
    source: sanitizeText(outcomePairs.source),
    sourceId: sanitizeText(outcomePairs.source_id),
    action: sanitizeText(outcomePairs.action),
    matchedCidr: sanitizeText(outcomePairs.matched_cidr),
    fallback: sanitizeText(outcomePairs.fallback),
    location: sanitizeText(outcomePairs.location),
    detection: sanitizeText(taxonomyPairs.detection),
    level: sanitizeText(taxonomyPairs.level),
    actionId: sanitizeText(taxonomyPairs.action),
    signals: normalizeSignals(taxonomyPairs.signals),
    rawOutcome
  };
};

export const classifyIpRangeFallback = (reason, parsedOutcome = {}) => {
  const fallback = sanitizeText(parsedOutcome.fallback).toLowerCase();
  if (fallback) return fallback;

  const reasonKey = sanitizeText(reason).toLowerCase();
  if (reasonKey.includes('fallback_maze')) return 'maze';
  if (reasonKey.includes('fallback_challenge')) return 'challenge';
  if (reasonKey.includes('fallback_block')) return 'block';
  if (reasonKey.includes('redirect_missing_url')) return 'block_missing_redirect';
  return 'none';
};

export const isIpRangeBanLike = (ban = {}) => {
  const reason = sanitizeText(ban?.reason).toLowerCase();
  if (isIpRangeReason(reason)) return true;
  const fingerprintSignals = Array.isArray(ban?.fingerprint?.signals)
    ? ban.fingerprint.signals
    : [];
  if (
    fingerprintSignals.some((signal) => sanitizeText(signal).toLowerCase().includes('ip_range'))
  ) {
    return true;
  }
  const parsed = parseIpRangeOutcome(ban?.fingerprint?.summary);
  return Boolean(parsed.source || parsed.sourceId || parsed.action || parsed.matchedCidr);
};

