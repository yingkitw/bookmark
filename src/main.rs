use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod browser;
mod config;
mod deduplication;
mod exporter;
mod graph;
mod organization;
mod processor;
mod search;

use browser::Browser;
use deduplication::MergeStrategy;
use exporter::export_data;
use processor::{BookmarkProcessor, ProcessingConfig};
use search::{open_bookmark, search_bookmarks};

#[derive(Parser)]
#[command(name = "bookmark")]
#[command(about = "Import, search, and manage bookmarks from all browsers", version = "0.1.1")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export bookmarks/history from browsers
    Export {
        /// Browser (chrome, firefox, safari, edge, all)
        #[arg(short, long, default_value = "all")]
        browser: String,
        /// Data type (bookmarks, history, both)
        #[arg(short, long, default_value = "bookmarks")]
        data_type: String,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Custom browser data directory
        #[arg(long)]
        profile_dir: Option<PathBuf>,
    },

    /// List available browsers
    List {
        /// Filter by specific browser
        browser: Option<String>,
    },

    /// Search bookmarks
    Search {
        /// Search query
        query: String,
        /// Search in title only
        #[arg(long)]
        title_only: bool,
        /// Search in URL only
        #[arg(long)]
        url_only: bool,
        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Open bookmark in browser
    Open {
        /// Search query
        query: String,
        /// Open first match without asking
        #[arg(short, long)]
        first: bool,
    },

    /// Process bookmarks (deduplicate, organize, or both)
    Process {
        /// Input file
        #[arg(short, long)]
        input: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
        /// Processing mode (dedupe, organize, both)
        #[arg(short, long, default_value = "both")]
        mode: String,
        /// Merge strategy (first, last, recent, merge)
        #[arg(long, default_value = "merge")]
        strategy: String,
        /// Organization strategy (domain, category, custom)
        #[arg(long, default_value = "custom")]
        org_strategy: String,
        /// Preview without applying
        #[arg(long)]
        preview: bool,
        /// Create backup
        #[arg(long)]
        backup: bool,
    },

    /// Generate knowledge graph
    Graph {
        /// Browser source
        #[arg(short, long, default_value = "all")]
        browser: String,
        /// Data type (bookmarks, history, both)
        #[arg(short, long, default_value = "both")]
        data_type: String,
        /// Output format (dot, json, gexf)
        #[arg(short, long, default_value = "dot")]
        format: String,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
        /// Minimum bookmarks for domain node
        #[arg(long, default_value = "2")]
        min_threshold: usize,
    },

    /// Manage configuration
    Config {
        /// Show current config
        #[arg(long)]
        show: bool,
        /// Create sample config
        #[arg(long)]
        create_sample: Option<PathBuf>,
        /// List custom rules
        #[arg(long)]
        list_rules: bool,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Export {
            browser,
            data_type,
            output,
            profile_dir,
        } => {
            if browser == "all" {
                export_all_browsers(&data_type, output, profile_dir)?;
            } else {
                export_data(&browser, &data_type, output, profile_dir)?;
            }
        }

        Commands::List { browser } => {
            if let Some(b) = browser {
                list_browser_profiles(&b)?;
            } else {
                list_all_browsers()?;
            }
        }

        Commands::Search {
            query,
            title_only,
            url_only,
            limit,
        } => {
            search_bookmarks(&query, title_only, url_only, limit)?;
        }

        Commands::Open { query, first } => {
            open_bookmark(&query, first)?;
        }

        Commands::Process {
            input,
            output,
            mode,
            strategy,
            org_strategy,
            preview,
            backup,
        } => {
            process_bookmarks(&input, &output, &mode, &strategy, &org_strategy, preview, backup)?;
        }

        Commands::Graph {
            browser,
            data_type,
            format,
            output,
            min_threshold,
        } => {
                generate_graph(&browser, &data_type, &format, output, min_threshold)?;
        }

        Commands::Config {
            show,
            create_sample,
            list_rules,
        } => {
            handle_config(show, create_sample, list_rules)?;
        }
    }

    Ok(())
}

fn export_all_browsers(data_type: &str, output_dir: Option<PathBuf>, profile_dir: Option<PathBuf>) -> Result<()> {
    let browsers = ["Chrome", "Firefox", "Safari", "Edge"];
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)?;

    println!("Scanning browsers...");
    let mut found = Vec::new();

    for browser_name in browsers {
        if let Ok(browser) = Browser::from_str(browser_name) {
            if let Ok(profiles) = browser.find_profiles(profile_dir.as_deref()) {
                if !profiles.is_empty() {
                    found.push(browser_name);
                    let output_file = output_dir.join(format!("{}-{}.yaml", browser_name.to_lowercase(), data_type));
                    println!("Exporting {}...", browser_name);
                    match export_data(browser_name, data_type, Some(output_file), profile_dir.clone()) {
                        Ok(_) => println!("  ✓ Success"),
                        Err(e) => println!("  ✗ Failed: {}", e),
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

fn process_bookmarks(
    input: &PathBuf,
    output: &PathBuf,
    mode: &str,
    strategy: &str,
    org_strategy: &str,
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
            organize_by_domain: org_strategy == "domain" || org_strategy == "custom",
            organize_by_category: org_strategy == "category" || org_strategy == "custom",
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

fn generate_graph(
    browser: &str,
    data_type: &str,
    format: &str,
    output: PathBuf,
    min_threshold: usize,
) -> Result<()> {
    println!("Generating knowledge graph...");

    let (bookmarks, history) = load_browser_data(browser, data_type)?;

    let config = graph::GraphConfig {
        min_domain_threshold: min_threshold,
        ..Default::default()
    };

    let mut builder = graph::GraphBuilder::new(config);
    let graph = match data_type {
        "bookmarks" => builder.from_bookmarks(&bookmarks)?,
        "history" => builder.from_history(&history)?,
        "both" => builder.from_both(&bookmarks, &history)?,
        _ => return Err(anyhow::anyhow!("Invalid data type")),
    };

    let content = match format {
        "dot" => graph::formats::to_dot(&graph),
        "json" => graph::formats::to_json(&graph),
        "gexf" => graph::formats::to_gexf(&graph),
        _ => return Err(anyhow::anyhow!("Invalid format")),
    };

    fs::write(&output, content)?;

    println!("✓ Graph generated: {}", output.display());
    println!("  Nodes: {} (bookmarks: {}, domains: {}, folders: {})",
        graph.metadata.total_nodes,
        graph.metadata.bookmark_count,
        graph.metadata.domain_count,
        graph.metadata.folder_count
    );
    println!("  Edges: {}", graph.metadata.total_edges);

    Ok(())
}

fn load_browser_data(browser: &str, data_type: &str) -> Result<(Vec<exporter::Bookmark>, Vec<exporter::UrlEntry>)> {
    let temp_dir = PathBuf::from("/tmp/bookmark_graph");

    if browser == "all" {
        fs::create_dir_all(&temp_dir)?;
        export_all_browsers(data_type, Some(temp_dir.clone()), None)?;
    } else {
        fs::create_dir_all(&temp_dir)?;
        export_data(browser, data_type, Some(temp_dir.join(format!("{}.yaml", browser))), None)?;
    }

    let mut all_bookmarks = Vec::new();
    let mut all_history = Vec::new();

    let browsers: Vec<&str> = if browser == "all" {
        vec!["chrome", "firefox", "safari", "edge"]
    } else {
        vec![browser]
    };

    for browser_name in browsers {
        let file = temp_dir.join(format!("{}-{}.yaml", browser_name, data_type));
        if file.exists() {
            let content = fs::read_to_string(&file)?;
            let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;
            for data in browser_data {
                if let Some(b) = data.bookmarks {
                    all_bookmarks.extend(b);
                }
                if let Some(h) = data.history {
                    all_history.extend(h.urls);
                }
            }
        }
    }

    Ok((all_bookmarks, all_history))
}

fn handle_config(show: bool, create_sample: Option<PathBuf>, list_rules: bool) -> Result<()> {
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

fn list_all_browsers() -> Result<()> {
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

fn list_browser_profiles(browser_name: &str) -> Result<()> {
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
