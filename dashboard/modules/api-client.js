// @ts-check

(function (global) {
  /**
   * @typedef {Object} AdminContext
   * @property {string} endpoint
   * @property {string} apikey
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

  class DashboardApiError extends Error {
    /**
     * @param {string} message
     * @param {number} status
     * @param {string} path
     * @param {string} method
     */
    constructor(message, status, path, method) {
      super(message);
      this.name = 'DashboardApiError';
      /** @type {number} */
      this.status = Number.isInteger(status) ? status : 0;
      /** @type {string} */
      this.path = String(path || '');
      /** @type {string} */
      this.method = String(method || 'GET').toUpperCase();
    }
  }

  /**
   * @param {unknown} value
   * @returns {Record<string, unknown>}
   */
  function asRecord(value) {
    return value && typeof value === 'object' ? /** @type {Record<string, unknown>} */ (value) : {};
  }

  /**
   * @param {unknown} value
   * @returns {string}
   */
  function errorMessageFromPayload(value) {
    if (typeof value === 'string' && value.trim()) return value.trim();
    if (value && typeof value === 'object') {
      const body = /** @type {Record<string, unknown>} */ (value);
      if (typeof body.error === 'string' && body.error.trim()) return body.error.trim();
      if (typeof body.message === 'string' && body.message.trim()) return body.message.trim();
      if (typeof body.detail === 'string' && body.detail.trim()) return body.detail.trim();
    }
    return 'Request failed';
  }

  /**
   * @param {unknown} value
   * @returns {Array<Record<string, unknown>>}
   */
  function asObjectArray(value) {
    if (!Array.isArray(value)) return [];
    return value.filter((entry) => entry && typeof entry === 'object');
  }

  /**
   * @param {unknown} value
   * @returns {Array<[string, number]>}
   */
  function adaptTopIps(value) {
    if (!Array.isArray(value)) return [];
    return value
      .filter((entry) => Array.isArray(entry) && entry.length >= 2)
      .map((entry) => [String(entry[0] || ''), Number(entry[1] || 0)]);
  }

  /**
   * @param {unknown} payload
   */
  function adaptAnalytics(payload) {
    const source = asRecord(payload);
    return {
      ban_count: Number(source.ban_count || 0),
      test_mode: source.test_mode === true,
      fail_mode: source.fail_mode || 'open'
    };
  }

  /**
   * @param {unknown} payload
   */
  function adaptEvents(payload) {
    const source = asRecord(payload);
    return {
      recent_events: asObjectArray(source.recent_events),
      event_counts: asRecord(source.event_counts),
      top_ips: adaptTopIps(source.top_ips),
      unique_ips: Number(source.unique_ips || 0)
    };
  }

  /**
   * @param {unknown} payload
   */
  function adaptBans(payload) {
    const source = asRecord(payload);
    return {
      bans: asObjectArray(source.bans)
    };
  }

  /**
   * @param {unknown} payload
   */
  function adaptMaze(payload) {
    const source = asRecord(payload);
    return {
      total_hits: Number(source.total_hits || 0),
      unique_crawlers: Number(source.unique_crawlers || 0),
      maze_auto_bans: Number(source.maze_auto_bans || 0),
      top_crawlers: Array.isArray(source.top_crawlers) ? source.top_crawlers : []
    };
  }

  /**
   * @param {unknown} payload
   */
  function adaptCdp(payload) {
    const source = asRecord(payload);
    return {
      stats: asRecord(source.stats),
      config: asRecord(source.config)
    };
  }

  /**
   * @param {unknown} payload
   */
  function adaptCdpEvents(payload) {
    const source = asRecord(payload);
    return {
      events: asObjectArray(source.events)
    };
  }

  /**
   * @param {unknown} payload
   */
  function adaptConfig(payload) {
    return asRecord(payload);
  }

  /**
   * @param {unknown} payload
   * @returns {{ content: string }}
   */
  function adaptRobots(payload) {
    if (payload && typeof payload === 'object') {
      const source = /** @type {Record<string, unknown>} */ (payload);
      return {
        content: typeof source.preview === 'string' ? source.preview : ''
      };
    }
    return {
      content: typeof payload === 'string' ? payload : ''
    };
  }

  function create(options = {}) {
    const getAdminContext =
      typeof options.getAdminContext === 'function' ? options.getAdminContext : null;
    const onUnauthorized =
      typeof options.onUnauthorized === 'function' ? options.onUnauthorized : null;
    const onApiError = typeof options.onApiError === 'function' ? options.onApiError : null;

    /**
     * Parse response payloads defensively because some local/runtime paths may
     * omit content-type headers even when returning JSON.
     *
     * @param {Response} response
     * @returns {Promise<unknown>}
     */
    async function parseResponsePayload(response) {
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
    }

    /**
     * @param {string} path
     * @param {RequestOptions} [options]
     */
    async function request(path, options = {}) {
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

      /** @type {BodyInit | null | undefined} */
      let body = options.body;
      if (options.json !== undefined) {
        if (!headers.has('Content-Type')) headers.set('Content-Type', JSON_CONTENT_TYPE);
        body = JSON.stringify(options.json);
      }

      const response = await fetch(`${context.endpoint}${path}`, {
        method,
        headers,
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
    }

    async function getAnalytics() {
      return adaptAnalytics(await request('/admin/analytics'));
    }

    /**
     * @param {number} hours
     */
    async function getEvents(hours = 24) {
      return adaptEvents(await request(`/admin/events?hours=${encodeURIComponent(String(hours))}`));
    }

    async function getBans() {
      return adaptBans(await request('/admin/ban'));
    }

    async function getMaze() {
      return adaptMaze(await request('/admin/maze'));
    }

    async function getCdp() {
      return adaptCdp(await request('/admin/cdp'));
    }

    /**
     * @param {{hours?: number, limit?: number}} [options]
     */
    async function getCdpEvents(options = {}) {
      const hours = Number.isFinite(options.hours) ? Number(options.hours) : 24;
      const limit = Number.isFinite(options.limit) ? Number(options.limit) : 500;
      return adaptCdpEvents(
        await request(
          `/admin/cdp/events?hours=${encodeURIComponent(String(hours))}&limit=${encodeURIComponent(String(limit))}`
        )
      );
    }

    async function getConfig() {
      return adaptConfig(await request('/admin/config'));
    }

    async function getRobotsPreview() {
      return adaptRobots(await request('/admin/robots'));
    }

    /**
     * @param {Record<string, unknown>} configPatch
     */
    async function updateConfig(configPatch) {
      return adaptConfig(
        await request('/admin/config', {
          method: 'POST',
          json: configPatch
        })
      );
    }

    /**
     * @param {string} ip
     * @param {number} duration
     * @param {string} [reason]
     */
    async function banIp(ip, duration, reason = 'manual_ban') {
      return request('/admin/ban', {
        method: 'POST',
        json: {
          ip: String(ip || ''),
          reason: String(reason || 'manual_ban'),
          duration: Number(duration || 0)
        }
      });
    }

    /**
     * @param {string} ip
     */
    async function unbanIp(ip) {
      return request(`/admin/unban?ip=${encodeURIComponent(String(ip || ''))}`, {
        method: 'POST'
      });
    }

    return {
      request,
      getAnalytics,
      getEvents,
      getBans,
      getMaze,
      getCdp,
      getCdpEvents,
      getConfig,
      getRobotsPreview,
      updateConfig,
      banIp,
      unbanIp
    };
  }

  global.ShumaDashboardApiClient = {
    create,
    DashboardApiError,
    adaptAnalytics,
    adaptEvents,
    adaptBans,
    adaptMaze,
    adaptCdp,
    adaptCdpEvents,
    adaptConfig
  };
})(window);
