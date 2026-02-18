// @ts-check

/**
 * @typedef {Object} AdminContext
 * @property {string} endpoint
 * @property {string} apikey
 * @property {boolean} [sessionAuth]
 * @property {string} [csrfToken]
 */

/**
 * @typedef {Object} RequestOptions
 * @property {string} [method]
 * @property {HeadersInit} [headers]
 * @property {unknown} [json]
 * @property {BodyInit | null} [body]
 * @property {AbortSignal} [signal]
 * @property {HTMLElement | null} [messageTarget]
 */

const JSON_CONTENT_TYPE = 'application/json';
const isWriteMethod = (method) => {
  const upper = String(method || 'GET').toUpperCase();
  return upper === 'POST' || upper === 'PUT' || upper === 'PATCH' || upper === 'DELETE';
};

/**
 * @param {string} message
 * @param {number} status
 * @param {string} path
 * @param {string} method
 */
export function DashboardApiError(message, status, path, method) {
  const error = new Error(message);
  error.name = 'DashboardApiError';
  /** @type {number} */
  error.status = Number.isInteger(status) ? status : 0;
  /** @type {string} */
  error.path = String(path || '');
  /** @type {string} */
  error.method = String(method || 'GET').toUpperCase();
  return error;
}

/**
 * @param {unknown} value
 * @returns {Record<string, unknown>}
 */
const asRecord = (value) =>
  value && typeof value === 'object' ? /** @type {Record<string, unknown>} */ (value) : {};

/**
 * @param {unknown} value
 * @returns {string}
 */
const errorMessageFromPayload = (value) => {
  if (typeof value === 'string' && value.trim()) return value.trim();
  if (value && typeof value === 'object') {
    const body = /** @type {Record<string, unknown>} */ (value);
    if (typeof body.error === 'string' && body.error.trim()) return body.error.trim();
    if (typeof body.message === 'string' && body.message.trim()) return body.message.trim();
    if (typeof body.detail === 'string' && body.detail.trim()) return body.detail.trim();
  }
  return 'Request failed';
};

/**
 * @param {unknown} value
 * @returns {Array<Record<string, unknown>>}
 */
const asObjectArray = (value) => {
  if (!Array.isArray(value)) return [];
  return value.filter((entry) => entry && typeof entry === 'object');
};

/**
 * @param {unknown} value
 * @returns {Array<[string, number]>}
 */
const adaptTopIps = (value) => {
  if (!Array.isArray(value)) return [];
  return value
    .filter((entry) => Array.isArray(entry) && entry.length >= 2)
    .map((entry) => [String(entry[0] || ''), Number(entry[1] || 0)]);
};

/**
 * @param {unknown} payload
 */
export const adaptAnalytics = (payload) => {
  const source = asRecord(payload);
  return {
    ban_count: Number(source.ban_count || 0),
    test_mode: source.test_mode === true,
    fail_mode: source.fail_mode || 'open'
  };
};

/**
 * @param {unknown} payload
 */
export const adaptEvents = (payload) => {
  const source = asRecord(payload);
  return {
    recent_events: asObjectArray(source.recent_events),
    event_counts: asRecord(source.event_counts),
    top_ips: adaptTopIps(source.top_ips),
    unique_ips: Number(source.unique_ips || 0)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptBans = (payload) => {
  const source = asRecord(payload);
  return {
    bans: asObjectArray(source.bans)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptMaze = (payload) => {
  const source = asRecord(payload);
  return {
    total_hits: Number(source.total_hits || 0),
    unique_crawlers: Number(source.unique_crawlers || 0),
    maze_auto_bans: Number(source.maze_auto_bans || 0),
    top_crawlers: Array.isArray(source.top_crawlers) ? source.top_crawlers : []
  };
};

/**
 * @param {unknown} payload
 */
export const adaptCdp = (payload) => {
  const source = asRecord(payload);
  return {
    stats: asRecord(source.stats),
    config: asRecord(source.config),
    fingerprint_stats: asRecord(source.fingerprint_stats)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptCdpEvents = (payload) => {
  const source = asRecord(payload);
  return {
    events: asObjectArray(source.events)
  };
};

/**
 * @param {unknown} payload
 */
export const adaptMonitoring = (payload) => {
  const source = asRecord(payload);
  const detailsSource = asRecord(source.details);
  const details = {
    analytics: adaptAnalytics(detailsSource.analytics),
    events: adaptEvents(detailsSource.events),
    bans: adaptBans(detailsSource.bans),
    maze: adaptMaze(detailsSource.maze),
    cdp: adaptCdp(detailsSource.cdp),
    cdp_events: adaptCdpEvents(detailsSource.cdp_events || detailsSource.cdpEvents)
  };
  return {
    summary: asRecord(source.summary),
    prometheus: asRecord(source.prometheus),
    details
  };
};

/**
 * @param {unknown} payload
 */
export const adaptConfig = (payload) => asRecord(payload);

/**
 * @param {unknown} payload
 * @returns {{ content: string }}
 */
const adaptRobots = (payload) => {
  if (payload && typeof payload === 'object') {
    const source = /** @type {Record<string, unknown>} */ (payload);
    return {
      content: typeof source.preview === 'string' ? source.preview : ''
    };
  }
  return {
    content: typeof payload === 'string' ? payload : ''
  };
};

export const create = (options = {}) => {
  const getAdminContext =
    typeof options.getAdminContext === 'function' ? options.getAdminContext : null;
  const onUnauthorized =
    typeof options.onUnauthorized === 'function' ? options.onUnauthorized : null;
  const onApiError = typeof options.onApiError === 'function' ? options.onApiError : null;
  const requestImpl =
    typeof options.request === 'function'
      ? options.request
      : fetch.bind(globalThis);

  /**
   * Parse response payloads defensively because some local/runtime paths may
   * omit content-type headers even when returning JSON.
   *
   * @param {Response} response
   * @returns {Promise<unknown>}
   */
  const parseResponsePayload = async (response) => {
    const contentType = String(response.headers.get('content-type') || '').toLowerCase();
    if (contentType.includes(JSON_CONTENT_TYPE)) {
      try {
        return await response.json();
      } catch (_e) {
        return await response.text();
      }
    }

    const text = await response.text();
    if (!text) return '';
    const trimmed = text.trim();
    if (!trimmed) return '';
    if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
      try {
        return JSON.parse(trimmed);
      } catch (_e) {
        return text;
      }
    }
    return text;
  };

  /**
   * @param {string} path
   * @param {RequestOptions} [options]
   */
  const request = async (path, options = {}) => {
    if (!getAdminContext) {
      throw new DashboardApiError('API client is not configured', 0, path, options.method || 'GET');
    }

    /** @type {AdminContext | null} */
    const context = getAdminContext(options.messageTarget || null);
    if (!context) {
      throw new DashboardApiError(
        'Login required. Go to /dashboard/login.html.',
        0,
        path,
        options.method || 'GET'
      );
    }

    const method = String(options.method || (options.json ? 'POST' : 'GET')).toUpperCase();
    const headers = new Headers(options.headers || {});
    if (!headers.has('Accept')) headers.set('Accept', JSON_CONTENT_TYPE);
    if (!headers.has('Authorization') && String(context.apikey || '').trim()) {
      headers.set('Authorization', `Bearer ${String(context.apikey).trim()}`);
    }
    const authHeader = headers.get('Authorization') || headers.get('authorization') || '';
    if (/^Bearer\s*$/i.test(authHeader.trim())) {
      headers.delete('Authorization');
      headers.delete('authorization');
    }
    if (
      context &&
      context.sessionAuth === true &&
      isWriteMethod(method) &&
      String(context.csrfToken || '').trim()
    ) {
      headers.set('X-Shuma-CSRF', String(context.csrfToken).trim());
    }

    /** @type {BodyInit | null | undefined} */
    let body = options.body;
    if (options.json !== undefined) {
      if (!headers.has('Content-Type')) headers.set('Content-Type', JSON_CONTENT_TYPE);
      body = JSON.stringify(options.json);
    }

    const response = await requestImpl(`${context.endpoint}${path}`, {
      method,
      headers,
      credentials: context && context.sessionAuth === true ? 'same-origin' : undefined,
      body: method === 'GET' || method === 'HEAD' ? undefined : body,
      signal: options.signal
    });

    const payload = await parseResponsePayload(response);

    if (response.status === 401) {
      if (onUnauthorized) onUnauthorized();
      const unauthorizedError = new DashboardApiError(
        'Unauthorized',
        response.status,
        path,
        method
      );
      if (onApiError) onApiError(unauthorizedError);
      throw unauthorizedError;
    }

    if (!response.ok) {
      const apiError = new DashboardApiError(
        errorMessageFromPayload(payload),
        response.status,
        path,
        method
      );
      if (onApiError) onApiError(apiError);
      throw apiError;
    }

    return payload;
  };

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getAnalytics = async (requestOptions = {}) =>
    adaptAnalytics(await request('/admin/analytics', requestOptions));

  /**
   * @param {number} hours
   * @param {RequestOptions} [requestOptions]
   */
  const getEvents = async (hours = 24, requestOptions = {}) =>
    adaptEvents(await request(`/admin/events?hours=${encodeURIComponent(String(hours))}`, requestOptions));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getBans = async (requestOptions = {}) => adaptBans(await request('/admin/ban', requestOptions));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getMaze = async (requestOptions = {}) => adaptMaze(await request('/admin/maze', requestOptions));

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getCdp = async (requestOptions = {}) => adaptCdp(await request('/admin/cdp', requestOptions));

  /**
   * @param {{hours?: number, limit?: number}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getCdpEvents = async (options = {}, requestOptions = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 500;
    return adaptCdpEvents(
      await request(
        `/admin/cdp/events?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}`,
        requestOptions
      )
    );
  };

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getConfig = async (requestOptions = {}) =>
    adaptConfig(await request('/admin/config', requestOptions));

  /**
   * @param {{hours?: number, limit?: number}} [options]
   * @param {RequestOptions} [requestOptions]
   */
  const getMonitoring = async (options = {}, requestOptions = {}) => {
    const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
    const limit = Number.isFinite(options.limit) ? Number(options.limit) : 10;
    return adaptMonitoring(
      await request(
        `/admin/monitoring?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}`,
        requestOptions
      )
    );
  };

  /**
   * @param {RequestOptions} [requestOptions]
   */
  const getRobotsPreview = async (requestOptions = {}) =>
    adaptRobots(await request('/admin/robots', requestOptions));

  /**
   * @param {Record<string, unknown>} configPatch
   */
  const updateConfig = async (configPatch) =>
    adaptConfig(
      await request('/admin/config', {
        method: 'POST',
        json: configPatch
      })
    );

  /**
   * @param {string} ip
   * @param {number} duration
   * @param {string} [reason]
   */
  const banIp = async (ip, duration, reason = 'manual_ban') =>
    request('/admin/ban', {
      method: 'POST',
      json: {
        ip: String(ip || ''),
        reason: String(reason || 'manual_ban'),
        duration: Number(duration || 0)
      }
    });

  /**
   * @param {string} ip
   */
  const unbanIp = async (ip) =>
    request(`/admin/unban?ip=${encodeURIComponent(String(ip || ''))}`, {
      method: 'POST'
    });

  return {
    request,
    getAnalytics,
    getEvents,
    getBans,
    getMaze,
    getCdp,
    getCdpEvents,
    getMonitoring,
    getConfig,
    getRobotsPreview,
    updateConfig,
    banIp,
    unbanIp
  };
};
