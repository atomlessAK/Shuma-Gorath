//! robots.txt generation with AI crawler blocking and honeypot integration
//!
//! Generates configurable robots.txt that:
//! - Blocks known AI training crawlers
//! - Supports Cloudflare Content-Signal directive
//! - Seeds honeypot/maze paths for bad bots
//! - Allows legitimate search engine crawlers

use crate::config::Config;
use std::collections::HashSet;

/// Known AI training crawler user-agents
pub const AI_TRAINING_BOTS: &[&str] = &[
    "GPTBot",
    "ChatGPT-User",
    "CCBot",
    "Google-Extended",
    "Applebot-Extended",
    "anthropic-ai",
    "ClaudeBot",
    "Claude-Web",
    "Bytespider",
    "FacebookBot",
    "Meta-ExternalAgent",
    "Meta-ExternalFetcher",
    "Diffbot",
    "Omgilibot",
    "Omgili",
    "cohere-ai",
    "cohere-training-data-crawler",
    "Timpibot",
    "PanguBot",
    "Kangaroo Bot",
    "AI2Bot",
    "Ai2Bot-Dolma",
    "img2dataset",
];

/// AI search/assistant crawlers (real-time fetching)
pub const AI_SEARCH_BOTS: &[&str] = &[
    "PerplexityBot",
    "Perplexity-User",
    "YouBot",
    "OAI-SearchBot",
    "Claude-SearchBot",
    "DuckAssistBot",
    "Amazonbot",
];

/// Legitimate search engine crawlers to allow
pub const SEARCH_ENGINE_BOTS: &[&str] = &[
    "Googlebot",
    "Bingbot",
    "Slurp", // Yahoo
    "DuckDuckBot",
    "Baiduspider",
    "YandexBot",
    "facebot", // Facebook link preview (not training)
    "Twitterbot",
    "LinkedInBot",
];

/// Generate robots.txt content based on configuration
pub fn generate_robots_txt(cfg: &Config) -> String {
    let mut lines: Vec<String> = Vec::new();
    let trap_paths = collect_trap_paths(cfg);

    // Header comment with Content-Signal
    lines.push("# Bot Defence - Robots Exclusion Protocol".to_string());
    lines.push(format!(
        "# Generated dynamically - Policy: {}",
        get_policy_name(cfg)
    ));
    lines.push("#".to_string());

    // Add Content-Signal as comment (some crawlers may parse it)
    let ai_train = if cfg.robots_block_ai_training {
        "no"
    } else {
        "yes"
    };
    let ai_input = if cfg.robots_block_ai_search {
        "no"
    } else {
        "yes"
    };
    let search = if cfg.robots_allow_search_engines {
        "yes"
    } else {
        "no"
    };
    lines.push(format!(
        "# Content-Signal: ai-train={}, search={}, ai-input={}",
        ai_train, search, ai_input
    ));
    lines.push("#".to_string());
    lines.push("".to_string());

    // Block AI training bots
    if cfg.robots_block_ai_training {
        lines.push("# AI Training Crawlers - BLOCKED".to_string());
        for bot in AI_TRAINING_BOTS {
            lines.push(format!("User-agent: {}", bot));
            lines.push("Disallow: /".to_string());
            lines.push("".to_string());
        }
    }

    // Block AI search/assistant bots
    if cfg.robots_block_ai_search {
        lines.push("# AI Search/Assistant Crawlers - BLOCKED".to_string());
        for bot in AI_SEARCH_BOTS {
            lines.push(format!("User-agent: {}", bot));
            lines.push("Disallow: /".to_string());
            lines.push("".to_string());
        }
    }

    // Allow legitimate search engines with crawl delay
    if cfg.robots_allow_search_engines {
        lines.push("# Search Engine Crawlers - ALLOWED".to_string());
        for bot in SEARCH_ENGINE_BOTS {
            lines.push(format!("User-agent: {}", bot));
            lines.push("Allow: /".to_string());
            if cfg.robots_crawl_delay > 0 {
                lines.push(format!("Crawl-delay: {}", cfg.robots_crawl_delay));
            }
            // Tell good bots to stay out of trap paths.
            for trap_path in &trap_paths {
                lines.push(format!("Disallow: {}", trap_path));
            }
            lines.push("".to_string());
        }
    }

    // Default rule for all other bots
    lines.push("# Default rule for all other bots".to_string());
    lines.push("User-agent: *".to_string());
    if cfg.robots_allow_search_engines {
        lines.push("Allow: /".to_string());
        if cfg.robots_crawl_delay > 0 {
            lines.push(format!("Crawl-delay: {}", cfg.robots_crawl_delay));
        }
    } else {
        lines.push("Disallow: /".to_string());
    }

    // Add trap paths that bad bots might follow.
    if !trap_paths.is_empty() {
        lines.push("".to_string());
        lines.push("# Trap paths - Good bots stay out, bad bots get trapped".to_string());
        for trap_path in &trap_paths {
            lines.push(format!("Disallow: {}", trap_path));
        }

        // Add enticing honeypot links as comments
        // Bad bots often parse these looking for "hidden" paths
        lines.push("".to_string());
        lines.push("# Internal paths (do not crawl)".to_string());
        if cfg.maze_enabled {
            lines.push("# /maze/secret-admin/".to_string());
            lines.push("# /trap/internal-archive/".to_string());
        }
        if let Some(first_honeypot) = cfg.honeypots.first() {
            lines.push(format!("# {}", first_honeypot));
        }
    }

    // Sitemap reference (if applicable)
    lines.push("".to_string());
    lines.push("# Sitemap".to_string());
    lines.push("# Sitemap: https://example.com/sitemap.xml".to_string());

    lines.join("\n")
}

/// Get a human-readable policy name
fn get_policy_name(cfg: &Config) -> &'static str {
    match (
        cfg.robots_block_ai_training,
        cfg.robots_block_ai_search,
        cfg.robots_allow_search_engines,
    ) {
        (true, true, true) => "Search Only (Block All AI)",
        (true, true, false) => "Block Everything",
        (true, false, true) => "Block AI Training, Allow AI Search & Search Engines",
        (true, false, false) => "Block AI Training & Search Engines, Allow AI Search",
        (false, true, true) => "Allow AI Training, Block AI Search, Allow Search Engines",
        (false, true, false) => "Allow AI Training, Block AI Search & Search Engines",
        (false, false, true) => "Allow Everything",
        (false, false, false) => "Block Search Engines Only",
    }
}

fn collect_trap_paths(cfg: &Config) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    if cfg.maze_enabled {
        paths.push("/maze/".to_string());
        paths.push("/trap/".to_string());
    }
    paths.extend(
        cfg.honeypots
            .iter()
            .map(|path| path.trim())
            .filter(|path| !path.is_empty())
            .map(|path| path.to_string()),
    );

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for path in paths {
        if seen.insert(path.clone()) {
            deduped.push(path);
        }
    }
    deduped
}

/// Generate Content-Signal HTTP header value
pub fn get_content_signal_header(cfg: &Config) -> String {
    let ai_train = if cfg.robots_block_ai_training {
        "no"
    } else {
        "yes"
    };
    let ai_input = if cfg.robots_block_ai_search {
        "no"
    } else {
        "yes"
    };
    let search = if cfg.robots_allow_search_engines {
        "yes"
    } else {
        "no"
    };
    format!(
        "ai-train={}, search={}, ai-input={}",
        ai_train, search, ai_input
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BanDurations;

    fn test_config() -> Config {
        Config {
            ban_duration: 21600,
            ban_durations: BanDurations::default(),
            rate_limit: 80,
            honeypots: vec!["/instaban".to_string()],
            browser_block: vec![],
            browser_whitelist: vec![],
            geo_risk: vec![],
            geo_allow: vec![],
            geo_challenge: vec![],
            geo_maze: vec![],
            geo_block: vec![],
            whitelist: vec![],
            path_whitelist: vec![],
            test_mode: false,
            maze_enabled: true,
            maze_auto_ban: true,
            maze_auto_ban_threshold: 50,
            robots_enabled: true,
            robots_block_ai_training: true,
            robots_block_ai_search: false,
            robots_allow_search_engines: true,
            robots_crawl_delay: 2,
            cdp_detection_enabled: false,
            cdp_detection_threshold: 3.0,
            cdp_auto_ban: false,
            js_required_enforced: true,
            pow_enabled: true,
            pow_difficulty: crate::config::POW_DIFFICULTY_MIN,
            pow_ttl_seconds: crate::config::POW_TTL_MIN,
            challenge_transform_count: 6,
            challenge_risk_threshold: 3,
            botness_maze_threshold: 6,
            botness_weights: crate::config::BotnessWeights::default(),
            defence_modes: crate::config::DefenceModes::default(),
            provider_backends: crate::config::ProviderBackends::default(),
        }
    }

    #[test]
    fn test_generate_robots_txt_blocks_ai_training() {
        let cfg = test_config();
        let robots = generate_robots_txt(&cfg);

        // Should block GPTBot
        assert!(robots.contains("User-agent: GPTBot"));
        assert!(robots.contains("Disallow: /"));

        // Should allow Googlebot
        assert!(robots.contains("User-agent: Googlebot"));
        assert!(robots.contains("Allow: /"));
    }

    #[test]
    fn test_generate_robots_txt_includes_honeypot() {
        let cfg = test_config();
        let robots = generate_robots_txt(&cfg);

        // Should align with active trap routes and configured honeypots.
        assert!(robots.contains("Disallow: /maze/"));
        assert!(robots.contains("Disallow: /trap/"));
        assert!(robots.contains("Disallow: /instaban"));
        assert!(!robots.contains("/.well-known/maze/"));
    }

    #[test]
    fn test_content_signal_header() {
        let cfg = test_config();
        let header = get_content_signal_header(&cfg);

        assert!(header.contains("ai-train=no"));
        assert!(header.contains("search=yes"));
    }

    #[test]
    fn test_crawl_delay() {
        let cfg = test_config();
        let robots = generate_robots_txt(&cfg);

        assert!(robots.contains("Crawl-delay: 2"));
    }

    #[test]
    fn test_honeypot_path_still_advertised_when_maze_disabled() {
        let mut cfg = test_config();
        cfg.maze_enabled = false;
        let robots = generate_robots_txt(&cfg);

        assert!(robots.contains("Disallow: /instaban"));
        assert!(!robots.contains("Disallow: /maze/"));
        assert!(!robots.contains("Disallow: /trap/"));
    }
}
