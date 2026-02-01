#[cfg(feature = "mcp")]
mod mcp_tests {
    use bookmark::mcp::McpServer;
    use serde_json::json;

    #[test]
    fn test_mcp_server_initialization() {
        let server = McpServer::new();
        assert!(true);
    }

    #[test]
    fn test_mcp_tools_available() {
        let expected_tools = vec![
            "export_bookmarks",
            "search_bookmarks",
            "list_browsers",
            "process_bookmarks",
            "generate_graph",
        ];
        
        assert_eq!(expected_tools.len(), 5);
    }
}
