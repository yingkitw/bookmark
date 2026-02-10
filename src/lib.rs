//! Bookmark Manager - Import, search, organize, and generate knowledge graphs from browser bookmarks
//!
//! # Features
//!
//! - Multi-browser support: Chrome, Firefox, Safari, Edge
//! - Search and open bookmarks
//! - Export bookmarks and history
//! - Remove duplicates and organize into folders
//! - Generate knowledge graphs (DOT, JSON, GEXF)
//!
//! # Library Usage
//!
//! ```rust,no_run
//! use bookmark::BookmarkManager;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = BookmarkManager::new();
//!
//!     // Export bookmarks
//!     let bookmarks = manager.export_bookmarks("chrome")?;
//!
//!     // Search
//!     let results = manager.search("github")?;
//!
//!     // Generate knowledge graph
//!     let graph = manager.graph_from_bookmarks(&bookmarks)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # CLI Usage
//!
//! ```bash
//! # Build with CLI feature (default)
//! cargo build --release
//!
//! # Export bookmarks
//! bookmark export --browser all
//!
//! # Search
//! bookmark search github
//!
//! # Generate graph
//! bookmark graph --format dot -o graph.dot
//! ```
//!
//! # MCP Server Usage
//!
//! ```bash
//! # Build with MCP feature
//! cargo build --features mcp
//!
//! # Run MCP server
//! bookmark-mcp
//! ```

pub mod browser;
pub mod config;
pub mod deduplication;
pub mod exporter;
pub mod graph;
pub mod graph_output;
pub mod organization;
pub mod processor;
pub mod search;
pub mod utils;

#[cfg(feature = "mcp")]
pub mod mcp;

use std::path::PathBuf;

/// Re-export commonly used types
pub use crate::exporter::{Bookmark, UrlEntry};
pub use crate::graph::{GraphConfig, GraphBuilder, KnowledgeGraph};

/// Main bookmark manager API
pub struct BookmarkManager {
    export_dir: Option<PathBuf>,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    pub fn new() -> Self {
        Self {
            export_dir: None,
        }
    }

    /// Set the default export directory
    pub fn with_export_dir(mut self, dir: PathBuf) -> Self {
        self.export_dir = Some(dir);
        self
    }

    /// Export bookmarks from a browser (reads live from browser databases)
    pub fn export_bookmarks(&self, browser: &str) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        let (bookmarks, _) = crate::exporter::load_browser_data(browser, "bookmarks")?;
        Ok(bookmarks)
    }

    /// Search bookmarks by query
    pub fn search(&self, query: &str) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        use crate::search::{search_bookmarks_internal, SearchOptions};

        let options = SearchOptions {
            title_only: false,
            url_only: false,
            limit: 100,
        };

        Ok(search_bookmarks_internal(query, &options)?)
    }

    /// Generate knowledge graph from bookmarks
    pub fn graph_from_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<KnowledgeGraph, Box<dyn std::error::Error>> {
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        Ok(builder.from_bookmarks(bookmarks)?)
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = BookmarkManager::new();
        assert!(manager.export_dir.is_none());
    }

    #[test]
    fn test_manager_with_export_dir() {
        let manager = BookmarkManager::new().with_export_dir(PathBuf::from("/tmp"));
        assert_eq!(manager.export_dir, Some(PathBuf::from("/tmp")));
    }
}
