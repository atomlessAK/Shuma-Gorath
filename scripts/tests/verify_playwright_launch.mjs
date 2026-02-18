#!/usr/bin/env node
import { chromium } from "@playwright/test";

const SANDBOX_PATTERNS = [
  /settings\.dat: Operation not permitted/i,
  /crashpad\.child_port_handshake.*Permission denied \(1100\)/i,
  /mach_port_rendezvous.*Permission denied/i,
  /signal=SIGABRT/i
];

function isLikelySandboxLaunchFailure(message) {
  return SANDBOX_PATTERNS.some((pattern) => pattern.test(message));
}

function formatErrorExcerpt(message) {
  return message
    .split("\n")
    .slice(0, 18)
    .join("\n")
    .trim();
}

async function main() {
  let browser;
  try {
    browser = await chromium.launch({
      channel: "chromium",
      headless: true,
      args: ["--disable-crashpad", "--disable-crash-reporter"]
    });
    const page = await browser.newPage();
    await page.goto("about:blank");
    await browser.close();
  } catch (error) {
    const message = String(error?.message || error || "unknown Playwright launch failure");
    const sandboxFailure = isLikelySandboxLaunchFailure(message);

    const lines = [
      "Playwright Chromium preflight failed before dashboard e2e execution.",
      "",
      `HOME=${process.env.HOME || "<unset>"}`,
      `PLAYWRIGHT_BROWSERS_PATH=${process.env.PLAYWRIGHT_BROWSERS_PATH || "<unset>"}`,
      "",
      "Error excerpt:",
      formatErrorExcerpt(message)
    ];

    if (sandboxFailure) {
      lines.push(
        "",
        "Detected sandbox-level browser launch restrictions.",
        "Run dashboard e2e from a terminal/session with browser launch permissions.",
        "Optional local bypass for restricted sandboxes only: PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1"
      );
      console.error(lines.join("\n"));
      process.exit(42);
    }

    lines.push("", "Launch failure did not match known sandbox signatures.");
    lines.push("Re-run with DEBUG=pw:browser for detailed Playwright browser logs.");
    console.error(lines.join("\n"));
    process.exit(1);
  } finally {
    if (browser) {
      await browser.close().catch(() => {});
    }
  }
}

await main();
