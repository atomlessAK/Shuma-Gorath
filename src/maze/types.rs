/// Configuration for maze page generation.
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
