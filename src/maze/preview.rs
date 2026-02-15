use super::content::{
    capitalize, generate_link_text, generate_paragraph, generate_title, DEPARTMENTS, NOUNS,
};
use super::renders::{
    generate_polymorphic_maze_page, AdvancedMazeLink, AdvancedMazeRenderOptions, MazeStyleTier,
};
use super::rng::{generate_path_segment, SeededRng};
use super::types::MazeConfig;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_PREVIEW_PATH: &str = "/maze/preview";
const PREVIEW_SITE_ID: &str = "admin-preview";
const PREVIEW_IP_BUCKET: &str = "admin-preview-ip";
const PREVIEW_UA_BUCKET: &str = "admin-preview-ua";
const PREVIEW_CHAIN_NONCE: &str = "admin-preview";

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn preview_secret_from_env() -> String {
    std::env::var("SHUMA_MAZE_PREVIEW_SECRET")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("preview::{}", super::token::secret_from_env()))
}

fn is_safe_preview_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 256
        && path
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '~'))
}

pub(crate) fn normalize_preview_path(requested_path: Option<&str>) -> String {
    let candidate = requested_path.unwrap_or(DEFAULT_PREVIEW_PATH).trim();
    if candidate.is_empty() {
        return DEFAULT_PREVIEW_PATH.to_string();
    }

    let path_only = candidate.split('?').next().unwrap_or(DEFAULT_PREVIEW_PATH);
    if !super::is_maze_path(path_only) || !is_safe_preview_path(path_only) {
        return DEFAULT_PREVIEW_PATH.to_string();
    }

    path_only.to_string()
}

fn preview_href(path: &str) -> String {
    let encoded = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
    format!("/admin/maze/preview?path={}", encoded)
}

fn preview_breadcrumb(rng: &mut SeededRng) -> String {
    let dept = rng.pick(DEPARTMENTS);
    let noun = capitalize(rng.pick(NOUNS));
    format!("Portal > {} > {} Operations", dept, noun)
}

pub(crate) fn render_admin_preview(
    cfg: &crate::config::Config,
    requested_path: Option<&str>,
) -> String {
    let current_path = normalize_preview_path(requested_path);
    let path_prefix = if current_path.starts_with("/trap/") {
        "/trap/"
    } else {
        "/maze/"
    };

    let now = now_secs();
    let entropy_bucket = now / cfg.maze_entropy_window_seconds.max(1);
    let seed = super::token::entropy_seed(
        preview_secret_from_env().as_str(),
        PREVIEW_SITE_ID,
        PREVIEW_IP_BUCKET,
        PREVIEW_UA_BUCKET,
        current_path.as_str(),
        entropy_bucket,
        PREVIEW_CHAIN_NONCE,
    );
    let mut rng = SeededRng::new(seed);

    let render_cfg = MazeConfig::default();
    let paragraph_count = rng
        .range(render_cfg.min_paragraphs, render_cfg.max_paragraphs)
        .min(cfg.maze_max_paragraphs.max(1) as usize)
        .max(1);
    let seed_focus = capitalize(NOUNS[(seed as usize) % NOUNS.len()]);
    let mut paragraphs = Vec::with_capacity(paragraph_count);
    for index in 0..paragraph_count {
        let mut paragraph = generate_paragraph(&mut rng);
        if index == 0 {
            paragraph.push_str(format!(" Priority stream: {}.", seed_focus).as_str());
        }
        paragraphs.push(paragraph);
    }

    let link_count = rng
        .range(render_cfg.min_links, render_cfg.max_links)
        .min(cfg.maze_max_links.max(1) as usize)
        .max(1);
    let segment_len = cfg.maze_path_entropy_segment_len.max(8) as usize;

    let mut links = Vec::with_capacity(link_count);
    for _ in 0..link_count {
        let next_path = format!(
            "{}{}",
            path_prefix,
            generate_path_segment(&mut rng, segment_len)
        );
        let topical_suffix = if rng.next() % 2 == 0 {
            Some(capitalize(rng.pick(NOUNS)))
        } else {
            None
        };
        let link_text = if let Some(term) = topical_suffix.as_deref() {
            format!("{} {}", generate_link_text(&mut rng), term)
        } else {
            generate_link_text(&mut rng)
        };
        let link_description = if let Some(term) = topical_suffix.as_deref() {
            format!(
                "{} Coordination lane: {}.",
                generate_paragraph(&mut rng),
                term
            )
        } else {
            generate_paragraph(&mut rng)
        };
        links.push(AdvancedMazeLink {
            href: preview_href(next_path.as_str()),
            text: link_text,
            description: link_description,
            pow_difficulty: None,
        });
    }

    let bootstrap_json = serde_json::json!({
        "flow_id": "maze-preview",
        "depth": 0,
        "checkpoint_token": "",
        "path_prefix": path_prefix,
        "entropy_nonce": "preview",
        "assets": {
            "worker_url": super::assets::MAZE_WORKER_PATH
        },
        "client_expansion": {
            "enabled": false,
            "seed": seed,
            "seed_sig": "",
            "hidden_count": 0,
            "segment_len": segment_len,
            "issue_path": "/maze/issue-links"
        }
    })
    .to_string();

    let variant_layout = (seed & 0xff) as u8 % 3;
    let variant_palette = ((seed >> 8) & 0xff) as u8 % 3;
    let options = AdvancedMazeRenderOptions {
        title: generate_title(&mut rng),
        breadcrumb: preview_breadcrumb(&mut rng),
        paragraphs,
        links,
        bootstrap_json,
        variant_layout,
        variant_palette,
        style_tier: MazeStyleTier::Full,
        style_sheet_url: Some(super::assets::MAZE_STYLE_PATH.to_string()),
        script_url: super::assets::MAZE_SCRIPT_PATH.to_string(),
    };

    generate_polymorphic_maze_page(&options)
}

#[cfg(test)]
mod tests {
    use super::{normalize_preview_path, preview_secret_from_env, render_admin_preview};

    #[test]
    fn normalize_preview_path_rejects_invalid_input() {
        assert_eq!(
            normalize_preview_path(Some("/admin/config")),
            "/maze/preview".to_string()
        );
        assert_eq!(
            normalize_preview_path(Some("/maze/<script>")),
            "/maze/preview".to_string()
        );
        assert_eq!(normalize_preview_path(None), "/maze/preview".to_string());
    }

    #[test]
    fn preview_secret_uses_dedicated_namespace_when_unset() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_MAZE_SECRET", "live-secret");
        std::env::remove_var("SHUMA_MAZE_PREVIEW_SECRET");
        let preview_secret = preview_secret_from_env();
        assert_eq!(preview_secret, "preview::live-secret");
        std::env::remove_var("SHUMA_MAZE_SECRET");
    }

    #[test]
    fn preview_render_is_non_operational() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_DEBUG_HEADERS", "true");
        let cfg = crate::config::defaults().clone();
        let html = render_admin_preview(&cfg, Some("/maze/preview-segment"));
        assert!(!html.contains("Maze Preview"));
        assert!(!html.contains("Preview-only path."));
        assert!(html.contains("/admin/maze/preview?path="));
        assert!(!html.contains("Variant maze-v"));
        assert!(!html.contains("Synthetic navigation surface. Not authoritative content."));
        assert!(!html.contains("mt="));
        assert!(!html.contains("data-shuma-covert-decoy"));
    }
}
