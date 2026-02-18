// @ts-check

export const createRuntimeEffects = (options = {}) => {
  const win = options.window || window;
  const nav = options.navigator || navigator;
  const requestImpl =
    typeof options.request === 'function' ? options.request : null;

  const setTimeoutFn =
    typeof options.setTimeout === 'function'
      ? options.setTimeout
      : win.setTimeout.bind(win);
  const clearTimeoutFn =
    typeof options.clearTimeout === 'function'
      ? options.clearTimeout
      : win.clearTimeout.bind(win);
  const requestAnimationFrameFn =
    typeof options.requestAnimationFrame === 'function'
      ? options.requestAnimationFrame
      : win.requestAnimationFrame.bind(win);
  const cancelAnimationFrameFn =
    typeof options.cancelAnimationFrame === 'function'
      ? options.cancelAnimationFrame
      : (typeof win.cancelAnimationFrame === 'function'
        ? win.cancelAnimationFrame.bind(win)
        : (() => {}));

  const copyText = async (text = '') => {
    const value = String(text || '');
    if (!nav || !nav.clipboard || typeof nav.clipboard.writeText !== 'function') {
      throw new Error('Clipboard API unavailable');
    }
    await nav.clipboard.writeText(value);
  };

  // Resolve window.fetch at call time so late-installed wrappers (for example
  // admin session CSRF injection) are respected.
  const request = (input, init = {}) => {
    if (requestImpl) return requestImpl(input, init);
    return win.fetch(input, init);
  };

  const setTimer = (task, ms = 0) => setTimeoutFn(task, ms);
  const clearTimer = (id) => clearTimeoutFn(id);
  const requestFrame = (task) => requestAnimationFrameFn(task);
  const cancelFrame = (id) => cancelAnimationFrameFn(id);

  const readHash = () => String((win.location && win.location.hash) || '');
  const setHash = (value = '') => {
    const normalized = String(value || '').replace(/^#/, '');
    win.location.hash = `#${normalized}`;
  };
  const replaceHash = (value = '') => {
    const normalized = String(value || '').replace(/^#/, '');
    const nextHash = `#${normalized}`;
    win.history.replaceState(
      null,
      '',
      `${win.location.pathname}${win.location.search}${nextHash}`
    );
  };
  const onHashChange = (handler) => {
    if (typeof handler !== 'function') return () => {};
    win.addEventListener('hashchange', handler);
    return () => win.removeEventListener('hashchange', handler);
  };

  return {
    request,
    copyText,
    setTimer,
    clearTimer,
    requestFrame,
    cancelFrame,
    readHash,
    setHash,
    replaceHash,
    onHashChange
  };
};
