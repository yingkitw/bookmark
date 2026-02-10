//! Graph output handling for different formats

use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::graph;
use crate::utils;

/// Configuration for graph output
pub struct OutputConfig {
    /// Temp directory for data files
    pub temp_dir: PathBuf,
    /// HTML filename
    pub html_filename: String,
    /// Data filename
    pub data_filename: String,
}

impl OutputConfig {
    /// Create a new output config with timestamp-based filenames
    pub fn new() -> Self {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let temp_dir = std::env::temp_dir().join("bookmark-graph");
        fs::create_dir_all(&temp_dir).ok(); // Ignore errors, will handle later

        Self {
            temp_dir,
            html_filename: format!("graph_{}.html", timestamp),
            data_filename: format!("graph_{}.data.js", timestamp),
        }
    }

    /// Get the full path for the HTML file
    pub fn html_path(&self) -> PathBuf {
        self.temp_dir.join(&self.html_filename)
    }

    /// Get the full path for the data file
    pub fn data_path(&self) -> PathBuf {
        self.temp_dir.join(&self.data_filename)
    }
}

/// Write graph output for HTML format
pub fn write_html_output(
    graph: &graph::KnowledgeGraph,
    output: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let config = OutputConfig::new();
    let html_path = config.html_path();
    let data_path = config.data_path();

    // Write data file
    let js_content = graph::formats::to_js_data(graph);
    fs::write(&data_path, js_content)?;

    // Write HTML file (references data file by name only, for same-directory loading)
    let html_content = graph::formats::to_html_dynamic(&data_path);
    fs::write(&html_path, html_content)?;

    // Create redirect at requested output location if different from temp
    if output != &html_path {
        let target_url = html_path.display().to_string();
        utils::create_redirect_html(output, &target_url)?;
    }

    Ok((html_path, data_path))
}

/// Print summary of graph output
pub fn print_output_summary(html_path: &Path, data_path: &Path, graph: &graph::KnowledgeGraph) {
    println!("  Graph files created in temp directory:");
    println!("    HTML: {}", html_path.display());
    println!("    Data: {}", data_path.display());
    println!("  Opening {}", html_path.display());
    println!("  Note: Temp files can be safely deleted after viewing");
}
