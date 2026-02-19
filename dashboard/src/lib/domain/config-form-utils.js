// @ts-check

const ISO_ALPHA2_CODES = new Set([
    'AD', 'AE', 'AF', 'AG', 'AI', 'AL', 'AM', 'AO', 'AQ', 'AR', 'AS', 'AT', 'AU', 'AW', 'AX', 'AZ', 'BA', 'BB',
    'BD', 'BE', 'BF', 'BG', 'BH', 'BI', 'BJ', 'BL', 'BM', 'BN', 'BO', 'BQ', 'BR', 'BS', 'BT', 'BV', 'BW', 'BY',
    'BZ', 'CA', 'CC', 'CD', 'CF', 'CG', 'CH', 'CI', 'CK', 'CL', 'CM', 'CN', 'CO', 'CR', 'CU', 'CV', 'CW', 'CX',
    'CY', 'CZ', 'DE', 'DJ', 'DK', 'DM', 'DO', 'DZ', 'EC', 'EE', 'EG', 'EH', 'ER', 'ES', 'ET', 'FI', 'FJ', 'FK',
    'FM', 'FO', 'FR', 'GA', 'GB', 'GD', 'GE', 'GF', 'GG', 'GH', 'GI', 'GL', 'GM', 'GN', 'GP', 'GQ', 'GR', 'GS',
    'GT', 'GU', 'GW', 'GY', 'HK', 'HM', 'HN', 'HR', 'HT', 'HU', 'ID', 'IE', 'IL', 'IM', 'IN', 'IO', 'IQ', 'IR',
    'IS', 'IT', 'JE', 'JM', 'JO', 'JP', 'KE', 'KG', 'KH', 'KI', 'KM', 'KN', 'KP', 'KR', 'KW', 'KY', 'KZ', 'LA',
    'LB', 'LC', 'LI', 'LK', 'LR', 'LS', 'LT', 'LU', 'LV', 'LY', 'MA', 'MC', 'MD', 'ME', 'MF', 'MG', 'MH', 'MK',
    'ML', 'MM', 'MN', 'MO', 'MP', 'MQ', 'MR', 'MS', 'MT', 'MU', 'MV', 'MW', 'MX', 'MY', 'MZ', 'NA', 'NC', 'NE',
    'NF', 'NG', 'NI', 'NL', 'NO', 'NP', 'NR', 'NU', 'NZ', 'OM', 'PA', 'PE', 'PF', 'PG', 'PH', 'PK', 'PL', 'PM',
    'PN', 'PR', 'PS', 'PT', 'PW', 'PY', 'QA', 'RE', 'RO', 'RS', 'RU', 'RW', 'SA', 'SB', 'SC', 'SD', 'SE', 'SG',
    'SH', 'SI', 'SJ', 'SK', 'SL', 'SM', 'SN', 'SO', 'SR', 'SS', 'ST', 'SV', 'SX', 'SY', 'SZ', 'TC', 'TD', 'TF',
    'TG', 'TH', 'TJ', 'TK', 'TL', 'TM', 'TN', 'TO', 'TR', 'TT', 'TV', 'TW', 'TZ', 'UA', 'UG', 'UM', 'US', 'UY',
    'UZ', 'VA', 'VC', 'VE', 'VG', 'VI', 'VN', 'VU', 'WF', 'WS', 'YE', 'YT', 'ZA', 'ZM', 'ZW'
  ]);

  function sanitizeGeoTextareaValue(value) {
    return (value || '')
      .replace(/[^a-zA-Z,]/g, '')
      .toUpperCase();
  }

export function parseCountryCodesStrict(raw) {
    const sanitized = sanitizeGeoTextareaValue(raw);
    if (!sanitized) return [];
    if (!/^[A-Z]{2}(,[A-Z]{2})*$/.test(sanitized)) {
      throw new Error('Use comma-separated 2-letter country codes only (example: GB,US,RU).');
    }

    const values = sanitized.split(',');
    const seen = new Set();
    const parsed = [];
    for (const value of values) {
      if (!ISO_ALPHA2_CODES.has(value)) {
        throw new Error(`Invalid country code: ${value}. Use valid ISO 3166-1 alpha-2 codes.`);
      }
      if (!seen.has(value)) {
        seen.add(value);
        parsed.push(value);
      }
    }
    return parsed;
  }

export function normalizeCountryCodesForCompare(raw) {
    return (raw || '')
      .split(',')
      .map((value) => value.trim())
      .filter((value) => value.length > 0)
      .map((value) => value.toUpperCase())
      .join(',');
  }

export function parseListTextarea(raw) {
    const source = String(raw || '');
    const parts = source.split(/[\n,]/);
    const seen = new Set();
    const parsed = [];
    for (const part of parts) {
      const trimmed = part.trim();
      if (!trimmed) continue;
      if (seen.has(trimmed)) continue;
      seen.add(trimmed);
      parsed.push(trimmed);
    }
    return parsed;
  }

export function formatListTextarea(values) {
    if (!Array.isArray(values) || values.length === 0) return '';
    return values.map((value) => String(value || '').trim()).filter(Boolean).join('\n');
  }

export function normalizeListTextareaForCompare(raw) {
    return parseListTextarea(raw).join('\n');
  }

export function parseHoneypotPathsTextarea(raw) {
    const paths = parseListTextarea(raw);
    for (const path of paths) {
      if (!path.startsWith('/')) {
        throw new Error(`Invalid honeypot path '${path}'. Paths must start with '/'.`);
      }
    }
    return paths;
  }

export function formatBrowserRulesTextarea(rules) {
    if (!Array.isArray(rules) || rules.length === 0) return '';
    return rules
      .filter((rule) => Array.isArray(rule) && rule.length >= 2)
      .map((rule) => `${String(rule[0] || '').trim()},${Number.parseInt(rule[1], 10)}`)
      .filter((line) => !line.startsWith(',') && !line.endsWith(',NaN'))
      .join('\n');
  }

export function parseBrowserRulesTextarea(raw) {
    const lines = String(raw || '')
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line.length > 0);
    const parsed = [];
    const seen = new Set();
    for (const line of lines) {
      const firstComma = line.indexOf(',');
      if (firstComma <= 0 || firstComma === line.length - 1) {
        throw new Error(`Invalid browser rule '${line}'. Use BrowserName,min_major.`);
      }
      const browser = line.slice(0, firstComma).trim();
      const versionText = line.slice(firstComma + 1).trim();
      const version = Number.parseInt(versionText, 10);
      if (!browser) {
        throw new Error(`Invalid browser rule '${line}'. Browser name is required.`);
      }
      if (!Number.isInteger(version) || version < 0) {
        throw new Error(`Invalid browser rule '${line}'. Version must be a whole number >= 0.`);
      }
      const dedupeKey = `${browser}|${version}`;
      if (seen.has(dedupeKey)) continue;
      seen.add(dedupeKey);
      parsed.push([browser, version]);
    }
    return parsed;
  }

export function normalizeBrowserRulesForCompare(raw) {
  try {
    return parseBrowserRulesTextarea(raw)
      .map((rule) => `${rule[0]},${rule[1]}`)
      .join('\n');
  } catch (_e) {
    return '__invalid__';
  }
}
