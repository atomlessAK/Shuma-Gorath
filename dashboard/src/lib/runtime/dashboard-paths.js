const DEFAULT_DASHBOARD_BASE_PATH = '/dashboard';

export function normalizeDashboardBasePath(value = DEFAULT_DASHBOARD_BASE_PATH) {
  const raw = String(value || '').trim();
  const prefixed = raw.startsWith('/') ? raw : `/${raw}`;
  const normalized = prefixed.replace(/\/{2,}/g, '/').replace(/\/+$/, '');
  return normalized || DEFAULT_DASHBOARD_BASE_PATH;
}

export function resolveDashboardBasePathFromLocation(locationLike, fallback = DEFAULT_DASHBOARD_BASE_PATH) {
  const safeFallback = normalizeDashboardBasePath(fallback);
  const pathname = String(locationLike && locationLike.pathname ? locationLike.pathname : '').trim();
  if (!pathname) return safeFallback;

  const marker = '/dashboard';
  const markerIndex = pathname.indexOf(marker);
  if (markerIndex === -1) return safeFallback;

  const candidate = pathname.slice(0, markerIndex + marker.length);
  return normalizeDashboardBasePath(candidate);
}

export function dashboardIndexPath(basePath = DEFAULT_DASHBOARD_BASE_PATH) {
  return `${normalizeDashboardBasePath(basePath)}/index.html`;
}

export function resolveDashboardAssetPath(basePath, assetRelativePath) {
  const relativePath = String(assetRelativePath || '').trim().replace(/^\/+/, '');
  return `${normalizeDashboardBasePath(basePath)}/${relativePath}`;
}

export function buildDashboardLoginPath(options = {}) {
  const basePath = normalizeDashboardBasePath(options.basePath || DEFAULT_DASHBOARD_BASE_PATH);
  const nextPath = String(options.nextPath || '').trim();
  if (!nextPath) return `${basePath}/login.html`;
  return `${basePath}/login.html?next=${encodeURIComponent(nextPath)}`;
}
