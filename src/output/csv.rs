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
            formats: vec![OutputFormat::Csv],
            metadata: false,
            prefix: Some("test".to_string()),
        }
    }

    #[test]
    fn test_csv_solutions_output() {
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

        let writer = CsvWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        assert_eq!(paths.len(), 1);

        let content = std::fs::read_to_string(&paths[0]).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "name,value");
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert_eq!(lines[1], "alice,42");
        assert_eq!(lines[2], "bob,99");
    }

    #[test]
    fn test_csv_escaping() {
        let tmp_dir = TempDir::new().unwrap();
        let config = make_test_config(tmp_dir.path().to_str().unwrap());
        let metadata = make_test_metadata();

        let output = make_test_output(
            QueryResultsData::Solutions {
                variables: vec!["col".to_string()],
                rows: vec![
                    vec!["has,comma".to_string()],
                    vec!["has\"quote".to_string()],
                    vec!["has\nnewline".to_string()],
                ],
            },
            3,
        );

        let writer = CsvWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();

        // Values with commas, quotes, or newlines should be wrapped in quotes
        assert!(content.contains("\"has,comma\""));
        // Quotes inside should be doubled
        assert!(content.contains("\"has\"\"quote\""));
        // Newlines should be wrapped in quotes
        assert!(content.contains("\"has\nnewline\""));
    }

    #[test]
    fn test_csv_boolean_output() {
        let tmp_dir = TempDir::new().unwrap();
        let config = make_test_config(tmp_dir.path().to_str().unwrap());
        let metadata = make_test_metadata();

        let output = make_test_output(QueryResultsData::Boolean(true), 1);

        let writer = CsvWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines[0], "result");
        assert_eq!(lines[1], "true");
    }

    #[test]
    fn test_csv_graph_output() {
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

        let writer = CsvWriter;
        let paths = writer.write(&output, &metadata, &config).unwrap();
        let content = std::fs::read_to_string(&paths[0]).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines[0], "subject,predicate,object");
        assert_eq!(lines.len(), 2); // header + 1 triple
    }
}
