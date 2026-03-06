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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = PipelineError::Config("bad config".to_string());
        assert_eq!(err.to_string(), "Configuration error: bad config");
    }

    #[test]
    fn test_source_error_display() {
        let err = PipelineError::Source {
            source_name: "s3:bucket".to_string(),
            message: "connection failed".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Source error: s3:bucket - connection failed"
        );
    }

    #[test]
    fn test_processor_error_display() {
        let err = PipelineError::Processor {
            processor_name: "unzip".to_string(),
            message: "corrupt archive".to_string(),
        };
        assert_eq!(err.to_string(), "Processor error: unzip - corrupt archive");
    }

    #[test]
    fn test_query_error_display() {
        let err = PipelineError::Query("timeout".to_string());
        assert_eq!(err.to_string(), "Query error: timeout");
    }

    #[test]
    fn test_output_error_display() {
        let err = PipelineError::Output("disk full".to_string());
        assert_eq!(err.to_string(), "Output error: disk full");
    }

    #[test]
    fn test_rdf_parse_error_display() {
        let err = PipelineError::RdfParse("invalid XML".to_string());
        assert_eq!(err.to_string(), "RDF parsing error: invalid XML");
    }

    #[test]
    fn test_sparql_error_display() {
        let err = PipelineError::Sparql("syntax error".to_string());
        assert_eq!(err.to_string(), "SPARQL error: syntax error");
    }

    #[test]
    fn test_zip_error_display() {
        let err = PipelineError::Zip("bad archive".to_string());
        assert_eq!(err.to_string(), "Zip error: bad archive");
    }
}
