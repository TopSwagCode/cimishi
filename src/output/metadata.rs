//! Metadata output writer.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tracing::info;

use super::{OutputMetadata, OutputWriter};
use crate::config::OutputConfig;
use crate::error::{PipelineError, Result};
use crate::query::QueryOutput;

/// Metadata output writer.
pub struct MetadataWriter;

impl OutputWriter for MetadataWriter {
    fn write(
        &self,
        output: &QueryOutput,
        metadata: &OutputMetadata,
        config: &OutputConfig,
    ) -> Result<Vec<String>> {
        fs::create_dir_all(&config.dir)?;

        let timestamp = metadata.timestamp.format("%Y%m%d_%H%M%S").to_string();
        let prefix = config.prefix.as_deref().unwrap_or("results");
        let filename = format!("{}_{}.metadata", prefix, timestamp);
        let filepath = Path::new(&config.dir).join(&filename);

        let mut file = File::create(&filepath).map_err(|e| {
            PipelineError::Output(format!("Failed to create '{}': {}", filepath.display(), e))
        })?;

        writeln!(file, "pipeline_name: {}", metadata.pipeline_name)?;
        writeln!(
            file,
            "timestamp: {}",
            metadata.timestamp.to_rfc3339()
        )?;
        writeln!(file, "files_loaded: {}", output.files_loaded)?;
        writeln!(file, "triples_loaded: {}", output.triples_loaded)?;
        writeln!(file, "load_time_ms: {}", output.load_time.as_millis())?;
        writeln!(
            file,
            "load_time_secs: {:.3}",
            output.load_time.as_secs_f64()
        )?;
        writeln!(file, "query_time_ms: {}", output.query_time.as_millis())?;
        writeln!(
            file,
            "query_time_secs: {:.3}",
            output.query_time.as_secs_f64()
        )?;
        writeln!(file, "result_count: {}", output.count)?;

        // Memory usage
        if let Some(mem_bytes) = output.peak_memory_bytes {
            writeln!(file, "peak_memory_bytes: {}", mem_bytes)?;
            writeln!(file, "peak_memory_mb: {:.2}", mem_bytes as f64 / 1024.0 / 1024.0)?;
        }

        info!("Metadata written to: {}", filepath.display());
        Ok(vec![filepath.to_string_lossy().to_string()])
    }
}
