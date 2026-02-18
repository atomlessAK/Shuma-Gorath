export function createDashboardEffects(options = {}) {
  const win = options.window || window;
  const doc = options.document || document;

  const request =
    typeof options.request === 'function'
      ? options.request
      : (input, init = {}) => win.fetch(input, init);

  const setTimer =
    typeof options.setTimeout === 'function'
      ? options.setTimeout
      : win.setTimeout.bind(win);

  const clearTimer =
    typeof options.clearTimeout === 'function'
      ? options.clearTimeout
      : win.clearTimeout.bind(win);

  const requestFrame =
    typeof options.requestAnimationFrame === 'function'
      ? options.requestAnimationFrame
      : win.requestAnimationFrame.bind(win);

  const now =
    typeof options.now === 'function'
      ? options.now
      : () => {
          if (win.performance && typeof win.performance.now === 'function') {
            return win.performance.now();
          }
          return Date.now();
        };

  const readHashTab = () => String(win.location.hash || '').replace(/^#/, '');

  const writeHashTab = (tab, opts = {}) => {
    const normalized = String(tab || '').replace(/^#/, '');
    if (!normalized) return;
    const nextHash = `#${normalized}`;
    if (win.location.hash === nextHash) return;

    const replace = opts && opts.replace === true;
    if (replace) {
      const nextUrl = `${win.location.pathname}${win.location.search}${nextHash}`;
      win.history.replaceState(null, '', nextUrl);
      return;
    }
    win.location.hash = normalized;
  };

  const onHashChange = (handler) => {
    if (typeof handler !== 'function') return () => {};
    win.addEventListener('hashchange', handler);
    return () => win.removeEventListener('hashchange', handler);
  };

  const onVisibilityChange = (handler) => {
    if (typeof handler !== 'function') return () => {};
    doc.addEventListener('visibilitychange', handler);
    return () => doc.removeEventListener('visibilitychange', handler);
  };

  const isPageVisible = () => doc.visibilityState !== 'hidden';

  const redirect = (path) => {
    win.location.replace(String(path || '/dashboard/login.html'));
  };

  const buildLoginRedirectPath =
    typeof options.buildLoginRedirectPath === 'function'
      ? options.buildLoginRedirectPath
      : () => {
          const pathname = String((win.location && win.location.pathname) || '/dashboard/index.html');
          const search = String((win.location && win.location.search) || '');
          const hash = String((win.location && win.location.hash) || '');
          const next = encodeURIComponent(`${pathname}${search}${hash}`);
          return `/dashboard/login.html?next=${next}`;
        };

  const focusTab =
    typeof options.focusTab === 'function'
      ? options.focusTab
      : (tab) => {
          const normalized = String(tab || '').replace(/^#/, '');
          if (!normalized) return false;
          const target = doc.getElementById(`dashboard-tab-${normalized}`);
          if (target && typeof target.focus === 'function') {
            target.focus();
            return true;
          }
          return false;
        };

  return {
    request,
    setTimer,
    clearTimer,
    now,
    requestFrame,
    readHashTab,
    writeHashTab,
    onHashChange,
    onVisibilityChange,
    isPageVisible,
    redirect,
    buildLoginRedirectPath,
    focusTab
  };
}
