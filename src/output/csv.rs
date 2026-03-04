//! CSV output writer.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tracing::info;

use super::{OutputMetadata, OutputWriter};
use crate::config::OutputConfig;
use crate::error::{PipelineError, Result};
use crate::query::{QueryOutput, QueryResultsData};

/// CSV output writer.
pub struct CsvWriter;

impl OutputWriter for CsvWriter {
    fn write(
        &self,
        output: &QueryOutput,
        metadata: &OutputMetadata,
        config: &OutputConfig,
    ) -> Result<Vec<String>> {
        fs::create_dir_all(&config.dir)?;

        let timestamp = metadata.timestamp.format("%Y%m%d_%H%M%S").to_string();
        let prefix = config.prefix.as_deref().unwrap_or("results");
        let filename = format!("{}_{}.csv", prefix, timestamp);
        let filepath = Path::new(&config.dir).join(&filename);

        let mut file = File::create(&filepath).map_err(|e| {
            PipelineError::Output(format!("Failed to create '{}': {}", filepath.display(), e))
        })?;

        match &output.results {
            QueryResultsData::Solutions { variables, rows } => {
                // Write header
                writeln!(file, "{}", variables.join(","))?;

                // Write rows
                for row in rows {
                    let escaped: Vec<String> = row
                        .iter()
                        .map(|cell| {
                            if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                                format!("\"{}\"", cell.replace('"', "\"\""))
                            } else {
                                cell.clone()
                            }
                        })
                        .collect();
                    writeln!(file, "{}", escaped.join(","))?;
                }
            }
            QueryResultsData::Boolean(result) => {
                writeln!(file, "result")?;
                writeln!(file, "{}", result)?;
            }
            QueryResultsData::Graph { triples } => {
                writeln!(file, "subject,predicate,object")?;
                for (s, p, o) in triples {
                    writeln!(file, "{},{},{}", s, p, o)?;
                }
            }
        }

        info!("CSV written to: {}", filepath.display());
        Ok(vec![filepath.to_string_lossy().to_string()])
    }
}
