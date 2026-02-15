use super::content::{
    capitalize, generate_fake_date, generate_link_text, generate_paragraph, generate_title,
    DEPARTMENTS, NOUNS,
};
use super::rng::{generate_path_segment, path_to_seed, SeededRng};
use super::types::MazeConfig;

pub(crate) struct AdvancedMazeLink {
    pub href: String,
    pub text: String,
    pub description: String,
    pub pow_difficulty: Option<u8>,
}

pub(crate) struct AdvancedMazeRenderOptions {
    pub title: String,
    pub breadcrumb: String,
    pub paragraphs: Vec<String>,
    pub links: Vec<AdvancedMazeLink>,
    pub server_visible_links: usize,
    pub bootstrap_json: String,
    pub variant_layout: u8,
    pub variant_palette: u8,
    pub variant_id: String,
    pub rollout_phase: String,
}

/// Generate a maze page HTML response.
pub fn generate_maze_page(path: &str, config: &MazeConfig) -> String {
    let seed = path_to_seed(path);
    let mut rng = SeededRng::new(seed);

    let title = generate_title(&mut rng);
    let num_links = rng.range(config.min_links, config.max_links);
    let num_paragraphs = rng.range(config.min_paragraphs, config.max_paragraphs);

    // Generate breadcrumb parts
    let dept = rng.pick(DEPARTMENTS);
    let breadcrumb_noun = capitalize(rng.pick(NOUNS));

    // Determine the base path (keep /trap/ or /maze/ prefix)
    let base_prefix = if path.starts_with("/trap/") {
        "/trap/"
    } else {
        "/maze/"
    };

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }}
        .container {{ 
            max-width: 1200px; 
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        header {{
            background: linear-gradient(90deg, #1a1a2e 0%, #16213e 100%);
            color: white;
            padding: 30px 40px;
        }}
        header h1 {{ font-size: 1.8rem; font-weight: 600; }}
        .breadcrumb {{ 
            color: #888; 
            font-size: 0.9rem; 
            margin-top: 8px;
        }}
        .content {{ padding: 40px; }}
        .description {{ 
            color: #555; 
            line-height: 1.8; 
            margin-bottom: 30px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}
        .nav-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 20px;
            margin-top: 30px;
        }}
        .nav-card {{
            background: white;
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            padding: 20px;
            transition: all 0.3s ease;
            text-decoration: none;
            color: inherit;
            display: block;
        }}
        .nav-card:hover {{
            border-color: #667eea;
            box-shadow: 0 8px 25px rgba(102, 126, 234, 0.15);
            transform: translateY(-2px);
        }}
        .nav-card h3 {{
            color: #1a1a2e;
            font-size: 1rem;
            margin-bottom: 8px;
        }}
        .nav-card p {{
            color: #666;
            font-size: 0.85rem;
            line-height: 1.5;
        }}
        .nav-card .arrow {{
            color: #667eea;
            margin-top: 10px;
            font-size: 0.9rem;
        }}
        footer {{
            background: #f8f9fa;
            padding: 20px 40px;
            color: #888;
            font-size: 0.85rem;
            border-top: 1px solid #e0e0e0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>{title}</h1>
            <div class="breadcrumb">Portal &gt; {dept} &gt; {breadcrumb_noun} Management</div>
        </header>
        <div class="content">
"#
    );

    // Add description paragraphs
    for _ in 0..num_paragraphs {
        let para = generate_paragraph(&mut rng);
        html.push_str(&format!(
            "            <p class=\"description\">{}</p>\n",
            para
        ));
    }

    // Add navigation grid with links
    html.push_str("            <div class=\"nav-grid\">\n");

    for _ in 0..num_links {
        let link_path = format!("{}{}", base_prefix, generate_path_segment(&mut rng, 16));
        let link_text = generate_link_text(&mut rng);
        let link_desc = generate_paragraph(&mut rng);
        // Truncate description
        let short_desc: String = link_desc.chars().take(80).collect();

        html.push_str(&format!(
            r#"                <a href="{}" class="nav-card">
                    <h3>{}</h3>
                    <p>{}...</p>
                    <div class="arrow">Access →</div>
                </a>
"#,
            link_path, link_text, short_desc
        ));
    }

    html.push_str("            </div>\n");
    html.push_str("        </div>\n");

    // Footer
    let footer_date = generate_fake_date(&mut rng);
    let session_id = generate_path_segment(&mut rng, 8);
    html.push_str(&format!(
        r#"        <footer>
            <p>Internal Portal • Last updated: {} • Session ID: {}</p>
        </footer>
    </div>
</body>
</html>"#,
        footer_date, session_id
    ));

    html
}

fn palette(variant_palette: u8) -> (&'static str, &'static str, &'static str, &'static str) {
    match variant_palette % 3 {
        0 => ("#0f172a", "#e2e8f0", "#38bdf8", "#f8fafc"),
        1 => ("#1f2937", "#fef3c7", "#f59e0b", "#fffbeb"),
        _ => ("#1e293b", "#dcfce7", "#22c55e", "#f0fdf4"),
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub(crate) fn generate_polymorphic_maze_page(options: &AdvancedMazeRenderOptions) -> String {
    let (header_bg, header_fg, accent, panel_bg) = palette(options.variant_palette);
    let layout_class = match options.variant_layout % 3 {
        0 => "layout-grid",
        1 => "layout-stacked",
        _ => "layout-ribbon",
    };

    let server_visible = options.server_visible_links.min(options.links.len());
    let visible_links = &options.links[..server_visible];

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <meta name="robots" content="noindex,nofollow,noarchive">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: "IBM Plex Sans", "Segoe UI", system-ui, sans-serif;
            min-height: 100vh;
            padding: 24px;
            background: radial-gradient(circle at 15% 15%, #0b1020 0%, #020617 70%);
            color: #111827;
        }}
        .wrap {{
            max-width: 1120px;
            margin: 0 auto;
            background: #ffffff;
            border-radius: 16px;
            overflow: hidden;
            border: 1px solid #e5e7eb;
            box-shadow: 0 28px 60px rgba(2, 6, 23, 0.35);
        }}
        header {{
            background: {};
            color: {};
            padding: 24px 30px;
        }}
        header h1 {{ font-size: 1.7rem; letter-spacing: 0.01em; }}
        .crumb {{ margin-top: 8px; opacity: 0.82; font-size: 0.9rem; }}
        .meta {{
            margin-top: 8px;
            font-size: 0.78rem;
            text-transform: uppercase;
            letter-spacing: 0.08em;
            opacity: 0.8;
        }}
        .content {{ padding: 28px; background: {}; }}
        .description {{
            background: #fff;
            border-left: 4px solid {};
            border-radius: 8px;
            padding: 14px;
            line-height: 1.7;
            margin-bottom: 14px;
        }}
        .nav-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
            gap: 14px;
            margin-top: 16px;
        }}
        .layout-stacked .nav-grid {{
            grid-template-columns: 1fr;
        }}
        .layout-ribbon .nav-grid {{
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        }}
        .nav-card {{
            display: block;
            text-decoration: none;
            color: inherit;
            border: 1px solid #e5e7eb;
            border-radius: 10px;
            background: #ffffff;
            padding: 16px;
            transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
        }}
        .nav-card:hover {{
            transform: translateY(-2px);
            border-color: {};
            box-shadow: 0 12px 24px rgba(15, 23, 42, 0.15);
        }}
        .nav-card h3 {{ color: #0f172a; font-size: 0.98rem; margin-bottom: 6px; }}
        .nav-card p {{ color: #475569; font-size: 0.86rem; line-height: 1.5; }}
        .nav-card .arrow {{ margin-top: 8px; color: {}; font-size: 0.84rem; }}
        .pow-hint {{
            margin-top: 6px;
            font-size: 0.75rem;
            color: #7c2d12;
            background: #ffedd5;
            border-radius: 999px;
            display: inline-block;
            padding: 2px 8px;
        }}
        .hidden-link {{
            position: absolute !important;
            width: 1px;
            height: 1px;
            margin: -1px;
            padding: 0;
            border: 0;
            clip: rect(0 0 0 0);
            clip-path: inset(50%);
            overflow: hidden;
            white-space: nowrap;
        }}
        footer {{
            background: #f8fafc;
            border-top: 1px solid #e2e8f0;
            font-size: 0.8rem;
            color: #64748b;
            padding: 12px 28px;
        }}
    </style>
</head>
<body>
    <div class="wrap {}">
        <header>
            <h1>{}</h1>
            <div class="crumb">{}</div>
            <div class="meta">Variant {} • Rollout {}</div>
        </header>
        <div class="content">
"#,
        options.title,
        header_bg,
        header_fg,
        panel_bg,
        accent,
        accent,
        accent,
        layout_class,
        options.title,
        options.breadcrumb,
        options.variant_id,
        options.rollout_phase
    );

    for paragraph in &options.paragraphs {
        html.push_str(&format!(
            r#"            <p class="description">{}</p>
"#,
            paragraph
        ));
    }

    html.push_str(r#"            <div class="nav-grid" id="maze-nav-grid">
"#);

    for link in visible_links {
        let pow_hint = link
            .pow_difficulty
            .map(|difficulty| {
                format!(
                    r#"<div class="pow-hint">Deep-traversal proof required ({} bits)</div>"#,
                    difficulty
                )
            })
            .unwrap_or_default();
        let pow_attr = link
            .pow_difficulty
            .map(|difficulty| format!(r#" data-pow-difficulty="{}""#, difficulty))
            .unwrap_or_default();
        html.push_str(&format!(
            r#"                <a href="{}" class="nav-card"{} data-link-kind="maze">
                    <h3>{}</h3>
                    <p>{}</p>
                    {}
                    <div class="arrow">Continue →</div>
                </a>
"#,
            link.href, pow_attr, link.text, link.description, pow_hint
        ));
    }

    html.push_str(r#"            </div>
        </div>
        <footer>
            <p>Synthetic navigation surface. Not authoritative content.</p>
        </footer>
    </div>
    <script>
        (function () {
"#);
    html.push_str("            const bootstrap = JSON.parse(\"");
    html.push_str(escape_json(options.bootstrap_json.as_str()).as_str());
    html.push_str(
        r#"\");
            const navGrid = document.getElementById('maze-nav-grid');
            if (!bootstrap || !Array.isArray(bootstrap.hidden_links) || !navGrid) {
                return;
            }

            async function sendCheckpoint() {
                if (!bootstrap.checkpoint_token) return;
                try {
                    await fetch('/maze/checkpoint', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            token: bootstrap.checkpoint_token,
                            flow_id: bootstrap.flow_id,
                            depth: bootstrap.depth,
                            checkpoint_reason: 'page_load'
                        }})
                    }});
                }} catch (_e) {{
                    // Best effort only.
                }}
            }

            function attachPowHandler(anchor) {
                const difficultyRaw = anchor.getAttribute('data-pow-difficulty');
                if (!difficultyRaw) return;
                const difficulty = parseInt(difficultyRaw, 10);
                if (!Number.isFinite(difficulty) || difficulty <= 0) return;

                anchor.addEventListener('click', async function (event) {
                    if (anchor.dataset.powReady === '1') return;
                    event.preventDefault();
                    anchor.dataset.powReady = '0';

                    const href = new URL(anchor.href, window.location.origin);
                    const token = href.searchParams.get('mt') || '';
                    let nonce = 0;
                    const maxIterations = 600000;

                    while (nonce < maxIterations) {
                        const raw = new TextEncoder().encode(`${token}:${nonce}`);
                        const hash = await crypto.subtle.digest('SHA-256', raw);
                        const bytes = new Uint8Array(hash);
                        let bits = difficulty;
                        let ok = true;
                        for (let i = 0; i < bytes.length; i += 1) {
                            if (bits <= 0) break;
                            const value = bytes[i];
                            if (bits >= 8) {
                                if (value !== 0) {{ ok = false; break; }}
                                bits -= 8;
                            } else {
                                const mask = 0xff << (8 - bits);
                                if ((value & mask) !== 0) {{ ok = false; }}
                                break;
                            }
                        }
                        if (ok) {{
                            href.searchParams.set('mpn', String(nonce));
                            anchor.dataset.powReady = '1';
                            window.location.assign(href.toString());
                            return;
                        }}
                        nonce += 1;
                    }
                    anchor.dataset.powReady = '0';
                });
            }

            for (const hidden of bootstrap.hidden_links) {
                const link = document.createElement('a');
                link.href = hidden.href;
                link.className = 'hidden-link';
                link.textContent = hidden.text || 'detail';
                if (hidden.pow_difficulty) {
                    link.setAttribute('data-pow-difficulty', String(hidden.pow_difficulty));
                }
                navGrid.appendChild(link);
                attachPowHandler(link);
            }

            const anchors = navGrid.querySelectorAll('a[data-pow-difficulty]');
            anchors.forEach(attachPowHandler);
            sendCheckpoint();
        })();
    </script>
</body>
</html>
"#,
    );

    html
}
