use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::exporter::{Bookmark, UrlEntry};

/// Node types in the knowledge graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Bookmark,
    Domain,
    Folder,
}

/// Edge types representing different relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeType {
    BelongsToDomain,
    InFolder,
    SameDomain,
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

/// Configuration for graph generation
#[derive(Debug, Clone)]
pub struct GraphConfig {
    pub include_domain_edges: bool,
    pub include_folder_edges: bool,
    pub include_same_domain_edges: bool,
    pub min_domain_threshold: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            include_domain_edges: true,
            include_folder_edges: true,
            include_same_domain_edges: true,
            min_domain_threshold: 2,
        }
    }
}

/// Builder for creating knowledge graphs
pub struct GraphBuilder {
    config: GraphConfig,
    domain_counts: HashMap<String, usize>,
    folder_counts: HashMap<String, usize>,
    domain_to_bookmarks: HashMap<String, Vec<String>>,
    folder_to_bookmarks: HashMap<String, Vec<String>>,
}

impl GraphBuilder {
    pub fn new(config: GraphConfig) -> Self {
        Self {
            config,
            domain_counts: HashMap::new(),
            folder_counts: HashMap::new(),
            domain_to_bookmarks: HashMap::new(),
            folder_to_bookmarks: HashMap::new(),
        }
    }

    /// Build a graph from bookmarks
    pub fn from_bookmarks(&mut self, bookmarks: &[Bookmark]) -> Result<KnowledgeGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Create bookmark nodes and collect statistics
        let mut bookmark_ids = Vec::new();
        for bookmark in bookmarks {
            let bookmark_id = bookmark.id.clone();
            bookmark_ids.push(bookmark_id.clone());

            let domain = bookmark.url.as_ref().and_then(|u| self.extract_domain(u));
            let folder = bookmark.folder.clone();

            // Track domain counts
            if let Some(ref d) = domain {
                *self.domain_counts.entry(d.clone()).or_insert(0) += 1;
                self.domain_to_bookmarks
                    .entry(d.clone())
                    .or_insert_with(Vec::new)
                    .push(bookmark_id.clone());
            }

            // Track folder counts
            if let Some(ref f) = folder {
                *self.folder_counts.entry(f.clone()).or_insert(0) += 1;
                self.folder_to_bookmarks
                    .entry(f.clone())
                    .or_insert_with(Vec::new)
                    .push(bookmark_id.clone());
            }

            nodes.push(GraphNode {
                id: bookmark_id.clone(),
                title: bookmark.title.clone(),
                node_type: NodeType::Bookmark,
                url: bookmark.url.clone(),
                domain: domain.clone(),
                folder: folder.clone(),
                size: 1,
            });
        }

        // Create domain nodes
        let domain_nodes = self.create_domain_nodes();
        let domain_count = domain_nodes.len();
        nodes.extend(domain_nodes);

        // Create folder nodes
        let folder_nodes = self.create_folder_nodes();
        let folder_count = folder_nodes.len();
        nodes.extend(folder_nodes);

        // Create edges
        if self.config.include_domain_edges {
            self.create_domain_edges(&mut edges);
        }
        if self.config.include_folder_edges {
            self.create_folder_edges(&mut edges);
        }
        if self.config.include_same_domain_edges {
            self.create_same_domain_edges(&mut edges);
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            bookmark_count: bookmark_ids.len(),
            domain_count,
            folder_count,
            generated_at: Utc::now(),
        };

        Ok(KnowledgeGraph {
            nodes,
            edges,
            metadata,
        })
    }

    /// Build a graph from history entries
    pub fn from_history(&mut self, history: &[UrlEntry]) -> Result<KnowledgeGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Convert history entries to bookmark-like nodes
        let mut bookmark_ids = Vec::new();
        for (i, entry) in history.iter().enumerate() {
            let bookmark_id = format!("hist_{}", i);
            bookmark_ids.push(bookmark_id.clone());

            let domain = self.extract_domain(&entry.url);

            // Track domain counts
            if let Some(ref d) = domain {
                *self.domain_counts.entry(d.clone()).or_insert(0) += 1;
                self.domain_to_bookmarks
                    .entry(d.clone())
                    .or_insert_with(Vec::new)
                    .push(bookmark_id.clone());
            }

            nodes.push(GraphNode {
                id: bookmark_id.clone(),
                title: entry.title.clone(),
                node_type: NodeType::Bookmark,
                url: Some(entry.url.clone()),
                domain: domain.clone(),
                folder: None,
                size: entry.visit_count as usize,
            });
        }

        // Create domain nodes
        let domain_nodes = self.create_domain_nodes();
        let domain_count = domain_nodes.len();
        nodes.extend(domain_nodes);

        // Create edges
        if self.config.include_domain_edges {
            self.create_domain_edges(&mut edges);
        }
        if self.config.include_same_domain_edges {
            self.create_same_domain_edges(&mut edges);
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            bookmark_count: bookmark_ids.len(),
            domain_count,
            folder_count: 0,
            generated_at: Utc::now(),
        };

        Ok(KnowledgeGraph {
            nodes,
            edges,
            metadata,
        })
    }

    /// Build a graph from both bookmarks and history
    pub fn from_both(
        &mut self,
        bookmarks: &[Bookmark],
        history: &[UrlEntry],
    ) -> Result<KnowledgeGraph> {
        // First process bookmarks
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let mut bookmark_ids = Vec::new();
        for bookmark in bookmarks {
            let bookmark_id = bookmark.id.clone();
            bookmark_ids.push(bookmark_id.clone());

            let domain = bookmark.url.as_ref().and_then(|u| self.extract_domain(u));
            let folder = bookmark.folder.clone();

            if let Some(ref d) = domain {
                *self.domain_counts.entry(d.clone()).or_insert(0) += 1;
                self.domain_to_bookmarks
                    .entry(d.clone())
                    .or_insert_with(Vec::new)
                    .push(bookmark_id.clone());
            }

            if let Some(ref f) = folder {
                *self.folder_counts.entry(f.clone()).or_insert(0) += 1;
                self.folder_to_bookmarks
                    .entry(f.clone())
                    .or_insert_with(Vec::new)
                    .push(bookmark_id.clone());
            }

            nodes.push(GraphNode {
                id: bookmark_id.clone(),
                title: bookmark.title.clone(),
                node_type: NodeType::Bookmark,
                url: bookmark.url.clone(),
                domain: domain.clone(),
                folder: folder.clone(),
                size: 1,
            });
        }

        // Then process history
        for (i, entry) in history.iter().enumerate() {
            let bookmark_id = format!("hist_{}", i);
            bookmark_ids.push(bookmark_id.clone());

            let domain = self.extract_domain(&entry.url);

            if let Some(ref d) = domain {
                *self.domain_counts.entry(d.clone()).or_insert(0) += 1;
                self.domain_to_bookmarks
                    .entry(d.clone())
                    .or_insert_with(Vec::new)
                    .push(bookmark_id.clone());
            }

            nodes.push(GraphNode {
                id: bookmark_id.clone(),
                title: entry.title.clone(),
                node_type: NodeType::Bookmark,
                url: Some(entry.url.clone()),
                domain: domain.clone(),
                folder: None,
                size: entry.visit_count as usize,
            });
        }

        // Create domain nodes
        let domain_nodes = self.create_domain_nodes();
        let domain_count = domain_nodes.len();
        nodes.extend(domain_nodes);

        // Create folder nodes
        let folder_nodes = self.create_folder_nodes();
        let folder_count = folder_nodes.len();
        nodes.extend(folder_nodes);

        // Create edges
        if self.config.include_domain_edges {
            self.create_domain_edges(&mut edges);
        }
        if self.config.include_folder_edges {
            self.create_folder_edges(&mut edges);
        }
        if self.config.include_same_domain_edges {
            self.create_same_domain_edges(&mut edges);
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            bookmark_count: bookmark_ids.len(),
            domain_count,
            folder_count,
            generated_at: Utc::now(),
        };

        Ok(KnowledgeGraph {
            nodes,
            edges,
            metadata,
        })
    }

    fn create_domain_nodes(&self) -> Vec<GraphNode> {
        let mut nodes = Vec::new();
        for (domain, &count) in &self.domain_counts {
            if count >= self.config.min_domain_threshold {
                nodes.push(GraphNode {
                    id: format!("domain_{}", domain),
                    title: domain.clone(),
                    node_type: NodeType::Domain,
                    url: None,
                    domain: Some(domain.clone()),
                    folder: None,
                    size: count,
                });
            }
        }
        nodes
    }

    fn create_folder_nodes(&self) -> Vec<GraphNode> {
        let mut nodes = Vec::new();
        for (folder, &count) in &self.folder_counts {
            nodes.push(GraphNode {
                id: format!("folder_{}", folder.replace('/', "_")),
                title: folder.clone(),
                node_type: NodeType::Folder,
                url: None,
                domain: None,
                folder: Some(folder.clone()),
                size: count,
            });
        }
        nodes
    }

    fn create_domain_edges(&self, edges: &mut Vec<GraphEdge>) {
        for (domain, bookmark_ids) in &self.domain_to_bookmarks {
            if *self.domain_counts.get(domain).unwrap_or(&0) >= self.config.min_domain_threshold {
                let domain_id = format!("domain_{}", domain);
                for bookmark_id in bookmark_ids {
                    edges.push(GraphEdge {
                        source: bookmark_id.clone(),
                        target: domain_id.clone(),
                        edge_type: EdgeType::BelongsToDomain,
                        weight: 1.0,
                    });
                }
            }
        }
    }

    fn create_folder_edges(&self, edges: &mut Vec<GraphEdge>) {
        for (folder, bookmark_ids) in &self.folder_to_bookmarks {
            let folder_id = format!("folder_{}", folder.replace('/', "_"));
            for bookmark_id in bookmark_ids {
                edges.push(GraphEdge {
                    source: bookmark_id.clone(),
                    target: folder_id.clone(),
                    edge_type: EdgeType::InFolder,
                    weight: 1.0,
                });
            }
        }
    }

    fn create_same_domain_edges(&self, edges: &mut Vec<GraphEdge>) {
        for bookmark_ids in self.domain_to_bookmarks.values() {
            if bookmark_ids.len() > 1 {
                for i in 0..bookmark_ids.len() {
                    for j in (i + 1)..bookmark_ids.len() {
                        edges.push(GraphEdge {
                            source: bookmark_ids[i].clone(),
                            target: bookmark_ids[j].clone(),
                            edge_type: EdgeType::SameDomain,
                            weight: 0.5,
                        });
                    }
                }
            }
        }
    }

    fn extract_domain(&self, url: &str) -> Option<String> {
        match url::Url::parse(url) {
            Ok(parsed) => {
                let host = parsed.host_str()?;
                // Remove www. prefix if present
                Some(host.strip_prefix("www.").unwrap_or(host).to_string())
            }
            Err(_) => None,
        }
    }
}

/// Format exporters
pub mod formats {
    use super::*;

    /// Export graph to DOT format (Graphviz)
    pub fn to_dot(graph: &KnowledgeGraph) -> String {
        let mut dot = String::from("digraph BookmarkKnowledgeGraph {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box];\n\n");

        // Define nodes with styling based on type
        for node in &graph.nodes {
            let (color, shape) = match node.node_type {
                NodeType::Bookmark => ("lightblue", "box"),
                NodeType::Domain => ("lightgreen", "ellipse"),
                NodeType::Folder => ("lightyellow", "folder"),
            };
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\", fillcolor={}, style=filled, shape={}];\n",
                escape_dot_id(&node.id),
                escape_dot_label(&node.title),
                color,
                shape
            ));
        }

        dot.push_str("\n");

        // Define edges with styling
        for edge in &graph.edges {
            let style = match edge.edge_type {
                EdgeType::BelongsToDomain => "[color=blue, penwidth=2]",
                EdgeType::InFolder => "[color=green, penwidth=1]",
                EdgeType::SameDomain => "[color=gray, penwidth=0.5, style=dashed]",
            };
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" {};\n",
                escape_dot_id(&edge.source),
                escape_dot_id(&edge.target),
                style
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Export graph to JSON format
    pub fn to_json(graph: &KnowledgeGraph) -> String {
        #[derive(Serialize)]
        struct JsonGraph {
            nodes: Vec<JsonNode>,
            edges: Vec<JsonEdge>,
            metadata: JsonMetadata,
        }

        #[derive(Serialize)]
        struct JsonNode {
            id: String,
            title: String,
            node_type: String,
            url: Option<String>,
            domain: Option<String>,
            folder: Option<String>,
            size: usize,
        }

        #[derive(Serialize)]
        struct JsonEdge {
            source: String,
            target: String,
            edge_type: String,
            weight: f64,
        }

        #[derive(Serialize)]
        struct JsonMetadata {
            total_nodes: usize,
            total_edges: usize,
            bookmark_count: usize,
            domain_count: usize,
            folder_count: usize,
            generated_at: DateTime<Utc>,
        }

        let json_nodes: Vec<JsonNode> = graph
            .nodes
            .iter()
            .map(|n| JsonNode {
                id: n.id.clone(),
                title: n.title.clone(),
                node_type: format!("{:?}", n.node_type).to_lowercase(),
                url: n.url.clone(),
                domain: n.domain.clone(),
                folder: n.folder.clone(),
                size: n.size,
            })
            .collect();

        let json_edges: Vec<JsonEdge> = graph
            .edges
            .iter()
            .map(|e| JsonEdge {
                source: e.source.clone(),
                target: e.target.clone(),
                edge_type: format!("{:?}", e.edge_type).to_lowercase(),
                weight: e.weight,
            })
            .collect();

        let json_graph = JsonGraph {
            nodes: json_nodes,
            edges: json_edges,
            metadata: JsonMetadata {
                total_nodes: graph.metadata.total_nodes,
                total_edges: graph.metadata.total_edges,
                bookmark_count: graph.metadata.bookmark_count,
                domain_count: graph.metadata.domain_count,
                folder_count: graph.metadata.folder_count,
                generated_at: graph.metadata.generated_at,
            },
        };

        serde_json::to_string_pretty(&json_graph).unwrap_or_default()
    }

    /// Export graph to GEXF format (Gephi)
    pub fn to_gexf(graph: &KnowledgeGraph) -> String {
        let mut gexf = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gexf xmlns="http://www.gexf.net/1.2draft" version="1.2">
    <graph mode="static" defaultedgetype="directed">
"#);

        // Node attributes definition
        gexf.push_str(r#"        <attributes class="node">
            <attribute id="0" title="node_type" type="string"/>
            <attribute id="1" title="url" type="string"/>
            <attribute id="2" title="domain" type="string"/>
            <attribute id="3" title="folder" type="string"/>
        </attributes>
"#);

        // Nodes
        gexf.push_str("        <nodes>\n");
        for node in &graph.nodes {
            let node_type_str = format!("{:?}", node.node_type).to_lowercase();
            gexf.push_str(&format!(
                r#"            <node id="{}" label="{}">
                <attvalues>
                    <attvalue for="0" value="{}"/>"#,
                escape_xml(&node.id),
                escape_xml(&node.title),
                escape_xml(&node_type_str)
            ));

            if let Some(ref url) = node.url {
                gexf.push_str(&format!(
                    r#"
                    <attvalue for="1" value="{}"/>"#,
                    escape_xml(url)
                ));
            }

            if let Some(ref domain) = node.domain {
                gexf.push_str(&format!(
                    r#"
                    <attvalue for="2" value="{}"/>"#,
                    escape_xml(domain)
                ));
            }

            if let Some(ref folder) = node.folder {
                gexf.push_str(&format!(
                    r#"
                    <attvalue for="3" value="{}"/>"#,
                    escape_xml(folder)
                ));
            }

            gexf.push_str(r#"
                </attvalues>
            </node>"#);
            gexf.push('\n');
        }
        gexf.push_str("        </nodes>\n");

        // Edges
        gexf.push_str("        <edges>\n");
        for (i, edge) in graph.edges.iter().enumerate() {
            let edge_type_str = format!("{:?}", edge.edge_type).to_lowercase();
            gexf.push_str(&format!(
                r#"            <edge id="{}" source="{}" target="{}" weight="{}" label="{}"/>"#,
                i,
                escape_xml(&edge.source),
                escape_xml(&edge.target),
                edge.weight,
                escape_xml(&edge_type_str)
            ));
            gexf.push('\n');
        }
        gexf.push_str("        </edges>\n");

        gexf.push_str("    </graph>\n</gexf>");
        gexf
    }

    fn escape_dot_id(s: &str) -> String {
        s.replace('"', "_")
            .replace('\\', "_")
            .replace(|c: char| c.is_whitespace(), "_")
    }

    fn escape_dot_label(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('|', "\\|")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('<', "\\<")
            .replace('>', "\\>")
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_bookmarks() -> Vec<Bookmark> {
        vec![
            Bookmark {
                id: "1".to_string(),
                title: "GitHub Home".to_string(),
                url: Some("https://github.com".to_string()),
                folder: Some("Development".to_string()),
                date_added: Some(Utc::now()),
                children: None,
            },
            Bookmark {
                id: "2".to_string(),
                title: "GitHub Repo".to_string(),
                url: Some("https://github.com/user/repo".to_string()),
                folder: Some("Development".to_string()),
                date_added: Some(Utc::now()),
                children: None,
            },
            Bookmark {
                id: "3".to_string(),
                title: "Rust Docs".to_string(),
                url: Some("https://doc.rust-lang.org".to_string()),
                folder: Some("Development".to_string()),
                date_added: Some(Utc::now()),
                children: None,
            },
            Bookmark {
                id: "4".to_string(),
                title: "Amazon".to_string(),
                url: Some("https://www.amazon.com".to_string()),
                folder: Some("Shopping".to_string()),
                date_added: Some(Utc::now()),
                children: None,
            },
        ]
    }

    fn create_test_history() -> Vec<UrlEntry> {
        vec![
            UrlEntry {
                url: "https://github.com".to_string(),
                title: "GitHub".to_string(),
                visit_count: 10,
                last_visit: Some(Utc::now()),
            },
            UrlEntry {
                url: "https://www.reddit.com".to_string(),
                title: "Reddit".to_string(),
                visit_count: 5,
                last_visit: Some(Utc::now()),
            },
        ]
    }

    #[test]
    fn test_graph_from_bookmarks() {
        let bookmarks = create_test_bookmarks();
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        // Should have 4 bookmark nodes + domain nodes + folder nodes
        assert!(graph.nodes.len() >= 4);
        assert_eq!(graph.metadata.bookmark_count, 4);

        // Check that we have domain nodes (only github.com with 2+ bookmarks meets threshold)
        let domain_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Domain)
            .collect();
        assert_eq!(domain_nodes.len(), 1); // Only github.com has 2 bookmarks

        // Check that we have folder nodes (Development, Shopping)
        let folder_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Folder)
            .collect();
        assert_eq!(folder_nodes.len(), 2);
    }

    #[test]
    fn test_graph_from_history() {
        let history = create_test_history();
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_history(&history).unwrap();

        // Should have 2 history nodes
        assert_eq!(graph.metadata.bookmark_count, 2);
        assert_eq!(graph.metadata.folder_count, 0); // History has no folders

        // With default threshold of 2, no domain nodes are created (each domain has only 1 entry)
        let domain_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Domain)
            .collect();
        assert_eq!(domain_nodes.len(), 0);
    }

    #[test]
    fn test_domain_threshold() {
        let bookmarks = create_test_bookmarks();
        let config = GraphConfig {
            min_domain_threshold: 3, // Require 3+ bookmarks per domain
            ..Default::default()
        };
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        // github.com has 2 bookmarks, should not create a domain node with threshold of 3
        let github_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Domain && n.domain == Some("github.com".to_string()))
            .collect();
        assert_eq!(github_nodes.len(), 0);
    }

    #[test]
    fn test_edge_creation() {
        let bookmarks = create_test_bookmarks();
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        // Should have edges
        assert!(graph.edges.len() > 0);

        // Check domain edges
        let domain_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::BelongsToDomain)
            .collect();
        assert!(domain_edges.len() > 0);

        // Check folder edges
        let folder_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::InFolder)
            .collect();
        assert!(folder_edges.len() > 0);
    }

    #[test]
    fn test_dot_export() {
        let bookmarks = create_test_bookmarks();
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        let dot = formats::to_dot(&graph);

        // Check DOT format structure
        assert!(dot.contains("digraph BookmarkKnowledgeGraph"));
        assert!(dot.contains("rankdir=LR"));
        assert!(dot.contains("fillcolor")); // Should have colors

        // Check for nodes
        assert!(dot.contains("node"));
        assert!(dot.contains("->")); // Should have edges
    }

    #[test]
    fn test_json_export() {
        let bookmarks = create_test_bookmarks();
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        let json = formats::to_json(&graph);

        // Check JSON structure
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
        assert!(json.contains("\"metadata\""));

        // Parse and validate
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["nodes"].is_array());
        assert!(parsed["edges"].is_array());
        assert!(parsed["metadata"]["total_nodes"].is_number());
    }

    #[test]
    fn test_gexf_export() {
        let bookmarks = create_test_bookmarks();
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        let gexf = formats::to_gexf(&graph);

        // Check GEXF format structure
        assert!(gexf.contains("<?xml version=\"1.0\""));
        assert!(gexf.contains("<gexf"));
        assert!(gexf.contains("<nodes>"));
        assert!(gexf.contains("<edges>"));
        assert!(gexf.contains("</gexf>"));
    }

    #[test]
    fn test_edge_type_toggles() {
        let bookmarks = create_test_bookmarks();

        // Test with only domain edges
        let config = GraphConfig {
            include_folder_edges: false,
            include_domain_edges: true,
            include_same_domain_edges: false,
            ..Default::default()
        };
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        // Should only have domain edges
        assert!(graph.edges.iter().all(|e| e.edge_type == EdgeType::BelongsToDomain));
    }

    #[test]
    fn test_empty_bookmarks() {
        let bookmarks: Vec<Bookmark> = vec![];
        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.metadata.bookmark_count, 0);
    }

    #[test]
    fn test_bookmark_without_url() {
        let bookmarks = vec![Bookmark {
            id: "1".to_string(),
            title: "No URL Bookmark".to_string(),
            url: None,
            folder: Some("Misc".to_string()),
            date_added: Some(Utc::now()),
            children: None,
        }];

        let config = GraphConfig::default();
        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_bookmarks(&bookmarks).unwrap();

        // Should still create the bookmark node
        assert_eq!(graph.metadata.bookmark_count, 1);

        // Should not create domain node (no URL)
        let domain_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Domain)
            .collect();
        assert_eq!(domain_nodes.len(), 0);

        // Should still create folder node
        let folder_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Folder)
            .collect();
        assert_eq!(folder_nodes.len(), 1);
    }

    #[test]
    fn test_extract_domain() {
        let config = GraphConfig::default();
        let builder = GraphBuilder::new(config);

        assert_eq!(builder.extract_domain("https://github.com"), Some("github.com".to_string()));
        assert_eq!(builder.extract_domain("https://www.github.com"), Some("github.com".to_string()));
        assert_eq!(builder.extract_domain("https://doc.rust-lang.org"), Some("doc.rust-lang.org".to_string()));
        assert_eq!(builder.extract_domain("not-a-url"), None);
    }
}
