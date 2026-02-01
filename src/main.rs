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
use config::AppConfig;
use deduplication::MergeStrategy;
use exporter::export_data;
use organization::OrganizationRule;
use processor::{BookmarkProcessor, ProcessingConfig};
use search::{open_bookmark, search_bookmarks};

#[derive(Parser)]
#[command(name = "bookmark-manager")]
#[command(about = "Import, search, and open bookmarks from all browsers")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Export {
        /// Browser to export from (chrome, firefox, safari, edge, all)
        #[arg(short, long, default_value = "all")]
        browser: String,
        /// Type of data to export (bookmarks, history, passwords, all)
        #[arg(short, long, default_value = "bookmarks")]
        data_type: String,
        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Custom browser data directory
        #[arg(long)]
        profile_dir: Option<PathBuf>,
    },
    List {
        /// List available browser profiles
        #[arg(short, long)]
        browser: Option<String>,
    },
    Scan {
        /// Scan and export all available browsers
        #[arg(short, long, default_value = "bookmarks")]
        data_type: String,
        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Search {
        /// Search term to find bookmarks
        #[arg(short, long)]
        query: String,
        /// Search in title only
        #[arg(long)]
        title_only: bool,
        /// Search in URL only
        #[arg(long)]
        url_only: bool,
        /// Limit number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    Open {
        /// Search term to find and open bookmark
        #[arg(short, long)]
        query: String,
        /// Open the first match without asking
        #[arg(short, long)]
        first: bool,
    },
    Dedupe {
        /// Input file containing bookmarks to deduplicate
        #[arg(short, long)]
        input: PathBuf,
        /// Output file for deduplicated bookmarks
        #[arg(short, long)]
        output: PathBuf,
        /// Merge strategy for duplicates (first, last, recent, frequent, merge)
        #[arg(long, default_value = "merge")]
        strategy: String,
        /// Preview changes without applying them
        #[arg(long)]
        preview: bool,
        /// Create backup of original file
        #[arg(long)]
        backup: bool,
    },
    Organize {
        /// Input file containing bookmarks to organize
        #[arg(short, long)]
        input: PathBuf,
        /// Output file for organized bookmarks
        #[arg(short, long)]
        output: PathBuf,
        /// Organization strategy (domain, category, date, custom)
        #[arg(long, default_value = "custom")]
        strategy: String,
        /// Preserve existing folder structure
        #[arg(long)]
        preserve_existing: bool,
        /// Preview changes without applying them
        #[arg(long)]
        preview: bool,
        /// Create backup of original file
        #[arg(long)]
        backup: bool,
    },
    Process {
        /// Input file containing bookmarks to process
        #[arg(short, long)]
        input: PathBuf,
        /// Output file for processed bookmarks
        #[arg(short, long)]
        output: PathBuf,
        /// Merge strategy for duplicates (first, last, recent, frequent, merge)
        #[arg(long, default_value = "merge")]
        merge_strategy: String,
        /// Organization strategy (domain, category, date, custom)
        #[arg(long, default_value = "custom")]
        organization_strategy: String,
        /// Preserve existing folder structure
        #[arg(long)]
        preserve_existing: bool,
        /// Preview changes without applying them
        #[arg(long)]
        preview: bool,
        /// Create backup of original file
        #[arg(long)]
        backup: bool,
        /// Generate detailed report
        #[arg(long)]
        report: Option<PathBuf>,
        /// Use config file settings
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        /// Create sample configuration file
        #[arg(long)]
        create_sample: Option<PathBuf>,
        /// Add custom organization rule
        #[arg(long)]
        add_rule: Option<String>,
        /// Remove custom organization rule
        #[arg(long)]
        remove_rule: Option<String>,
        /// List all custom rules
        #[arg(long)]
        list_rules: bool,
        /// Validate configuration
        #[arg(long)]
        validate: bool,
        /// Configuration file path
        #[arg(short, long)]
        config_file: Option<PathBuf>,
    },
    Graph {
        /// Browser to generate graph from (chrome, firefox, safari, edge, all)
        #[arg(short, long, default_value = "all")]
        browser: String,
        /// Type of data to include (bookmarks, history, both)
        #[arg(short, long, default_value = "both")]
        data_type: String,
        /// Output format (dot, json, gexf)
        #[arg(short, long, default_value = "dot")]
        format: String,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        /// Include folder-based relationships
        #[arg(long, default_value = "true")]
        include_folder_edges: bool,
        /// Include domain-based relationships
        #[arg(long, default_value = "true")]
        include_domain_edges: bool,
        /// Include same-domain relationships
        #[arg(long, default_value = "true")]
        include_same_domain_edges: bool,
        /// Minimum bookmarks to create domain node
        #[arg(long, default_value = "2")]
        min_domain_threshold: usize,
        /// Custom browser data directory
        #[arg(long)]
        profile_dir: Option<PathBuf>,
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
                browser::list_profiles(&b)?;
            } else {
                browser::list_all_browsers()?;
            }
        }
        Commands::Scan { data_type, output } => {
            export_all_browsers(&data_type, output, None)?;
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
        Commands::Dedupe {
            input,
            output,
            strategy,
            preview,
            backup,
        } => {
            deduplicate_bookmarks(&input, &output, &strategy, preview, backup)?;
        }
        Commands::Organize {
            input,
            output,
            strategy,
            preserve_existing,
            preview,
            backup,
        } => {
            organize_bookmarks(
                &input,
                &output,
                &strategy,
                preserve_existing,
                preview,
                backup,
            )?;
        }
        Commands::Process {
            input,
            output,
            merge_strategy,
            organization_strategy,
            preserve_existing,
            preview,
            backup,
            report,
            config,
        } => {
            process_bookmarks(
                &input,
                &output,
                &merge_strategy,
                &organization_strategy,
                preserve_existing,
                preview,
                backup,
                &report,
                &config,
            )?;
        }
        Commands::Config {
            show,
            create_sample,
            add_rule,
            remove_rule,
            list_rules,
            validate,
            config_file,
        } => {
            handle_config_commands(
                show,
                create_sample,
                add_rule,
                remove_rule,
                list_rules,
                validate,
                config_file,
            )?;
        }
        Commands::Graph {
            browser,
            data_type,
            format,
            output,
            include_folder_edges,
            include_domain_edges,
            include_same_domain_edges,
            min_domain_threshold,
            profile_dir,
        } => {
            generate_knowledge_graph(
                &browser,
                &data_type,
                &format,
                output,
                include_folder_edges,
                include_domain_edges,
                include_same_domain_edges,
                min_domain_threshold,
                profile_dir,
            )?;
        }
    }

    Ok(())
}

fn export_all_browsers(
    data_type: &str,
    output_dir: Option<PathBuf>,
    profile_dir: Option<PathBuf>,
) -> Result<()> {
    let browsers = ["Chrome", "Firefox", "Safari", "Edge"];
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("."));

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    println!("Scanning for available browsers...");
    let mut found_browsers = Vec::new();

    for browser_name in browsers.iter() {
        if let Ok(browser) = Browser::from_str(browser_name) {
            if let Ok(profiles) = browser.find_profiles(profile_dir.as_deref()) {
                if !profiles.is_empty() {
                    found_browsers.push((browser_name.to_string(), profiles.len()));
                }
            }
        }
    }

    if found_browsers.is_empty() {
        println!("No browser profiles found!");
        return Ok(());
    }

    println!("Found {} browsers:", found_browsers.len());
    for (name, profile_count) in &found_browsers {
        println!("  {} ({} profiles)", name, profile_count);
    }
    println!();

    // Export bookmarks from each found browser
    for (browser_name, _) in found_browsers {
        let output_file = output_dir.join(format!(
            "{}-{}.yaml",
            browser_name.to_lowercase(),
            data_type
        ));

        println!(
            "Exporting {} {} to {}",
            browser_name,
            data_type,
            output_file.display()
        );

        match export_data(
            &browser_name,
            &data_type,
            Some(output_file.clone()),
            profile_dir.clone(),
        ) {
            Ok(_) => println!("✓ Successfully exported {}", browser_name),
            Err(e) => {
                if browser_name == "Safari" && e.to_string().contains("protected") {
                    println!(
                        "⚠ {} requires manual copy. See error for instructions.",
                        browser_name
                    );
                } else if browser_name == "Firefox" && e.to_string().contains("locked") {
                    println!("⚠ {} is running. Please close it first.", browser_name);
                } else {
                    println!("✗ Failed to export {}: {}", browser_name, e);
                }
            }
        }
        println!();
    }

    println!("Export complete! Check the output directory for YAML files.");
    Ok(())
}

fn deduplicate_bookmarks(
    input: &PathBuf,
    output: &PathBuf,
    strategy: &str,
    preview: bool,
    backup: bool,
) -> Result<()> {
    println!("Loading bookmarks from {}...", input.display());

    // Load bookmarks from input file
    let content = fs::read_to_string(input)?;
    let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;

    // Collect all bookmarks from all browser data
    let mut all_bookmarks = Vec::new();
    for data in browser_data {
        if let Some(bookmarks) = data.bookmarks {
            all_bookmarks.extend(bookmarks);
        }
    }

    println!("Loaded {} bookmarks", all_bookmarks.len());

    // Parse merge strategy
    let merge_strategy = match strategy {
        "first" => MergeStrategy::KeepFirst,
        "last" => MergeStrategy::KeepLast,
        "recent" => MergeStrategy::KeepMostRecent,
        "frequent" => MergeStrategy::KeepMostFrequent,
        "merge" => MergeStrategy::MergeMetadata,
        _ => return Err(anyhow::anyhow!("Invalid merge strategy: {}", strategy)),
    };

    // Create processing configuration
    let dedup_config = deduplication::DeduplicationConfig {
        merge_strategy,
        ..Default::default()
    };

    let config = ProcessingConfig {
        deduplication_config: dedup_config,
        organization_config: organization::OrganizationConfig {
            organize_by_domain: false,
            organize_by_category: false,
            organize_by_date: false,
            ..Default::default()
        },
        dry_run: preview,
        backup_original: backup,
    };

    // Process bookmarks
    let processor = BookmarkProcessor::new(config);

    if preview {
        processor.preview_changes(&all_bookmarks)?;
        return Ok(());
    }

    let result = processor.process_bookmarks(&all_bookmarks)?;

    println!("Deduplication complete:");
    println!(
        "  Original bookmarks: {}",
        result.processing_summary.original_count
    );
    println!(
        "  Final bookmarks: {}",
        result.processing_summary.final_count
    );
    println!(
        "  Duplicates removed: {}",
        result.processing_summary.duplicates_removed
    );

    // Export processed bookmarks
    processor.export_processed_bookmarks(&result.processed_bookmarks, output)?;

    Ok(())
}

fn organize_bookmarks(
    input: &PathBuf,
    output: &PathBuf,
    strategy: &str,
    preserve_existing: bool,
    preview: bool,
    backup: bool,
) -> Result<()> {
    println!("Loading bookmarks from {}...", input.display());

    // Load bookmarks from input file
    let content = fs::read_to_string(input)?;
    let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;

    // Collect all bookmarks from all browser data
    let mut all_bookmarks = Vec::new();
    for data in browser_data {
        if let Some(bookmarks) = data.bookmarks {
            all_bookmarks.extend(bookmarks);
        }
    }

    println!("Loaded {} bookmarks", all_bookmarks.len());

    // Create organization configuration
    let organization_config = organization::OrganizationConfig {
        organize_by_domain: strategy == "domain" || strategy == "custom",
        organize_by_category: strategy == "category" || strategy == "custom",
        organize_by_date: strategy == "date",
        preserve_existing,
        ..Default::default()
    };

    let config = ProcessingConfig {
        deduplication_config: deduplication::DeduplicationConfig {
            normalize_urls: false, // Don't deduplicate in organize-only mode
            ..Default::default()
        },
        organization_config,
        dry_run: preview,
        backup_original: backup,
    };

    // Process bookmarks
    let processor = BookmarkProcessor::new(config);

    if preview {
        processor.preview_changes(&all_bookmarks)?;
        return Ok(());
    }

    let result = processor.process_bookmarks(&all_bookmarks)?;

    println!("Organization complete:");
    println!(
        "  Bookmarks processed: {}",
        result.processing_summary.final_count
    );
    println!(
        "  Folders created: {}",
        result.processing_summary.folders_created
    );

    // Show folder distribution
    println!("\nFolder distribution:");
    let mut sorted_folders: Vec<_> = result
        .processing_summary
        .folder_distribution
        .iter()
        .collect();
    sorted_folders.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count (descending)

    for (folder, count) in sorted_folders.iter().take(10) {
        println!("  {}: {} bookmarks", folder, count);
    }

    if sorted_folders.len() > 10 {
        println!("  ... and {} more folders", sorted_folders.len() - 10);
    }

    // Export processed bookmarks
    processor.export_processed_bookmarks(&result.processed_bookmarks, output)?;

    Ok(())
}

fn process_bookmarks(
    input: &PathBuf,
    output: &PathBuf,
    merge_strategy: &str,
    organization_strategy: &str,
    preserve_existing: bool,
    preview: bool,
    backup: bool,
    report_path: &Option<PathBuf>,
    config_path: &Option<PathBuf>,
) -> Result<()> {
    println!("Loading bookmarks from {}...", input.display());

    // Load bookmarks from input file
    let content = fs::read_to_string(input)?;
    let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;

    // Collect all bookmarks from all browser data
    let mut all_bookmarks = Vec::new();
    for data in browser_data {
        if let Some(bookmarks) = data.bookmarks {
            all_bookmarks.extend(bookmarks);
        }
    }

    println!("Loaded {} bookmarks", all_bookmarks.len());

    // Parse merge strategy
    let merge_strategy = match merge_strategy {
        "first" => MergeStrategy::KeepFirst,
        "last" => MergeStrategy::KeepLast,
        "recent" => MergeStrategy::KeepMostRecent,
        "frequent" => MergeStrategy::KeepMostFrequent,
        "merge" => MergeStrategy::MergeMetadata,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid merge strategy: {}",
                merge_strategy
            ))
        }
    };

    // Create organization configuration
    let organization_config = organization::OrganizationConfig {
        organize_by_domain: organization_strategy == "domain" || organization_strategy == "custom",
        organize_by_category: organization_strategy == "category"
            || organization_strategy == "custom",
        organize_by_date: organization_strategy == "date",
        preserve_existing,
        ..Default::default()
    };

    // Load config file if provided, otherwise use defaults
    let app_config = if let Some(config_path) = config_path {
        AppConfig::load_from_file(config_path)?
    } else if let Ok(default_config) = AppConfig::load_or_create() {
        default_config
    } else {
        AppConfig::default()
    };

    let config = ProcessingConfig {
        deduplication_config: deduplication::DeduplicationConfig {
            merge_strategy,
            ..app_config.deduplication
        },
        organization_config,
        dry_run: preview || app_config.dry_run_by_default,
        backup_original: backup || app_config.backup_enabled,
    };

    // Process bookmarks
    let processor = BookmarkProcessor::new(config);

    if preview {
        processor.preview_changes(&all_bookmarks)?;
        return Ok(());
    }

    let result = processor.process_bookmarks(&all_bookmarks)?;

    println!("Processing complete:");
    println!(
        "  Original bookmarks: {}",
        result.processing_summary.original_count
    );
    println!(
        "  Final bookmarks: {}",
        result.processing_summary.final_count
    );
    println!(
        "  Duplicates removed: {}",
        result.processing_summary.duplicates_removed
    );
    println!(
        "  Folders created: {}",
        result.processing_summary.folders_created
    );
    println!(
        "  Processing time: {:?}",
        result.processing_summary.processing_time
    );

    // Export processed bookmarks
    processor.export_processed_bookmarks(&result.processed_bookmarks, output)?;

    // Generate report if requested
    if let Some(report_path) = report_path {
        let report = processor.generate_report(&result);
        fs::write(report_path, report)?;
        println!("Detailed report generated: {}", report_path.display());
    }

    Ok(())
}

fn handle_config_commands(
    show: bool,
    create_sample: Option<PathBuf>,
    add_rule: Option<String>,
    remove_rule: Option<String>,
    list_rules: bool,
    validate: bool,
    config_file: Option<PathBuf>,
) -> Result<()> {
    let config_path = config_file.unwrap_or_else(|| AppConfig::get_default_config_path());

    if show {
        let config = AppConfig::load_from_file(&config_path)?;
        println!("Current configuration at {}:\n", config_path.display());
        println!("{}", serde_yaml::to_string(&config)?);
        return Ok(());
    }

    if let Some(sample_path) = create_sample {
        AppConfig::create_sample_config(&sample_path)?;
        println!("Sample configuration created at: {}", sample_path.display());
        return Ok(());
    }

    let mut config = AppConfig::load_from_file(&config_path)?;

    if let Some(rule_json) = add_rule {
        // Parse rule from JSON string
        let rule: OrganizationRule = serde_json::from_str(&rule_json)?;
        config.add_custom_rule(rule);
        config.save_to_file(&config_path)?;
        println!("Custom rule added successfully");
        return Ok(());
    }

    if let Some(rule_name) = remove_rule {
        config.remove_custom_rule(&rule_name)?;
        config.save_to_file(&config_path)?;
        println!("Custom rule '{}' removed successfully", rule_name);
        return Ok(());
    }

    if list_rules {
        let rules = config.list_rules();
        if rules.is_empty() {
            println!("No custom rules configured");
        } else {
            println!("Custom organization rules:");
            for rule in rules {
                println!("  Name: {} (Priority: {})", rule.name, rule.priority);
                println!("  Pattern: {}", rule.pattern);
                println!("  Folder: {}", rule.folder);
                println!();
            }
        }
        return Ok(());
    }

    if validate {
        match config.validate_config() {
            Ok(()) => println!("Configuration is valid"),
            Err(e) => {
                eprintln!("Configuration validation failed: {}", e);
                return Err(e);
            }
        }
        return Ok(());
    }

    // If no specific command, show help
    println!("Config commands available:");
    println!("  --show                    Show current configuration");
    println!("  --create-sample PATH       Create sample configuration file");
    println!("  --add-rule JSON           Add custom rule (JSON format)");
    println!("  --remove-rule NAME        Remove custom rule by name");
    println!("  --list-rules              List all custom rules");
    println!("  --validate                Validate configuration");
    println!("  --config-file PATH        Specify config file path");

    Ok(())
}

fn generate_knowledge_graph(
    browser: &str,
    data_type: &str,
    format: &str,
    output: PathBuf,
    include_folder_edges: bool,
    include_domain_edges: bool,
    include_same_domain_edges: bool,
    min_domain_threshold: usize,
    profile_dir: Option<PathBuf>,
) -> Result<()> {
    println!("Generating knowledge graph...");

    // Step 1: Load data based on browser and data_type
    let temp_file = PathBuf::from("/tmp/bookmark_graph_data.yaml");
    if browser == "all" {
        // Export all browsers to temp file
        let output_dir = Some(PathBuf::from("/tmp"));
        export_all_browsers(data_type, output_dir, profile_dir.clone())?;

        // Load from the exported files
        let mut all_bookmarks = Vec::new();
        let mut all_history = Vec::new();

        let browsers = ["chrome", "firefox", "safari", "edge"];
        for browser_name in browsers.iter() {
            let browser_file = PathBuf::from(format!(
                "/tmp/{}-{}.yaml",
                browser_name.to_lowercase(),
                data_type
            ));
            if browser_file.exists() {
                let content = fs::read_to_string(&browser_file)?;
                let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;

                for data in browser_data {
                    if let Some(bookmarks) = data.bookmarks {
                        all_bookmarks.extend(bookmarks);
                    }
                    if let Some(history) = data.history {
                        all_history.extend(history.urls);
                    }
                }
            }
        }

        // Step 2: Create graph configuration
        let config = graph::GraphConfig {
            include_folder_edges,
            include_domain_edges,
            include_same_domain_edges,
            min_domain_threshold,
        };

        // Step 3: Build graph
        let mut builder = graph::GraphBuilder::new(config);
        let graph = match data_type {
            "bookmarks" => builder.from_bookmarks(&all_bookmarks)?,
            "history" => builder.from_history(&all_history)?,
            "both" => builder.from_both(&all_bookmarks, &all_history)?,
            _ => return Err(anyhow::anyhow!("Invalid data type: {}", data_type)),
        };

        // Step 4: Export in requested format
        let output_content = match format {
            "dot" => graph::formats::to_dot(&graph),
            "json" => graph::formats::to_json(&graph),
            "gexf" => graph::formats::to_gexf(&graph),
            _ => return Err(anyhow::anyhow!("Invalid format: {}", format)),
        };

        // Step 5: Write to file
        fs::write(&output, output_content)?;

        // Step 6: Report statistics
        println!("Graph generated successfully!");
        println!(
            "  Nodes: {} ({} bookmarks, {} domains, {} folders)",
            graph.metadata.total_nodes,
            graph.metadata.bookmark_count,
            graph.metadata.domain_count,
            graph.metadata.folder_count
        );
        println!("  Edges: {}", graph.metadata.total_edges);
        println!("  Output: {}", output.display());
    } else {
        // Single browser
        export_data(browser, data_type, Some(temp_file.clone()), profile_dir)?;

        // Step 2: Parse data
        let content = fs::read_to_string(&temp_file)?;
        let browser_data: Vec<exporter::BrowserData> = serde_yaml::from_str(&content)?;

        // Step 3: Collect bookmarks and history
        let mut all_bookmarks = Vec::new();
        let mut all_history = Vec::new();

        for data in browser_data {
            if let Some(bookmarks) = data.bookmarks {
                all_bookmarks.extend(bookmarks);
            }
            if let Some(history) = data.history {
                all_history.extend(history.urls);
            }
        }

        // Step 4: Create graph configuration
        let config = graph::GraphConfig {
            include_folder_edges,
            include_domain_edges,
            include_same_domain_edges,
            min_domain_threshold,
        };

        // Step 5: Build graph
        let mut builder = graph::GraphBuilder::new(config);
        let graph = match data_type {
            "bookmarks" => builder.from_bookmarks(&all_bookmarks)?,
            "history" => builder.from_history(&all_history)?,
            "both" => builder.from_both(&all_bookmarks, &all_history)?,
            _ => return Err(anyhow::anyhow!("Invalid data type: {}", data_type)),
        };

        // Step 6: Export in requested format
        let output_content = match format {
            "dot" => graph::formats::to_dot(&graph),
            "json" => graph::formats::to_json(&graph),
            "gexf" => graph::formats::to_gexf(&graph),
            _ => return Err(anyhow::anyhow!("Invalid format: {}", format)),
        };

        // Step 7: Write to file
        fs::write(&output, output_content)?;

        // Step 8: Report statistics
        println!("Graph generated successfully!");
        println!(
            "  Nodes: {} ({} bookmarks, {} domains, {} folders)",
            graph.metadata.total_nodes,
            graph.metadata.bookmark_count,
            graph.metadata.domain_count,
            graph.metadata.folder_count
        );
        println!("  Edges: {}", graph.metadata.total_edges);
        println!("  Output: {}", output.display());
    }

    Ok(())
}
