// @ts-check

export const create = (options = {}) => {
  const query = typeof options.query === 'function' ? options.query : () => null;
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;

  const stateElement = (tab) => query(`[data-tab-state="${tab}"]`);

  const setMessage = (tab, kind, message) => {
    const stateEl = stateElement(tab);
    if (!stateEl) return;
    const normalizedKind = kind === 'error' || kind === 'loading' || kind === 'empty' ? kind : '';
    if (!normalizedKind) {
      stateEl.hidden = true;
      stateEl.textContent = '';
      stateEl.className = 'tab-state';
      return;
    }
    stateEl.hidden = false;
    stateEl.textContent = String(message || '');
    stateEl.className = `tab-state tab-state--${normalizedKind}`;
  };

  const withStateStore = (cb) => {
    const stateStore = getStateStore();
    if (!stateStore) return;
    cb(stateStore);
  };

  const showLoading = (tab, message = 'Loading...') => {
    withStateStore((stateStore) => {
      stateStore.setTabLoading(tab, true);
      stateStore.clearTabError(tab);
    });
    setMessage(tab, 'loading', message);
  };

  const showError = (tab, message) => {
    withStateStore((stateStore) => {
      stateStore.setTabError(tab, message);
      stateStore.setTabEmpty(tab, false);
    });
    setMessage(tab, 'error', message);
  };

  const showEmpty = (tab, message) => {
    withStateStore((stateStore) => {
      stateStore.setTabEmpty(tab, true);
      stateStore.clearTabError(tab);
      stateStore.markTabUpdated(tab);
    });
    setMessage(tab, 'empty', message);
  };

  const clear = (tab) => {
    withStateStore((stateStore) => {
      stateStore.setTabLoading(tab, false);
      stateStore.setTabEmpty(tab, false);
      stateStore.clearTabError(tab);
      stateStore.markTabUpdated(tab);
    });
    setMessage(tab, '', '');
  };

  return {
    showLoading,
    showError,
    showEmpty,
    clear
  };
};
