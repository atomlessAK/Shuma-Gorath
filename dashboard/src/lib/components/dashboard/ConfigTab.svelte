<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let analyticsSnapshot = null;

  $: hasAnalyticsSnapshot = analyticsSnapshot && typeof analyticsSnapshot === 'object';
  $: testModeEnabled = hasAnalyticsSnapshot ? analyticsSnapshot.test_mode === true : null;
  $: testModeStatusText = testModeEnabled === null
    ? 'LOADING...'
    : (testModeEnabled ? 'ENABLED (LOGGING ONLY)' : 'DISABLED (BLOCKING ACTIVE)');
  $: testModeStatusClass = `text-muted status-value ${
    testModeEnabled === null
      ? ''
      : (testModeEnabled ? 'test-mode-status--enabled' : 'test-mode-status--disabled')
  }`.trim();
</script>

<section
  id="dashboard-panel-config"
  class="admin-group"
  data-dashboard-tab-panel="config"
  aria-labelledby="dashboard-tab-config"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
  tabindex="-1"
>
          <TabStateMessage tab="config" status={tabStatus} />
          <p id="config-mode-subtitle" class="admin-group-subtitle text-muted">
            Admin page configuration state is <strong>LOADING</strong>.
          </p>
          <div class="controls-grid controls-grid--config">
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Test Mode</h3>
              <p class="control-desc text-muted">Use for safe tuning. Enabled logs detections without blocking; disable to enforce defenses.</p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="test-mode-toggle">Enable Test Mode</label>
                  <label class="toggle-switch" for="test-mode-toggle">
                    <input type="checkbox" id="test-mode-toggle" aria-label="Enable Test Mode">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
              </div>
              <span id="test-mode-status" class={testModeStatusClass}>{testModeStatusText}</span>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>JS Required</h3>
              <p class="control-desc text-muted">Toggle whether normal requests require JS verification. Disable only for non-JS clients; this weakens bot defense and bypasses PoW on normal paths.</p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="js-required-enforced-toggle">Enforce JS Required</label>
                  <label class="toggle-switch" for="js-required-enforced-toggle">
                    <input type="checkbox" id="js-required-enforced-toggle" aria-label="Enforce JS required">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
              </div>
              <button id="save-js-required-config" class="btn btn-submit" disabled>Save JS Required</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Proof-of-Work (PoW)</h3>
              <p class="control-desc text-muted">Set verification work cost. Higher values increase scraper cost but can add friction on slower devices.</p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="pow-enabled-toggle">Enable PoW</label>
                  <label class="toggle-switch" for="pow-enabled-toggle">
                    <input type="checkbox" id="pow-enabled-toggle" checked aria-label="Enable PoW challenge verification">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="input-row">
                  <label class="control-label" for="pow-difficulty">Difficulty (bits)</label>
                  <input class="input-field" type="number" id="pow-difficulty" min="12" max="20" step="1" inputmode="numeric" value="15" aria-label="PoW difficulty in leading zero bits">
                </div>
                <div class="input-row">
                  <label class="control-label" for="pow-ttl">Seed TTL (seconds)</label>
                  <input class="input-field" type="number" id="pow-ttl" min="30" max="300" step="1" inputmode="numeric" value="90" aria-label="PoW seed TTL in seconds">
                </div>
              </div>
              <button id="save-pow-config" class="btn btn-submit" disabled>Save PoW Settings</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Challenge Puzzle</h3>
              <p class="control-desc text-muted">Set how many transform options are shown in puzzle challenges (higher values can increase solve time).</p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="challenge-puzzle-enabled-toggle">Enable Challenge Puzzle</label>
                  <label class="toggle-switch" for="challenge-puzzle-enabled-toggle">
                    <input type="checkbox" id="challenge-puzzle-enabled-toggle" aria-label="Enable challenge puzzle routing">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="input-row">
                  <label class="control-label" for="challenge-puzzle-transform-count">Transform Options</label>
                  <input class="input-field" type="number" id="challenge-puzzle-transform-count" min="4" max="8" step="1" inputmode="numeric" value="6" aria-label="Challenge transform option count">
                </div>
              </div>
              <button id="save-challenge-puzzle-config" class="btn btn-submit" disabled>Save Challenge Puzzle</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Rate Limiting</h3>
              <p class="control-desc text-muted">Set allowed requests per minute. Lower values are stricter but can affect legitimate burst traffic.</p>
              <div class="admin-controls">
                <div class="input-row">
                  <label class="control-label control-label--wide" for="rate-limit-threshold">Requests Per Minute</label>
                  <input class="input-field" type="number" id="rate-limit-threshold" min="1" max="1000000" step="1" inputmode="numeric" value="80" aria-label="Rate limit requests per minute">
                </div>
              </div>
              <button id="save-rate-limit-config" class="btn btn-submit" disabled>Save Rate Limit</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Honeypot Paths</h3>
              <p class="control-desc text-muted">One path per line. Requests that hit these paths are treated as high-confidence bot behavior. Paths must start with <code>/</code>.</p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="honeypot-enabled-toggle">Enable Honeypot</label>
                  <label class="toggle-switch" for="honeypot-enabled-toggle">
                    <input type="checkbox" id="honeypot-enabled-toggle" aria-label="Enable honeypot">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="geo-field">
                  <label class="control-label" for="honeypot-paths">Paths</label>
                  <textarea class="input-field geo-textarea" id="honeypot-paths" rows="3" aria-label="Honeypot paths" spellcheck="false"></textarea>
                </div>
              </div>
              <button id="save-honeypot-config" class="btn btn-submit" disabled>Save Honeypots</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Browser Policy</h3>
              <p class="control-desc text-muted">Use one rule per line in <code>BrowserName,min_major</code> format (for example <code>Chrome,120</code>).</p>
              <div class="admin-controls">
                <div class="geo-field">
                  <label class="control-label" for="browser-block-rules">Minimum Versions (Block)</label>
                  <textarea class="input-field geo-textarea" id="browser-block-rules" rows="3" aria-label="Browser block minimum versions" spellcheck="false"></textarea>
                </div>
                <div class="geo-field">
                  <label class="control-label" for="browser-whitelist-rules">Allowlist Exceptions</label>
                  <textarea class="input-field geo-textarea" id="browser-whitelist-rules" rows="2" aria-label="Browser allowlist exceptions" spellcheck="false"></textarea>
                </div>
              </div>
              <button id="save-browser-policy-config" class="btn btn-submit" disabled>Save Browser Policy</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Bypass Allowlists</h3>
              <p class="control-desc text-muted">Define trusted bypass entries. Use one entry per line.</p>
              <div class="admin-controls">
                <div class="geo-field">
                  <label class="control-label" for="network-whitelist">IP/CIDR Allowlist</label>
                  <textarea class="input-field geo-textarea" id="network-whitelist" rows="3" aria-label="IP and CIDR allowlist" spellcheck="false"></textarea>
                </div>
                <div class="geo-field">
                  <label class="control-label" for="path-whitelist">Path Allowlist</label>
                  <textarea class="input-field geo-textarea" id="path-whitelist" rows="3" aria-label="Path allowlist" spellcheck="false"></textarea>
                </div>
              </div>
              <button id="save-whitelist-config" class="btn btn-submit" disabled>Save Allowlists</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Maze</h3>
              <p class="control-desc text-muted">
                Control trap-page routing and optional auto-ban. Lower thresholds ban faster but may increase false positives.
                <a id="preview-maze-link" href="/admin/maze/preview" target="_blank" rel="noopener noreferrer">Preview Maze</a>
                in a non-operational view (admin session required).
              </p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label" for="maze-enabled-toggle">Maze Enabled</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="maze-enabled-toggle" checked aria-label="Enable maze">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="toggle-row">
                  <label class="control-label" for="maze-auto-ban-toggle">Ban on entry</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="maze-auto-ban-toggle" checked aria-label="Enable maze ban on entry">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="input-row">
                  <label class="control-label" for="maze-threshold">Ban Threshold (pages)</label>
                  <input class="input-field" type="number" id="maze-threshold" value="50" min="5" max="500" step="1" inputmode="numeric" aria-label="Maze ban threshold in pages">
                </div>
              </div>
              <button id="save-maze-config" class="btn btn-submit" disabled>Save Maze Settings</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>CDP (Detect Browser Automation)</h3>
              <p class="control-desc text-muted">Control automation-signal detection and optional auto-ban. Stricter thresholds catch more bots but may increase false positives.</p>
              <div class="admin-controls">
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="cdp-enabled-toggle">Enable Detection</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="cdp-enabled-toggle" checked aria-label="Enable CDP detection">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="cdp-auto-ban-toggle">Auto-ban on Detection</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="cdp-auto-ban-toggle" checked aria-label="Enable CDP auto-ban">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="slider-control">
                  <div class="slider-header">
                    <label class="control-label control-label--wide" for="cdp-threshold-slider">Detection Threshold</label>
                    <span id="cdp-threshold-value" class="slider-badge">0.6</span>
                  </div>
                  <input type="range" id="cdp-threshold-slider" min="0.3" max="1.0" step="0.1" value="0.6" aria-label="CDP detection threshold">
                  <div class="slider-labels">
                    <span>Strict</span>
                    <span>Permissive</span>
                  </div>
                </div>
              </div>
              <button id="save-cdp-config" class="btn btn-submit" disabled>Save CDP Settings</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Edge Integration Mode</h3>
              <p class="control-desc text-muted">Control how external edge bot outcomes influence local policy: off ignores edge outcomes, advisory records them without direct enforcement, authoritative allows strong edge outcomes to short-circuit.</p>
              <div class="admin-controls">
                <div class="input-row">
                  <label class="control-label control-label--wide" for="edge-integration-mode-select">Mode</label>
                  <select class="input-field" id="edge-integration-mode-select" aria-label="Edge integration mode">
                    <option value="off">off</option>
                    <option value="advisory">advisory</option>
                    <option value="authoritative">authoritative</option>
                  </select>
                </div>
              </div>
              <button id="save-edge-integration-mode-config" class="btn btn-submit" disabled>Save Edge Integration Mode</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>GEO Risk Based Scoring</h3>
              <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be receive added botness risk to contribute to the combined score.</p>
              <div class="admin-controls geo-controls">
                <div class="geo-field">
                  <label class="control-label" for="geo-risk-list">Scoring Countries</label>
                  <textarea class="input-field geo-textarea" id="geo-risk-list" rows="1" aria-label="GEO scoring countries" spellcheck="false"></textarea>
                </div>
              </div>
              <button id="save-geo-scoring-config" class="btn btn-submit" disabled>Save GEO Scoring</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>GEO Risk Based Routing</h3>
              <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be automatically routed. Precedence is Block &gt; Maze &gt; Challenge &gt; Allow.</p>
              <div class="admin-controls geo-controls">
                <div class="geo-field">
                  <label class="control-label" for="geo-block-list">Block Countries</label>
                  <textarea class="input-field geo-textarea" id="geo-block-list" rows="1" aria-label="GEO block countries" spellcheck="false"></textarea>
                </div>
                <div class="geo-field">
                  <label class="control-label" for="geo-maze-list">Maze Countries</label>
                  <textarea class="input-field geo-textarea" id="geo-maze-list" rows="1" aria-label="GEO maze countries" spellcheck="false"></textarea>
                </div>
                <div class="geo-field">
                  <label class="control-label" for="geo-challenge-list">Challenge Countries</label>
                  <textarea class="input-field geo-textarea" id="geo-challenge-list" rows="1" aria-label="GEO challenge countries" spellcheck="false"></textarea>
                </div>
                <div class="geo-field">
                  <label class="control-label" for="geo-allow-list">Allow Countries</label>
                  <textarea class="input-field geo-textarea" id="geo-allow-list" rows="1" aria-label="GEO allow countries" spellcheck="false"></textarea>
                </div>
              </div>
              <button id="save-geo-routing-config" class="btn btn-submit" disabled>Save GEO Routing</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Ban Durations</h3>
              <p class="control-desc text-muted">Set ban length in days, hours, and minutes per trigger type. Longer bans increase deterrence but slow recovery from false positives.</p>
              <div class="duration-grid">
                <div class="duration-row">
                  <label class="control-label" for="dur-honeypot-days">Maze Threshold Exceeded</label>
                  <div class="duration-inputs">
                    <label class="duration-input" for="dur-honeypot-days">
                      <input id="dur-honeypot-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" value="1" />
                      <span class="input-unit">days</span>
                    </label>
                    <label class="duration-input" for="dur-honeypot-hours">
                      <input id="dur-honeypot-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">hrs</span>
                    </label>
                    <label class="duration-input" for="dur-honeypot-minutes">
                      <input id="dur-honeypot-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">mins</span>
                    </label>
                  </div>
                </div>
                <div class="duration-row">
                  <label class="control-label" for="dur-rate-limit-days">Rate Limit Exceeded</label>
                  <div class="duration-inputs">
                    <label class="duration-input" for="dur-rate-limit-days">
                      <input id="dur-rate-limit-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">days</span>
                    </label>
                    <label class="duration-input" for="dur-rate-limit-hours">
                      <input id="dur-rate-limit-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" value="1" />
                      <span class="input-unit">hrs</span>
                    </label>
                    <label class="duration-input" for="dur-rate-limit-minutes">
                      <input id="dur-rate-limit-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">mins</span>
                    </label>
                  </div>
                </div>
                <div class="duration-row">
                  <label class="control-label" for="dur-browser-days">Browser Automation Detected</label>
                  <div class="duration-inputs">
                    <label class="duration-input" for="dur-browser-days">
                      <input id="dur-browser-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">days</span>
                    </label>
                    <label class="duration-input" for="dur-browser-hours">
                      <input id="dur-browser-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" value="6" />
                      <span class="input-unit">hrs</span>
                    </label>
                    <label class="duration-input" for="dur-browser-minutes">
                      <input id="dur-browser-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">mins</span>
                    </label>
                  </div>
                </div>
                <div class="duration-row">
                  <label class="control-label" for="dur-cdp-days">CDP Automation Detected</label>
                  <div class="duration-inputs">
                    <label class="duration-input" for="dur-cdp-days">
                      <input id="dur-cdp-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">days</span>
                    </label>
                    <label class="duration-input" for="dur-cdp-hours">
                      <input id="dur-cdp-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" value="12" />
                      <span class="input-unit">hrs</span>
                    </label>
                    <label class="duration-input" for="dur-cdp-minutes">
                      <input id="dur-cdp-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">mins</span>
                    </label>
                  </div>
                </div>
                <div class="duration-row">
                  <label class="control-label" for="dur-admin-days">Admin Manual Ban Default</label>
                  <div class="duration-inputs">
                    <label class="duration-input" for="dur-admin-days">
                      <input id="dur-admin-days" class="input-field" type="number" min="0" max="365" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">days</span>
                    </label>
                    <label class="duration-input" for="dur-admin-hours">
                      <input id="dur-admin-hours" class="input-field" type="number" min="0" max="23" step="1" inputmode="numeric" value="6" />
                      <span class="input-unit">hrs</span>
                    </label>
                    <label class="duration-input" for="dur-admin-minutes">
                      <input id="dur-admin-minutes" class="input-field" type="number" min="0" max="59" step="1" inputmode="numeric" value="0" />
                      <span class="input-unit">mins</span>
                    </label>
                  </div>
                </div>
              </div>
              <button id="save-durations-btn" class="btn btn-submit" disabled>Save Durations</button>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Robots and AI Bot Policy</h3>
              <p class="control-desc text-muted">Keep robots.txt serving controls separate from AI bot policy controls.</p>
              <div class="admin-controls">
                <h4 class="control-subtitle">robots.txt Serving</h4>
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="robots-enabled-toggle">Serve robots.txt</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="robots-enabled-toggle" checked aria-label="Serve robots.txt">
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="input-row">
                  <label class="control-label control-label--wide" for="robots-crawl-delay">Crawl Delay (seconds)</label>
                  <input class="input-field" type="number" id="robots-crawl-delay" value="2" min="0" max="60" step="1" inputmode="numeric" aria-label="Robots crawl delay in seconds">
                </div>
                <button id="save-robots-config" class="btn btn-submit" disabled>Save robots serving</button>
                <h4 class="control-subtitle">AI Bot Policy</h4>
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="robots-block-training-toggle">Opt-out AI Training</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="robots-block-training-toggle" checked aria-label="Opt-out AI training">
                    <span class="toggle-slider"></span>
                  </label>
                  <span class="toggle-hint">GPTBot, CCBot, ClaudeBot</span>
                </div>
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="robots-block-search-toggle">Opt-out AI Search</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="robots-block-search-toggle" aria-label="Opt-out AI search">
                    <span class="toggle-slider"></span>
                  </label>
                  <span class="toggle-hint">PerplexityBot, etc.</span>
                </div>
                <div class="toggle-row">
                  <label class="control-label control-label--wide" for="robots-allow-search-toggle">Restrict Search Engines</label>
                  <label class="toggle-switch">
                    <input type="checkbox" id="robots-allow-search-toggle" aria-label="Restrict search engines">
                    <span class="toggle-slider"></span>
                  </label>
                  <span class="toggle-hint">Google, Bing, etc.</span>
                </div>
              </div>
              <button id="save-ai-policy-config" class="btn btn-submit" disabled>Save AI bot policy</button>
              <button id="preview-robots" class="btn btn-subtle">Show robots.txt</button>
              <div id="robots-preview" class="robots-preview panel hidden pad-sm">
                <h4>robots.txt Preview</h4>
                <pre id="robots-preview-content"></pre>
              </div>
            </div>
            <div class="control-group panel-soft pad-md config-edit-pane hidden">
              <h3>Advanced Config JSON</h3>
              <p class="control-desc text-muted">Directly edit writable config keys as a JSON object. This exposes advanced keys that do not yet have dedicated pane controls.</p>
              <div class="admin-controls">
                <div class="geo-field">
                  <label class="control-label" for="advanced-config-json">JSON Patch</label>
                  <textarea class="input-field geo-textarea" id="advanced-config-json" rows="8" aria-label="Advanced config JSON patch" spellcheck="false"></textarea>
                </div>
              </div>
              <button id="save-advanced-config" class="btn btn-submit" disabled>Save Advanced Config</button>
            </div>
          </div>
        </section>
