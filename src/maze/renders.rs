use super::content::{
    capitalize, generate_fake_date, generate_link_text, generate_paragraph, generate_title,
    DEPARTMENTS, NOUNS,
};
use super::rng::{generate_path_segment, path_to_seed, SeededRng};
use super::types::MazeConfig;

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
