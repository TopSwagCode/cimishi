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
                    let comma = if i < triples.len() - 1 { "," } else { "" };

                    writeln!(
                        file,
                        "    {{\"subject\": \"{}\", \"predicate\": \"{}\", \"object\": \"{}\"}}{}",
                        s_esc, p_esc, o_esc, comma
                    )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{OutputConfig, OutputFormat};
    use crate::output::OutputMetadata;
    use crate::query::{QueryOutput, QueryResultsData};
    use std::time::Duration;
    use tempfile::TempDir;

    fn make_test_output(results: QueryResultsData, count: usize) -> QueryOutput {
        QueryOutput {
            results,
            count,
            load_time: Duration::from_millis(100),
            query_time: Duration::from_millis(50),
            files_loaded: 1,
            triples_loaded: 10,
            peak_memory_bytes: Some(1024),
        }
    }

    fn make_test_metadata() -> OutputMetadata {
        OutputMetadata {
            pipeline_name: "test".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    fn make_test_config(dir: &str) -> OutputConfig {
        OutputConfig {
            dir: dir.to_string(),
            formats: vec![OutputFormat::Json],
            metadata: false,
            prefix: Some("test".to_string()),
        }
    }

    #[test]
    fn test_json_solutions_output() {
        let tmp_dir = TempDir::new().unwrap();
        let config = make_test_config(tmp_dir.path().to_str().unwrap());
        let metadata = make_test_metadata();

        let output = make_test_output(
            QueryResultsData::Solutions {
                variables: vec!["name".to_string(), "value".to_string()],
                rows: vec![
                    vec!["alice".to_string(), "42".to_string()],
                    vec!["bob".to_string(), "99".to_string()],
                ],
            },
            2,
        );

        let writer = JsonWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["type"], "solutions");
        assert_eq!(parsed["count"], 2);
        assert!(parsed["results"].is_array());
        assert_eq!(parsed["results"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_json_boolean_output() {
        let tmp_dir = TempDir::new().unwrap();
        let config = make_test_config(tmp_dir.path().to_str().unwrap());
        let metadata = make_test_metadata();

        let output = make_test_output(QueryResultsData::Boolean(false), 1);

        let writer = JsonWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["type"], "boolean");
        assert_eq!(parsed["result"], false);
    }

    #[test]
    fn test_json_graph_output() {
        let tmp_dir = TempDir::new().unwrap();
        let config = make_test_config(tmp_dir.path().to_str().unwrap());
        let metadata = make_test_metadata();

        let output = make_test_output(
            QueryResultsData::Graph {
                triples: vec![(
                    "http://example.org/s".to_string(),
                    "http://example.org/p".to_string(),
                    "http://example.org/o".to_string(),
                )],
            },
            1,
        );

        let writer = JsonWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["type"], "graph");
        assert!(parsed["triples"].is_array());
        assert_eq!(parsed["triples"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_json_escaping() {
        let tmp_dir = TempDir::new().unwrap();
        let config = make_test_config(tmp_dir.path().to_str().unwrap());
        let metadata = make_test_metadata();

        let output = make_test_output(
            QueryResultsData::Solutions {
                variables: vec!["col".to_string()],
                rows: vec![
                    vec!["back\\slash".to_string()],
                    vec!["has\"quote".to_string()],
                ],
            },
            2,
        );

        let writer = JsonWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();

        // Must parse as valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let results = parsed["results"].as_array().unwrap();
        assert_eq!(results[0]["col"], "back\\slash");
        assert_eq!(results[1]["col"], "has\"quote");
    }
}
