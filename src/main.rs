//! RDF Query Pipeline CLI
//!
//! A modular pipeline for fetching, processing, and querying RDF data.

use cimishi::{Pipeline, PipelineConfig};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// RDF Query Pipeline - A modular tool for processing and querying RDF data.
#[derive(Parser, Debug)]
#[command(name = "cimishi")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the query pipeline with a given configuration.
    Query {
        /// Path to the pipeline configuration file (TOML format).
        #[arg(short, long, default_value = "pipeline.toml")]
        config: PathBuf,

        /// Enable verbose output (debug logging).
        #[arg(short, long)]
        verbose: bool,
    },

    /// Compare query results (not yet implemented).
    Compare,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query { config, verbose } => {
            // Initialize logging
            let filter = if verbose {
                EnvFilter::new("debug")
            } else {
                EnvFilter::new("info")
            };

            fmt().with_env_filter(filter).with_target(false).init();

            info!("RDF Query Pipeline v{}", env!("CARGO_PKG_VERSION"));

            // Load configuration
            let config = if config.exists() {
                info!("Loading config from: {}", config.display());
                PipelineConfig::from_file(&config)?
            } else {
                info!("Config file not found, using defaults for /input -> /output");
                default_config()
            };

            // Run pipeline
            let pipeline = Pipeline::new(config);
            pipeline.run().await?;
        }
        Commands::Compare => {
            eprintln!("Error: compare is not implemented");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Create a default configuration for Docker usage (backward compatible).
fn default_config() -> PipelineConfig {
    use cimishi::config::*;

    PipelineConfig {
        pipeline: PipelineMetadata {
            name: "default".to_string(),
            parallel: true,
            max_concurrent: 10,
        },
        sources: vec![SourceConfig::Local(LocalSourceConfig {
            path: "/input".to_string(),
            files: vec![],
            patterns: vec!["*.xml".to_string(), "*.rdf".to_string()],
            recursive: true,
        })],
        processors: vec![],
        query: QueryConfig {
            file: Some("/input/query.sparql".to_string()),
            query: None,
            base_iri: "http://example.org/".to_string(),
        },
        output: OutputConfig {
            dir: "/output".to_string(),
            formats: vec![OutputFormat::Csv],
            metadata: true,
            prefix: Some("results".to_string()),
        },
    }
}
