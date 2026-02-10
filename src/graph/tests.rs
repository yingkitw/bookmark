use super::*;
use crate::exporter::{Bookmark, UrlEntry};
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
    let config = GraphConfig {
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    assert!(graph.nodes.len() >= 4);
    assert_eq!(graph.metadata.bookmark_count, 4);

    let domain_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Domain)
        .collect();
    assert_eq!(domain_nodes.len(), 1);

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

    assert_eq!(graph.metadata.bookmark_count, 2);
    assert_eq!(graph.metadata.folder_count, 0);

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
        min_domain_threshold: 3,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

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
    let config = GraphConfig {
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    assert!(graph.edges.len() > 0);

    let domain_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::BelongsToDomain)
        .collect();
    assert!(domain_edges.len() > 0);

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
    let config = GraphConfig {
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let dot = formats::to_dot(&graph);

    assert!(dot.contains("digraph BookmarkKnowledgeGraph"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.contains("fillcolor"));
    assert!(dot.contains("node"));
    assert!(dot.contains("->"));
}

#[test]
fn test_json_export() {
    let bookmarks = create_test_bookmarks();
    let config = GraphConfig {
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let json = formats::to_json(&graph);

    assert!(json.contains("\"nodes\""));
    assert!(json.contains("\"edges\""));
    assert!(json.contains("\"metadata\""));

    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["nodes"].is_array());
    assert!(parsed["edges"].is_array());
    assert!(parsed["metadata"]["total_nodes"].is_number());
}

#[test]
fn test_gexf_export() {
    let bookmarks = create_test_bookmarks();
    let config = GraphConfig {
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let gexf = formats::to_gexf(&graph);

    assert!(gexf.contains("<?xml version=\"1.0\""));
    assert!(gexf.contains("<gexf"));
    assert!(gexf.contains("<nodes>"));
    assert!(gexf.contains("<edges>"));
    assert!(gexf.contains("</gexf>"));
}

#[test]
fn test_edge_type_toggles() {
    let bookmarks = create_test_bookmarks();

    let config = GraphConfig {
        include_folder_edges: false,
        include_domain_edges: true,
        include_same_domain_edges: false,
        include_tag_edges: false,
        include_category_edges: false,
        include_similarity_edges: false,
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    assert!(graph
        .edges
        .iter()
        .all(|e| e.edge_type == EdgeType::BelongsToDomain));
}

#[test]
fn test_tag_nodes_created() {
    let bookmarks = vec![
        Bookmark {
            id: "1".to_string(),
            title: "Rust Programming Guide".to_string(),
            url: Some("https://rust-lang.org/learn".to_string()),
            folder: Some("Dev".to_string()),
            date_added: Some(Utc::now()),
            children: None,
        },
        Bookmark {
            id: "2".to_string(),
            title: "Rust Async Programming".to_string(),
            url: Some("https://rust-lang.org/async".to_string()),
            folder: Some("Dev".to_string()),
            date_added: Some(Utc::now()),
            children: None,
        },
    ];
    let config = GraphConfig {
        min_tag_threshold: 2,
        include_tag_edges: true,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let tag_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Tag)
        .collect();
    assert!(
        !tag_nodes.is_empty(),
        "Should create tag nodes for shared keywords"
    );

    let tag_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::HasTag)
        .collect();
    assert!(!tag_edges.is_empty(), "Should create HasTag edges");
}

#[test]
fn test_category_nodes_created() {
    let bookmarks = create_test_bookmarks();
    let config = GraphConfig {
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let cat_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Category)
        .collect();
    assert!(!cat_nodes.is_empty(), "Should create category nodes");

    let cat_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::InCategory)
        .collect();
    assert!(!cat_edges.is_empty(), "Should create InCategory edges");
}

#[test]
fn test_similarity_edges() {
    let bookmarks = vec![
        Bookmark {
            id: "1".to_string(),
            title: "Rust Programming Language".to_string(),
            url: Some("https://rust-lang.org".to_string()),
            folder: None,
            date_added: None,
            children: None,
        },
        Bookmark {
            id: "2".to_string(),
            title: "Rust Programming Tutorial".to_string(),
            url: Some("https://example.com/rust".to_string()),
            folder: None,
            date_added: None,
            children: None,
        },
    ];
    let config = GraphConfig {
        similarity_threshold: 0.2,
        include_similarity_edges: true,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let sim_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::SimilarContent)
        .collect();
    assert!(
        !sim_edges.is_empty(),
        "Should create SimilarContent edges for bookmarks with shared tags"
    );
}

#[test]
fn test_html_export() {
    let bookmarks = create_test_bookmarks();
    let config = GraphConfig {
        min_domain_threshold: 2,
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    let html = formats::to_html(&graph);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("d3.v7.min.js"));
    assert!(html.contains("Knowledge Graph"));
    assert!(html.contains("\"nodes\""));
    assert!(html.contains("\"edges\""));
}

#[test]
fn test_extract_tags() {
    let config = GraphConfig::default();
    let builder = GraphBuilder::new(config);

    let tags = builder.extract_tags("Rust Programming Language", Some("https://rust-lang.org/learn"));
    assert!(tags.contains(&"rust".to_string()));
    assert!(tags.contains(&"programming".to_string()));
    assert!(tags.contains(&"language".to_string()));
    assert!(tags.contains(&"learn".to_string()));
}

#[test]
fn test_categorize() {
    let config = GraphConfig::default();
    let builder = GraphBuilder::new(config);

    assert_eq!(
        builder.categorize(
            "GitHub Repository",
            Some("https://github.com/user/repo"),
            Some("github.com")
        ),
        "Development"
    );
    assert_eq!(
        builder.categorize("Amazon Shopping", Some("https://amazon.com"), Some("amazon.com")),
        "Shopping"
    );
    assert_eq!(
        builder.categorize(
            "ChatGPT AI",
            Some("https://chat.openai.com"),
            Some("chat.openai.com")
        ),
        "AI & ML"
    );
    assert_eq!(
        builder.categorize("Random Page", Some("https://example.com"), Some("example.com")),
        "Other"
    );
}

#[test]
fn test_empty_bookmarks() {
    let bookmarks: Vec<Bookmark> = vec![];
    let config = GraphConfig {
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
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

    let config = GraphConfig {
        detail_level: super::DetailLevel::Detailed,
        max_bookmarks_per_domain: None,
        max_total_bookmarks: None,
        ..Default::default()
    };
    let mut builder = GraphBuilder::new(config);
    let graph = builder.from_bookmarks(&bookmarks).unwrap();

    assert_eq!(graph.metadata.bookmark_count, 1);

    let domain_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Domain)
        .collect();
    assert_eq!(domain_nodes.len(), 0);

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

    assert_eq!(
        builder.extract_domain("https://github.com"),
        Some("github.com".to_string())
    );
    assert_eq!(
        builder.extract_domain("https://www.github.com"),
        Some("github.com".to_string())
    );
    assert_eq!(
        builder.extract_domain("https://doc.rust-lang.org"),
        Some("doc.rust-lang.org".to_string())
    );
    assert_eq!(builder.extract_domain("not-a-url"), None);
}

#[test]
fn test_analyzer_jaccard_similarity() {
    use std::collections::HashSet;
    let a: HashSet<String> = ["rust", "programming", "language"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let b: HashSet<String> = ["rust", "programming", "tutorial"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let sim = analyzer::jaccard_similarity(&a, &b);
    assert!(sim > 0.0 && sim < 1.0);

    let empty: HashSet<String> = HashSet::new();
    assert_eq!(analyzer::jaccard_similarity(&a, &empty), 0.0);
}
