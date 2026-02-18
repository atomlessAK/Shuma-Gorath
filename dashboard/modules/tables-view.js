// @ts-check

import * as format from './core/format.js';
import * as domModule from './core/dom.js';

export const create = (options = {}) => {
  const escapeHtml = typeof options.escapeHtml === 'function'
    ? options.escapeHtml
    : format.escapeHtml;
  const onQuickUnban = typeof options.onQuickUnban === 'function'
    ? options.onQuickUnban
    : async () => {};
  const domCache = domModule.createCache({ document });
  const query = domCache.query;
  const byId = domCache.byId;
  const expandedBanDetails = new Set();
  const delegatedBanBodies = new WeakSet();

  const canPatchTableRows = (tbody) => (
    Boolean(tbody) &&
    typeof tbody.appendChild === 'function' &&
    typeof tbody.replaceChild === 'function' &&
    typeof tbody.removeChild === 'function' &&
    typeof document !== 'undefined' &&
    typeof document.createElement === 'function'
  );

  const tableRows = (tbody) => {
    if (!tbody || !tbody.children) return [];
    return Array.from(tbody.children);
  };

  const buildStableKey = (parts, duplicates) => {
    const base = parts.map((part) => String(part || '')).join('|');
    const count = duplicates.get(base) || 0;
    duplicates.set(base, count + 1);
    return count === 0 ? base : `${base}#${count}`;
  };

  const ensureBanTableActionDelegation = (tbody) => {
    if (!tbody || delegatedBanBodies.has(tbody)) return;
    if (typeof tbody.addEventListener !== 'function') return;

    tbody.addEventListener('click', async (event) => {
      const target = event.target;
      const button = target && typeof target.closest === 'function'
        ? target.closest('button')
        : null;
      const withinTbody = typeof tbody.contains === 'function' ? tbody.contains(button) : true;
      if (!button || !withinTbody) return;

      if (button.classList.contains('ban-details-toggle')) {
        const detailId = String(button.dataset.target || '').trim();
        if (!detailId) return;
        const detailRow = byId(detailId);
        if (!detailRow || !detailRow.classList) return;

        detailRow.classList.toggle('hidden');
        const isHidden = detailRow.classList.contains('hidden');
        if (isHidden) {
          expandedBanDetails.delete(detailId);
        } else {
          expandedBanDetails.add(detailId);
        }
        button.textContent = isHidden ? 'Details' : 'Hide';
        return;
      }

      if (button.classList.contains('unban-quick')) {
        const ip = String(button.dataset.ip || '').trim();
        if (!ip) return;
        const previousText = button.textContent;
        button.disabled = true;
        try {
          await onQuickUnban(ip);
        } finally {
          button.disabled = false;
          button.textContent = previousText || 'Unban';
        }
      }
    });

    delegatedBanBodies.add(tbody);
    if (tbody.dataset && typeof tbody.dataset === 'object') {
      tbody.dataset.banDelegationBound = 'true';
    }
  };

  const updateBansTable = (bans) => {
    const tbody = query('#bans-table tbody');
    if (!tbody) return;
    ensureBanTableActionDelegation(tbody);

    if (!Array.isArray(bans) || bans.length === 0) {
      expandedBanDetails.clear();
      domModule.setHtml(
        tbody,
        '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No active bans</td></tr>'
      );
      return;
    }

    const duplicates = new Map();
    const rows = [];

    for (const ban of bans) {
      const now = Math.floor(Date.now() / 1000);
      const isExpired = Number(ban.expires || 0) < now;
      const bannedAt = ban.banned_at ? new Date(ban.banned_at * 1000).toLocaleString() : '-';
      const expiresAt = new Date(Number(ban.expires || 0) * 1000).toLocaleString();
      const safeIp = escapeHtml(ban.ip || '-');
      const safeReason = escapeHtml(ban.reason || 'unknown');
      const signals = (ban.fingerprint && Array.isArray(ban.fingerprint.signals))
        ? ban.fingerprint.signals
        : [];
      const signalBadges = signals.length
        ? signals.map((signal) => `<span class="ban-signal-badge">${escapeHtml(signal)}</span>`).join('')
        : '<span class="text-muted">none</span>';
      const baseKey = buildStableKey(
        [ban.ip, ban.reason, ban.banned_at, ban.expires],
        duplicates
      );
      const detailsId = `ban-detail-${baseKey.replace(/[^a-zA-Z0-9_-]/g, '-')}`;
      const detailExpanded = expandedBanDetails.has(detailsId);

      const summaryHtml = `
      <td><code>${safeIp}</code></td>
      <td>${safeReason}</td>
      <td>${bannedAt}</td>
      <td class="${isExpired ? 'expired' : ''}">${isExpired ? 'Expired' : expiresAt}</td>
      <td>${signalBadges}</td>
      <td class="ban-action-cell">
        <button class="ban-details-toggle" data-target="${detailsId}">${detailExpanded ? 'Hide' : 'Details'}</button>
        <button class="unban-quick" data-ip="${ban.ip}">Unban</button>
      </td>
    `;

      const score =
        ban.fingerprint && typeof ban.fingerprint.score === 'number' ? ban.fingerprint.score : null;
      const summary = ban.fingerprint && ban.fingerprint.summary
        ? ban.fingerprint.summary
        : 'No additional fingerprint details.';
      const safeSummary = escapeHtml(summary);
      const detailHtml = `
      <td colspan="6">
        <div class="ban-detail-content">
          <div><strong>Score:</strong> ${score === null ? 'n/a' : score}</div>
          <div><strong>Summary:</strong> ${safeSummary}</div>
        </div>
      </td>
    `;
      rows.push({
        key: `summary:${baseKey}`,
        html: summaryHtml,
        attrs: { className: 'ban-summary-row', id: '' }
      });
      rows.push({
        key: `detail:${baseKey}`,
        html: detailHtml,
        attrs: {
          className: `ban-detail-row${detailExpanded ? '' : ' hidden'}`,
          id: detailsId
        }
      });
    }

    const nextDetailIds = new Set(
      rows
        .filter((row) => row.attrs && row.attrs.id)
        .map((row) => row.attrs.id)
    );
    Array.from(expandedBanDetails).forEach((detailId) => {
      if (!nextDetailIds.has(detailId)) {
        expandedBanDetails.delete(detailId);
      }
    });

    const decoratedRows = rows.map((row) => ({
      key: row.key,
      html: row.html,
      attrs: row.attrs || null
    }));

    if (!canPatchTableRows(tbody)) {
      const fallbackHtml = decoratedRows
        .map((row) => {
          const classAttr = row.attrs && row.attrs.className ? ` class=\"${escapeHtml(row.attrs.className)}\"` : '';
          const idAttr = row.attrs && row.attrs.id ? ` id=\"${escapeHtml(row.attrs.id)}\"` : '';
          return `<tr data-row-key=\"${escapeHtml(row.key)}\"${classAttr}${idAttr}>${row.html}</tr>`;
        })
        .join('');
      domModule.setHtml(tbody, fallbackHtml);
      return;
    }

    const existingRows = tableRows(tbody);
    for (let index = 0; index < decoratedRows.length; index += 1) {
      const nextRow = decoratedRows[index];
      const currentRow = existingRows[index];
      if (currentRow) {
        const currentKey = String(currentRow?.dataset?.rowKey || '');
        if (currentKey === nextRow.key) {
          if (currentRow.innerHTML !== nextRow.html) {
            domModule.setHtml(currentRow, nextRow.html);
          }
          if (nextRow.attrs) {
            currentRow.className = nextRow.attrs.className || '';
            currentRow.id = nextRow.attrs.id || '';
          }
          continue;
        }

        const replacement = document.createElement('tr');
        replacement.dataset.rowKey = nextRow.key;
        if (nextRow.attrs) {
          replacement.className = nextRow.attrs.className || '';
          replacement.id = nextRow.attrs.id || '';
        }
        domModule.setHtml(replacement, nextRow.html);
        tbody.replaceChild(replacement, currentRow);
        existingRows[index] = replacement;
        continue;
      }

      const appended = document.createElement('tr');
      appended.dataset.rowKey = nextRow.key;
      if (nextRow.attrs) {
        appended.className = nextRow.attrs.className || '';
        appended.id = nextRow.attrs.id || '';
      }
      domModule.setHtml(appended, nextRow.html);
      tbody.appendChild(appended);
      existingRows.push(appended);
    }

    const nextCount = decoratedRows.length;
    while (tableRows(tbody).length > nextCount) {
      const rowList = tableRows(tbody);
      const tail = rowList[rowList.length - 1];
      if (!tail) break;
      tbody.removeChild(tail);
    }
  };

  return {
    updateBansTable
  };
};
