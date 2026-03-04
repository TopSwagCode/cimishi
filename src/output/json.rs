//! JSON output writer.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tracing::info;

use super::{OutputMetadata, OutputWriter};
use crate::config::OutputConfig;
use crate::error::{PipelineError, Result};
use crate::query::{QueryOutput, QueryResultsData};

/// JSON output writer.
pub struct JsonWriter;

impl OutputWriter for JsonWriter {
    fn write(
        &self,
        output: &QueryOutput,
        metadata: &OutputMetadata,
        config: &OutputConfig,
    ) -> Result<Vec<String>> {
        fs::create_dir_all(&config.dir)?;

        let timestamp = metadata.timestamp.format("%Y%m%d_%H%M%S").to_string();
        let prefix = config.prefix.as_deref().unwrap_or("results");
        let filename = format!("{}_{}.json", prefix, timestamp);
        let filepath = Path::new(&config.dir).join(&filename);

        let mut file = File::create(&filepath).map_err(|e| {
            PipelineError::Output(format!("Failed to create '{}': {}", filepath.display(), e))
        })?;

        // Simple JSON serialization without serde_json
        match &output.results {
            QueryResultsData::Solutions { variables, rows } => {
                writeln!(file, "{{")?;
                writeln!(file, "  \"type\": \"solutions\",")?;
                writeln!(file, "  \"variables\": {:?},", variables)?;
                writeln!(file, "  \"results\": [")?;

                for (i, row) in rows.iter().enumerate() {
                    write!(file, "    {{")?;
                    for (j, (var, val)) in variables.iter().zip(row.iter()).enumerate() {
                        let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                        if j > 0 {
                            write!(file, ", ")?;
                        }
                        write!(file, "\"{}\": \"{}\"", var, escaped)?;
                    }
                    if i < rows.len() - 1 {
                        writeln!(file, "}},")?;
                    } else {
                        writeln!(file, "}}")?;
                    }
                }

                writeln!(file, "  ],")?;
                writeln!(file, "  \"count\": {}", output.count)?;
                writeln!(file, "}}")?;
            }
            QueryResultsData::Boolean(result) => {
                writeln!(file, "{{")?;
                writeln!(file, "  \"type\": \"boolean\",")?;
                writeln!(file, "  \"result\": {}", result)?;
                writeln!(file, "}}")?;
            }
            QueryResultsData::Graph { triples } => {
                writeln!(file, "{{")?;
                writeln!(file, "  \"type\": \"graph\",")?;
                writeln!(file, "  \"triples\": [")?;

                for (i, (s, p, o)) in triples.iter().enumerate() {
                    let s_esc = s.replace('\\', "\\\\").replace('"', "\\\"");
                    let p_esc = p.replace('\\', "\\\\").replace('"', "\\\"");
                    let o_esc = o.replace('\\', "\\\\").replace('"', "\\\"");

                    if i < triples.len() - 1 {
                        writeln!(
                            file,
                            "    {{\"subject\": \"{}\", \"predicate\": \"{}\", \"object\": \"{}\"}}",
                            s_esc, p_esc, o_esc
                        )?;
                    } else {
                        writeln!(
                            file,
                            "    {{\"subject\": \"{}\", \"predicate\": \"{}\", \"object\": \"{}\"}}",
                            s_esc, p_esc, o_esc
                        )?;
                    }
                }

                writeln!(file, "  ],")?;
                writeln!(file, "  \"count\": {}", output.count)?;
                writeln!(file, "}}")?;
            }
        }

        info!("JSON written to: {}", filepath.display());
        Ok(vec![filepath.to_string_lossy().to_string()])
    }
}
