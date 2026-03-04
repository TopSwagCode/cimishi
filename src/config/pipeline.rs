//! Pipeline configuration structures.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{PipelineError, Result};

/// Root configuration for a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Pipeline metadata.
    pub pipeline: PipelineMetadata,

    /// Data sources to fetch from.
    #[serde(default)]
    pub sources: Vec<SourceConfig>,

    /// Processors to apply to fetched data.
    #[serde(default)]
    pub processors: Vec<ProcessorConfig>,

    /// Query configuration.
    pub query: QueryConfig,

    /// Output configuration.
    pub output: OutputConfig,
}

impl PipelineConfig {
    /// Load configuration from a file (TOML, YAML, or JSON).
    /// Format is detected from file extension.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        // Detect format from extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            "yaml" | "yml" => Self::from_yaml(&content),
            "json" => Self::from_json(&content),
            "toml" | _ => Self::from_toml(&content),
        }
    }

    /// Load configuration from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self> {
        let config: PipelineConfig = toml::from_str(content)?;
        Ok(config)
    }

    /// Load configuration from a YAML string.
    pub fn from_yaml(content: &str) -> Result<Self> {
        let config: PipelineConfig = serde_yaml::from_str(content)?;
        Ok(config)
    }

    /// Load configuration from a JSON string.
    pub fn from_json(content: &str) -> Result<Self> {
        let config: PipelineConfig = serde_json::from_str(content)?;
        Ok(config)
    }

    /// Alias for from_toml for backwards compatibility.
    pub fn from_str(content: &str) -> Result<Self> {
        Self::from_toml(content)
    }
}

/// Pipeline metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    /// Name of the pipeline.
    pub name: String,

    /// Whether to run stages in parallel where possible.
    #[serde(default = "default_parallel")]
    pub parallel: bool,

    /// Maximum concurrent operations.
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
}

fn default_parallel() -> bool {
    true
}

fn default_max_concurrent() -> usize {
    10
}

/// Configuration for a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceConfig {
    /// Local filesystem source.
    Local(LocalSourceConfig),

    /// S3-compatible object storage.
    S3(S3SourceConfig),

    /// Azure Blob Storage.
    Azure(AzureSourceConfig),

    /// Google Cloud Storage.
    Gcs(GcsSourceConfig),
}

/// Local filesystem source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSourceConfig {
    /// Path to the directory (for pattern matching).
    #[serde(default)]
    pub path: String,

    /// Explicit list of file paths to fetch.
    /// Use this to specify individual files instead of or in addition to patterns.
    #[serde(default)]
    pub files: Vec<String>,

    /// Glob patterns to match files within the path.
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,

    /// Whether to search recursively.
    #[serde(default = "default_recursive")]
    pub recursive: bool,
}

/// S3 source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3SourceConfig {
    /// S3 bucket name.
    pub bucket: String,

    /// Prefix (folder path) within the bucket.
    #[serde(default)]
    pub prefix: String,

    /// AWS region.
    pub region: String,

    /// Optional endpoint URL (for S3-compatible services like MinIO).
    pub endpoint: Option<String>,

    /// Explicit list of object keys to fetch.
    /// Use this to specify individual files instead of or in addition to patterns.
    #[serde(default)]
    pub files: Vec<String>,

    /// Glob patterns to match files.
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,
}

/// Azure Blob Storage source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSourceConfig {
    /// Storage account name.
    pub account: String,

    /// Container name.
    pub container: String,

    /// Prefix (folder path) within the container.
    #[serde(default)]
    pub prefix: String,

    /// Optional endpoint URL (for Azurite emulator).
    pub endpoint: Option<String>,

    /// Skip request signing (for emulators like Azurite).
    #[serde(default)]
    pub skip_signature: bool,

    /// Explicit list of blob paths to fetch.
    /// Use this to specify individual files instead of or in addition to patterns.
    #[serde(default)]
    pub files: Vec<String>,

    /// Glob patterns to match files.
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,
}

/// Google Cloud Storage source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsSourceConfig {
    /// GCS bucket name.
    pub bucket: String,

    /// Prefix (folder path) within the bucket.
    #[serde(default)]
    pub prefix: String,

    /// Custom endpoint URL (for fake-gcs-server emulator).
    /// When set, credentials are skipped for emulator testing.
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Custom service account key JSON (not needed for emulators).
    pub service_account_key: Option<String>,

    /// Explicit list of object paths to fetch.
    /// Use this to specify individual files instead of or in addition to patterns.
    #[serde(default)]
    pub files: Vec<String>,

    /// Glob patterns to match files.
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,
}

fn default_patterns() -> Vec<String> {
    vec!["*.xml".to_string(), "*.rdf".to_string()]
}

fn default_recursive() -> bool {
    true
}

/// Configuration for a processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProcessorConfig {
    /// Unzip/decompress processor for extracting archives.
    Unzip(UnzipProcessorConfig),

    /// Filter processor for filtering files by pattern.
    Filter(FilterProcessorConfig),
}

/// Unzip/decompress processor configuration.
/// Supports both ZIP archives and GZIP compressed files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnzipProcessorConfig {
    /// Patterns for files to extract (within archives).
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,

    /// ZIP archive patterns to process.
    #[serde(default = "default_archive_patterns")]
    pub archive_patterns: Vec<String>,

    /// GZIP file patterns to decompress.
    #[serde(default = "default_gzip_patterns")]
    pub gzip_patterns: Vec<String>,
}

fn default_archive_patterns() -> Vec<String> {
    vec!["*.zip".to_string()]
}

fn default_gzip_patterns() -> Vec<String> {
    vec!["*.gz".to_string(), "*.gzip".to_string()]
}

/// Filter processor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterProcessorConfig {
    /// Include patterns (files matching any pattern are kept).
    #[serde(default)]
    pub include: Vec<String>,

    /// Exclude patterns (files matching any pattern are removed).
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Query configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// Path to SPARQL query file.
    pub file: Option<String>,

    /// Inline SPARQL query (used if file is not specified).
    pub query: Option<String>,

    /// Base IRI for RDF parsing.
    #[serde(default = "default_base_iri")]
    pub base_iri: String,
}

fn default_base_iri() -> String {
    "http://example.org/".to_string()
}

impl QueryConfig {
    /// Get the SPARQL query string.
    pub fn get_query(&self) -> Result<String> {
        if let Some(ref file) = self.file {
            std::fs::read_to_string(file).map_err(|e| {
                PipelineError::Config(format!("Failed to read query file '{}': {}", file, e))
            })
        } else if let Some(ref query) = self.query {
            Ok(query.clone())
        } else {
            // Default query
            Ok(DEFAULT_QUERY.to_string())
        }
    }
}

/// Default SPARQL query for CIM/IGM data.
pub const DEFAULT_QUERY: &str = r#"
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX cim: <http://iec.ch/TC57/CIM100#>
PREFIX md: <http://iec.ch/TC57/61970-552/ModelDescription/1#>

SELECT ?subject ?predicate ?object
WHERE {
    ?subject ?predicate ?object .
}
LIMIT 100
"#;

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output directory.
    pub dir: String,

    /// Output formats to generate.
    #[serde(default = "default_formats")]
    pub formats: Vec<OutputFormat>,

    /// Whether to write metadata file.
    #[serde(default = "default_metadata")]
    pub metadata: bool,

    /// Custom filename prefix.
    pub prefix: Option<String>,
}

fn default_formats() -> Vec<OutputFormat> {
    vec![OutputFormat::Csv]
}

fn default_metadata() -> bool {
    true
}

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Csv,
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let config_str = r#"
[pipeline]
name = "test-pipeline"
parallel = true

[[sources]]
type = "local"
path = "./input"
patterns = ["*.xml"]

[[processors]]
type = "unzip"
patterns = ["*.xml"]

[query]
file = "./query.sparql"

[output]
dir = "./output"
formats = ["csv", "json"]
metadata = true
"#;

        let config = PipelineConfig::from_str(config_str).unwrap();
        assert_eq!(config.pipeline.name, "test-pipeline");
        assert!(config.pipeline.parallel);
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.processors.len(), 1);
    }
}
