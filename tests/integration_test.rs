use bookmark::{BookmarkManager, Bookmark};
use tempfile::TempDir;

#[test]
fn test_library_api_manager_creation() {
    let _manager = BookmarkManager::new();
    assert!(true);
}

#[test]
fn test_library_api_with_export_dir() {
    let temp_dir = TempDir::new().unwrap();
    let _manager = BookmarkManager::new()
        .with_export_dir(temp_dir.path().to_path_buf());
    assert!(true);
}

#[test]
fn test_graph_generation() {
    let bookmarks = vec![
        Bookmark {
            id: "1".to_string(),
            title: "GitHub".to_string(),
            url: Some("https://github.com".to_string()),
            folder: Some("Dev".to_string()),
            date_added: None,
            children: None,
        },
        Bookmark {
            id: "2".to_string(),
            title: "Rust".to_string(),
            url: Some("https://rust-lang.org".to_string()),
            folder: Some("Dev".to_string()),
            date_added: None,
            children: None,
        },
    ];

    let manager = BookmarkManager::new();
    let result = manager.graph_from_bookmarks(&bookmarks);
    assert!(result.is_ok());
    
    let graph = result.unwrap();
    assert!(graph.metadata.total_nodes > 0);
}

#[cfg(feature = "mcp")]
#[test]
fn test_mcp_server_creation() {
    use bookmark::mcp::McpServer;
    let server = McpServer::new();
    assert!(true);
}
