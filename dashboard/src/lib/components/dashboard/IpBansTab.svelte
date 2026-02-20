<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import TableWrapper from './primitives/TableWrapper.svelte';
  import {
    classifyIpRangeFallback,
    formatIpRangeReasonLabel,
    isIpRangeBanLike,
    isIpRangeReason,
    parseIpRangeOutcome
  } from '../../domain/ip-range-policy.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let bansSnapshot = null;
  export let configSnapshot = null;
  export let onBan = null;
  export let onUnban = null;

  const MANUAL_BAN_FALLBACK_SECONDS = 21600;

  let expandedRows = {};
  let banIp = '';
  let unbanIp = '';
  let banDurationDays = 0;
  let banDurationHours = 6;
  let banDurationMinutes = 0;
  let lastAppliedConfigSnapshot = null;
  let banning = false;
  let unbanning = false;
  let banFilter = 'all';

  const formatTimestamp = (rawTs) => {
    const ts = Number(rawTs || 0);
    if (!Number.isFinite(ts) || ts <= 0) return '-';
    return new Date(ts * 1000).toLocaleString();
  };

  const isValidIpv4 = (value) => {
    const segments = String(value || '').split('.');
    if (segments.length !== 4) return false;
    return segments.every((segment) => {
      if (!/^\d{1,3}$/.test(segment)) return false;
      const numeric = Number(segment);
      return Number.isInteger(numeric) && numeric >= 0 && numeric <= 255;
    });
  };

  const isValidIpv6 = (value) => {
    const source = String(value || '').trim();
    if (!source.includes(':')) return false;
    return /^[0-9a-fA-F:]+$/.test(source);
  };

  const isValidIp = (value) => {
    const trimmed = String(value || '').trim();
    if (!trimmed || trimmed.length > 45) return false;
    return isValidIpv4(trimmed) || isValidIpv6(trimmed);
  };

  const durationPartsFromSeconds = (seconds, fallbackSeconds = MANUAL_BAN_FALLBACK_SECONDS) => {
    const parsed = Number.parseInt(seconds, 10);
    const safe = Number.isFinite(parsed) && parsed > 0 ? parsed : fallbackSeconds;
    const days = Math.floor(safe / 86400);
    const remainingAfterDays = safe - (days * 86400);
    const hours = Math.floor(remainingAfterDays / 3600);
    const remainingAfterHours = remainingAfterDays - (hours * 3600);
    const minutes = Math.floor(remainingAfterHours / 60);
    return {
      days,
      hours,
      minutes
    };
  };

  const applyConfiguredBanDuration = (config) => {
    const rawAdminDuration = config && typeof config === 'object' && config.ban_durations
      ? config.ban_durations.admin
      : undefined;
    const parts = durationPartsFromSeconds(rawAdminDuration, MANUAL_BAN_FALLBACK_SECONDS);
    banDurationDays = parts.days;
    banDurationHours = parts.hours;
    banDurationMinutes = parts.minutes;
  };

  const toKey = (ban, index) =>
    `${String(ban?.ip || '-')}:${String(ban?.reason || '-')}:${String(ban?.banned_at || 0)}:${String(ban?.expires || 0)}:${index}`;

  const isExpanded = (key) => expandedRows[key] === true;

  const formatIpRangeSourceLabel = (source) => {
    const normalized = String(source || '').trim().toLowerCase();
    if (normalized === 'managed') return 'Managed Set';
    if (normalized === 'custom') return 'Custom Rule';
    return normalized ? normalized : '-';
  };

  const deriveIpRangeBanMeta = (ban = {}) => {
    const reason = String(ban?.reason || '').trim();
    const parsed = parseIpRangeOutcome(ban?.fingerprint?.summary);
    const fallback = classifyIpRangeFallback(reason, parsed);
    const isIpRange = isIpRangeBanLike(ban) || isIpRangeReason(reason);
    return {
      isIpRange,
      reasonLabel: isIpRange ? formatIpRangeReasonLabel(reason) : '',
      source: parsed.source || '',
      sourceLabel: formatIpRangeSourceLabel(parsed.source),
      sourceId: parsed.sourceId || '',
      action: parsed.action || '',
      matchedCidr: parsed.matchedCidr || '',
      detection: parsed.detection || '',
      fallback: fallback !== 'none' ? fallback : ''
    };
  };

  function toggleDetails(key) {
    expandedRows = {
      ...expandedRows,
      [key]: !isExpanded(key)
    };
  }

  $: bans = Array.isArray(bansSnapshot?.bans) ? bansSnapshot.bans : [];
  $: if (configSnapshot && configSnapshot !== lastAppliedConfigSnapshot) {
    applyConfiguredBanDuration(configSnapshot);
    lastAppliedConfigSnapshot = configSnapshot;
  }
  $: banRows = bans.map((ban, index) => ({
    ban,
    key: toKey(ban, index),
    originalIndex: index,
    meta: deriveIpRangeBanMeta(ban)
  }));
  $: filteredBanRows = banFilter === 'ip-range'
    ? banRows.filter((row) => row.meta.isIpRange)
    : banRows;
  $: banDurationSeconds = (
    (Number(banDurationDays) * 24 * 60 * 60) +
    (Number(banDurationHours) * 60 * 60) +
    (Number(banDurationMinutes) * 60)
  );
  $: canBan = isValidIp(banIp) && banDurationSeconds > 0 && !banning;
  $: canUnban = isValidIp(unbanIp) && !unbanning;

  async function submitBan() {
    if (!canBan || typeof onBan !== 'function') return;
    banning = true;
    try {
      await onBan({
        ip: String(banIp || '').trim(),
        duration: Number(banDurationSeconds)
      });
      banIp = '';
    } finally {
      banning = false;
    }
  }

  async function submitUnban() {
    if (!canUnban || typeof onUnban !== 'function') return;
    unbanning = true;
    try {
      await onUnban({
        ip: String(unbanIp || '').trim()
      });
      unbanIp = '';
    } finally {
      unbanning = false;
    }
  }
</script>

<section
  id="dashboard-panel-ip-bans"
  class="admin-group admin-group--status"
  data-dashboard-tab-panel="ip-bans"
  aria-labelledby="dashboard-tab-ip-bans"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
          <TabStateMessage tab="ip-bans" status={tabStatus} />
          <div class="control-group panel-soft pad-sm">
            <div class="input-row">
              <label class="control-label control-label--wide" for="ip-ban-filter">Ban View</label>
              <select id="ip-ban-filter" class="input-field" aria-label="Filter ban table" bind:value={banFilter}>
                <option value="all">All Active Bans</option>
                <option value="ip-range">IP Range Policy Only</option>
              </select>
            </div>
            <p class="control-desc text-muted">
              {filteredBanRows.length} shown of {bans.length}. For false positives from CIDR policy,
              add a safe CIDR to <code>ip_range_emergency_allowlist</code> or disable the offending rule in Config.
            </p>
          </div>
          <TableWrapper>
            <table id="bans-table" class="panel panel-border bans-table-admin">
              <thead>
                <tr>
                  <th class="caps-label">IP Address</th>
                  <th class="caps-label">Reason</th>
                  <th class="caps-label">Banned At</th>
                  <th class="caps-label">Expires</th>
                  <th class="caps-label">Signals</th>
                  <th class="caps-label">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#if filteredBanRows.length === 0}
                  <tr>
                    <td colspan="6" style="text-align: center; color: #6b7280;">
                      {banFilter === 'ip-range' ? 'No active IP range policy bans' : 'No active bans'}
                    </td>
                  </tr>
                {:else}
                  {#each filteredBanRows as row}
                    {@const ban = row.ban}
                    {@const meta = row.meta}
                    {@const rowKey = row.key}
                    {@const detailVisible = isExpanded(rowKey)}
                    {@const detailsId = `ban-detail-${row.originalIndex}`}
                    {@const signals = Array.isArray(ban?.fingerprint?.signals) ? ban.fingerprint.signals : []}
                    {@const expiresTs = Number(ban?.expires || 0)}
                    {@const isExpired = Number.isFinite(expiresTs) && expiresTs > 0
                      ? expiresTs < Math.floor(Date.now() / 1000)
                      : false}
                    <tr class="ban-summary-row">
                      <td><code>{ban?.ip || '-'}</code></td>
                      <td>
                        <code>{ban?.reason || '-'}</code>
                        {#if meta.isIpRange}
                          <div class="ban-detail-content">
                            <span class="ban-signal-badge">IP range</span>
                            <span class="text-muted">{meta.reasonLabel}</span>
                            {#if meta.sourceId}
                              <span><code>{meta.sourceId}</code></span>
                            {/if}
                          </div>
                        {/if}
                      </td>
                      <td>{formatTimestamp(ban?.banned_at)}</td>
                      <td class={isExpired ? 'expired' : ''}>
                        {isExpired ? 'Expired' : formatTimestamp(expiresTs)}
                      </td>
                      <td>
                        {#if signals.length === 0}
                          <span class="text-muted">none</span>
                        {:else}
                          {#each signals as signal}
                            <span class="ban-signal-badge">{signal}</span>
                          {/each}
                        {/if}
                      </td>
                      <td class="ban-action-cell">
                        <button
                          class="ban-details-toggle"
                          data-target={detailsId}
                          type="button"
                          on:click={() => toggleDetails(rowKey)}
                        >{detailVisible ? 'Hide' : 'Details'}</button>
                      </td>
                    </tr>
                    <tr id={detailsId} class={`ban-detail-row${detailVisible ? '' : ' hidden'}`}>
                      <td colspan="6">
                        <div class="ban-detail-content">
                          <div><strong>Score:</strong> {Number.isFinite(Number(ban?.fingerprint?.score)) ? Number(ban.fingerprint.score) : 'n/a'}</div>
                          <div><strong>Summary:</strong> {ban?.fingerprint?.summary || 'No additional fingerprint details.'}</div>
                          {#if meta.isIpRange}
                            <div><strong>IP Range Source:</strong> {meta.sourceLabel}</div>
                            <div><strong>Source ID:</strong> {meta.sourceId ? meta.sourceId : '-'}</div>
                            <div><strong>Policy Action:</strong> {meta.action ? meta.action : '-'}</div>
                            <div><strong>Matched CIDR:</strong> {meta.matchedCidr ? meta.matchedCidr : '-'}</div>
                            <div><strong>Detection:</strong> {meta.detection ? meta.detection : '-'}</div>
                            {#if meta.fallback}
                              <div><strong>Fallback:</strong> {meta.fallback}</div>
                            {/if}
                          {/if}
                        </div>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </TableWrapper>
          <div class="controls-grid controls-grid--manual">
            <div class="control-group panel-soft pad-md">
              <h3>Ban IP</h3>
              <input id="ban-ip" class="input-field" type="text" placeholder="IP address" aria-label="IP address to ban" maxlength="45" spellcheck="false" autocomplete="off" bind:value={banIp} />
              <input id="ban-reason" class="input-field" type="text" value="manual_ban" aria-label="Ban reason (fixed)" readonly disabled />
              <label class="control-label" for="ban-duration-days">Duration</label>
              <div class="duration-inputs">
                <label class="duration-input" for="ban-duration-days">
                  <input id="ban-duration-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" aria-label="Manual ban duration days" bind:value={banDurationDays} />
                  <span class="input-unit">days</span>
                </label>
                <label class="duration-input" for="ban-duration-hours">
                  <input id="ban-duration-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" aria-label="Manual ban duration hours" bind:value={banDurationHours} />
                  <span class="input-unit">hrs</span>
                </label>
                <label class="duration-input" for="ban-duration-minutes">
                  <input id="ban-duration-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" aria-label="Manual ban duration minutes" bind:value={banDurationMinutes} />
                  <span class="input-unit">mins</span>
                </label>
              </div>
              <button id="ban-btn" class="btn btn-submit" disabled={!canBan} on:click={submitBan}>Ban</button>
            </div>
            <div class="control-group panel-soft pad-md">
              <h3>Unban IP</h3>
              <input id="unban-ip" class="input-field" type="text" placeholder="IP address" aria-label="IP address to unban" maxlength="45" spellcheck="false" autocomplete="off" bind:value={unbanIp} />
              <button id="unban-btn" class="btn btn-submit" disabled={!canUnban} on:click={submitUnban}>Unban</button>
            </div>
          </div>
</section>
