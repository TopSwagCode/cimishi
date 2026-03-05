//! Terminal (stdout) output writer — tab-separated for piping.

use std::io::Write;

use super::{OutputMetadata, OutputWriter};
use crate::config::OutputConfig;
use crate::error::Result;
use crate::query::{QueryOutput, QueryResultsData};

/// Writes query results as TSV to stdout.
pub struct TerminalWriter;

impl OutputWriter for TerminalWriter {
    fn write(
        &self,
        output: &QueryOutput,
        _metadata: &OutputMetadata,
        _config: &OutputConfig,
    ) -> Result<Vec<String>> {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        match &output.results {
            QueryResultsData::Solutions { variables, rows } => {
                writeln!(out, "{}", variables.join("\t"))?;
                for row in rows {
                    writeln!(out, "{}", row.join("\t"))?;
                }
            }
            QueryResultsData::Boolean(result) => {
                writeln!(out, "{}", result)?;
            }
            QueryResultsData::Graph { triples } => {
                writeln!(out, "subject\tpredicate\tobject")?;
                for (s, p, o) in triples {
                    writeln!(out, "{}\t{}\t{}", s, p, o)?;
                }
            }
        }

        Ok(vec![])
    }
}
