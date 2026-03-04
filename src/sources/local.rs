//! Local filesystem source implementation.

use async_trait::async_trait;
use bytes::Bytes;
use glob::Pattern;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

use super::{FetchedFile, Source};
use crate::config::LocalSourceConfig;
use crate::error::{PipelineError, Result};

/// Source for reading files from local filesystem.
pub struct LocalSource {
    config: LocalSourceConfig,
    name: String,
}

impl LocalSource {
    pub fn new(config: LocalSourceConfig) -> Self {
        let name = if !config.path.is_empty() {
            format!("local:{}", config.path)
        } else if !config.files.is_empty() {
            format!("local:[{} files]", config.files.len())
        } else {
            "local:empty".to_string()
        };
        Self { config, name }
    }

    fn matches_patterns(&self, filename: &str) -> bool {
        if self.config.patterns.is_empty() {
            return true;
        }

        for pattern in &self.config.patterns {
            if let Ok(pat) = Pattern::new(pattern) {
                if pat.matches(filename) {
                    return true;
                }
            }
        }
        false
    }

    /// Fetch a single file by its explicit path.
    async fn fetch_single_file(&self, file_path: &str) -> Result<Option<FetchedFile>> {
        let path = Path::new(file_path);

        if !path.exists() {
            warn!("Explicit file does not exist: {}", file_path);
            return Ok(None);
        }

        if !path.is_file() {
            warn!("Path is not a file: {}", file_path);
            return Ok(None);
        }

        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string());

        debug!("Reading explicit file: {}", file_path);
        let content = fs::read(path).await.map_err(|e| PipelineError::Source {
            source_name: self.name.clone(),
            message: format!("Failed to read file '{}': {}", file_path, e),
        })?;

        Ok(Some(FetchedFile {
            path: file_path.to_string(),
            filename,
            content: Bytes::from(content),
            source: self.name.clone(),
        }))
    }

    /// Fetch all explicitly listed files.
    async fn fetch_explicit_files(&self) -> Result<Vec<FetchedFile>> {
        let mut files = Vec::new();

        for file_path in &self.config.files {
            if let Some(fetched) = self.fetch_single_file(file_path).await? {
                files.push(fetched);
            }
        }

        if !self.config.files.is_empty() {
            info!(
                "Fetched {} of {} explicit files",
                files.len(),
                self.config.files.len()
            );
        }

        Ok(files)
    }

    async fn fetch_dir(&self, dir: &Path) -> Result<Vec<FetchedFile>> {
        let mut files = Vec::new();

        let mut entries = fs::read_dir(dir).await.map_err(|e| PipelineError::Source {
            source_name: self.name.clone(),
            message: format!("Failed to read directory '{}': {}", dir.display(), e),
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| PipelineError::Source {
            source_name: self.name.clone(),
            message: format!("Failed to read entry: {}", e),
        })? {
            let path = entry.path();
            let metadata = entry.metadata().await.map_err(|e| PipelineError::Source {
                source_name: self.name.clone(),
                message: format!("Failed to get metadata for '{}': {}", path.display(), e),
            })?;

            if metadata.is_file() {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if self.matches_patterns(&filename) {
                    debug!("Reading file: {}", path.display());
                    let content = fs::read(&path).await.map_err(|e| PipelineError::Source {
                        source_name: self.name.clone(),
                        message: format!("Failed to read file '{}': {}", path.display(), e),
                    })?;

                    files.push(FetchedFile {
                        path: path.to_string_lossy().to_string(),
                        filename,
                        content: Bytes::from(content),
                        source: self.name.clone(),
                    });
                }
            } else if metadata.is_dir() && self.config.recursive {
                let sub_files = Box::pin(self.fetch_dir(&path)).await?;
                files.extend(sub_files);
            }
        }

        Ok(files)
    }
}

#[async_trait]
impl Source for LocalSource {
    async fn fetch(&self) -> Result<Vec<FetchedFile>> {
        let mut all_files = Vec::new();

        // First, fetch any explicitly listed files
        let explicit_files = self.fetch_explicit_files().await?;
        all_files.extend(explicit_files);

        // Then, if a path is specified, fetch from it
        if !self.config.path.is_empty() {
            let path = Path::new(&self.config.path);

            if !path.exists() {
                return Err(PipelineError::Source {
                    source_name: self.name.clone(),
                    message: format!("Path does not exist: {}", self.config.path),
                });
            }

            info!("Fetching from local path: {}", self.config.path);

            if path.is_file() {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let content = fs::read(path).await.map_err(|e| PipelineError::Source {
                    source_name: self.name.clone(),
                    message: format!("Failed to read file: {}", e),
                })?;

                all_files.push(FetchedFile {
                    path: self.config.path.clone(),
                    filename,
                    content: Bytes::from(content),
                    source: self.name.clone(),
                });
            } else {
                let dir_files = self.fetch_dir(path).await?;
                all_files.extend(dir_files);
            }
        }

        // Check if we got anything
        if all_files.is_empty() && self.config.files.is_empty() && self.config.path.is_empty() {
            warn!("No path or files specified for local source");
        }

        Ok(all_files)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
