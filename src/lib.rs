//! RDF Query Pipeline Library
//!
//! A modular, async pipeline for fetching, processing, and querying RDF data.
//!
//! # Architecture
//!
//! The pipeline consists of four stages:
//! 1. **Sources** - Fetch data from various locations (local, S3, Azure, GCS)
//! 2. **Processors** - Transform data (unzip, filter, etc.)
//! 3. **Query** - Load RDF and execute SPARQL queries
//! 4. **Output** - Write results (CSV, JSON, metadata)
//!
//! # Example
//!
//! ```rust,no_run
//! use rdf_query::{PipelineConfig, Pipeline};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = PipelineConfig::from_file("pipeline.toml")?;
//!     let pipeline = Pipeline::new(config);
//!     pipeline.run().await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod output;
pub mod pipeline;
pub mod processors;
pub mod query;
pub mod sources;

// Re-export main types for convenience
pub use config::PipelineConfig;
pub use error::{PipelineError, Result};
pub use pipeline::Pipeline;
