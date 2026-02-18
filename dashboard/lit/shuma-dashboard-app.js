// @ts-check

import { LitElement, html } from 'lit';

/**
 * Root dashboard app shell for Lit migration.
 *
 * This first cut keeps existing light-DOM markup and bootstraps the legacy
 * dashboard orchestrator from a single app-surface custom element.
 */
class ShumaDashboardApp extends LitElement {
  static properties = {
    bootState: { state: true }
  };

  constructor() {
    super();
    this.bootState = 'pending';
    this.bootError = '';
    this.bootPromise = null;
    this.unmountDashboard = null;
  }

  connectedCallback() {
    super.connectedCallback();
    if (!this.bootPromise) {
      this.bootPromise = this.bootstrapLegacyDashboard();
    }
  }

  disconnectedCallback() {
    if (typeof this.unmountDashboard === 'function') {
      this.unmountDashboard();
      this.unmountDashboard = null;
    }
    super.disconnectedCallback();
  }

  async bootstrapLegacyDashboard() {
    try {
      const dashboard = await import('../dashboard.js');
      if (typeof dashboard.mountDashboard !== 'function') {
        throw new Error('mountDashboard export not found');
      }
      this.unmountDashboard = dashboard.mountDashboard();
      this.bootState = 'ready';
      this.dispatchEvent(new CustomEvent('shuma-dashboard-booted', {
        bubbles: true,
        composed: true
      }));
    } catch (err) {
      this.bootState = 'error';
      this.bootError = err && err.message ? String(err.message) : 'Unknown dashboard boot error';
      console.error('Lit dashboard app bootstrap failed:', err);
    }
  }

  render() {
    return html`
      <slot></slot>
      ${this.bootState === 'error'
        ? html`<div class="message error">Dashboard bootstrap failed: ${this.bootError}</div>`
        : ''}
    `;
  }
}

customElements.define('shuma-dashboard-app', ShumaDashboardApp);
