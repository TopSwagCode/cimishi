//! Object store source implementation (S3, Azure, GCS).

use async_trait::async_trait;
use futures::TryStreamExt;
use glob::Pattern;
use object_store::aws::AmazonS3Builder;
use object_store::azure::MicrosoftAzureBuilder;
use object_store::gcp::GoogleCloudStorageBuilder;
use object_store::path::Path as ObjectPath;
use object_store::ObjectStore;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::{FetchedFile, Source};
use crate::config::{AzureSourceConfig, GcsSourceConfig, S3SourceConfig};
use crate::error::Result;

/// Source for reading files from object storage (S3, Azure, GCS).
pub struct ObjectStoreSource {
    store: Arc<dyn ObjectStore>,
    prefix: String,
    patterns: Vec<String>,
    files: Vec<String>,
    name: String,
}

impl ObjectStoreSource {
    /// Create an S3 source.
    pub fn s3(config: S3SourceConfig) -> Self {
        let mut builder = AmazonS3Builder::from_env()
            .with_bucket_name(&config.bucket)
            .with_region(&config.region);

        if let Some(ref endpoint) = config.endpoint {
            builder = builder.with_endpoint(endpoint);
        }

        let store = builder.build().expect("Failed to build S3 client");
        let name = if !config.files.is_empty() {
            format!("s3://{}/[{} files]", config.bucket, config.files.len())
        } else {
            format!("s3://{}/{}", config.bucket, config.prefix)
        };

        Self {
            store: Arc::new(store),
            prefix: config.prefix,
            patterns: config.patterns,
            files: config.files,
            name,
        }
    }

    /// Create an Azure Blob Storage source.
    pub fn azure(config: AzureSourceConfig) -> Self {
        let mut builder = MicrosoftAzureBuilder::from_env()
            .with_account(&config.account)
            .with_container_name(&config.container)
            .with_allow_http(true);

        if let Some(ref endpoint) = config.endpoint {
            builder = builder.with_endpoint(endpoint.clone());
        }

        if config.skip_signature {
            builder = builder.with_skip_signature(true);
        }

        let store = builder.build().expect("Failed to build Azure client");

        let name = if !config.files.is_empty() {
            format!("azure://{}/[{} files]", config.container, config.files.len())
        } else {
            format!("azure://{}/{}", config.container, config.prefix)
        };

        Self {
            store: Arc::new(store),
            prefix: config.prefix,
            patterns: config.patterns,
            files: config.files,
            name,
        }
    }

    /// Create a GCS source.
    /// For emulators (fake-gcs-server), set these environment variables:
    /// - STORAGE_EMULATOR_HOST: http://fake-gcs:4443
    /// - GOOGLE_APPLICATION_CREDENTIALS: /path/to/credentials.json
    pub fn gcs(config: GcsSourceConfig) -> Self {
        let mut builder = GoogleCloudStorageBuilder::from_env()
            .with_bucket_name(&config.bucket);

        // Override service account key if provided in config
        if let Some(ref key) = config.service_account_key {
            builder = builder.with_service_account_key(key);
        }

        let store = builder.build().expect("Failed to build GCS client");

        let name = if !config.files.is_empty() {
            format!("gs://{}/[{} files]", config.bucket, config.files.len())
        } else {
            format!("gs://{}/{}", config.bucket, config.prefix)
        };

        Self {
            store: Arc::new(store),
            prefix: config.prefix,
            patterns: config.patterns,
            files: config.files,
            name,
        }
    }

    fn matches_patterns(&self, filename: &str) -> bool {
        if self.patterns.is_empty() {
            return true;
        }

        for pattern in &self.patterns {
            if let Ok(pat) = Pattern::new(pattern) {
                if pat.matches(filename) {
                    return true;
                }
            }
        }
        false
    }

    fn extract_filename(path: &str) -> String {
        path.rsplit('/').next().unwrap_or(path).to_string()
    }

    /// Fetch explicitly listed files by their object keys.
    async fn fetch_explicit_files(&self) -> Result<Vec<FetchedFile>> {
        let mut fetched = Vec::new();

        for file_key in &self.files {
            let path = ObjectPath::from(file_key.as_str());
            let filename = Self::extract_filename(file_key);

            debug!("Fetching explicit file: {}", file_key);

            match self.store.get(&path).await {
                Ok(data) => {
                    let content = data.bytes().await?;
                    fetched.push(FetchedFile {
                        path: file_key.clone(),
                        filename,
                        content,
                        source: self.name.clone(),
                    });
                }
                Err(e) => {
                    warn!("Failed to fetch '{}': {}", file_key, e);
                    // Continue with other files
                }
            }
        }

        if !self.files.is_empty() {
            info!(
                "Fetched {} of {} explicit files from {}",
                fetched.len(),
                self.files.len(),
                self.name
            );
        }

        Ok(fetched)
    }
}

#[async_trait]
impl Source for ObjectStoreSource {
    async fn fetch(&self) -> Result<Vec<FetchedFile>> {
        info!("Fetching from object store: {}", self.name);

        let mut all_files = Vec::new();

        // First, fetch any explicitly listed files
        let explicit_files = self.fetch_explicit_files().await?;
        all_files.extend(explicit_files);

        // Then, if files list is empty or we have a prefix, do pattern-based listing
        // Only skip listing if we have explicit files AND no prefix is set
        let should_list = self.files.is_empty() || !self.prefix.is_empty();

        if should_list {
            let prefix = if self.prefix.is_empty() {
                None
            } else {
                Some(ObjectPath::from(self.prefix.as_str()))
            };

            let list_stream = self.store.list(prefix.as_ref());
            let objects: Vec<_> = list_stream.try_collect().await?;

            for meta in objects {
                let path_str = meta.location.to_string();
                let filename = Self::extract_filename(&path_str);

                if !self.matches_patterns(&filename) {
                    continue;
                }

                // Skip if already fetched via explicit files
                if self.files.contains(&path_str) {
                    continue;
                }

                debug!("Fetching: {}", path_str);
                let data = self.store.get(&meta.location).await?;
                let content = data.bytes().await?;

                all_files.push(FetchedFile {
                    path: path_str,
                    filename,
                    content,
                    source: self.name.clone(),
                });
            }
        }

        info!("Fetched {} files from {}", all_files.len(), self.name);
        Ok(all_files)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
