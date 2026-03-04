//! Custom error types for the RDF query pipeline.

use thiserror::Error;

/// Main error type for pipeline operations.
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Source error: {source_name} - {message}")]
    Source {
        source_name: String,
        message: String,
    },

    #[error("Processor error: {processor_name} - {message}")]
    Processor {
        processor_name: String,
        message: String,
    },

    #[error("Query error: {0}")]
    Query(String),

    #[error("Output error: {0}")]
    Output(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RDF parsing error: {0}")]
    RdfParse(String),

    #[error("SPARQL error: {0}")]
    Sparql(String),

    #[error("Object store error: {0}")]
    ObjectStore(#[from] object_store::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Zip error: {0}")]
    Zip(String),
}

/// Result type alias for pipeline operations.
pub type Result<T> = std::result::Result<T, PipelineError>;
