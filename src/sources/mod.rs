//! Data source module - trait and implementations for fetching data.

mod local;
mod object_store_source;

pub use local::LocalSource;
pub use object_store_source::ObjectStoreSource;

use async_trait::async_trait;
use bytes::Bytes;

use crate::config::SourceConfig;
use crate::error::Result;

/// A file fetched from a source.
#[derive(Debug, Clone)]
pub struct FetchedFile {
    /// Original path/key of the file.
    pub path: String,

    /// Filename (without directory).
    pub filename: String,

    /// File content as bytes.
    pub content: Bytes,

    /// Source name that provided this file.
    pub source: String,
}

/// Trait for data sources that fetch files.
#[async_trait]
pub trait Source: Send + Sync {
    /// Fetch all matching files from the source.
    async fn fetch(&self) -> Result<Vec<FetchedFile>>;

    /// Get the name of this source for logging/identification.
    fn name(&self) -> &str;
}

/// Create a source from configuration.
pub fn create_source(config: &SourceConfig) -> Box<dyn Source> {
    match config {
        SourceConfig::Local(cfg) => Box::new(LocalSource::new(cfg.clone())),
        SourceConfig::S3(cfg) => Box::new(ObjectStoreSource::s3(cfg.clone())),
        SourceConfig::Azure(cfg) => Box::new(ObjectStoreSource::azure(cfg.clone())),
        SourceConfig::Gcs(cfg) => Box::new(ObjectStoreSource::gcs(cfg.clone())),
    }
}
