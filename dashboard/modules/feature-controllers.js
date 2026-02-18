// @ts-check

const noop = () => {};

const createFeatureController = (options = {}) => {
  const tab = String(options.tab || '').trim();
  const onInit = typeof options.onInit === 'function' ? options.onInit : noop;
  const onMount = typeof options.onMount === 'function' ? options.onMount : noop;
  const onUnmount = typeof options.onUnmount === 'function' ? options.onUnmount : noop;
  const onRefresh = typeof options.onRefresh === 'function' ? options.onRefresh : async () => {};

  return {
    init: (context = {}) => onInit({ ...context, tab }),
    mount: (context = {}) => onMount({ ...context, tab }),
    unmount: (context = {}) => onUnmount({ ...context, tab }),
    refresh: (context = {}) => onRefresh({ ...context, tab })
  };
};

const createMountedRefreshController = (tab, options = {}) => {
  const notifyTabMounted =
    typeof options.notifyTabMounted === 'function' ? options.notifyTabMounted : noop;
  const notifyTabUnmounted =
    typeof options.notifyTabUnmounted === 'function' ? options.notifyTabUnmounted : noop;
  const refreshTab = typeof options.refreshTab === 'function' ? options.refreshTab : async () => {};
  const hasValidApiContext =
    typeof options.hasValidApiContext === 'function' ? options.hasValidApiContext : () => false;

  return createFeatureController({
    tab,
    onMount: async ({ tab: activeTab }) => {
      notifyTabMounted(activeTab);
      if (hasValidApiContext()) {
        await refreshTab(activeTab, 'tab-mount');
      }
    },
    onUnmount: ({ tab: activeTab }) => {
      notifyTabUnmounted(activeTab);
    },
    onRefresh: ({ tab: activeTab, reason = 'manual' }) => refreshTab(activeTab, reason)
  });
};

const createMonitoringController = (options = {}) =>
  createMountedRefreshController('monitoring', options);

const createIpBansController = (options = {}) =>
  createMountedRefreshController('ip-bans', options);

const createStatusController = (options = {}) =>
  createMountedRefreshController('status', options);

const createConfigController = (options = {}) =>
  createMountedRefreshController('config', options);

const createTuningController = (options = {}) =>
  createMountedRefreshController('tuning', options);

export const createDashboardFeatureControllers = (options = {}) => ({
  monitoring: createMonitoringController(options),
  'ip-bans': createIpBansController(options),
  status: createStatusController(options),
  config: createConfigController(options),
  tuning: createTuningController(options)
});

