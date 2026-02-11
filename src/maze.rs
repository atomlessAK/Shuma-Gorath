// src/maze.rs
// Maze - Traps web crawlers in infinite loops
// Generates deterministic fake pages with links that lead to more fake pages

use spin_sdk::http;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Configuration for the maze
pub struct MazeConfig {
    pub min_links: usize,
    pub max_links: usize,
    pub min_paragraphs: usize,
    pub max_paragraphs: usize,
}

impl Default for MazeConfig {
    fn default() -> Self {
        MazeConfig {
            min_links: 8,
            max_links: 15,
            min_paragraphs: 3,
            max_paragraphs: 6,
        }
    }
}

/// Simple seeded pseudo-random number generator (xorshift64)
/// We use this instead of rand crate to keep WASM size small
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        // Ensure non-zero state
        SeededRng {
            state: if seed == 0 { 0xDEADBEEF } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next() as usize % (max - min + 1))
    }

    /// Pick a random item from a static string slice
    fn pick(&mut self, items: &[&'static str]) -> &'static str {
        let idx = self.next() as usize % items.len();
        items[idx]
    }
}

/// Hash a path to get a deterministic seed
fn path_to_seed(path: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

/// Generate a random hex string for link paths
fn generate_path_segment(rng: &mut SeededRng, len: usize) -> String {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    (0..len)
        .map(|_| HEX_CHARS[rng.next() as usize % HEX_CHARS.len()] as char)
        .collect()
}

/// Word lists for generating fake content
const NOUNS: &[&str] = &[
    "system",
    "data",
    "server",
    "network",
    "client",
    "database",
    "file",
    "user",
    "admin",
    "config",
    "backup",
    "report",
    "dashboard",
    "analytics",
    "service",
    "process",
    "resource",
    "module",
    "component",
    "interface",
    "protocol",
    "session",
    "transaction",
    "record",
    "entry",
    "request",
    "response",
    "cache",
    "storage",
    "cluster",
    "node",
    "instance",
    "container",
    "deployment",
    "pipeline",
    "workflow",
];

const VERBS: &[&str] = &[
    "configure",
    "manage",
    "update",
    "delete",
    "create",
    "view",
    "export",
    "import",
    "sync",
    "backup",
    "restore",
    "monitor",
    "analyze",
    "optimize",
    "validate",
    "process",
    "submit",
    "review",
    "approve",
    "deploy",
    "migrate",
    "transform",
];

const ADJECTIVES: &[&str] = &[
    "advanced",
    "secure",
    "internal",
    "external",
    "primary",
    "secondary",
    "legacy",
    "updated",
    "archived",
    "active",
    "pending",
    "completed",
    "failed",
    "critical",
    "standard",
    "custom",
    "automated",
    "manual",
    "scheduled",
    "temporary",
    "permanent",
];

const DEPARTMENTS: &[&str] = &[
    "Sales",
    "Marketing",
    "Engineering",
    "HR",
    "Finance",
    "Operations",
    "Support",
    "IT",
    "Legal",
    "Compliance",
    "Security",
    "Development",
    "QA",
    "DevOps",
];

const MONTHS: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Generate a fake title for a page
fn generate_title(rng: &mut SeededRng) -> String {
    let pattern = rng.next() % 4;
    match pattern {
        0 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Management", adj, noun)
        }
        1 => {
            let dept = rng.pick(DEPARTMENTS);
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Portal", dept, noun)
        }
        2 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Dashboard", adj, noun)
        }
        _ => {
            let verb = capitalize(rng.pick(VERBS));
            let noun = capitalize(rng.pick(NOUNS));
            let adj = capitalize(rng.pick(ADJECTIVES));
            format!("{} {} - {} Access", verb, noun, adj)
        }
    }
}

/// Generate a fake link text
fn generate_link_text(rng: &mut SeededRng) -> String {
    let pattern = rng.next() % 7;
    match pattern {
        0 => {
            let verb = capitalize(rng.pick(VERBS));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {}", verb, noun)
        }
        1 => {
            let dept = rng.pick(DEPARTMENTS);
            let noun = rng.pick(NOUNS);
            format!("{} {} Portal", dept, noun)
        }
        2 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Settings", adj, noun)
        }
        3 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("View {} {}", adj, noun)
        }
        4 => {
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} Management", noun)
        }
        5 => {
            let dept = rng.pick(DEPARTMENTS);
            format!("{} Dashboard", dept)
        }
        _ => {
            let dept = rng.pick(DEPARTMENTS);
            let noun = rng.pick(NOUNS);
            format!("{} {} Report", dept, noun)
        }
    }
}

/// Generate a fake date string
fn generate_fake_date(rng: &mut SeededRng) -> String {
    let month = rng.pick(MONTHS);
    let day = rng.range(1, 28);
    let year_suffix = rng.range(3, 6);
    format!("{} {}, 202{}", month, day, year_suffix)
}

/// Generate a fake paragraph of text
fn generate_paragraph(rng: &mut SeededRng) -> String {
    let pattern = rng.next() % 5;
    match pattern {
        0 => {
            let adj1 = rng.pick(ADJECTIVES);
            let noun1 = rng.pick(NOUNS);
            let adj2 = rng.pick(ADJECTIVES);
            let adj3 = rng.pick(ADJECTIVES);
            let noun2 = rng.pick(NOUNS);
            let noun3 = rng.pick(NOUNS);
            format!(
                "The {} {} requires {} access to the {} {}. Please ensure all {} are properly configured before proceeding.",
                adj1, noun1, adj2, adj3, noun2, noun3
            )
        }
        1 => {
            let noun1 = rng.pick(NOUNS);
            let verb = rng.pick(VERBS);
            let adj = rng.pick(ADJECTIVES);
            let noun2 = rng.pick(NOUNS);
            let noun3 = rng.pick(NOUNS);
            format!(
                "This {} allows you to {} the {} {}. All changes are logged and can be reviewed in the {} section.",
                noun1, verb, adj, noun2, noun3
            )
        }
        2 => {
            let adj1 = rng.pick(ADJECTIVES);
            let noun = rng.pick(NOUNS);
            let adj2 = rng.pick(ADJECTIVES);
            let dept = rng.pick(DEPARTMENTS);
            format!(
                "Access to {} {} is restricted to {} personnel only. Contact {} for authorization requests.",
                adj1, noun, adj2, dept
            )
        }
        3 => {
            let noun1 = rng.pick(NOUNS);
            let noun2 = rng.pick(NOUNS);
            let date = generate_fake_date(rng);
            let noun3 = rng.pick(NOUNS);
            let noun4 = rng.pick(NOUNS);
            format!(
                "The {} {} was last updated on {}. Review the {} for recent changes and {}.",
                noun1, noun2, date, noun3, noun4
            )
        }
        _ => {
            let noun1 = rng.pick(NOUNS);
            let verb1 = rng.pick(VERBS);
            let noun2 = rng.pick(NOUNS);
            let noun3 = rng.pick(NOUNS);
            let noun4 = rng.pick(NOUNS);
            let verb2 = rng.pick(VERBS);
            format!(
                "Use this {} to {} {} across all {}. The {} will be {} automatically.",
                noun1, verb1, noun2, noun3, noun4, verb2
            )
        }
    }
}

/// Check if a path is a maze entry point
pub fn is_maze_path(path: &str) -> bool {
    path.starts_with("/trap/") || path.starts_with("/maze/")
}

/// Handle a maze request and return an HTTP response
pub fn handle_maze_request(path: &str) -> http::Response {
    let config = MazeConfig::default();
    let html = generate_maze_page(path, &config);

    http::Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store, no-cache, must-revalidate")
        .header("X-Robots-Tag", "noindex, nofollow")
        .body(html)
        .build()
}

/// Generate a maze page HTML response
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_maze_path() {
        assert!(is_maze_path("/trap/abc123"));
        assert!(is_maze_path("/maze/def456"));
        assert!(!is_maze_path("/admin/config"));
        assert!(!is_maze_path("/api/data"));
    }

    #[test]
    fn test_deterministic_generation() {
        let config = MazeConfig::default();
        let page1 = generate_maze_page("/trap/test123", &config);
        let page2 = generate_maze_page("/trap/test123", &config);
        assert_eq!(page1, page2, "Same path should generate identical pages");
    }

    #[test]
    fn test_different_paths_different_pages() {
        let config = MazeConfig::default();
        let page1 = generate_maze_page("/trap/path1", &config);
        let page2 = generate_maze_page("/trap/path2", &config);
        assert_ne!(
            page1, page2,
            "Different paths should generate different pages"
        );
    }

    #[test]
    fn test_page_contains_links() {
        let config = MazeConfig::default();
        let page = generate_maze_page("/trap/test", &config);
        assert!(
            page.contains("nav-card"),
            "Page should contain navigation cards"
        );
        assert!(
            page.contains("href=\"/trap/"),
            "Page should contain trap links"
        );
    }

    #[test]
    fn test_maze_links_stay_in_maze() {
        let config = MazeConfig::default();
        let page = generate_maze_page("/maze/entry", &config);
        assert!(
            page.contains("href=\"/maze/"),
            "Maze pages should link to maze paths"
        );
        assert!(
            !page.contains("href=\"/trap/"),
            "Maze pages should not link to trap paths"
        );
    }

    #[test]
    fn test_seeded_rng_deterministic() {
        let mut rng1 = SeededRng::new(12345);
        let mut rng2 = SeededRng::new(12345);

        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let mut rng1 = SeededRng::new(11111);
        let mut rng2 = SeededRng::new(22222);

        // Very unlikely to match
        assert_ne!(rng1.next(), rng2.next());
    }
}
