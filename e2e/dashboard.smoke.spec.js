const { test, expect } = require("@playwright/test");
const { seedDashboardData } = require("./seed-dashboard-data");

const BASE_URL = process.env.SHUMA_BASE_URL || "http://127.0.0.1:3000";
const API_KEY = process.env.SHUMA_API_KEY || "changeme-dev-only-api-key";

async function openDashboard(page) {
  await page.goto(`${BASE_URL}/dashboard/index.html`);
  await page.waitForTimeout(250);
  if (page.url().includes("/dashboard/login.html")) {
    await page.fill("#login-apikey", API_KEY);
    await page.click("#login-submit");
    await expect(page).toHaveURL(/\/dashboard\/index\.html/);
  }
  await page.waitForSelector("#logout-btn", { timeout: 15000 });
  await expect(page.locator("#logout-btn")).toBeEnabled();
  await page.waitForFunction(() => {
    const total = document.getElementById("total-events")?.textContent?.trim();
    return Boolean(total && total !== "-" && total !== "...");
  }, { timeout: 15000 });
}

test.beforeAll(async () => {
  await seedDashboardData();
});

test("dashboard loads and shows seeded operational data", async ({ page }) => {
  await openDashboard(page);

  await expect(page.locator("h1")).toHaveText("Shuma-Gorath");
  await expect(page.locator("h3", { hasText: "API Access" })).toHaveCount(0);

  await expect(page.locator("#last-updated")).toContainText("updated:");
  await expect(page.locator("#config-mode-subtitle")).toContainText("Admin page configuration enabled.");

  await expect(page.locator("#total-events")).not.toHaveText("-");
  await expect(page.locator("#events tbody tr").first()).toBeVisible();
  await expect(page.locator("#events tbody")).toContainText(/manual_ban|cdp_detected|events_view|analytics_view/);

  await expect(page.locator("#cdp-events tbody tr").first()).toBeVisible();
  await expect(page.locator("#cdp-total-detections")).not.toHaveText("-");
});

test("ban form enforces IP validity and submit state", async ({ page }) => {
  await openDashboard(page);

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

  const mazeSave = page.locator("#save-maze-config");
  const durationsSave = page.locator("#save-durations-btn");

  await expect(mazeSave).toBeDisabled();
  await expect(durationsSave).toBeDisabled();

  const mazeThreshold = page.locator("#maze-threshold");
  const initialMazeThreshold = await mazeThreshold.inputValue();
  const nextMazeThreshold = String(Math.min(500, Number(initialMazeThreshold || "50") + 1));
  await mazeThreshold.fill(nextMazeThreshold);
  await mazeThreshold.dispatchEvent("input");
  await expect(mazeSave).toBeEnabled();
  await mazeThreshold.fill(initialMazeThreshold);
  await mazeThreshold.dispatchEvent("input");
  await expect(mazeSave).toBeDisabled();

  const durationField = page.locator("#dur-admin-minutes");
  const initialDuration = await durationField.inputValue();
  const nextDuration = String((Number(initialDuration || "0") + 1) % 60);
  await durationField.fill(nextDuration);
  await durationField.dispatchEvent("input");
  await expect(durationsSave).toBeEnabled();
  await durationField.fill(initialDuration);
  await durationField.dispatchEvent("input");
  await expect(durationsSave).toBeDisabled();
});

test("dashboard tables keep sticky headers", async ({ page }) => {
  await openDashboard(page);

  const eventsHeaderPosition = await page
    .locator("#events thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);
  const cdpHeaderPosition = await page
    .locator("#cdp-events thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);
  const bansHeaderPosition = await page
    .locator("#bans-table thead th")
    .first()
    .evaluate((el) => getComputedStyle(el).position);

  expect(eventsHeaderPosition).toBe("sticky");
  expect(cdpHeaderPosition).toBe("sticky");
  expect(bansHeaderPosition).toBe("sticky");
});
