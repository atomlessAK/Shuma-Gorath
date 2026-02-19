// @ts-check

export const escapeHtml = (value) => String(value ?? '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;');

export const formatNumber = (value, fallback = '0') => {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return String(fallback);
  return parsed.toLocaleString();
};

export const formatDateTimeSeconds = (epochSeconds, fallback = '-') => {
  const parsed = Number(epochSeconds);
  if (!Number.isFinite(parsed)) return fallback;
  return new Date(parsed * 1000).toLocaleString();
};

export const arraysEqualShallow = (a, b) => {
  if (!Array.isArray(a) || !Array.isArray(b)) return false;
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) return false;
  }
  return true;
};
