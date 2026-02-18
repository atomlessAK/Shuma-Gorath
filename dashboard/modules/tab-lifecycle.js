// @ts-check

export const DASHBOARD_TABS = Object.freeze(['monitoring', 'ip-bans', 'status', 'config', 'tuning']);
export const DEFAULT_DASHBOARD_TAB = 'monitoring';

export const normalizeTab = (raw) => {
  const normalized = String(raw || '').trim().toLowerCase();
  return DASHBOARD_TABS.includes(normalized) ? normalized : DEFAULT_DASHBOARD_TAB;
};

const resolveController = (controller) => {
  const source = controller || {};
  return {
    init: typeof source.init === 'function' ? source.init : () => {},
    mount: typeof source.mount === 'function' ? source.mount : () => {},
    unmount: typeof source.unmount === 'function' ? source.unmount : () => {},
    refresh: typeof source.refresh === 'function' ? source.refresh : async () => {}
  };
};

export const createTabLifecycleCoordinator = (options = {}) => {
  const linkSelector = options.linkSelector || '[data-dashboard-tab-link]';
  const monitoringPanelId = options.monitoringPanelId || 'dashboard-panel-monitoring';
  const adminSectionId = options.adminSectionId || 'dashboard-admin-section';
  const adminPanelSelector = options.adminPanelSelector || '#dashboard-admin-section [data-dashboard-tab-panel]';
  const onActiveTabChange =
    typeof options.onActiveTabChange === 'function' ? options.onActiveTabChange : null;
  const effects = options.effects || {};
  const readHash = typeof effects.readHash === 'function'
    ? effects.readHash
    : () => String((window.location && window.location.hash) || '');
  const replaceHash = typeof effects.replaceHash === 'function'
    ? effects.replaceHash
    : (value) => {
      const normalized = String(value || '').replace(/^#/, '');
      history.replaceState(
        null,
        '',
        `${window.location.pathname}${window.location.search}#${normalized}`
      );
    };
  const setHash = typeof effects.setHash === 'function'
    ? effects.setHash
    : (value) => {
      window.location.hash = String(value || '').replace(/^#/, '');
    };
  const onHashChange = typeof effects.onHashChange === 'function'
    ? effects.onHashChange
    : (handler) => {
      window.addEventListener('hashchange', handler);
      return () => window.removeEventListener('hashchange', handler);
    };
  const requestFrame = typeof effects.requestFrame === 'function'
    ? effects.requestFrame
    : (task) => window.requestAnimationFrame(task);

  const controllerSource = options.controllers || {};
  const controllers = {};
  DASHBOARD_TABS.forEach((tab) => {
    controllers[tab] = resolveController(controllerSource[tab]);
  });

  let activeTab = DEFAULT_DASHBOARD_TAB;
  let initialized = false;
  let unbindFns = [];

  const applyDomState = (tabName) => {
    const tab = normalizeTab(tabName);
    const links = Array.from(document.querySelectorAll(linkSelector));
    links.forEach((link) => {
      const linkTab = normalizeTab(link.dataset.dashboardTabLink);
      const selected = linkTab === tab;
      link.setAttribute('aria-selected', selected ? 'true' : 'false');
      link.tabIndex = selected ? 0 : -1;
      link.classList.toggle('active', selected);
    });

    const monitoringPanel = document.getElementById(monitoringPanelId);
    if (monitoringPanel) {
      const isMonitoringVisible = tab === 'monitoring';
      monitoringPanel.hidden = !isMonitoringVisible;
      monitoringPanel.setAttribute('aria-hidden', isMonitoringVisible ? 'false' : 'true');
      monitoringPanel.tabIndex = isMonitoringVisible ? 0 : -1;
    }

    const adminSection = document.getElementById(adminSectionId);
    if (adminSection) {
      const isAdminVisible = tab !== 'monitoring';
      adminSection.hidden = !isAdminVisible;
      adminSection.setAttribute('aria-hidden', isAdminVisible ? 'false' : 'true');
    }

    const adminPanels = Array.from(document.querySelectorAll(adminPanelSelector));
    if (tab === 'monitoring') {
      adminPanels.forEach((panel) => {
        panel.hidden = true;
      });
      return;
    }

    let matched = false;
    adminPanels.forEach((panel) => {
      const panelTab = normalizeTab(panel.dataset.dashboardTabPanel);
      const show = panelTab === tab;
      panel.hidden = !show;
      panel.setAttribute('aria-hidden', show ? 'false' : 'true');
      panel.tabIndex = show ? 0 : -1;
      if (show) matched = true;
    });

    if (!matched) {
      adminPanels.forEach((panel) => {
        const panelTab = normalizeTab(panel.dataset.dashboardTabPanel);
        const show = panelTab === 'config';
        panel.hidden = !show;
        panel.setAttribute('aria-hidden', show ? 'false' : 'true');
        panel.tabIndex = show ? 0 : -1;
      });
    }
  };

  const setActiveTab = (tabName, reason) => {
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
  };

  const syncFromHash = () => {
    const hash = readHash();
    const requested = normalizeTab(String(hash || '').replace(/^#/, ''));
    if (hash !== `#${requested}`) {
      replaceHash(requested);
    }
    setActiveTab(requested, 'hash');
  };

  const focusActivePanel = (tabName) => {
    const tab = normalizeTab(tabName);
    const selector = tab === 'monitoring'
      ? `#${monitoringPanelId}`
      : `${adminPanelSelector}[data-dashboard-tab-panel="${tab}"]`;
    const panel = document.querySelector(selector);
    if (panel && typeof panel.focus === 'function') {
      panel.focus();
    }
  };

  const activate = (tabName, reason = 'programmatic') => {
    const tab = normalizeTab(tabName);
    if (readHash() !== `#${tab}`) {
      setHash(tab);
    } else {
      setActiveTab(tab, reason);
    }
    if (reason === 'keyboard') {
      requestFrame(() => focusActivePanel(tab));
    }
  };

  const focusByOffset = (offset) => {
    const links = Array.from(document.querySelectorAll(linkSelector));
    if (links.length === 0) return;
    const currentIndex = links.findIndex((link) => link.getAttribute('aria-selected') === 'true');
    const startIndex = currentIndex >= 0 ? currentIndex : 0;
    const nextIndex = (startIndex + offset + links.length) % links.length;
    const target = links[nextIndex];
    target.focus();
    activate(target.dataset.dashboardTabLink, 'keyboard');
  };

  const bindLinkInteractions = () => {
    document.querySelectorAll(linkSelector).forEach((link) => {
      const onClick = (event) => {
        event.preventDefault();
        activate(link.dataset.dashboardTabLink, 'click');
      };
      const onKeydown = (event) => {
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
      };
      link.addEventListener('click', onClick);
      link.addEventListener('keydown', onKeydown);
      unbindFns.push(() => {
        link.removeEventListener('click', onClick);
        link.removeEventListener('keydown', onKeydown);
      });
    });
  };

  const init = () => {
    if (initialized) return;
    DASHBOARD_TABS.forEach((tab) => {
      controllers[tab].init({ tab });
    });
    bindLinkInteractions();
    unbindFns.push(onHashChange(syncFromHash));
    initialized = true;
    syncFromHash();
  };

  const destroy = () => {
    if (!initialized) return;
    controllers[activeTab].unmount({ tab: activeTab, nextTab: null, reason: 'destroy' });
    unbindFns.forEach((unbind) => unbind());
    unbindFns = [];
    initialized = false;
  };

  const refreshActive = async (context = {}) => controllers[activeTab].refresh({
    tab: activeTab,
    reason: context.reason || 'manual'
  });

  return {
    init,
    destroy,
    activate,
    refreshActive,
    getActiveTab: () => activeTab,
    normalizeTab
  };
};
