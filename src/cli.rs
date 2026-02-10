use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::browser::Browser;
use crate::deduplication::MergeStrategy;
use crate::exporter::export_data;
use crate::processor::{BookmarkProcessor, ProcessingConfig};
use crate::{config, deduplication, exporter, graph, organization};

pub fn export_all_browsers(
    data_type: &str,
    output_dir: Option<PathBuf>,
    profile_dir: Option<PathBuf>,
) -> Result<()> {
    let browsers = ["Chrome", "Firefox", "Safari", "Edge"];
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)?;

    println!("Scanning browsers...");
    let mut found = Vec::new();

    // Handle "both" data type by exporting bookmarks and history separately
    let export_types = match data_type {
        "both" => vec!["bookmarks", "history"],
        _ => vec![data_type],
    };

    for browser_name in browsers {
        if let Ok(browser) = Browser::from_str(browser_name) {
            if let Ok(profiles) = browser.find_profiles(profile_dir.as_deref()) {
                if !profiles.is_empty() {
                    found.push(browser_name);
                    for export_type in &export_types {
                        let output_file = output_dir.join(format!(
                            "{}-{}.yaml",
                            browser_name.to_lowercase(),
                            export_type
                        ));
                        println!("Exporting {} ({})...", browser_name, export_type);
                        match export_data(
                            browser_name,
                            export_type,
                            Some(output_file),
                            profile_dir.clone(),
                        ) {
                            Ok(_) => println!("  ✓ Success"),
                            Err(e) => println!("  ✗ Failed: {}", e),
                        }
                    }
                }
            }
        }
    }

    if found.is_empty() {
        println!("No browsers found");
    } else {
        println!("\nExported: {}", found.join(", "));
    }
    Ok(())
}

pub fn process_bookmarks(
    input: &PathBuf,
    output: &PathBuf,
    mode: &str,
    strategy: &str,
    _org_strategy: &str,
    preview: bool,
    backup: bool,
) -> Result<()> {
    println!("Loading {}...", input.display());
    let content = fs::read_to_string(input)?;
    let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;

    let mut all_bookmarks = Vec::new();
    for data in browser_data {
        if let Some(bookmarks) = data.bookmarks {
            all_bookmarks.extend(bookmarks);
        }
    }

    println!("Loaded {} bookmarks", all_bookmarks.len());

    let merge_strategy = match strategy {
        "first" => MergeStrategy::KeepFirst,
        "last" => MergeStrategy::KeepLast,
        "recent" => MergeStrategy::KeepMostRecent,
        "merge" => MergeStrategy::MergeMetadata,
        _ => return Err(anyhow::anyhow!("Invalid strategy: {}", strategy)),
    };

    let dedupe_enabled = mode == "dedupe" || mode == "both";

    let config = ProcessingConfig {
        deduplication_config: deduplication::DeduplicationConfig {
            merge_strategy,
            normalize_urls: dedupe_enabled,
            ..Default::default()
        },
        organization_config: organization::OrganizationConfig {
            organize_by_domain: _org_strategy == "domain" || _org_strategy == "custom",
            organize_by_category: _org_strategy == "category" || _org_strategy == "custom",
            ..Default::default()
        },
        dry_run: preview,
        backup_original: backup,
    };

    let processor = BookmarkProcessor::new(config);
    let result = processor.process_bookmarks(&all_bookmarks)?;

    if !preview {
        processor.export_processed_bookmarks(&result.processed_bookmarks, output)?;
    }

    println!(
        "Original: {} | Final: {} | Duplicates removed: {}",
        result.processing_summary.original_count,
        result.processing_summary.final_count,
        result.processing_summary.duplicates_removed
    );

    Ok(())
}

pub fn generate_graph(
    browser: &str,
    data_type: &str,
    format: &str,
    output: PathBuf,
    min_threshold: usize,
    detail: &str,
    max_per_domain: Option<usize>,
    max_total: Option<usize>,
    domain_only: bool,
    since: Option<String>,
) -> Result<()> {
    println!("Generating knowledge graph...");

    let (bookmarks, history) = exporter::load_browser_data(browser, data_type)?;

    // Parse detail level
    let detail_level = match detail.to_lowercase().as_str() {
        "overview" => graph::DetailLevel::Overview,
        "standard" => graph::DetailLevel::Standard,
        "detailed" => graph::DetailLevel::Detailed,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid detail level: {}. Use overview, standard, or detailed",
                detail
            ))
        }
    };

    // Parse date filter
    let min_date = if let Some(date_str) = since {
        Some(
            chrono::DateTime::parse_from_rfc3339(&date_str)
                .map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid date format: {}. Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)",
                        date_str
                    )
                })?
                .with_timezone(&chrono::Utc),
        )
    } else {
        None
    };

    let config = graph::GraphConfig {
        min_domain_threshold: min_threshold,
        detail_level,
        max_bookmarks_per_domain: max_per_domain,
        max_total_bookmarks: max_total,
        domain_only,
        min_date,
        ..Default::default()
    };

    println!("Graph configuration:");
    println!("  Detail level: {:?}", detail_level);
    println!("  Min domain threshold: {}", min_threshold);
    if let Some(max_per) = max_per_domain {
        println!("  Max bookmarks per domain: {}", max_per);
    }
    if let Some(max_tot) = max_total {
        println!("  Max total bookmarks: {}", max_tot);
    }
    if domain_only {
        println!("  Domain-only mode: enabled");
    }
    if let Some(ref date) = min_date {
        println!("  Only bookmarks newer than: {}", date);
    }

    let mut builder = graph::GraphBuilder::new(config);
    let graph = match data_type {
        "bookmarks" => builder.from_bookmarks(&bookmarks)?,
        "history" => builder.from_history(&history)?,
        "both" => builder.from_both(&bookmarks, &history)?,
        _ => return Err(anyhow::anyhow!("Invalid data type")),
    };

    // For HTML output, write both HTML and data to temp folder (keeps personal data out of project)
    if format == "html" {
        let temp_dir = std::env::temp_dir().join("bookmark-graph");
        fs::create_dir_all(&temp_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let html_filename = format!("graph_{}.html", timestamp);
        let data_filename = format!("graph_{}.data.js", timestamp);

        let temp_html_path = temp_dir.join(&html_filename);
        let data_path = temp_dir.join(&data_filename);

        let js_content = graph::formats::to_js_data(&graph);
        fs::write(&data_path, js_content)?;

        let html_content = graph::formats::to_html_dynamic(&data_path);
        fs::write(&temp_html_path, html_content)?;

        println!("  Graph files created in temp directory:");
        println!("    HTML: {}", temp_html_path.display());
        println!("    Data: {}", data_path.display());
        println!("  Opening {}", temp_html_path.display());

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open").arg(&temp_html_path).spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("xdg-open").arg(&temp_html_path).spawn()?;
        }
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("start").arg(&temp_html_path).spawn()?;
        }

        if output != temp_html_path {
            let output_content = format!(
                r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0;url={}"></head>
<body>
Redirecting to <a href="{}">{}</a>...
</body>
</html>"#,
                temp_html_path.display(),
                temp_html_path.display(),
                temp_html_path.display()
            );
            fs::write(&output, output_content)?;
            println!("  Redirect link: {}", output.display());
        }
    } else {
        let content = match format {
            "dot" => graph::formats::to_dot(&graph),
            "json" => graph::formats::to_json(&graph),
            "gexf" => graph::formats::to_gexf(&graph),
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid format: {}. Use dot, json, gexf, or html",
                    format
                ))
            }
        };
        fs::write(&output, content)?;
    }

    println!("✓ Graph generated: {}", output.display());
    println!(
        "  Nodes: {} (bookmarks: {}, domains: {}, folders: {})",
        graph.metadata.total_nodes,
        graph.metadata.bookmark_count,
        graph.metadata.domain_count,
        graph.metadata.folder_count
    );
    println!("  Edges: {}", graph.metadata.total_edges);

    Ok(())
}

pub fn handle_config(show: bool, create_sample: Option<PathBuf>, list_rules: bool) -> Result<()> {
    if let Some(path) = create_sample {
        config::AppConfig::create_sample_config(&path)?;
        println!("Created: {}", path.display());
        return Ok(());
    }

    if show {
        let config = config::AppConfig::load_or_create()?;
        println!("{}", serde_yaml::to_string(&config)?);
        return Ok(());
    }

    if list_rules {
        let config = config::AppConfig::load_or_create()?;
        let rules = config.list_rules();
        if rules.is_empty() {
            println!("No custom rules");
        } else {
            for rule in rules {
                println!("{}: {} -> {}", rule.name, rule.pattern, rule.folder);
            }
        }
        return Ok(());
    }

    println!("Config commands:");
    println!("  --show           Show current configuration");
    println!("  --create-sample  Create sample config file");
    println!("  --list-rules     List custom organization rules");

    Ok(())
}

pub fn list_all_browsers() -> Result<()> {
    println!("Available browsers:");
    for browser_name in &["Chrome", "Firefox", "Safari", "Edge"] {
        if let Ok(browser) = Browser::from_str(browser_name) {
            if let Ok(profiles) = browser.find_profiles(None) {
                println!("  {}: {} profile(s)", browser_name, profiles.len());
            }
        }
    }
    Ok(())
}

pub fn list_browser_profiles(browser_name: &str) -> Result<()> {
    let browser = Browser::from_str(browser_name)?;
    let profiles = browser.find_profiles(None)?;
    if profiles.is_empty() {
        println!("No profiles found for {}", browser_name);
    } else {
        println!("Profiles for {}:", browser_name);
        for (i, p) in profiles.iter().enumerate() {
            println!("  {}: {}", i + 1, p.display());
        }
    }
    Ok(())
}
