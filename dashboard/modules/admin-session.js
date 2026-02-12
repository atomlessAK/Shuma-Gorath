(function (global) {
  function create(options = {}) {
    const state = {
      authenticated: false,
      csrfToken: ''
    };

    const nativeFetch = global.fetch.bind(global);

    function isWriteMethod(method) {
      const upper = String(method || 'GET').toUpperCase();
      return upper === 'POST' || upper === 'PUT' || upper === 'PATCH' || upper === 'DELETE';
    }

    function requestUrlOf(input) {
      if (typeof input === 'string') return input;
      if (input && typeof input.url === 'string') return input.url;
      return '';
    }

    function requestMethodOf(input, init) {
      if (init && init.method) return init.method;
      if (input && input.method) return input.method;
      return 'GET';
    }

    function isAdminRequestUrl(url) {
      try {
        const resolved = new URL(url, global.location.origin);
        return resolved.pathname.startsWith('/admin/');
      } catch (_e) {
        return false;
      }
    }

    function refreshUiState() {
      if (typeof options.refreshCoreActionButtonsState === 'function') {
        options.refreshCoreActionButtonsState();
      }
    }

    function resolveEndpoint() {
      if (typeof options.resolveAdminApiEndpoint !== 'function') return '';
      const resolved = options.resolveAdminApiEndpoint();
      if (!resolved) return '';
      if (typeof resolved === 'string') return resolved;
      if (typeof resolved.endpoint === 'string') return resolved.endpoint;
      return '';
    }

    function setAdminSession(authenticated, csrfToken = '') {
      state.authenticated = Boolean(authenticated);
      state.csrfToken = state.authenticated ? String(csrfToken || '') : '';
      refreshUiState();
    }

    global.fetch = function patchedFetch(input, init = {}) {
      const url = requestUrlOf(input);
      if (!isAdminRequestUrl(url)) {
        return nativeFetch(input, init);
      }

      const method = requestMethodOf(input, init);
      const headers = new Headers(init.headers || (input instanceof Request ? input.headers : undefined));
      const authHeader = headers.get('Authorization') || headers.get('authorization') || '';

      if (/^Bearer\s*$/i.test(authHeader.trim())) {
        headers.delete('Authorization');
        headers.delete('authorization');
      }

      if (state.authenticated && isWriteMethod(method) && state.csrfToken) {
        if (!headers.has('X-Shuma-CSRF')) {
          headers.set('X-Shuma-CSRF', state.csrfToken);
        }
      }

      const nextInit = {
        ...init,
        headers,
        credentials: 'same-origin'
      };
      return nativeFetch(input, nextInit);
    };

    function hasValidApiContext() {
      return state.authenticated;
    }

    function getAdminContext(messageTarget) {
      const endpoint = resolveEndpoint();
      if (!endpoint) {
        if (messageTarget) {
          messageTarget.textContent = 'Unable to resolve admin API endpoint from the current page origin.';
          messageTarget.className = 'message error';
        }
        refreshUiState();
        return null;
      }

      if (!state.authenticated) {
        if (messageTarget) {
          messageTarget.textContent = 'Login required. Go to /dashboard/login.html.';
          messageTarget.className = 'message warning';
        }
        refreshUiState();
        return null;
      }

      refreshUiState();
      return { endpoint, apikey: '', sessionAuth: true, csrfToken: state.csrfToken };
    }

    async function restoreAdminSession() {
      const endpoint = resolveEndpoint();
      if (!endpoint) {
        setAdminSession(false);
        return false;
      }
      try {
        const resp = await fetch(`${endpoint}/admin/session`);
        if (!resp.ok) {
          setAdminSession(false);
          return false;
        }
        const data = await resp.json();
        if (data && data.authenticated === true && data.method === 'session') {
          setAdminSession(true, data.csrf_token || '');
          return true;
        }
        setAdminSession(false);
        return false;
      } catch (_e) {
        setAdminSession(false);
        return false;
      }
    }

    function bindLogoutButton(buttonId = 'logout-btn', messageId = 'admin-msg') {
      const button = document.getElementById(buttonId);
      if (!button) return;
      button.onclick = async function onLogoutClick() {
        const message = document.getElementById(messageId);
        const endpoint = resolveEndpoint();
        if (!endpoint) return;

        this.disabled = true;
        this.textContent = 'Logging out...';
        try {
          await fetch(`${endpoint}/admin/logout`, { method: 'POST' });
        } catch (_e) {}
        setAdminSession(false);
        if (message) {
          message.textContent = 'Logged out';
          message.className = 'message success';
        }
        this.textContent = 'Logout';
        refreshUiState();
        if (typeof options.redirectToLogin === 'function') {
          options.redirectToLogin();
        }
      };
    }

    return {
      hasValidApiContext,
      getAdminContext,
      restoreAdminSession,
      bindLogoutButton,
      getState: () => ({ ...state })
    };
  }

  global.ShumaDashboardAdminSession = {
    create
  };
})(window);
