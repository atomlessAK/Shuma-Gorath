use rand::Rng;
use spin_sdk::http::Request;

use super::token::make_seed_token;
use super::types::NotABotSeed;

pub(crate) fn render_not_a_bot(
    req: &Request,
    cfg: &crate::config::Config,
) -> spin_sdk::http::Response {
    let ip = crate::extract_client_ip(req);
    let ua = req
        .header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let now = crate::admin::now_ts();
    let mut rng = rand::rng();
    let seed = NotABotSeed {
        operation_id: format!("{:016x}{:016x}", rng.random::<u64>(), rng.random::<u64>()),
        flow_id: crate::challenge::operation_envelope::FLOW_NOT_A_BOT.to_string(),
        step_id: crate::challenge::operation_envelope::STEP_NOT_A_BOT_SUBMIT.to_string(),
        step_index: crate::challenge::operation_envelope::STEP_INDEX_NOT_A_BOT_SUBMIT,
        issued_at: now,
        expires_at: now.saturating_add(cfg.not_a_bot_nonce_ttl_seconds),
        token_version: crate::challenge::operation_envelope::TOKEN_VERSION_V1,
        ip_bucket: crate::signals::ip_identity::bucket_ip(&ip),
        ua_bucket: crate::challenge::operation_envelope::user_agent_bucket(ua),
        path_class: crate::challenge::operation_envelope::PATH_CLASS_NOT_A_BOT_SUBMIT.to_string(),
        return_to: normalize_return_to(req.uri()),
    };
    let seed_token = make_seed_token(&seed);

    let html = format!(
        r#"
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Please confrim you are not a bot</title>
  <style>
    :root {{
      --bg: #fffafd;
      --panel: #ffffff;
      --ink: #111111;
      --muted: #5b6472;
      --border: #dfd5e3;
      --accent: #2b021f;
      --accent-ink: #ffffff;
      --focus: #a86f97;
    }}
    body {{
      margin: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: repeating-linear-gradient(-45deg, var(--bg), var(--bg) 10px, #f8f0f9 10px, #f8f0f9 20px);
      color: var(--ink);
      min-height: 100vh;
      display: grid;
      place-items: center;
      padding: 16px;
    }}
    .panel {{
      width: min(520px, 100%);
      border: 1px solid var(--border);
      background: var(--panel);
      padding: 20px;
      box-sizing: border-box;
    }}
    h1 {{
      margin: 0 0 8px;
      font-size: 1.35rem;
      line-height: 1.3;
    }}
    p {{
      margin: 0 0 16px;
      color: var(--muted);
      line-height: 1.5;
    }}
    .checkbox-row {{
      margin: 16px 0;
    }}
    .checkbox-control {{
      width: 100%;
      border: 1px solid var(--border);
      background: #faf6fa;
      color: var(--ink);
      display: inline-flex;
      align-items: center;
      gap: 12px;
      padding: 12px 14px;
      cursor: pointer;
      text-align: left;
      font-size: 1rem;
      line-height: 1.3;
    }}
    .checkbox-control:hover {{
      background: #f6eff7;
    }}
    .checkbox-control[aria-checked="true"] {{
      border-color: var(--accent);
      background: #f4edf6;
    }}
    .checkbox-control[disabled] {{
      cursor: progress;
      opacity: 0.8;
    }}
    .checkbox-box {{
      position: relative;
      inline-size: 20px;
      block-size: 20px;
      border: 2px solid #5d2f4f;
      background: #ffffff;
      flex: 0 0 20px;
      box-sizing: border-box;
    }}
    .checkbox-mark {{
      position: absolute;
      left: 5px;
      top: 0px;
      width: 6px;
      height: 12px;
      border-right: 3px solid var(--accent);
      border-bottom: 3px solid var(--accent);
      transform: rotate(45deg) scale(0);
      transform-origin: center;
      transition: transform 0.12s ease-out;
    }}
    .checkbox-control[aria-checked="true"] .checkbox-mark {{
      transform: rotate(45deg) scale(1);
    }}
    .checkbox-control:focus-visible {{
      outline: 2px solid var(--focus);
      outline-offset: 2px;
    }}
  </style>
</head>
<body>
  <main class="panel">
    <h1>Please confrim you are not a bot</h1>
    <form id="not-a-bot-form" method="POST" action="{not_a_bot_path}">
      <input type="hidden" name="seed" value="{seed_token}" />
      <input type="hidden" id="not-a-bot-checked" name="checked" value="0" />
      <input type="hidden" id="not-a-bot-telemetry" name="telemetry" value="" />
      <div class="checkbox-row">
        <button
          id="not-a-bot-checkbox"
          class="checkbox-control"
          type="button"
          role="checkbox"
          aria-checked="false"
        >
          <span class="checkbox-box" aria-hidden="true"><span class="checkbox-mark"></span></span>
          <span>I am not a bot</span>
        </button>
      </div>
    </form>
  </main>
  <script>
    (function () {{
      const form = document.getElementById('not-a-bot-form');
      const checkbox = document.getElementById('not-a-bot-checkbox');
      const checked = document.getElementById('not-a-bot-checked');
      const telemetryField = document.getElementById('not-a-bot-telemetry');

      const start = performance.now();
      let pointerDownAt = 0;
      let lastPoint = null;
      let lastAngle = null;
      let lastInputModality = 'unknown';
      let submissionStarted = false;

      const telemetry = {{
        has_pointer: false,
        pointer_move_count: 0,
        pointer_path_length: 0,
        pointer_direction_changes: 0,
        down_up_ms: 0,
        focus_changes: 0,
        visibility_changes: 0,
        interaction_elapsed_ms: 0,
        keyboard_used: false,
        touch_used: false,
        events_order_valid: false,
        activation_method: 'unknown',
        activation_trusted: false,
        activation_count: 0,
        control_focused: false
      }};

      function boundedIncrement(key, max) {{
        telemetry[key] = Math.min(max, (telemetry[key] || 0) + 1);
      }}

      function resolveActivationMethod() {{
        if (lastInputModality === 'unknown') {{
          if (telemetry.touch_used) return 'touch';
          if (telemetry.keyboard_used) return 'keyboard';
          if (telemetry.has_pointer) return 'pointer';
          return 'unknown';
        }}
        return lastInputModality;
      }}

      function submitVerification(event) {{
        if (submissionStarted) {{
          return;
        }}
        submissionStarted = true;
        checked.value = '1';
        telemetry.events_order_valid = true;
        telemetry.activation_method = resolveActivationMethod();
        telemetry.activation_trusted = event && event.isTrusted === true;
        telemetry.activation_count = Math.min(255, (telemetry.activation_count || 0) + 1);
        telemetry.control_focused = (document.activeElement === checkbox);
        telemetry.interaction_elapsed_ms = Math.min(
          600000,
          Math.max(0, Math.round(performance.now() - start))
        );
        telemetryField.value = JSON.stringify(telemetry);
        checkbox.setAttribute('aria-checked', 'true');
        checkbox.disabled = true;
        if (typeof form.requestSubmit === 'function') {{
          form.requestSubmit();
          return;
        }}
        form.submit();
      }}

      document.addEventListener('pointermove', function (event) {{
        telemetry.has_pointer = true;
        boundedIncrement('pointer_move_count', 60000);
        const point = {{ x: event.clientX || 0, y: event.clientY || 0 }};
        if (lastPoint) {{
          const dx = point.x - lastPoint.x;
          const dy = point.y - lastPoint.y;
          const segmentLength = Math.sqrt((dx * dx) + (dy * dy));
          telemetry.pointer_path_length = Math.min(100000, telemetry.pointer_path_length + segmentLength);
          const angle = Math.atan2(dy, dx);
          if (lastAngle !== null && Math.abs(angle - lastAngle) > 0.5) {{
            boundedIncrement('pointer_direction_changes', 60000);
          }}
          lastAngle = angle;
        }}
        lastPoint = point;
      }}, {{ passive: true }});

      document.addEventListener('pointerdown', function (event) {{
        lastInputModality = event.pointerType === 'touch' ? 'touch' : 'pointer';
        pointerDownAt = performance.now();
        if (event.pointerType === 'touch') {{
          telemetry.touch_used = true;
        }}
      }}, {{ passive: true }});

      document.addEventListener('pointerup', function () {{
        if (pointerDownAt > 0) {{
          const ms = Math.max(0, Math.round(performance.now() - pointerDownAt));
          telemetry.down_up_ms = Math.min(600000, ms);
          pointerDownAt = 0;
        }}
      }}, {{ passive: true }});

      document.addEventListener('keydown', function () {{
        lastInputModality = 'keyboard';
        telemetry.keyboard_used = true;
      }});

      window.addEventListener('focus', function () {{
        boundedIncrement('focus_changes', 255);
      }}, {{ passive: true }});

      document.addEventListener('visibilitychange', function () {{
        boundedIncrement('visibility_changes', 255);
      }}, {{ passive: true }});

      checkbox.addEventListener('click', function (event) {{
        submitVerification(event);
      }});

      form.addEventListener('submit', function (event) {{
        if (checked.value !== '1') {{
          event.preventDefault();
          return;
        }}
        telemetry.interaction_elapsed_ms = Math.min(
          600000,
          Math.max(0, Math.round(performance.now() - start))
        );
        telemetryField.value = JSON.stringify(telemetry);
      }});
    }})();
  </script>
</body>
</html>
"#,
        not_a_bot_path = crate::challenge::NOT_A_BOT_PATH,
        seed_token = seed_token
    );

    crate::challenge::challenge_response(200, html.as_str())
}

pub(crate) fn normalize_return_to(uri: &str) -> String {
    let candidate = uri
        .split('#')
        .next()
        .unwrap_or("/")
        .trim();
    if candidate.is_empty()
        || !candidate.starts_with('/')
        || candidate.starts_with("//")
        || candidate.starts_with(crate::challenge::NOT_A_BOT_PATH)
        || candidate.len() > 512
    {
        return "/".to_string();
    }
    candidate.to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalize_return_to, render_not_a_bot};
    use spin_sdk::http::{Method, Request};

    #[test]
    fn render_not_a_bot_uses_checkbox_like_control_and_auto_submit_flow() {
        let req = Request::builder()
            .method(Method::Get)
            .uri("/challenge/not-a-bot-checkbox")
            .build();
        let response = render_not_a_bot(&req, crate::config::defaults());
        let body = String::from_utf8(response.into_body()).expect("render body should be utf8");
        assert!(body.contains("role=\"checkbox\""));
        assert!(!body.contains("type=\"checkbox\""));
        assert!(!body.contains("id=\"not-a-bot-submit\""));
        assert!(body.contains("function submitVerification"));
    }

    #[test]
    fn normalize_return_to_rejects_not_a_bot_self_route() {
        assert_eq!(
            normalize_return_to("/challenge/not-a-bot-checkbox?next=/dashboard"),
            "/"
        );
    }
}
