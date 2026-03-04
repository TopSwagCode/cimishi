//! RDF Query Pipeline CLI
//!
//! A modular pipeline for fetching, processing, and querying RDF data.

use clap::Parser;
use cimishi::{Pipeline, PipelineConfig};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// RDF Query Pipeline - A modular tool for processing and querying RDF data.
#[derive(Parser, Debug)]
#[command(name = "cimishi")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the pipeline configuration file (TOML format).
    #[arg(short, long, default_value = "pipeline.toml")]
    config: PathBuf,

    /// Enable verbose output (debug logging).
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!("RDF Query Pipeline v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = if cli.config.exists() {
        info!("Loading config from: {}", cli.config.display());
        PipelineConfig::from_file(&cli.config)?
    } else {
        info!("Config file not found, using defaults for /input -> /output");
        default_config()
    };

    // Run pipeline
    let pipeline = Pipeline::new(config);
    pipeline.run().await?;

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
