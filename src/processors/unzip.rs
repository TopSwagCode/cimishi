//! Decompress processor for extracting archives (ZIP and GZIP).

use async_compression::tokio::bufread::GzipDecoder;
use async_trait::async_trait;
use async_zip::tokio::read::seek::ZipFileReader;
use bytes::Bytes;
use futures::io::AsyncReadExt;
use glob::Pattern;
use std::io::Cursor;
use tokio::io::AsyncReadExt as TokioAsyncReadExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, info, warn};

use super::{ProcessedFile, Processor};
use crate::config::UnzipProcessorConfig;
use crate::error::{PipelineError, Result};

/// Processor that extracts files from ZIP and GZIP archives in-memory.
pub struct UnzipProcessor {
    config: UnzipProcessorConfig,
}

impl UnzipProcessor {
    pub fn new(config: UnzipProcessorConfig) -> Self {
        Self { config }
    }

    /// Check if file matches ZIP archive patterns.
    fn is_zip_archive(&self, filename: &str) -> bool {
        for pattern in &self.config.archive_patterns {
            if let Ok(pat) = Pattern::new(pattern) {
                if pat.matches(filename) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if file matches GZIP patterns.
    fn is_gzip_file(&self, filename: &str) -> bool {
        for pattern in &self.config.gzip_patterns {
            if let Ok(pat) = Pattern::new(pattern) {
                if pat.matches(filename) {
                    return true;
                }
            }
        }
        // Also check by extension as fallback
        filename.ends_with(".gz") || filename.ends_with(".gzip")
    }

    fn matches_extract_pattern(&self, filename: &str) -> bool {
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

    /// Decompress a GZIP file in-memory.
    async fn decompress_gzip(&self, file: &ProcessedFile) -> Result<ProcessedFile> {
        debug!("Decompressing GZIP: {}", file.filename);

        let cursor = Cursor::new(file.content.to_vec());
        let reader = tokio::io::BufReader::new(cursor);
        let mut decoder = GzipDecoder::new(reader);

        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).await.map_err(|e| {
            PipelineError::Zip(format!(
                "Failed to decompress gzip '{}': {}",
                file.filename, e
            ))
        })?;

        // Remove .gz extension from filename
        let decompressed_name = file
            .filename
            .strip_suffix(".gz")
            .or_else(|| file.filename.strip_suffix(".gzip"))
            .unwrap_or(&file.filename)
            .to_string();

        debug!(
            "Decompressed GZIP: {} -> {} ({} bytes -> {} bytes)",
            file.filename,
            decompressed_name,
            file.content.len(),
            decompressed.len()
        );

        Ok(ProcessedFile {
            path: format!("{}:decompressed", file.path),
            filename: decompressed_name,
            content: Bytes::from(decompressed),
            source: file.source.clone(),
        })
    }

    /// Extract files from a ZIP archive in-memory.
    async fn extract_zip(&self, file: &ProcessedFile) -> Result<Vec<ProcessedFile>> {
        let cursor = Cursor::new(file.content.to_vec());
        // Wrap with tokio's async support, then convert to futures-compatible
        let reader = tokio::io::BufReader::new(cursor).compat();

        let mut zip = ZipFileReader::new(reader).await.map_err(|e| {
            PipelineError::Zip(format!("Failed to open zip '{}': {}", file.filename, e))
        })?;

        let mut extracted = Vec::new();
        let entries_count = zip.file().entries().len();

        for i in 0..entries_count {
            // Get entry info and convert to owned strings before mutably borrowing zip
            let (entry_filename, simple_name) =
                {
                    let entry = zip.file().entries().get(i).ok_or_else(|| {
                        PipelineError::Zip(format!("Entry {} not found in zip", i))
                    })?;

                    let filename = entry.filename().as_str().map_err(|e| {
                        PipelineError::Zip(format!("Invalid filename in zip: {}", e))
                    })?;

                    // Skip directories
                    if filename.ends_with('/') {
                        continue;
                    }

                    // Extract just the filename without path
                    let simple_name = filename.rsplit('/').next().unwrap_or(filename).to_string();

                    (filename.to_string(), simple_name)
                };

            if !self.matches_extract_pattern(&simple_name) {
                continue;
            }

            debug!("Extracting: {} from {}", simple_name, file.filename);

            let mut entry_reader = zip.reader_with_entry(i).await.map_err(|e| {
                PipelineError::Zip(format!("Failed to read entry '{}': {}", entry_filename, e))
            })?;

            let mut content = Vec::new();
            entry_reader.read_to_end(&mut content).await.map_err(|e| {
                PipelineError::Zip(format!("Failed to extract '{}': {}", entry_filename, e))
            })?;

            extracted.push(ProcessedFile {
                path: format!("{}:{}", file.path, entry_filename),
                filename: simple_name.clone(),
                content: Bytes::from(content),
                source: file.source.clone(),
            });

            // Check if extracted file is a GZIP and decompress it too
            if self.is_gzip_file(&simple_name) {
                let gzip_file = extracted.pop().unwrap();
                match self.decompress_gzip(&gzip_file).await {
                    Ok(decompressed) => {
                        extracted.push(decompressed);
                    }
                    Err(e) => {
                        warn!("Failed to decompress nested gzip '{}': {}", simple_name, e);
                        extracted.push(gzip_file); // Keep original on failure
                    }
                }
            }
        }

        Ok(extracted)
    }
}

#[async_trait]
impl Processor for UnzipProcessor {
    async fn process(&self, files: Vec<ProcessedFile>) -> Result<Vec<ProcessedFile>> {
        info!(
            "Decompress processor: processing {} files (ZIP patterns: {:?}, GZIP patterns: {:?})",
            files.len(),
            self.config.archive_patterns,
            self.config.gzip_patterns
        );

        let mut result = Vec::new();
        let mut zip_archives_processed = 0;
        let mut gzip_files_processed = 0;
        let mut files_extracted = 0;

        for file in files {
            if self.is_zip_archive(&file.filename) {
                // Handle ZIP archive
                match self.extract_zip(&file).await {
                    Ok(extracted) => {
                        zip_archives_processed += 1;
                        files_extracted += extracted.len();
                        result.extend(extracted);
                    }
                    Err(e) => {
                        warn!("Failed to extract ZIP '{}': {}", file.filename, e);
                        // Continue processing other files
                    }
                }
            } else if self.is_gzip_file(&file.filename) {
                // Handle GZIP file
                match self.decompress_gzip(&file).await {
                    Ok(decompressed) => {
                        gzip_files_processed += 1;

                        // Check if decompressed file matches extract patterns
                        if self.matches_extract_pattern(&decompressed.filename) {
                            result.push(decompressed);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to decompress GZIP '{}': {}", file.filename, e);
                        // Continue processing other files
                    }
                }
            } else {
                // Pass through non-archive files
                result.push(file);
            }
        }

        info!(
            "Decompress processor: {} ZIP archives ({} files extracted), {} GZIP files decompressed, {} total output files",
            zip_archives_processed,
            files_extracted,
            gzip_files_processed,
            result.len()
        );

        Ok(result)
    }

    fn name(&self) -> &str {
        "decompress"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnzipProcessorConfig;
    use bytes::Bytes;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn default_processor() -> UnzipProcessor {
        UnzipProcessor::new(UnzipProcessorConfig {
            patterns: vec!["*.xml".to_string(), "*.rdf".to_string()],
            archive_patterns: vec!["*.zip".to_string()],
            gzip_patterns: vec!["*.gz".to_string(), "*.gzip".to_string()],
        })
    }

    #[test]
    fn test_is_zip_archive() {
        let processor = default_processor();
        assert!(processor.is_zip_archive("data.zip"));
        assert!(!processor.is_zip_archive("data.xml"));
    }

    #[test]
    fn test_is_gzip_file() {
        let processor = default_processor();
        assert!(processor.is_gzip_file("data.gz"));
        assert!(processor.is_gzip_file("data.gzip"));
        assert!(!processor.is_gzip_file("data.xml"));
    }

    #[test]
    fn test_matches_extract_pattern() {
        let config = UnzipProcessorConfig {
            patterns: vec!["*.xml".to_string()],
            archive_patterns: vec!["*.zip".to_string()],
            gzip_patterns: vec!["*.gz".to_string(), "*.gzip".to_string()],
        };
        let processor = UnzipProcessor::new(config);
        assert!(processor.matches_extract_pattern("test.xml"));
        assert!(!processor.matches_extract_pattern("test.zip"));
    }

    #[test]
    fn test_matches_extract_pattern_empty() {
        let config = UnzipProcessorConfig {
            patterns: vec![],
            archive_patterns: vec!["*.zip".to_string()],
            gzip_patterns: vec!["*.gz".to_string(), "*.gzip".to_string()],
        };
        let processor = UnzipProcessor::new(config);
        assert!(processor.matches_extract_pattern("anything.txt"));
        assert!(processor.matches_extract_pattern("test.xml"));
        assert!(processor.matches_extract_pattern("archive.zip"));
    }

    #[tokio::test]
    async fn test_gzip_decompression() {
        let original_content = b"<rdf:RDF>hello world</rdf:RDF>";

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original_content).unwrap();
        let compressed = encoder.finish().unwrap();

        let file = ProcessedFile {
            path: "test/path".to_string(),
            filename: "test.xml.gz".to_string(),
            content: Bytes::from(compressed),
            source: "test-source".to_string(),
        };

        let processor = default_processor();
        let result = processor.process(vec![file]).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].filename, "test.xml");
        assert_eq!(result[0].content, Bytes::from(&original_content[..]));
    }

    #[tokio::test]
    async fn test_passthrough_non_archive() {
        let content = b"plain xml content";
        let file = ProcessedFile {
            path: "some/path".to_string(),
            filename: "plain.xml".to_string(),
            content: Bytes::from(&content[..]),
            source: "test-source".to_string(),
        };

        let processor = default_processor();
        let result = processor.process(vec![file]).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].filename, "plain.xml");
        assert_eq!(result[0].content, Bytes::from(&content[..]));
    }
}
