// @ts-check

export const cloneJsonValue = (value) => {
  if (value === null || value === undefined) return null;
  if (typeof value !== 'object') return value;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (_e) {
    return null;
  }
};

export const readValueAtPath = (obj, path) => {
  const segments = String(path || '').split('.');
  let cursor = obj;
  for (const segment of segments) {
    if (!segment || cursor === null || typeof cursor !== 'object') return undefined;
    if (!Object.prototype.hasOwnProperty.call(cursor, segment)) return undefined;
    cursor = cursor[segment];
  }
  return cursor;
};

export const writeValueAtPath = (target, path, value) => {
  const segments = String(path || '').split('.');
  if (segments.length === 0) return;
  let cursor = target;
  for (let i = 0; i < segments.length; i += 1) {
    const segment = segments[i];
    if (!segment) return;
    const isLeaf = i === segments.length - 1;
    if (isLeaf) {
      cursor[segment] = value;
      return;
    }
    if (!cursor[segment] || typeof cursor[segment] !== 'object' || Array.isArray(cursor[segment])) {
      cursor[segment] = {};
    }
    cursor = cursor[segment];
  }
};

export const buildTemplateFromPaths = (source, paths = []) => {
  const template = {};
  paths.forEach((path) => {
    const rawValue = readValueAtPath(source, path);
    if (rawValue === undefined) return;
    const cloned = cloneJsonValue(rawValue);
    writeValueAtPath(template, path, cloned === null && rawValue !== null ? rawValue : cloned);
  });
  return template;
};

export const normalizeJsonObjectForCompare = (raw) => {
  try {
    const parsed = JSON.parse(String(raw || '{}'));
    if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') return null;
    return JSON.stringify(parsed);
  } catch (_e) {
    return null;
  }
};
