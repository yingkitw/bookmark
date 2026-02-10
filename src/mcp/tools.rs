use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::browser::Browser;
use crate::exporter::{export_data, Bookmark};
use crate::graph::{GraphBuilder, GraphConfig};
use crate::processor::{BookmarkProcessor, ProcessingConfig};
use crate::search::{search_bookmarks_internal, SearchOptions};

use super::McpServer;

impl McpServer {
    pub(super) fn tool_export_bookmarks(&self, args: Value) -> Result<Value> {
        let browser = args["browser"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing browser"))?;
        let data_type = args["data_type"].as_str().unwrap_or("bookmarks");

        let temp_dir = PathBuf::from("/tmp/bookmark_mcp");
        std::fs::create_dir_all(&temp_dir)?;

        let output_file = temp_dir.join(format!("{}-{}.yaml", browser, data_type));
        export_data(browser, data_type, Some(output_file.clone()), None)?;

        let content = std::fs::read_to_string(&output_file)?;
        let data: Vec<crate::exporter::BrowserData> = serde_yaml::from_str(&content)?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&data)?
            }]
        }))
    }

    pub(super) fn tool_search_bookmarks(&self, args: Value) -> Result<Value> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let title_only = args["title_only"].as_bool().unwrap_or(false);
        let url_only = args["url_only"].as_bool().unwrap_or(false);
        let limit = args["limit"].as_u64().unwrap_or(20) as usize;

        let options = SearchOptions {
            title_only,
            url_only,
            limit,
        };

        let results = search_bookmarks_internal(query, &options)?;

        let formatted_results: Vec<String> = results
            .iter()
            .map(|b| format!("{} - {}", b.title, b.url.as_deref().unwrap_or("N/A")))
            .collect();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": formatted_results.join("\n")
            }]
        }))
    }

    pub(super) fn tool_list_browsers(&self, args: Value) -> Result<Value> {
        let mut output = Vec::new();

        let browsers = if let Some(browser_name) = args["browser"].as_str() {
            vec![browser_name]
        } else {
            vec!["chrome", "firefox", "safari", "edge"]
        };

        for browser_name in browsers {
            if let Ok(browser) = Browser::from_str(browser_name) {
                if let Ok(profiles) = browser.find_profiles(None) {
                    output.push(format!(
                        "{}: {} profile(s)",
                        browser_name,
                        profiles.len()
                    ));
                }
            }
        }

        Ok(json!({
            "content": [{
                "type": "text",
                "text": output.join("\n")
            }]
        }))
    }

    pub(super) fn tool_process_bookmarks(&self, args: Value) -> Result<Value> {
        let bookmarks_json = args["bookmarks"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Missing bookmarks array"))?;
        let mode = args["mode"].as_str().unwrap_or("both");
        let strategy = args["strategy"].as_str().unwrap_or("merge");

        let bookmarks: Vec<Bookmark> = serde_json::from_value(json!(bookmarks_json))?;

        let merge_strategy = match strategy {
            "first" => crate::deduplication::MergeStrategy::KeepFirst,
            "last" => crate::deduplication::MergeStrategy::KeepLast,
            "recent" => crate::deduplication::MergeStrategy::KeepMostRecent,
            _ => crate::deduplication::MergeStrategy::MergeMetadata,
        };

        let config = ProcessingConfig {
            deduplication_config: crate::deduplication::DeduplicationConfig {
                merge_strategy,
                normalize_urls: mode == "dedupe" || mode == "both",
                ..Default::default()
            },
            organization_config: crate::organization::OrganizationConfig::default(),
            dry_run: false,
            backup_original: false,
        };

        let processor = BookmarkProcessor::new(config);
        let result = processor.process_bookmarks(&bookmarks)?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Processed {} bookmarks:\n- Original: {}\n- Final: {}\n- Duplicates removed: {}",
                    result.processing_summary.original_count,
                    result.processing_summary.original_count,
                    result.processing_summary.final_count,
                    result.processing_summary.duplicates_removed
                )
            }],
            "processed_bookmarks": result.processed_bookmarks
        }))
    }

    pub(super) fn tool_generate_graph(&self, args: Value) -> Result<Value> {
        let browser = args["browser"].as_str().unwrap_or("all");
        let format = args["format"].as_str().unwrap_or("json");
        let min_threshold = args["min_threshold"].as_u64().unwrap_or(2) as usize;

        let (all_bookmarks, all_history) = crate::exporter::load_browser_data(browser, "both")?;

        let config = GraphConfig {
            min_domain_threshold: min_threshold,
            ..Default::default()
        };

        let mut builder = GraphBuilder::new(config);
        let graph = builder.from_both(&all_bookmarks, &all_history)?;

        let graph_content = match format {
            "dot" => crate::graph::formats::to_dot(&graph),
            "json" => crate::graph::formats::to_json(&graph),
            "gexf" => crate::graph::formats::to_gexf(&graph),
            "html" => crate::graph::formats::to_html(&graph),
            _ => return Err(anyhow::anyhow!("Invalid format: {}. Use dot, json, gexf, or html", format)),
        };

        Ok(json!({
            "content": [{
                "type": "text",
                "text": graph_content
            }],
            "metadata": {
                "nodes": graph.metadata.total_nodes,
                "edges": graph.metadata.total_edges,
                "bookmarks": graph.metadata.bookmark_count,
                "domains": graph.metadata.domain_count
            }
        }))
    }
}
