(function (global) {
  const DASHBOARD_TABS = Object.freeze(['monitoring', 'ip-bans', 'status', 'config', 'tuning']);
  const DEFAULT_DASHBOARD_TAB = 'monitoring';

  function normalizeTab(raw) {
    const normalized = String(raw || '').trim().toLowerCase();
    return DASHBOARD_TABS.includes(normalized) ? normalized : DEFAULT_DASHBOARD_TAB;
  }

  function resolveController(controller) {
    const source = controller || {};
    return {
      init: typeof source.init === 'function' ? source.init : function noopInit() {},
      mount: typeof source.mount === 'function' ? source.mount : function noopMount() {},
      unmount: typeof source.unmount === 'function' ? source.unmount : function noopUnmount() {},
      refresh: typeof source.refresh === 'function' ? source.refresh : async function noopRefresh() {}
    };
  }

  function createTabLifecycleCoordinator(options = {}) {
    const linkSelector = options.linkSelector || '[data-dashboard-tab-link]';
    const monitoringPanelId = options.monitoringPanelId || 'dashboard-panel-monitoring';
    const adminSectionId = options.adminSectionId || 'dashboard-admin-section';
    const adminPanelSelector = options.adminPanelSelector || '#dashboard-admin-section [data-dashboard-tab-panel]';
    const onActiveTabChange =
      typeof options.onActiveTabChange === 'function' ? options.onActiveTabChange : null;

    const controllerSource = options.controllers || {};
    const controllers = {};
    DASHBOARD_TABS.forEach(tab => {
      controllers[tab] = resolveController(controllerSource[tab]);
    });

    let activeTab = DEFAULT_DASHBOARD_TAB;
    let initialized = false;

    function applyDomState(tabName) {
      const tab = normalizeTab(tabName);
      const links = Array.from(document.querySelectorAll(linkSelector));
      links.forEach(link => {
        const linkTab = normalizeTab(link.dataset.dashboardTabLink);
        const selected = linkTab === tab;
        link.setAttribute('aria-selected', selected ? 'true' : 'false');
        link.tabIndex = selected ? 0 : -1;
        link.classList.toggle('active', selected);
      });

      const monitoringPanel = document.getElementById(monitoringPanelId);
      if (monitoringPanel) {
        monitoringPanel.hidden = tab !== 'monitoring';
      }

      const adminSection = document.getElementById(adminSectionId);
      if (adminSection) {
        adminSection.hidden = tab === 'monitoring';
      }

      const adminPanels = Array.from(document.querySelectorAll(adminPanelSelector));
      if (tab === 'monitoring') {
        adminPanels.forEach(panel => {
          panel.hidden = true;
        });
        return;
      }

      let matched = false;
      adminPanels.forEach(panel => {
        const panelTab = normalizeTab(panel.dataset.dashboardTabPanel);
        const show = panelTab === tab;
        panel.hidden = !show;
        if (show) matched = true;
      });

      if (!matched) {
        adminPanels.forEach(panel => {
          const panelTab = normalizeTab(panel.dataset.dashboardTabPanel);
          panel.hidden = panelTab !== 'config';
        });
      }
    }

    function setActiveTab(tabName, reason) {
      const nextTab = normalizeTab(tabName);
      const prevTab = activeTab;
      if (initialized && prevTab === nextTab) {
        applyDomState(nextTab);
        return;
      }

      if (initialized) {
        controllers[prevTab].unmount({ tab: prevTab, nextTab, reason });
      }
      activeTab = nextTab;
      applyDomState(nextTab);
      controllers[nextTab].mount({ tab: nextTab, prevTab, reason });
      if (onActiveTabChange) {
        onActiveTabChange(nextTab, prevTab, reason);
      }
    }

    function syncFromHash() {
      const requested = normalizeTab((window.location.hash || '').replace(/^#/, ''));
      if (window.location.hash !== `#${requested}`) {
        history.replaceState(
          null,
          '',
          `${window.location.pathname}${window.location.search}#${requested}`
        );
      }
      setActiveTab(requested, 'hash');
    }

    function activate(tabName, reason = 'programmatic') {
      const tab = normalizeTab(tabName);
      if (window.location.hash !== `#${tab}`) {
        window.location.hash = tab;
      } else {
        setActiveTab(tab, reason);
      }
    }

    function focusByOffset(offset) {
      const links = Array.from(document.querySelectorAll(linkSelector));
      if (links.length === 0) return;
      const currentIndex = links.findIndex(link => link.getAttribute('aria-selected') === 'true');
      const startIndex = currentIndex >= 0 ? currentIndex : 0;
      const nextIndex = (startIndex + offset + links.length) % links.length;
      const target = links[nextIndex];
      target.focus();
      activate(target.dataset.dashboardTabLink, 'keyboard');
    }

    function bindLinkInteractions() {
      document.querySelectorAll(linkSelector).forEach(link => {
        link.addEventListener('click', event => {
          event.preventDefault();
          activate(link.dataset.dashboardTabLink, 'click');
        });
        link.addEventListener('keydown', event => {
          if (event.key === 'ArrowRight') {
            event.preventDefault();
            focusByOffset(1);
          } else if (event.key === 'ArrowLeft') {
            event.preventDefault();
            focusByOffset(-1);
          } else if (event.key === 'Home') {
            event.preventDefault();
            activate(DASHBOARD_TABS[0], 'keyboard');
          } else if (event.key === 'End') {
            event.preventDefault();
            activate(DASHBOARD_TABS[DASHBOARD_TABS.length - 1], 'keyboard');
          }
        });
      });
    }

    function init() {
      if (initialized) return;
      DASHBOARD_TABS.forEach(tab => {
        controllers[tab].init({ tab });
      });
      bindLinkInteractions();
      window.addEventListener('hashchange', syncFromHash);
      initialized = true;
      syncFromHash();
    }

    async function refreshActive(context = {}) {
      return controllers[activeTab].refresh({
        tab: activeTab,
        reason: context.reason || 'manual'
      });
    }

    return {
      init,
      activate,
      refreshActive,
      getActiveTab: function getActiveTab() {
        return activeTab;
      },
      normalizeTab
    };
  }

  global.ShumaDashboardTabLifecycle = {
    DASHBOARD_TABS,
    DEFAULT_DASHBOARD_TAB,
    normalizeTab,
    createTabLifecycleCoordinator
  };
})(window);
