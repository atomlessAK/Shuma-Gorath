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
    pub bootstrap_json: String,
    pub variant_layout: u8,
    pub variant_palette: u8,
    pub style_tier: MazeStyleTier,
    pub style_sheet_url: Option<String>,
    pub script_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeStyleTier {
    Full,
    Lite,
    Machine,
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

fn escape_script_json(value: &str) -> String {
    value.replace("</", "<\\/")
}

pub(crate) fn generate_polymorphic_maze_page(options: &AdvancedMazeRenderOptions) -> String {
    let (_header_bg, _header_fg, _accent, _panel_bg) = palette(options.variant_palette);
    let layout_class = match options.variant_layout % 3 {
        0 => "layout-grid",
        1 => "layout-stacked",
        _ => "layout-ribbon",
    };
    let style_class = match options.style_tier {
        MazeStyleTier::Full => "style-full",
        MazeStyleTier::Lite => "style-lite",
        MazeStyleTier::Machine => "style-machine",
    };
    let mut head_assets = String::new();
    if let Some(url) = &options.style_sheet_url {
        head_assets.push_str(format!(r#"<link rel="stylesheet" href="{}">"#, url).as_str());
    }
    head_assets
        .push_str(format!(r#"<script defer src="{}"></script>"#, options.script_url).as_str());

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <meta name="robots" content="noindex,nofollow,noarchive">
    {}
</head>
<body class="{}">
    <div class="wrap {} {}">
        <header>
            <h1>{}</h1>
            <div class="crumb">{}</div>
        </header>
        <div class="content">
"#,
        options.title,
        head_assets,
        style_class,
        layout_class,
        style_class,
        options.title,
        options.breadcrumb
    );

    for paragraph in &options.paragraphs {
        html.push_str(&format!(
            r#"            <p class="description">{}</p>
"#,
            paragraph
        ));
    }

    html.push_str(
        r#"            <div class="nav-grid" id="maze-nav-grid">
"#,
    );

    for link in &options.links {
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

    html.push_str(
        r#"            </div>
        </div>
"#,
    );
    html.push_str(
        r#"    </div>
    <script id="maze-bootstrap" type="application/json">"#,
    );
    html.push_str(escape_script_json(options.bootstrap_json.as_str()).as_str());
    html.push_str(
        r#"</script>
</body>
</html>"#,
    );

    html
}
