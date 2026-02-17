// @ts-check

const sanitizeEndpointText = (value) => (value || '').replace(/\s+/g, '').trim();

const isLoopbackHostname = (hostname) => {
  const normalized = String(hostname || '').trim().toLowerCase();
  return (
    normalized === 'localhost' ||
    normalized === '127.0.0.1' ||
    normalized === '::1' ||
    normalized === '[::1]'
  );
};

export const parseEndpointUrl = (value) => {
  const sanitized = sanitizeEndpointText(value);
  if (!sanitized) return null;
  try {
    const url = new URL(sanitized);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
    if (!url.hostname) return null;
    const pathname = url.pathname === '/' ? '' : url.pathname.replace(/\/+$/, '');
    return `${url.protocol}//${url.host}${pathname}`;
  } catch (_e) {
    return null;
  }
};

export const createAdminEndpointResolver = (options = {}) => {
  const win = options.window || window;
  let context = null;

  return () => {
    if (context) return context;

    const origin = win.location.origin || `${win.location.protocol}//${win.location.host}`;
    let endpoint = parseEndpointUrl(origin) || origin;

    // Local diagnostics only: allow ?api_endpoint=http://127.0.0.1:3000 override on loopback dashboards.
    if (isLoopbackHostname(win.location.hostname)) {
      const params = new URLSearchParams(win.location.search || '');
      const override = sanitizeEndpointText(params.get('api_endpoint') || '');
      if (override) {
        const parsed = parseEndpointUrl(override);
        if (parsed) {
          try {
            const parsedUrl = new URL(parsed);
            if (isLoopbackHostname(parsedUrl.hostname)) {
              endpoint = parsed;
            }
          } catch (_e) {}
        }
      }
    }

    context = { endpoint };
    return context;
  };
};
