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
