export function createDashboardTabStateRuntime(options = {}) {
  const getStateStore =
    typeof options.getStateStore === 'function' ? options.getStateStore : () => null;

  const withStore = (task) => {
    const store = getStateStore();
    if (!store) return false;
    task(store);
    return true;
  };

  function showTabLoading(tab, message = 'Loading...') {
    withStore((store) => {
      store.clearTabError(tab);
      store.setTabEmpty(tab, false, '');
      store.setTabLoading(tab, true, message);
    });
  }

  function showTabError(tab, message) {
    withStore((store) => {
      store.setTabEmpty(tab, false, '');
      store.setTabError(tab, message);
    });
  }

  function showTabEmpty(tab, message) {
    withStore((store) => {
      store.clearTabError(tab);
      store.setTabLoading(tab, false, '');
      store.setTabEmpty(tab, true, message);
      store.markTabUpdated(tab);
    });
  }

  function clearTabStateMessage(tab) {
    withStore((store) => {
      store.setTabLoading(tab, false, '');
      store.setTabEmpty(tab, false, '');
      store.clearTabError(tab);
      store.markTabUpdated(tab);
    });
  }

  return {
    showTabLoading,
    showTabError,
    showTabEmpty,
    clearTabStateMessage
  };
}
