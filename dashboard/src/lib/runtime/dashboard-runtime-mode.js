const DASHBOARD_RUNTIME_MODES = Object.freeze({
  native: 'native',
  legacy: 'legacy'
});

export const DEFAULT_DASHBOARD_RUNTIME_MODE = DASHBOARD_RUNTIME_MODES.native;

export function normalizeDashboardRuntimeMode(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === DASHBOARD_RUNTIME_MODES.legacy) {
    return DASHBOARD_RUNTIME_MODES.legacy;
  }
  return DEFAULT_DASHBOARD_RUNTIME_MODE;
}

export function resolveDashboardRuntimeMode(env = {}) {
  const source = env || {};
  const configuredMode =
    source.PUBLIC_SHUMA_DASHBOARD_RUNTIME_MODE ??
    source.PUBLIC_DASHBOARD_RUNTIME_MODE ??
    '';
  return normalizeDashboardRuntimeMode(configuredMode);
}
