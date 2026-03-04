//! Processor module - trait and implementations for transforming data.

mod filter;
mod unzip;

pub use filter::FilterProcessor;
pub use unzip::UnzipProcessor;

use async_trait::async_trait;
use bytes::Bytes;

use crate::config::ProcessorConfig;
use crate::error::Result;

/// A file after processing.
#[derive(Debug, Clone)]
pub struct ProcessedFile {
    /// Path/identifier of the file.
    pub path: String,

    /// Filename (without directory).
    pub filename: String,

    /// File content as bytes.
    pub content: Bytes,

    /// Source that originally provided this file.
    pub source: String,
}

impl From<crate::sources::FetchedFile> for ProcessedFile {
    fn from(f: crate::sources::FetchedFile) -> Self {
        Self {
            path: f.path,
            filename: f.filename,
            content: f.content,
            source: f.source,
        }
    }
}

/// Trait for processors that transform files.
#[async_trait]
pub trait Processor: Send + Sync {
    /// Process files, potentially transforming or filtering them.
    async fn process(&self, files: Vec<ProcessedFile>) -> Result<Vec<ProcessedFile>>;

    /// Get the name of this processor for logging/identification.
    fn name(&self) -> &str;
}

/// Create a processor from configuration.
pub fn create_processor(config: &ProcessorConfig) -> Box<dyn Processor> {
    match config {
        ProcessorConfig::Unzip(cfg) => Box::new(UnzipProcessor::new(cfg.clone())),
        ProcessorConfig::Filter(cfg) => Box::new(FilterProcessor::new(cfg.clone())),
    }
}
