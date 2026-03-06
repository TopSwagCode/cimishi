//! Filter processor for filtering files by pattern.

use async_trait::async_trait;
use glob::Pattern;
use tracing::info;

use super::{ProcessedFile, Processor};
use crate::config::FilterProcessorConfig;
use crate::error::Result;

/// Processor that filters files by include/exclude patterns.
pub struct FilterProcessor {
    config: FilterProcessorConfig,
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
}

impl FilterProcessor {
    pub fn new(config: FilterProcessorConfig) -> Self {
        let include_patterns: Vec<Pattern> = config
            .include
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        let exclude_patterns: Vec<Pattern> = config
            .exclude
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        Self {
            config,
            include_patterns,
            exclude_patterns,
        }
    }

    fn should_include(&self, filename: &str) -> bool {
        // If no include patterns, include everything
        let included = if self.include_patterns.is_empty() {
            true
        } else {
            self.include_patterns.iter().any(|p| p.matches(filename))
        };

        // If excluded, reject
        let excluded = self.exclude_patterns.iter().any(|p| p.matches(filename));

        included && !excluded
    }
}

#[async_trait]
impl Processor for FilterProcessor {
    async fn process(&self, files: Vec<ProcessedFile>) -> Result<Vec<ProcessedFile>> {
        let input_count = files.len();

        let result: Vec<ProcessedFile> = files
            .into_iter()
            .filter(|f| self.should_include(&f.filename))
            .collect();

        info!(
            "Filter processor: {} -> {} files (include: {:?}, exclude: {:?})",
            input_count,
            result.len(),
            self.config.include,
            self.config.exclude
        );

        Ok(result)
    }

    fn name(&self) -> &str {
        "filter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn make_file(name: &str) -> ProcessedFile {
        ProcessedFile {
            path: name.to_string(),
            filename: name.to_string(),
            content: Bytes::new(),
            source: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_include_filter() {
        let processor = FilterProcessor::new(FilterProcessorConfig {
            include: vec!["*.xml".to_string()],
            exclude: vec![],
        });

        let files = vec![
            make_file("test.xml"),
            make_file("test.zip"),
            make_file("data.xml"),
        ];

        let result = processor.process(files).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_exclude_filter() {
        let processor = FilterProcessor::new(FilterProcessorConfig {
            include: vec![],
            exclude: vec!["*.zip".to_string()],
        });

        let files = vec![
            make_file("test.xml"),
            make_file("test.zip"),
            make_file("data.xml"),
        ];

        let result = processor.process(files).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_combined_include_exclude() {
        let processor = FilterProcessor::new(FilterProcessorConfig {
            include: vec!["*.xml".to_string(), "*.rdf".to_string()],
            exclude: vec!["*_BD_*".to_string()],
        });

        let files = vec![
            make_file("test_EQ.xml"),
            make_file("test_BD_.xml"),
            make_file("test.rdf"),
            make_file("test.zip"),
        ];

        let result = processor.process(files).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|f| f.filename == "test_EQ.xml"));
        assert!(result.iter().any(|f| f.filename == "test.rdf"));
    }

    #[tokio::test]
    async fn test_empty_patterns_passes_all() {
        let processor = FilterProcessor::new(FilterProcessorConfig {
            include: vec![],
            exclude: vec![],
        });

        let files = vec![
            make_file("data.xml"),
            make_file("image.png"),
            make_file("report.csv"),
        ];

        let result = processor.process(files).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_no_matches_returns_empty() {
        let processor = FilterProcessor::new(FilterProcessorConfig {
            include: vec!["*.csv".to_string()],
            exclude: vec![],
        });

        let files = vec![make_file("test.xml"), make_file("data.xml")];

        let result = processor.process(files).await.unwrap();
        assert_eq!(result.len(), 0);
    }
}
