// @ts-check

import * as format from './core/format.js';
import * as domModule from './core/dom.js';
import { getChartConstructor } from './services/chart-runtime-adapter.js';

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
    label: `Top Offender (${count.toLocaleString()} ${unit})`
  };
};

const formatMetricLabel = (key, fallbackMap) => {
  if (fallbackMap && fallbackMap[key]) return fallbackMap[key];
  return String(key || '-')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase());
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
    .sort((a, b) => Number(b[1] || 0) - Number(a[1] || 0));

const deriveTrendSeries = (trend = []) => {
  const points = Array.isArray(trend) ? trend : [];
  return {
    labels: points.map((point) => formatTrendTimestamp(Number(point.ts || 0))),
    data: points.map((point) => Number(point.total || 0))
  };
};

export const deriveMazeStatsViewModel = (data = {}) => {
  const topCrawler =
    Array.isArray(data.top_crawlers) && data.top_crawlers.length ? data.top_crawlers[0] : null;
  return {
    totalHits: format.formatNumber(data.total_hits, '0'),
    uniqueCrawlers: format.formatNumber(data.unique_crawlers, '0'),
    mazeAutoBans: format.formatNumber(data.maze_auto_bans, '0'),
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
  const pow = summary.pow || {};
  const rate = summary.rate || {};
  const geo = summary.geo || {};

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

  return {
    honeypot: {
      totalHits: format.formatNumber(honeypot.total_hits, '0'),
      uniqueCrawlers: format.formatNumber(honeypot.unique_crawlers, '0'),
      topOffender: deriveTopOffenderViewModel(
        topHoneypotCrawler?.label,
        topHoneypotCrawler?.count,
        'hit',
        'hits'
      ),
      topPaths: Array.isArray(honeypot.top_paths) ? honeypot.top_paths : []
    },
    challenge: {
      totalFailures: format.formatNumber(challenge.total_failures, '0'),
      uniqueOffenders: format.formatNumber(challenge.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topChallengeOffender?.label,
        topChallengeOffender?.count,
        'hit',
        'hits'
      ),
      reasons: sortCountEntries(challenge.reasons),
      trend: deriveTrendSeries(challenge.trend)
    },
    pow: {
      totalFailures: format.formatNumber(pow.total_failures, '0'),
      uniqueOffenders: format.formatNumber(pow.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topPowOffender?.label,
        topPowOffender?.count,
        'hit',
        'hits'
      ),
      reasons: sortCountEntries(pow.reasons),
      trend: deriveTrendSeries(pow.trend)
    },
    rate: {
      totalViolations: format.formatNumber(rate.total_violations, '0'),
      uniqueOffenders: format.formatNumber(rate.unique_offenders, '0'),
      topOffender: deriveTopOffenderViewModel(
        topRateOffender?.label,
        topRateOffender?.count,
        'hit',
        'hits'
      ),
      outcomes: sortCountEntries(rate.outcomes)
    },
    geo: {
      totalViolations: format.formatNumber(geo.total_violations, '0'),
      actionMix: {
        block: Number(geo.actions?.block || 0).toLocaleString(),
        challenge: Number(geo.actions?.challenge || 0).toLocaleString(),
        maze: Number(geo.actions?.maze || 0).toLocaleString()
      },
      topCountries: Array.isArray(geo.top_countries) ? geo.top_countries : []
    }
  };
};

export const derivePrometheusHelperViewModel = (prometheusData = {}, origin = '') => {
  const readString = (value) => (typeof value === 'string' ? value.trim() : '');
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
    observabilityLink:
      typeof docs.observability === 'string' && docs.observability ? docs.observability : '',
    apiLink: typeof docs.api === 'string' && docs.api ? docs.api : ''
  };
};

const updateMonitoringTrendChart = (existingChart, canvas, title, color, trendSeries, chartCtor) => {
  if (!canvas || typeof chartCtor !== 'function') return existingChart;
  const ctx = canvas.getContext('2d');
  if (!ctx) return existingChart;

  if (!existingChart) {
    return new chartCtor(ctx, {
      type: 'line',
      data: {
        labels: trendSeries.labels,
        datasets: [{
          label: title,
          data: trendSeries.data,
          backgroundColor: color,
          fill: true,
          tension: 0.35,
          pointRadius: 0,
          pointHoverRadius: 0,
          borderWidth: 0
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: true,
        plugins: { legend: { display: false } },
        scales: {
          y: {
            beginAtZero: true,
            ticks: { stepSize: 1 }
          }
        }
      }
    });
  }

  const sameLabels = format.arraysEqualShallow(existingChart.data.labels || [], trendSeries.labels);
  const sameData = format.arraysEqualShallow(
    existingChart.data.datasets[0].data || [],
    trendSeries.data
  );
  if (sameLabels && sameData) {
    return existingChart;
  }

  existingChart.data.labels = trendSeries.labels;
  existingChart.data.datasets[0].data = trendSeries.data;
  existingChart.update();
  return existingChart;
};

const renderTopOffenderCard = (domRefs, domApi, valueId, labelId, viewModel) => {
  domApi.setText(domRefs[valueId], viewModel.value);
  domApi.setText(domRefs[labelId], viewModel.label);
};

const renderCountList = (domRefs, domApi, escapeHtml, containerId, entries, emptyText, valueSuffix = '') => {
  const container = domRefs[containerId];
  if (!container) return;
  const rows = Array.isArray(entries) ? entries : [];
  if (!rows.length) {
    domApi.setHtml(container, `<p class="no-data">${escapeHtml(emptyText)}</p>`);
    return;
  }
  domApi.setHtml(container, rows.map((row) => {
    const label = escapeHtml(row.label || '-');
    const count = Number(row.count || 0).toLocaleString();
    const suffix = valueSuffix ? ` ${escapeHtml(valueSuffix)}` : '';
    return `
      <div class="crawler-item panel panel-border">
        <span class="crawler-ip">${label}</span>
        <span class="crawler-hits">${count}${suffix}</span>
      </div>
    `;
  }).join(''));
};

const renderReasonTable = (domRefs, domApi, escapeHtml, tbodyId, rows, labels) => {
  const tbody = domRefs[tbodyId];
  if (!tbody) return;
  if (!rows.length) {
    domApi.setHtml(
      tbody,
      '<tr><td colspan="2" style="text-align: center; color: #6b7280;">No failures in window</td></tr>'
    );
    return;
  }
  domApi.setHtml(tbody, rows.map(([key, value]) => `
    <tr>
      <td>${escapeHtml(formatMetricLabel(key, labels))}</td>
      <td>${Number(value || 0).toLocaleString()}</td>
    </tr>
  `).join(''));
};

const renderOutcomeList = (domRefs, domApi, escapeHtml, listId, rows) => {
  const list = domRefs[listId];
  if (!list) return;
  if (!rows.length) {
    domApi.setHtml(list, '<li class="text-muted">No outcomes yet</li>');
    return;
  }
  domApi.setHtml(list, rows.map(([key, value]) => `
    <li><strong>${escapeHtml(formatMetricLabel(key, RATE_OUTCOME_LABELS))}:</strong> ${Number(value || 0).toLocaleString()}</li>
  `).join(''));
};

const createDefaultEffects = () => ({
  copyText: async (text) => {
    await navigator.clipboard.writeText(String(text || ''));
  },
  setTimer: (task, ms = 0) => window.setTimeout(task, ms)
});

export const create = (options = {}) => {
  const escapeHtml = typeof options.escapeHtml === 'function' ? options.escapeHtml : format.escapeHtml;
  const effects = options.effects && typeof options.effects === 'object'
    ? options.effects
    : createDefaultEffects();
  const domCache = domModule.createCache({ document });
  const byId = domCache.byId;
  const domRefs = {
    mazeTotalHits: byId('maze-total-hits'),
    mazeUniqueCrawlers: byId('maze-unique-crawlers'),
    mazeAutoBans: byId('maze-auto-bans'),
    mazeTopOffender: byId('maze-top-offender'),
    mazeTopOffenderLabel: byId('maze-top-offender-label'),
    honeypotTotalHits: byId('honeypot-total-hits'),
    honeypotUniqueCrawlers: byId('honeypot-unique-crawlers'),
    honeypotTopOffender: byId('honeypot-top-offender'),
    honeypotTopOffenderLabel: byId('honeypot-top-offender-label'),
    honeypotTopPaths: byId('honeypot-top-paths'),
    challengeFailuresTotal: byId('challenge-failures-total'),
    challengeFailuresUnique: byId('challenge-failures-unique'),
    challengeTopOffender: byId('challenge-top-offender'),
    challengeTopOffenderLabel: byId('challenge-top-offender-label'),
    challengeFailureReasons: byId('challenge-failure-reasons'),
    challengeFailuresTrendChart: byId('challengeFailuresTrendChart'),
    powFailuresTotal: byId('pow-failures-total'),
    powFailuresUnique: byId('pow-failures-unique'),
    powTopOffender: byId('pow-top-offender'),
    powTopOffenderLabel: byId('pow-top-offender-label'),
    powFailureReasons: byId('pow-failure-reasons'),
    powFailuresTrendChart: byId('powFailuresTrendChart'),
    rateViolationsTotal: byId('rate-violations-total'),
    rateViolationsUnique: byId('rate-violations-unique'),
    rateTopOffender: byId('rate-top-offender'),
    rateTopOffenderLabel: byId('rate-top-offender-label'),
    rateOutcomesList: byId('rate-outcomes-list'),
    geoViolationsTotal: byId('geo-violations-total'),
    geoActionMix: byId('geo-action-mix'),
    geoTopCountries: byId('geo-top-countries'),
    prometheusExample: byId('monitoring-prometheus-example'),
    prometheusCopyButton: byId('monitoring-prometheus-copy'),
    prometheusCopyCurlButton: byId('monitoring-prometheus-copy-curl'),
    prometheusFacts: byId('monitoring-prometheus-facts'),
    prometheusOutput: byId('monitoring-prometheus-output'),
    prometheusStats: byId('monitoring-prometheus-stats'),
    prometheusWindowed: byId('monitoring-prometheus-windowed'),
    prometheusSummaryStats: byId('monitoring-prometheus-summary-stats'),
    prometheusObservabilityLink: byId('monitoring-prometheus-observability-link'),
    prometheusApiLink: byId('monitoring-prometheus-api-link')
  };

  let challengeFailuresTrendChart = null;
  let powFailuresTrendChart = null;
  let prometheusCopyHandler = null;
  let prometheusCopyCurlHandler = null;
  let chartConstructor = typeof options.chartConstructor === 'function'
    ? options.chartConstructor
    : null;

  const resolveChartConstructor = () => {
    if (typeof chartConstructor === 'function') return chartConstructor;
    chartConstructor = getChartConstructor();
    return typeof chartConstructor === 'function' ? chartConstructor : null;
  };

  const renderMazeStats = (viewModel) => {
    domModule.setText(domRefs.mazeTotalHits, viewModel.totalHits);
    domModule.setText(domRefs.mazeUniqueCrawlers, viewModel.uniqueCrawlers);
    domModule.setText(domRefs.mazeAutoBans, viewModel.mazeAutoBans);
    renderTopOffenderCard(domRefs, domModule, 'mazeTopOffender', 'mazeTopOffenderLabel', viewModel.topOffender);
  };

  const renderMonitoringSummary = (viewModel) => {
    domModule.setText(domRefs.honeypotTotalHits, viewModel.honeypot.totalHits);
    domModule.setText(domRefs.honeypotUniqueCrawlers, viewModel.honeypot.uniqueCrawlers);
    renderTopOffenderCard(
      domRefs,
      domModule,
      'honeypotTopOffender',
      'honeypotTopOffenderLabel',
      viewModel.honeypot.topOffender
    );
    renderCountList(
      domRefs,
      domModule,
      escapeHtml,
      'honeypotTopPaths',
      viewModel.honeypot.topPaths,
      'No honeypot path data yet',
      'hits'
    );

    domModule.setText(domRefs.challengeFailuresTotal, viewModel.challenge.totalFailures);
    domModule.setText(domRefs.challengeFailuresUnique, viewModel.challenge.uniqueOffenders);
    renderTopOffenderCard(
      domRefs,
      domModule,
      'challengeTopOffender',
      'challengeTopOffenderLabel',
      viewModel.challenge.topOffender
    );
    renderReasonTable(
      domRefs,
      domModule,
      escapeHtml,
      'challengeFailureReasons',
      viewModel.challenge.reasons,
      CHALLENGE_REASON_LABELS
    );
    challengeFailuresTrendChart = updateMonitoringTrendChart(
      challengeFailuresTrendChart,
      domRefs.challengeFailuresTrendChart,
      'Challenge Failures',
      'rgba(255,205,235,0.95)',
      viewModel.challenge.trend,
      resolveChartConstructor()
    );

    domModule.setText(domRefs.powFailuresTotal, viewModel.pow.totalFailures);
    domModule.setText(domRefs.powFailuresUnique, viewModel.pow.uniqueOffenders);
    renderTopOffenderCard(domRefs, domModule, 'powTopOffender', 'powTopOffenderLabel', viewModel.pow.topOffender);
    renderReasonTable(
      domRefs,
      domModule,
      escapeHtml,
      'powFailureReasons',
      viewModel.pow.reasons,
      POW_REASON_LABELS
    );
    powFailuresTrendChart = updateMonitoringTrendChart(
      powFailuresTrendChart,
      domRefs.powFailuresTrendChart,
      'PoW Failures',
      'rgba(205,155,185,0.95)',
      viewModel.pow.trend,
      resolveChartConstructor()
    );

    domModule.setText(domRefs.rateViolationsTotal, viewModel.rate.totalViolations);
    domModule.setText(domRefs.rateViolationsUnique, viewModel.rate.uniqueOffenders);
    renderTopOffenderCard(
      domRefs,
      domModule,
      'rateTopOffender',
      'rateTopOffenderLabel',
      viewModel.rate.topOffender
    );
    renderOutcomeList(domRefs, domModule, escapeHtml, 'rateOutcomesList', viewModel.rate.outcomes);

    domModule.setText(domRefs.geoViolationsTotal, viewModel.geo.totalViolations);
    domModule.setText(
      domRefs.geoActionMix,
      `B:${viewModel.geo.actionMix.block} C:${viewModel.geo.actionMix.challenge} M:${viewModel.geo.actionMix.maze}`
    );
    renderCountList(
      domRefs,
      domModule,
      escapeHtml,
      'geoTopCountries',
      viewModel.geo.topCountries,
      'No GEO violations yet',
      'hits'
    );
  };

  const renderPrometheusHelper = (viewModel) => {
    domModule.setText(domRefs.prometheusExample, viewModel.exampleJs);
    if (domRefs.prometheusCopyCurlButton) {
      domRefs.prometheusCopyCurlButton.dataset.copyText = viewModel.copyCurlText;
    }
    domModule.setHtml(
      domRefs.prometheusFacts,
      viewModel.facts.map((entry) => `<li>${escapeHtml(entry)}</li>`).join('')
    );
    domModule.setText(domRefs.prometheusOutput, viewModel.exampleOutput);
    domModule.setText(domRefs.prometheusStats, viewModel.exampleStats);
    domModule.setText(domRefs.prometheusWindowed, viewModel.exampleWindowed);
    domModule.setText(domRefs.prometheusSummaryStats, viewModel.exampleSummaryStats);
    if (domRefs.prometheusObservabilityLink && viewModel.observabilityLink) {
      domRefs.prometheusObservabilityLink.href = viewModel.observabilityLink;
    }
    if (domRefs.prometheusApiLink && viewModel.apiLink) {
      domRefs.prometheusApiLink.href = viewModel.apiLink;
    }
  };

  const updateMazeStats = (data) => renderMazeStats(deriveMazeStatsViewModel(data));

  const updateMonitoringSummary = (summary) => {
    renderMonitoringSummary(deriveMonitoringSummaryViewModel(summary));
  };

  const updatePrometheusHelper = (prometheusData) => {
    const origin = window.location.origin || 'http://127.0.0.1:3000';
    renderPrometheusHelper(derivePrometheusHelperViewModel(prometheusData, origin));
  };

  const showLoadingState = () => {
    [
      'mazeTotalHits',
      'mazeUniqueCrawlers',
      'mazeAutoBans',
      'honeypotTotalHits',
      'honeypotUniqueCrawlers',
      'challengeFailuresTotal',
      'challengeFailuresUnique',
      'powFailuresTotal',
      'powFailuresUnique',
      'rateViolationsTotal',
      'rateViolationsUnique',
      'geoViolationsTotal'
    ].forEach((refKey) => {
      domModule.setText(domRefs[refKey], '...');
    });

    const loadingTopOffender = { value: '...', label: 'Top Offender' };
    renderTopOffenderCard(domRefs, domModule, 'mazeTopOffender', 'mazeTopOffenderLabel', loadingTopOffender);
    renderTopOffenderCard(
      domRefs,
      domModule,
      'honeypotTopOffender',
      'honeypotTopOffenderLabel',
      loadingTopOffender
    );
    renderTopOffenderCard(
      domRefs,
      domModule,
      'challengeTopOffender',
      'challengeTopOffenderLabel',
      loadingTopOffender
    );
    renderTopOffenderCard(domRefs, domModule, 'powTopOffender', 'powTopOffenderLabel', loadingTopOffender);
    renderTopOffenderCard(domRefs, domModule, 'rateTopOffender', 'rateTopOffenderLabel', loadingTopOffender);
  };

  const bindPrometheusCopyButtons = () => {
    const copyWithFeedback = async (targetButton, text, resetText) => {
      if (!targetButton || !text) return;
      try {
        await effects.copyText(String(text || '').trim());
        targetButton.textContent = 'Copied';
        effects.setTimer(() => {
          targetButton.textContent = resetText;
        }, 1200);
      } catch (_err) {
        targetButton.textContent = 'Copy Failed';
        effects.setTimer(() => {
          targetButton.textContent = resetText;
        }, 1500);
      }
    };

    if (domRefs.prometheusCopyButton && domRefs.prometheusExample) {
      if (prometheusCopyHandler) {
        domRefs.prometheusCopyButton.removeEventListener('click', prometheusCopyHandler);
      }
      prometheusCopyHandler = async () => {
        const text = String(domRefs.prometheusExample.textContent || '').trim();
        await copyWithFeedback(domRefs.prometheusCopyButton, text, 'Copy JS Example');
      };
      domRefs.prometheusCopyButton.addEventListener('click', prometheusCopyHandler);
    }

    if (domRefs.prometheusCopyCurlButton) {
      if (prometheusCopyCurlHandler) {
        domRefs.prometheusCopyCurlButton.removeEventListener('click', prometheusCopyCurlHandler);
      }
      prometheusCopyCurlHandler = async () => {
        const fallback = `curl -sS '${window.location.origin || 'http://127.0.0.1:3000'}/metrics'`;
        const text = String(domRefs.prometheusCopyCurlButton.dataset.copyText || fallback).trim();
        await copyWithFeedback(domRefs.prometheusCopyCurlButton, text, 'Copy Curl Example');
      };
      domRefs.prometheusCopyCurlButton.addEventListener('click', prometheusCopyCurlHandler);
    }
  };

  const destroy = () => {
    if (domRefs.prometheusCopyButton && prometheusCopyHandler) {
      domRefs.prometheusCopyButton.removeEventListener('click', prometheusCopyHandler);
      prometheusCopyHandler = null;
    }
    if (domRefs.prometheusCopyCurlButton && prometheusCopyCurlHandler) {
      domRefs.prometheusCopyCurlButton.removeEventListener('click', prometheusCopyCurlHandler);
      prometheusCopyCurlHandler = null;
    }
    if (challengeFailuresTrendChart && typeof challengeFailuresTrendChart.destroy === 'function') {
      challengeFailuresTrendChart.destroy();
    }
    if (powFailuresTrendChart && typeof powFailuresTrendChart.destroy === 'function') {
      powFailuresTrendChart.destroy();
    }
    challengeFailuresTrendChart = null;
    powFailuresTrendChart = null;
    chartConstructor = null;
  };

  return {
    showLoadingState,
    updateMazeStats,
    updateMonitoringSummary,
    updatePrometheusHelper,
    bindPrometheusCopyButtons,
    destroy
  };
};
