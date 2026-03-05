//! Output module for writing results.

mod csv;
mod json;
mod metadata;
mod terminal;

pub use self::csv::CsvWriter;
pub use self::json::JsonWriter;
pub use self::metadata::MetadataWriter;
pub use self::terminal::TerminalWriter;

use crate::config::{OutputConfig, OutputFormat};
use crate::error::Result;
use crate::query::QueryOutput;

/// Metadata for output writing.
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    pub pipeline_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for output writers.
pub trait OutputWriter: Send + Sync {
    /// Write query results to output.
    fn write(
        &self,
        output: &QueryOutput,
        metadata: &OutputMetadata,
        config: &OutputConfig,
    ) -> Result<Vec<String>>;
}

/// Create output writers from configuration.
pub fn create_writers(config: &OutputConfig) -> Vec<Box<dyn OutputWriter>> {
    let mut writers: Vec<Box<dyn OutputWriter>> = config
        .formats
        .iter()
        .map(|format| -> Box<dyn OutputWriter> {
            match format {
                OutputFormat::Csv => Box::new(CsvWriter),
                OutputFormat::Json => Box::new(JsonWriter),
                OutputFormat::Terminal => Box::new(TerminalWriter),
            }
        })
        .collect();

    if config.metadata {
        writers.push(Box::new(MetadataWriter));
    }

    writers
}
