mod analyzer;
mod builder;
pub mod formats;
#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export public API
pub use builder::GraphBuilder;

/// Node types in the knowledge graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Bookmark,
    Domain,
    Folder,
    Tag,
    Category,
}

/// Edge types representing different relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeType {
    BelongsToDomain,
    InFolder,
    SameDomain,
    HasTag,
    InCategory,
    SimilarContent,
}

/// Metadata for a graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub date_added: Option<DateTime<Utc>>,
    pub visit_count: Option<i64>,
    pub bookmark_count: usize,
}

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub node_type: NodeType,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub folder: Option<String>,
    pub size: usize,
}

/// An edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: f64,
}

/// Metadata for the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub bookmark_count: usize,
    pub domain_count: usize,
    pub folder_count: usize,
    pub generated_at: DateTime<Utc>,
}

/// The main knowledge graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

/// Detail level for graph generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailLevel {
    /// Overview: Only domains and folders, no individual bookmarks
    Overview,
    /// Standard: Domains, folders, and top N bookmarks per domain
    Standard,
    /// Detailed: Full graph with all bookmarks
    Detailed,
}

/// Configuration for graph generation
#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub include_domain_edges: bool,
    pub include_folder_edges: bool,
    pub include_same_domain_edges: bool,
    pub include_tag_edges: bool,
    pub include_category_edges: bool,
    pub include_similarity_edges: bool,
    pub min_domain_threshold: usize,
    pub min_tag_threshold: usize,
    pub similarity_threshold: f64,
    /// Level of detail for the graph
    pub detail_level: DetailLevel,
    /// Maximum number of bookmarks to show per domain (None = all)
    pub max_bookmarks_per_domain: Option<usize>,
    /// Maximum total bookmarks (None = no limit)
    pub max_total_bookmarks: Option<usize>,
    /// Only include bookmarks newer than this date (None = all time)
    pub min_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Domain-only mode (collapse all bookmarks into domains)
    pub domain_only: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            include_domain_edges: true,
            include_folder_edges: true,
            include_same_domain_edges: false,
            include_tag_edges: false,
            include_category_edges: true,
            include_similarity_edges: false,
            min_domain_threshold: 5,
            min_tag_threshold: 3,
            similarity_threshold: 0.3,
            detail_level: DetailLevel::Standard,
            max_bookmarks_per_domain: Some(10),
            max_total_bookmarks: Some(5000),
            min_date: None,
            domain_only: false,
        }
    }
}
