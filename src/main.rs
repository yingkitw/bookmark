use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod browser;
mod cli;
mod config;
mod deduplication;
mod exporter;
mod graph;
mod organization;
mod processor;
mod search;

use exporter::export_data;
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
        /// Output format (dot, json, gexf, html)
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
        /// Minimum bookmarks for domain node (default: 5)
        #[arg(long, default_value = "5")]
        min_threshold: usize,
        /// Detail level: overview, standard, detailed
        #[arg(long, default_value = "standard")]
        detail: String,
        /// Maximum bookmarks per domain (for standard detail level)
        #[arg(long)]
        max_per_domain: Option<usize>,
        /// Maximum total bookmarks
        #[arg(long)]
        max_total: Option<usize>,
        /// Domain-only mode (no individual bookmark nodes)
        #[arg(long)]
        domain_only: bool,
        /// Only include bookmarks newer than this date (ISO 8601 format)
        #[arg(long)]
        since: Option<String>,
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
    let args = Cli::parse();

    match args.command {
        Commands::Export {
            browser,
            data_type,
            output,
            profile_dir,
        } => {
            if browser == "all" {
                cli::export_all_browsers(&data_type, output, profile_dir)?;
            } else {
                export_data(&browser, &data_type, output, profile_dir)?;
            }
        }

        Commands::List { browser } => {
            if let Some(b) = browser {
                cli::list_browser_profiles(&b)?;
            } else {
                cli::list_all_browsers()?;
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
            cli::process_bookmarks(&input, &output, &mode, &strategy, &org_strategy, preview, backup)?;
        }

        Commands::Graph {
            browser,
            data_type,
            format,
            output,
            min_threshold,
            detail,
            max_per_domain,
            max_total,
            domain_only,
            since,
        } => {
            cli::generate_graph(&browser, &data_type, &format, output, min_threshold, &detail, max_per_domain, max_total, domain_only, since)?;
        }

        Commands::Config {
            show,
            create_sample,
            list_rules,
        } => {
            cli::handle_config(show, create_sample, list_rules)?;
        }
    }

    Ok(())
}
