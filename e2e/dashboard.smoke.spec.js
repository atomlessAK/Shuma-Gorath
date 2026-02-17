const { test, expect } = require("@playwright/test");
const { seedDashboardData } = require("./seed-dashboard-data");

const BASE_URL = process.env.SHUMA_BASE_URL || "http://127.0.0.1:3000";
const API_KEY = (process.env.SHUMA_API_KEY || "").trim();
const DASHBOARD_TABS = Object.freeze(["monitoring", "ip-bans", "status", "config", "tuning"]);
const ADMIN_TABS = Object.freeze(["ip-bans", "status", "config", "tuning"]);
const runtimeGuards = new WeakMap();

function ensureRequiredEnv() {
  if (!API_KEY) {
    throw new Error("Missing SHUMA_API_KEY for dashboard smoke tests.");
  }
  if (/^changeme/i.test(API_KEY)) {
    throw new Error("SHUMA_API_KEY is a placeholder value; run make setup or make api-key-generate.");
  }
}

function isStaticRuntimeRequest(request) {
  const resourceType = request.resourceType();
  return resourceType === "script" || resourceType === "stylesheet";
}

function ensureRuntimeGuard(page) {
  if (runtimeGuards.has(page)) {
    return runtimeGuards.get(page);
  }

  const guard = {
    failures: []
  };

  page.on("pageerror", (error) => {
    guard.failures.push(`pageerror: ${error.message}`);
  });

  page.on("requestfailed", (request) => {
    if (!isStaticRuntimeRequest(request)) {
      return;
    }
    const failure = request.failure();
    guard.failures.push(
      `requestfailed: ${request.method()} ${request.url()} (${failure ? failure.errorText : "unknown"})`
    );
  });

  page.on("response", (response) => {
    const request = response.request();
    if (!isStaticRuntimeRequest(request)) {
      return;
    }
    if (response.status() >= 400) {
      guard.failures.push(
        `asset-response: ${request.method()} ${response.url()} -> ${response.status()}`
      );
    }
  });

  runtimeGuards.set(page, guard);
  return guard;
}

function assertNoRuntimeFailures(page) {
  const guard = runtimeGuards.get(page);
  if (!guard || guard.failures.length === 0) {
    return;
  }
  throw new Error(`Unexpected dashboard runtime failures:\n${guard.failures.join("\n")}`);
}

async function assertActiveTabPanelVisibility(page, activeTab) {
  for (const tab of DASHBOARD_TABS) {
    await expect(page.locator(`#dashboard-tab-${tab}`)).toHaveAttribute(
      "aria-selected",
      tab === activeTab ? "true" : "false"
    );
  }

  if (activeTab === "monitoring") {
    await expect(page.locator("#dashboard-panel-monitoring")).toBeVisible();
    await expect(page.locator("#dashboard-admin-section")).toBeHidden();
    for (const tab of ADMIN_TABS) {
      await expect(page.locator(`#dashboard-panel-${tab}`)).toBeHidden();
    }
    return;
  }

  await expect(page.locator("#dashboard-panel-monitoring")).toBeHidden();
  await expect(page.locator("#dashboard-admin-section")).toBeVisible();
  for (const tab of ADMIN_TABS) {
    const panel = page.locator(`#dashboard-panel-${tab}`);
    if (tab === activeTab) {
      const forcedHidden = await panel.evaluate((element) => element.classList.contains("hidden"));
      if (forcedHidden) {
        await expect(panel).toBeHidden();
      } else {
        await expect(panel).toBeVisible();
      }
    } else {
      await expect(panel).toBeHidden();
    }
  }
}

async function openDashboard(page, options = {}) {
  const initialTab = typeof options.initialTab === "string" ? options.initialTab : "monitoring";
  const targetUrl = `${BASE_URL}/dashboard/index.html#${initialTab}`;
  ensureRuntimeGuard(page);
  await page.goto(targetUrl);
  await page.waitForTimeout(250);
  if (page.url().includes("/dashboard/login.html")) {
    await page.fill("#login-apikey", API_KEY);
    await page.click("#login-submit");
    await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  }
  if (!page.url().endsWith(`#${initialTab}`)) {
    await page.goto(targetUrl);
  }
  await page.waitForSelector("#logout-btn", { timeout: 15000 });
  await expect(page.locator("#logout-btn")).toBeEnabled();
  if (initialTab === "monitoring") {
    await page.waitForFunction(() => {
      const total = document.getElementById("total-events")?.textContent?.trim();
      return Boolean(total && total !== "-" && total !== "...");
    }, { timeout: 15000 });
  }
  await page.waitForFunction(() => {
    const subtitle = document.getElementById("config-mode-subtitle")?.textContent || "";
    return !subtitle.includes("LOADING");
  }, { timeout: 15000 });
  await assertActiveTabPanelVisibility(page, initialTab);
  assertNoRuntimeFailures(page);
}

async function openTab(page, tab) {
  await page.click(`#dashboard-tab-${tab}`);
  await expect(page).toHaveURL(new RegExp(`#${tab}$`));
  await assertActiveTabPanelVisibility(page, tab);
  assertNoRuntimeFailures(page);
}

async function assertChartsFillPanels(page) {
  const metrics = await page.evaluate(() => {
    const ids = ["eventTypesChart", "topIpsChart", "timeSeriesChart"];
    return ids.map((id) => {
      const canvas = document.getElementById(id);
      const panel = canvas ? canvas.closest(".chart-container") : null;
      if (!canvas || !panel) {
        return { id, missing: true };
      }
      const canvasRect = canvas.getBoundingClientRect();
      const panelRect = panel.getBoundingClientRect();
      return {
        id,
        missing: false,
        canvasWidth: canvasRect.width,
        canvasHeight: canvasRect.height,
        panelWidth: panelRect.width
      };
    });
  });

  for (const metric of metrics) {
    expect(metric.missing, `${metric.id} should exist in a chart panel`).toBe(false);
    expect(metric.canvasWidth, `${metric.id} should fill most of panel width`).toBeGreaterThan(
      metric.panelWidth * 0.8
    );
    expect(metric.canvasHeight, `${metric.id} should have non-squashed height`).toBeGreaterThan(170);
  }
}

test.beforeAll(async () => {
  ensureRequiredEnv();
  await seedDashboardData();
});

test.afterEach(async ({ page }) => {
  assertNoRuntimeFailures(page);
});

test("dashboard bare path redirects to canonical index route", async ({ request }) => {
  const response = await request.get(`${BASE_URL}/dashboard`, { maxRedirects: 0 });
  expect(response.status()).toBe(308);
  expect(response.headers().location).toBe("/dashboard/index.html");
});

test("dashboard clean-state renders explicit empty placeholders", async ({ page }) => {
  const emptyConfig = {
    admin_config_write_enabled: true,
    pow_enabled: true,
    challenge_puzzle_enabled: true,
    challenge_puzzle_transform_count: 6,
    challenge_puzzle_risk_threshold: 3,
    challenge_puzzle_risk_threshold_default: 3,
    botness_maze_threshold: 6,
    botness_maze_threshold_default: 6,
    botness_weights: {
      js_required: 1,
      geo_risk: 2,
      rate_medium: 1,
      rate_high: 2
    },
    ban_durations: {
      honeypot: 86400,
      rate_limit: 3600,
      browser: 21600,
      cdp: 43200,
      admin: 21600
    },
    honeypot_enabled: true,
    honeypots: ["/instaban"],
    browser_block: [["Chrome", 120], ["Firefox", 115], ["Safari", 15]],
    browser_whitelist: [],
    whitelist: [],
    path_whitelist: [],
    maze_enabled: true,
    maze_threshold: 50,
    maze_auto_ban: false,
    robots_enabled: true,
    ai_robots_block: true,
    ai_robots_aggressive: false,
    ai_robots_content_signal: true,
    robots_crawl_delay: 2,
    cdp_enabled: true,
    cdp_mode: "report-only",
    cdp_score_threshold: 0.8,
    cdp_auto_ban: false,
    cdp_auto_ban_threshold: 0.9,
    rate_limit: 80,
    js_required_enforced: true,
    test_mode: false,
    kv_store_fail_open: true,
    edge_integration_mode: "off",
    geo_risk: [],
    geo_allow: [],
    geo_challenge: [],
    geo_maze: [],
    geo_block: []
  };

  await page.route("**/admin/analytics", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ ban_count: 0, test_mode: false, fail_mode: "open" })
    });
  });
  await page.route("**/admin/events?hours=*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ recent_events: [], event_counts: {}, top_ips: [], unique_ips: 0 })
    });
  });
  await page.route("**/admin/ban", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ bans: [] })
    });
  });
  await page.route("**/admin/maze", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ total_hits: 0, unique_crawlers: 0, maze_auto_bans: 0, top_crawlers: [] })
    });
  });
  await page.route("**/admin/cdp", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ stats: { total_detections: 0, auto_bans: 0 }, config: {} })
    });
  });
  await page.route("**/admin/cdp/events?*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ events: [] })
    });
  });
  await page.route("**/admin/config", async (route) => {
    if (route.request().method() !== "GET") {
      await route.continue();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(emptyConfig)
    });
  });

  await openDashboard(page);
  await expect(page.locator("#total-events")).toHaveText("0");
  await expect(page.locator("#events tbody")).toContainText("No recent events");
  await expect(page.locator("#cdp-events tbody")).toContainText(
    "No CDP detections or auto-bans in the selected window"
  );
  await expect(page.locator("#maze-top-offender")).toHaveText("None");

  await openTab(page, "ip-bans");
  await expect(page.locator("#bans-table tbody")).toContainText("No active bans");
  await expect(page.locator('[data-tab-state="ip-bans"]')).toContainText("No active bans.");
});

test("dashboard loads and shows seeded operational data", async ({ page }) => {
  await openDashboard(page);
  await assertChartsFillPanels(page);

  await expect(page.locator("h1")).toHaveText("Shuma-Gorath");
  await expect(page.locator("h3", { hasText: "API Access" })).toHaveCount(0);

  await expect(page.locator("#last-updated")).toContainText("updated:");
  await expect(page.locator("#config-mode-subtitle")).toContainText("Admin page configuration");

  await expect(page.locator("#total-events")).not.toHaveText("-");
  await expect(page.locator("#events tbody tr").first()).toBeVisible();
  await expect(page.locator("#events tbody")).not.toContainText("undefined");

  await expect(page.locator("#cdp-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#cdp-total-detections")).not.toHaveText("-");
});

test("status tab resolves fail mode without requiring monitoring bootstrap", async ({ page }) => {
  await openDashboard(page, { initialTab: "status" });
  const failModeCard = page
    .locator("#status-items .status-item")
    .filter({ has: page.locator("h3", { hasText: "Fail Mode Policy" }) });

  await expect(failModeCard).toHaveCount(1);
  await expect(failModeCard.locator(".status-value")).toHaveText(/OPEN|CLOSED/);
  await expect(failModeCard.locator(".status-value")).not.toHaveText("UNKNOWN");

  const statusVarTables = page.locator("#status-vars-groups .status-vars-table");
  expect(await statusVarTables.count()).toBeGreaterThan(1);

  const statusVarRows = page.locator("#status-vars-groups .status-vars-table tbody tr");
  expect(await statusVarRows.count()).toBeGreaterThan(20);
  const testModeRow = page
    .locator("#status-vars-groups .status-vars-table tbody tr")
    .filter({ has: page.locator("code", { hasText: "test_mode" }) });
  await expect(testModeRow).toHaveCount(1);
  await expect(testModeRow).toHaveClass(/status-var-row--admin-write/);
  await expect(testModeRow.locator("td").nth(2)).not.toHaveText("");
});

test("ban form enforces IP validity and submit state", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "ip-bans");

  const banButton = page.locator("#ban-btn");
  await expect(banButton).toBeDisabled();

  await page.fill("#ban-ip", "not-an-ip");
  await page.dispatchEvent("#ban-ip", "input");
  await expect(banButton).toBeDisabled();

  await page.fill("#ban-ip", "198.51.100.42");
  await page.dispatchEvent("#ban-ip", "input");
  await expect(banButton).toBeEnabled();
});

test("maze and duration save buttons use shared dirty-state behavior", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "config");

  const mazeSave = page.locator("#save-maze-config");
  const durationsSave = page.locator("#save-durations-btn");
  const powSave = page.locator("#save-pow-config");
  const rateLimitSave = page.locator("#save-rate-limit-config");
  const jsRequiredSave = page.locator("#save-js-required-config");
  const honeypotSave = page.locator("#save-honeypot-config");
  const honeypotEnabledToggle = page.locator("#honeypot-enabled-toggle");
  const honeypotEnabledSwitch = page.locator("label.toggle-switch[for='honeypot-enabled-toggle']");
  const browserPolicySave = page.locator("#save-browser-policy-config");
  const whitelistSave = page.locator("#save-whitelist-config");
  const edgeModeSave = page.locator("#save-edge-integration-mode-config");
  const edgeModeSelect = page.locator("#edge-integration-mode-select");
  const advancedSave = page.locator("#save-advanced-config");
  const advancedField = page.locator("#advanced-config-json");

  await expect(mazeSave).toBeDisabled();
  await expect(durationsSave).toBeDisabled();
  await expect(powSave).toBeDisabled();
  await expect(rateLimitSave).toBeDisabled();
  await expect(jsRequiredSave).toBeDisabled();
  await expect(honeypotSave).toBeDisabled();
  await expect(browserPolicySave).toBeDisabled();
  await expect(whitelistSave).toBeDisabled();
  await expect(edgeModeSave).toBeDisabled();
  await expect(advancedSave).toBeDisabled();

  const mazeThreshold = page.locator("#maze-threshold");
  if (!(await mazeThreshold.isVisible())) {
    await expect(page.locator("#config-mode-subtitle")).toContainText(/disabled|read-only|Admin page configuration/i);
    return;
  }
  const initialMazeThreshold = await mazeThreshold.inputValue();
  const nextMazeThreshold = String(Math.min(500, Number(initialMazeThreshold || "50") + 1));
  await mazeThreshold.fill(nextMazeThreshold);
  await mazeThreshold.dispatchEvent("input");
  await expect(mazeSave).toBeEnabled();
  await mazeThreshold.fill(initialMazeThreshold);
  await mazeThreshold.dispatchEvent("input");
  await expect(mazeSave).toBeDisabled();

  const durationField = page.locator("#dur-cdp-minutes");
  const initialDuration = await durationField.inputValue();
  const nextDuration = String((Number(initialDuration || "0") + 1) % 60);
  await durationField.fill(nextDuration);
  await durationField.dispatchEvent("input");
  await expect(durationsSave).toBeEnabled();
  await durationField.fill(initialDuration);
  await durationField.dispatchEvent("input");
  await expect(durationsSave).toBeDisabled();

  const rateLimitField = page.locator("#rate-limit-threshold");
  const initialRateLimit = await rateLimitField.inputValue();
  const nextRateLimit = String(Math.max(1, Number(initialRateLimit || "80") + 1));
  await rateLimitField.fill(nextRateLimit);
  await rateLimitField.dispatchEvent("input");
  await expect(rateLimitSave).toBeEnabled();
  await rateLimitField.fill(initialRateLimit);
  await rateLimitField.dispatchEvent("input");
  await expect(rateLimitSave).toBeDisabled();

  const jsRequiredToggle = page.locator("#js-required-enforced-toggle");
  if (await jsRequiredToggle.isVisible()) {
    const jsRequiredInitial = await jsRequiredToggle.isChecked();
    await jsRequiredToggle.click();
    await expect(jsRequiredSave).toBeEnabled();
    if (jsRequiredInitial !== await jsRequiredToggle.isChecked()) {
      await jsRequiredToggle.click();
    }
    await expect(jsRequiredSave).toBeDisabled();
  }

  const powToggle = page.locator("#pow-enabled-toggle");
  const powToggleSwitch = page.locator("label.toggle-switch[for='pow-enabled-toggle']");
  if (await powToggleSwitch.isVisible() && await powToggle.isEnabled()) {
    const powInitial = await powToggle.isChecked();
    await powToggleSwitch.click();
    await expect(powSave).toBeEnabled();
    if (powInitial !== await powToggle.isChecked()) {
      await powToggleSwitch.click();
    }
    await expect(powSave).toBeDisabled();
  }

  const honeypotField = page.locator("#honeypot-paths");
  const initialHoneypots = await honeypotField.inputValue();
  await honeypotField.fill(`${initialHoneypots}\n/trap-e2e`);
  await honeypotField.dispatchEvent("input");
  await expect(honeypotSave).toBeEnabled();
  await honeypotField.fill(initialHoneypots);
  await honeypotField.dispatchEvent("input");
  await expect(honeypotSave).toBeDisabled();
  if (await honeypotEnabledSwitch.isVisible() && await honeypotEnabledToggle.isEnabled()) {
    const initialHoneypotEnabled = await honeypotEnabledToggle.isChecked();
    await honeypotEnabledSwitch.click();
    await expect(honeypotSave).toBeEnabled();
    if (initialHoneypotEnabled !== await honeypotEnabledToggle.isChecked()) {
      await honeypotEnabledSwitch.click();
    }
    await expect(honeypotSave).toBeDisabled();
  }

  const browserBlockField = page.locator("#browser-block-rules");
  const initialBrowserBlock = await browserBlockField.inputValue();
  await browserBlockField.fill(`${initialBrowserBlock}\nEdge,120`);
  await browserBlockField.dispatchEvent("input");
  await expect(browserPolicySave).toBeEnabled();
  await browserBlockField.fill(initialBrowserBlock);
  await browserBlockField.dispatchEvent("input");
  await expect(browserPolicySave).toBeDisabled();

  const networkWhitelistField = page.locator("#network-whitelist");
  const initialNetworkWhitelist = await networkWhitelistField.inputValue();
  await networkWhitelistField.fill(`${initialNetworkWhitelist}\n198.51.100.0/24`);
  await networkWhitelistField.dispatchEvent("input");
  await expect(whitelistSave).toBeEnabled();
  await networkWhitelistField.fill(initialNetworkWhitelist);
  await networkWhitelistField.dispatchEvent("input");
  await expect(whitelistSave).toBeDisabled();

  const initialEdgeMode = await edgeModeSelect.inputValue();
  const nextEdgeMode = initialEdgeMode === "off" ? "advisory" : "off";
  await edgeModeSelect.selectOption(nextEdgeMode);
  await expect(edgeModeSave).toBeEnabled();
  await edgeModeSelect.selectOption(initialEdgeMode);
  await expect(edgeModeSave).toBeDisabled();

  const initialAdvanced = await advancedField.inputValue();
  let parsedAdvanced;
  try {
    parsedAdvanced = JSON.parse(initialAdvanced);
  } catch (_e) {
    parsedAdvanced = {};
  }
  const nextAdvanced = {
    ...parsedAdvanced,
    rate_limit: Number(parsedAdvanced.rate_limit || 80) + 1
  };
  await advancedField.fill(JSON.stringify(nextAdvanced, null, 2));
  await advancedField.dispatchEvent("input");
  await expect(advancedSave).toBeEnabled();
  await advancedField.fill("{invalid");
  await advancedField.dispatchEvent("input");
  await expect(advancedSave).toBeDisabled();
  await advancedField.fill(initialAdvanced);
  await advancedField.dispatchEvent("input");
  await expect(advancedSave).toBeDisabled();

  const challengeTransformField = page.locator("#challenge-puzzle-transform-count");
  const challengeEnabledToggle = page.locator("#challenge-puzzle-enabled-toggle");
  const challengeEnabledSwitch = page.locator("label.toggle-switch[for='challenge-puzzle-enabled-toggle']");
  const challengeTransformSave = page.locator("#save-challenge-puzzle-config");
  if (await challengeTransformField.isEnabled()) {
    const initialTransformCount = await challengeTransformField.inputValue();
    const nextTransformCount = String(Math.min(8, Number(initialTransformCount || "6") + 1));
    await challengeTransformField.fill(nextTransformCount);
    await challengeTransformField.dispatchEvent("input");
    await expect(challengeTransformSave).toBeEnabled();
    await challengeTransformField.fill(initialTransformCount);
    await challengeTransformField.dispatchEvent("input");
    await expect(challengeTransformSave).toBeDisabled();

    if (await challengeEnabledSwitch.isVisible() && await challengeEnabledToggle.isEnabled()) {
      const initialEnabled = await challengeEnabledToggle.isChecked();
      await challengeEnabledSwitch.click();
      await expect(challengeTransformSave).toBeEnabled();
      if (initialEnabled !== await challengeEnabledToggle.isChecked()) {
        await challengeEnabledSwitch.click();
      }
      await expect(challengeTransformSave).toBeDisabled();
    }
  } else {
    await expect(challengeTransformSave).toBeDisabled();
  }
});

test("session survives reload and time-range controls refresh chart data", async ({ page }) => {
  await openDashboard(page);

  await openTab(page, "monitoring");
  await page.reload();
  await expect(page).toHaveURL(/\/dashboard\/index\.html#monitoring/);
  await expect(page.locator("#logout-btn")).toBeEnabled();

  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/admin/events?hours=168") && resp.ok()),
    page.click('.time-btn[data-range="week"]')
  ]);
  await expect(page.locator('.time-btn[data-range="week"]')).toHaveClass(/active/);

  await Promise.all([
    page.waitForResponse((resp) => resp.url().includes("/admin/events?hours=720") && resp.ok()),
    page.click('.time-btn[data-range="month"]')
  ]);
  await expect(page.locator('.time-btn[data-range="month"]')).toHaveClass(/active/);
});

test("dashboard tables keep sticky headers", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "monitoring");

  const eventsHeaderPosition = await page
    .locator("#events thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);
  const cdpHeaderPosition = await page
    .locator("#cdp-events thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);

  await openTab(page, "ip-bans");
  const bansHeaderPosition = await page
    .locator("#bans-table thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);

  expect(eventsHeaderPosition).toBe("sticky");
  expect(cdpHeaderPosition).toBe("sticky");
  expect(bansHeaderPosition).toBe("sticky");
});

test("tab hash route persists selected panel across reload", async ({ page }) => {
  await openDashboard(page);
  await openTab(page, "config");
  await expect(page.locator("#dashboard-panel-config")).toBeVisible();
  await expect(page.locator("#dashboard-panel-monitoring")).toBeHidden();

  await page.reload();
  await expect(page).toHaveURL(/\/dashboard\/index\.html#config/);
  await expect(page.locator("#dashboard-panel-config")).toBeVisible();
  await assertActiveTabPanelVisibility(page, "config");
});

test("tab keyboard navigation updates hash and selected state", async ({ page }) => {
  await openDashboard(page);
  const monitoringTab = page.locator("#dashboard-tab-monitoring");
  await monitoringTab.focus();
  await expect(monitoringTab).toHaveAttribute("aria-selected", "true");

  await page.keyboard.press("ArrowRight");
  await expect(page).toHaveURL(/#ip-bans$/);
  await expect(page.locator("#dashboard-tab-ip-bans")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("#dashboard-panel-ip-bans")).toBeVisible();
  await assertActiveTabPanelVisibility(page, "ip-bans");

  await page.locator("#dashboard-tab-ip-bans").focus();
  await page.keyboard.press("End");
  await expect(page).toHaveURL(/#tuning$/);
  await expect(page.locator("#dashboard-tab-tuning")).toHaveAttribute("aria-selected", "true");
  await assertActiveTabPanelVisibility(page, "tuning");

  await page.locator("#dashboard-tab-tuning").focus();
  await page.keyboard.press("Home");
  await expect(page).toHaveURL(/#monitoring$/);
  await expect(page.locator("#dashboard-tab-monitoring")).toHaveAttribute("aria-selected", "true");
  await assertActiveTabPanelVisibility(page, "monitoring");
});

test("tab error state is surfaced when tab-scoped fetch fails", async ({ page }) => {
  await openDashboard(page);

  await page.route("**/admin/ban", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "temporary ban endpoint outage" })
    });
  });

  await openTab(page, "ip-bans");
  await expect(page.locator('[data-tab-state="ip-bans"]')).toContainText("temporary ban endpoint outage");
  await page.unroute("**/admin/ban");
});

test("logout redirects back to login page", async ({ page }) => {
  await openDashboard(page);
  await page.click("#logout-btn");
  await expect(page).toHaveURL(/\/dashboard\/login\.html\?next=/);
});
