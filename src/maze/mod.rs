pub(crate) mod assets;
#[cfg(test)]
mod benchmark;
mod content;
pub(crate) mod covert_decoy;
mod http;
pub(crate) mod preview;
mod renders;
mod rng;
pub(crate) mod runtime;
pub(crate) mod seeds;
#[cfg(test)]
mod simulation;
pub(crate) mod state;
mod token;
mod types;

pub use http::{handle_maze_request, is_maze_path};
#[allow(dead_code)]
pub type MazeConfig = types::MazeConfig;

#[allow(dead_code)]
pub fn generate_maze_page(path: &str, config: &MazeConfig) -> String {
    renders::generate_maze_page(path, config)
}

#[cfg(test)]
mod tests {
    use super::rng::SeededRng;
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
