//! Pipeline orchestration module.

use chrono::Utc;
use futures::future::join_all;
use std::time::Instant;
use tracing::{info, warn};

use crate::config::PipelineConfig;
use crate::error::Result;
use crate::output::{create_writers, OutputMetadata};
use crate::processors::{create_processor, ProcessedFile};
use crate::query::{QueryEngine, SparqlEngine};
use crate::sources::create_source;

/// Pipeline runner that orchestrates all stages.
pub struct Pipeline {
    config: PipelineConfig,
}

impl Pipeline {
    /// Create a new pipeline from configuration.
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Run the complete pipeline.
    pub async fn run(&self) -> Result<()> {
        let start = Instant::now();
        info!("Starting pipeline: {}", self.config.pipeline.name);

        // Stage 1: Fetch from all sources
        info!("Stage 1: Fetching data from {} sources", self.config.sources.len());
        let files = self.fetch_all().await?;
        info!("Fetched {} files total", files.len());

        if files.is_empty() {
            warn!("No files fetched from any source");
            return Ok(());
        }

        // Stage 2: Process through all processors
        info!(
            "Stage 2: Processing through {} processors",
            self.config.processors.len()
        );
        let processed = self.process_all(files).await?;
        info!("Processing complete: {} files", processed.len());

        if processed.is_empty() {
            warn!("No files remaining after processing");
            return Ok(());
        }

        // Stage 3: Load and query
        info!("Stage 3: Loading RDF and executing query");
        let engine = SparqlEngine::new(self.config.query.base_iri.clone());
        let output = engine.execute(processed, &self.config.query)?;

        // Stage 4: Write outputs
        info!("Stage 4: Writing outputs");
        let metadata = OutputMetadata {
            pipeline_name: self.config.pipeline.name.clone(),
            timestamp: Utc::now(),
        };

        let writers = create_writers(&self.config.output);
        let mut output_files = Vec::new();

        for writer in &writers {
            match writer.write(&output, &metadata, &self.config.output) {
                Ok(files) => output_files.extend(files),
                Err(e) => warn!("Output writer failed: {}", e),
            }
        }

        let total_time = start.elapsed();
        info!(
            "Pipeline '{}' completed in {:?}",
            self.config.pipeline.name, total_time
        );
        info!("Output files: {:?}", output_files);

        Ok(())
    }

    /// Fetch from all sources (parallel if enabled).
    async fn fetch_all(&self) -> Result<Vec<ProcessedFile>> {
        if self.config.sources.is_empty() {
            return Ok(Vec::new());
        }

        let sources: Vec<_> = self.config.sources.iter().map(create_source).collect();

        if self.config.pipeline.parallel {
            // Parallel fetch
            let futures: Vec<_> = sources.iter().map(|s| s.fetch()).collect();
            let results = join_all(futures).await;

            let mut all_files = Vec::new();
            for result in results {
                match result {
                    Ok(files) => {
                        all_files.extend(files.into_iter().map(ProcessedFile::from));
                    }
                    Err(e) => {
                        warn!("Source fetch failed: {}", e);
                    }
                }
            }
            Ok(all_files)
        } else {
            // Sequential fetch
            let mut all_files = Vec::new();
            for source in &sources {
                match source.fetch().await {
                    Ok(files) => {
                        all_files.extend(files.into_iter().map(ProcessedFile::from));
                    }
                    Err(e) => {
                        warn!("Source '{}' fetch failed: {}", source.name(), e);
                    }
                }
            }
            Ok(all_files)
        }
    }

    /// Process through all processors in sequence.
    async fn process_all(&self, mut files: Vec<ProcessedFile>) -> Result<Vec<ProcessedFile>> {
        for processor_config in &self.config.processors {
            let processor = create_processor(processor_config);
            info!("Running processor: {}", processor.name());
            files = processor.process(files).await?;
        }
        Ok(files)
    }
}
