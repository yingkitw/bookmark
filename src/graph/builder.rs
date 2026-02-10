use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};

use super::analyzer;
use super::{
    DetailLevel, EdgeType, GraphConfig, GraphEdge, GraphMetadata, GraphNode, KnowledgeGraph,
    NodeType,
};
use crate::exporter::{Bookmark, UrlEntry};

/// A single item to ingest into the graph (unified representation)
struct IngestItem<'a> {
    id: String,
    title: &'a str,
    url: Option<&'a str>,
    folder: Option<&'a str>,
    size: usize,
}

/// Builder for creating knowledge graphs
pub struct GraphBuilder {
    config: GraphConfig,
    domain_counts: HashMap<String, usize>,
    folder_counts: HashMap<String, usize>,
    tag_counts: HashMap<String, usize>,
    category_counts: HashMap<String, usize>,
    domain_to_bookmarks: HashMap<String, Vec<String>>,
    folder_to_bookmarks: HashMap<String, Vec<String>>,
    tag_to_bookmarks: HashMap<String, Vec<String>>,
    category_to_bookmarks: HashMap<String, Vec<String>>,
    bookmark_tags: HashMap<String, HashSet<String>>,
}

impl GraphBuilder {
    pub fn new(config: GraphConfig) -> Self {
        Self {
            config,
            domain_counts: HashMap::new(),
            folder_counts: HashMap::new(),
            tag_counts: HashMap::new(),
            category_counts: HashMap::new(),
            domain_to_bookmarks: HashMap::new(),
            folder_to_bookmarks: HashMap::new(),
            tag_to_bookmarks: HashMap::new(),
            category_to_bookmarks: HashMap::new(),
            bookmark_tags: HashMap::new(),
        }
    }

    /// Build a graph from bookmarks
    pub fn from_bookmarks(&mut self, bookmarks: &[Bookmark]) -> Result<KnowledgeGraph> {
        let filtered = self.filter_bookmarks(bookmarks);
        let items: Vec<IngestItem> = filtered
            .iter()
            .map(|b| IngestItem {
                id: b.id.clone(),
                title: &b.title,
                url: b.url.as_deref(),
                folder: b.folder.as_deref(),
                size: 1,
            })
            .collect();

        let create_nodes = !self.config.domain_only
            && self.config.detail_level != DetailLevel::Overview;
        let nodes = self.ingest_items(&items, create_nodes);

        self.finalize_graph(nodes, filtered.len())
    }

    /// Build a graph from history entries
    pub fn from_history(&mut self, history: &[UrlEntry]) -> Result<KnowledgeGraph> {
        let items: Vec<IngestItem> = history
            .iter()
            .enumerate()
            .map(|(i, e)| IngestItem {
                id: format!("hist_{}", i),
                title: &e.title,
                url: Some(e.url.as_str()),
                folder: None,
                size: e.visit_count as usize,
            })
            .collect();

        let nodes = self.ingest_items(&items, true);
        self.finalize_graph(nodes, items.len())
    }

    /// Build a graph from both bookmarks and history
    pub fn from_both(
        &mut self,
        bookmarks: &[Bookmark],
        history: &[UrlEntry],
    ) -> Result<KnowledgeGraph> {
        let mut items: Vec<IngestItem> = bookmarks
            .iter()
            .map(|b| IngestItem {
                id: b.id.clone(),
                title: &b.title,
                url: b.url.as_deref(),
                folder: b.folder.as_deref(),
                size: 1,
            })
            .collect();

        items.extend(history.iter().enumerate().map(|(i, e)| IngestItem {
            id: format!("hist_{}", i),
            title: &e.title,
            url: Some(e.url.as_str()),
            folder: None,
            size: e.visit_count as usize,
        }));

        let nodes = self.ingest_items(&items, true);
        self.finalize_graph(nodes, items.len())
    }

    /// Unified ingestion: track stats and optionally create bookmark nodes
    fn ingest_items(&mut self, items: &[IngestItem], create_nodes: bool) -> Vec<GraphNode> {
        let mut nodes = Vec::new();

        for item in items {
            let domain = item.url.and_then(analyzer::extract_domain);

            // Track domain
            if let Some(ref d) = domain {
                *self.domain_counts.entry(d.clone()).or_insert(0) += 1;
                self.domain_to_bookmarks
                    .entry(d.clone())
                    .or_default()
                    .push(item.id.clone());
            }

            // Track folder
            if let Some(f) = item.folder {
                *self.folder_counts.entry(f.to_string()).or_insert(0) += 1;
                self.folder_to_bookmarks
                    .entry(f.to_string())
                    .or_default()
                    .push(item.id.clone());
            }

            // Extract tags
            let tags = analyzer::extract_tags(item.title, item.url);
            for tag in &tags {
                *self.tag_counts.entry(tag.clone()).or_insert(0) += 1;
                self.tag_to_bookmarks
                    .entry(tag.clone())
                    .or_default()
                    .push(item.id.clone());
            }
            self.bookmark_tags
                .insert(item.id.clone(), tags.into_iter().collect());

            // Assign category
            let category = analyzer::categorize(item.title, item.url, domain.as_deref());
            *self.category_counts.entry(category.clone()).or_insert(0) += 1;
            self.category_to_bookmarks
                .entry(category.clone())
                .or_default()
                .push(item.id.clone());

            if create_nodes {
                nodes.push(GraphNode {
                    id: item.id.clone(),
                    title: item.title.to_string(),
                    node_type: NodeType::Bookmark,
                    url: item.url.map(|s| s.to_string()),
                    domain: domain.clone(),
                    folder: item.folder.map(|s| s.to_string()),
                    size: item.size,
                });
            }
        }

        nodes
    }

    /// Filter bookmarks based on date and detail level config
    fn filter_bookmarks<'a>(&self, bookmarks: &'a [Bookmark]) -> Vec<&'a Bookmark> {
        let filtered: Vec<&Bookmark> = bookmarks
            .iter()
            .filter(|b| {
                if let Some(min_date) = self.config.min_date {
                    if let Some(date_added) = b.date_added {
                        if date_added < min_date {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            })
            .collect();

        match self.config.detail_level {
            DetailLevel::Overview => Vec::new(),
            DetailLevel::Standard => {
                let mut domain_counts: HashMap<String, usize> = HashMap::new();
                let mut result = Vec::new();
                for &bookmark in &filtered {
                    let domain = bookmark
                        .url
                        .as_ref()
                        .and_then(|u| analyzer::extract_domain(u))
                        .unwrap_or_else(|| "other".to_string());
                    let count = domain_counts.entry(domain).or_insert(0);
                    let under_per_domain = self
                        .config
                        .max_bookmarks_per_domain
                        .map_or(true, |max| *count < max);
                    let under_total = self
                        .config
                        .max_total_bookmarks
                        .map_or(true, |max| result.len() < max);
                    if under_per_domain && under_total {
                        result.push(bookmark);
                        *count += 1;
                    }
                }
                result
            }
            DetailLevel::Detailed => {
                if let Some(max_total) = self.config.max_total_bookmarks {
                    filtered.iter().take(max_total).copied().collect()
                } else {
                    filtered
                }
            }
        }
    }

    /// Create aggregate nodes and edges, build metadata
    fn finalize_graph(
        &self,
        mut nodes: Vec<GraphNode>,
        bookmark_count: usize,
    ) -> Result<KnowledgeGraph> {
        let mut edges = Vec::new();

        // Create aggregate nodes
        let domain_nodes = self.create_domain_nodes();
        let domain_count = domain_nodes.len();
        nodes.extend(domain_nodes);

        let folder_nodes = self.create_folder_nodes();
        let folder_count = folder_nodes.len();
        nodes.extend(folder_nodes);

        nodes.extend(self.create_tag_nodes());
        nodes.extend(self.create_category_nodes());

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
        if self.config.include_tag_edges {
            self.create_tag_edges(&mut edges);
        }
        if self.config.include_category_edges {
            self.create_category_edges(&mut edges);
        }
        if self.config.include_similarity_edges {
            self.create_similarity_edges(&mut edges);
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            bookmark_count,
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

    // --- Node creators ---

    fn create_domain_nodes(&self) -> Vec<GraphNode> {
        self.domain_counts
            .iter()
            .filter(|&(_, &count)| count >= self.config.min_domain_threshold)
            .map(|(domain, &count)| GraphNode {
                id: format!("domain_{}", domain),
                title: domain.clone(),
                node_type: NodeType::Domain,
                url: None,
                domain: Some(domain.clone()),
                folder: None,
                size: count,
            })
            .collect()
    }

    fn create_folder_nodes(&self) -> Vec<GraphNode> {
        self.folder_counts
            .iter()
            .map(|(folder, &count)| GraphNode {
                id: format!("folder_{}", folder.replace('/', "_")),
                title: folder.clone(),
                node_type: NodeType::Folder,
                url: None,
                domain: None,
                folder: Some(folder.clone()),
                size: count,
            })
            .collect()
    }

    fn create_tag_nodes(&self) -> Vec<GraphNode> {
        self.tag_counts
            .iter()
            .filter(|&(_, &count)| count >= self.config.min_tag_threshold)
            .map(|(tag, &count)| GraphNode {
                id: format!("tag_{}", tag),
                title: format!("#{}", tag),
                node_type: NodeType::Tag,
                url: None,
                domain: None,
                folder: None,
                size: count,
            })
            .collect()
    }

    fn create_category_nodes(&self) -> Vec<GraphNode> {
        self.category_counts
            .iter()
            .map(|(category, &count)| GraphNode {
                id: format!("cat_{}", category),
                title: category.clone(),
                node_type: NodeType::Category,
                url: None,
                domain: None,
                folder: None,
                size: count,
            })
            .collect()
    }

    // --- Edge creators ---

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

    fn create_tag_edges(&self, edges: &mut Vec<GraphEdge>) {
        for (tag, bookmark_ids) in &self.tag_to_bookmarks {
            if *self.tag_counts.get(tag).unwrap_or(&0) >= self.config.min_tag_threshold {
                let tag_id = format!("tag_{}", tag);
                for bookmark_id in bookmark_ids {
                    edges.push(GraphEdge {
                        source: bookmark_id.clone(),
                        target: tag_id.clone(),
                        edge_type: EdgeType::HasTag,
                        weight: 0.8,
                    });
                }
            }
        }
    }

    fn create_category_edges(&self, edges: &mut Vec<GraphEdge>) {
        for (category, bookmark_ids) in &self.category_to_bookmarks {
            let cat_id = format!("cat_{}", category);
            for bookmark_id in bookmark_ids {
                edges.push(GraphEdge {
                    source: bookmark_id.clone(),
                    target: cat_id.clone(),
                    edge_type: EdgeType::InCategory,
                    weight: 0.7,
                });
            }
        }
    }

    fn create_similarity_edges(&self, edges: &mut Vec<GraphEdge>) {
        let bookmark_ids: Vec<&String> = self.bookmark_tags.keys().collect();
        for i in 0..bookmark_ids.len() {
            for j in (i + 1)..bookmark_ids.len() {
                let tags_a = &self.bookmark_tags[bookmark_ids[i]];
                let tags_b = &self.bookmark_tags[bookmark_ids[j]];
                let jaccard = analyzer::jaccard_similarity(tags_a, tags_b);
                if jaccard >= self.config.similarity_threshold {
                    edges.push(GraphEdge {
                        source: bookmark_ids[i].clone(),
                        target: bookmark_ids[j].clone(),
                        edge_type: EdgeType::SimilarContent,
                        weight: jaccard,
                    });
                }
            }
        }
    }

    // --- Public helpers for backward compatibility ---

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        analyzer::extract_domain(url)
    }

    pub fn extract_tags(&self, title: &str, url: Option<&str>) -> Vec<String> {
        analyzer::extract_tags(title, url)
    }

    pub fn categorize(&self, title: &str, url: Option<&str>, domain: Option<&str>) -> String {
        analyzer::categorize(title, url, domain)
    }
}
