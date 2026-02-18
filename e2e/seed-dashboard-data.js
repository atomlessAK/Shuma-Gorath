const DEFAULT_BASE_URL = "http://127.0.0.1:3000";
const API_KEY = (process.env.SHUMA_API_KEY || "").trim();

function authHeaders() {
  return {
    Authorization: `Bearer ${API_KEY}`
  };
}

function adminHeaders() {
  return {
    ...authHeaders(),
    ...forwardedHeaders("127.0.0.1")
  };
}

function forwardedHeaders(ip) {
  const headers = {
    "X-Forwarded-For": ip
  };
  const secret = (process.env.SHUMA_FORWARDED_IP_SECRET || "").trim();
  if (secret) {
    headers["X-Shuma-Forwarded-Secret"] = secret;
  }
  return headers;
}

function ensureRequiredEnv() {
  if (!API_KEY) {
    throw new Error("Missing SHUMA_API_KEY for dashboard seed.");
  }
  if (/^changeme/i.test(API_KEY)) {
    throw new Error("SHUMA_API_KEY is a placeholder value; run make setup or make api-key-generate.");
  }
}

async function request(baseURL, path, options = {}) {
  const response = await fetch(`${baseURL}${path}`, options);
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`Seed request failed: ${options.method || "GET"} ${path} -> ${response.status} ${text}`);
  }
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

async function safeUnban(baseURL, ip) {
  try {
    await request(baseURL, `/admin/unban?ip=${encodeURIComponent(ip)}`, {
      method: "POST",
      headers: adminHeaders()
    });
  } catch {
    // Best-effort cleanup for synthetic seed IPs.
  }
}

async function seedDashboardData() {
  ensureRequiredEnv();
  const baseURL = process.env.SHUMA_BASE_URL || DEFAULT_BASE_URL;
  const now = Date.now();
  const banIp = `203.0.113.${(now % 200) + 20}`;
  const cdpIp = `198.51.100.${(Math.floor(now / 7) % 200) + 20}`;
  let originalTestMode = false;
  let restoreTestMode = false;

  const config = await request(baseURL, "/admin/config", {
    headers: adminHeaders()
  });
  if (config && typeof config.test_mode === "boolean") {
    originalTestMode = config.test_mode;
    restoreTestMode = true;
  }

  try {
    await request(baseURL, "/admin/config", {
      method: "POST",
      headers: {
        ...adminHeaders(),
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ test_mode: false })
    });

    await request(baseURL, "/admin/ban", {
      method: "POST",
      headers: {
        ...adminHeaders(),
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        ip: banIp,
        duration: 3600
      })
    });

    await request(baseURL, "/cdp-report", {
      method: "POST",
      headers: {
        ...forwardedHeaders(cdpIp),
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        cdp_detected: true,
        score: 0.92,
        checks: ["webdriver", "runtime_enable"]
      })
    });

    await safeUnban(baseURL, banIp);
    await safeUnban(baseURL, cdpIp);

    await request(baseURL, "/admin/analytics", {
      headers: adminHeaders()
    });
    await request(baseURL, "/admin/events?hours=24", {
      headers: adminHeaders()
    });
    const events = await request(baseURL, "/admin/events?hours=24", {
      headers: adminHeaders()
    });

    if (!events || !Array.isArray(events.recent_events) || events.recent_events.length === 0) {
      throw new Error("Seed verification failed: no recent events available");
    }

    return {
      banIp,
      cdpIp,
      eventCount: events.recent_events.length
    };
  } finally {
    if (restoreTestMode) {
      await request(baseURL, "/admin/config", {
        method: "POST",
        headers: {
          ...adminHeaders(),
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ test_mode: originalTestMode })
      }).catch(() => {});
    }
    await safeUnban(baseURL, banIp);
    await safeUnban(baseURL, cdpIp);
  }
}

module.exports = { seedDashboardData };

if (require.main === module) {
  seedDashboardData()
    .then((result) => {
      process.stdout.write(
        `Dashboard seed complete: banIp=${result.banIp}, cdpIp=${result.cdpIp}, events=${result.eventCount}\n`
      );
    })
    .catch((err) => {
      process.stderr.write(`Dashboard seed failed: ${err.message}\n`);
      process.exit(1);
    });
}
