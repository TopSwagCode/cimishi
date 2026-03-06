//! SPARQL query engine implementation using Oxigraph.

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Term;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::io::Cursor;
use std::time::{Duration, Instant};
use tracing::{debug, info};

use crate::config::QueryConfig;
use crate::error::{PipelineError, Result};
use crate::processors::ProcessedFile;

/// Format an RDF term for clean CSV/JSON output.
///
/// Converts RDF terms to user-friendly strings:
/// - IRIs: Full IRI string (without angle brackets)
/// - Literals: Just the value (without quotes or datatype)
/// - Blank nodes: The blank node identifier
/// - Triples (RDF-star): Formatted as nested triple
fn format_term(term: &Term) -> String {
    match term {
        Term::NamedNode(iri) => iri.as_str().to_string(),
        Term::BlankNode(bn) => format!("_:{}", bn.as_str()),
        Term::Literal(lit) => lit.value().to_string(),
        Term::Triple(triple) => {
            format!(
                "<<{} {} {}>>",
                format_term(&triple.subject.clone().into()),
                format_term(&triple.predicate.clone().into()),
                format_term(&triple.object.clone())
            )
        }
    }
}

/// Output from a query execution.
#[derive(Debug)]
pub struct QueryOutput {
    /// Query results type.
    pub results: QueryResultsData,

    /// Number of results.
    pub count: usize,

    /// Time to load data into the store.
    pub load_time: Duration,

    /// Time to execute the query.
    pub query_time: Duration,

    /// Number of files loaded.
    pub files_loaded: usize,

    /// Number of triples loaded.
    pub triples_loaded: usize,

    /// Peak memory usage in bytes (physical memory).
    pub peak_memory_bytes: Option<usize>,
}

/// Simplified query results for output.
#[derive(Debug)]
pub enum QueryResultsData {
    /// SELECT query results with variable names and rows.
    Solutions {
        variables: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// ASK query result.
    Boolean(bool),
    /// CONSTRUCT/DESCRIBE query results.
    Graph {
        triples: Vec<(String, String, String)>,
    },
}

/// Trait for query engines.
pub trait QueryEngine: Send + Sync {
    /// Load files and execute a query.
    fn execute(&self, files: Vec<ProcessedFile>, config: &QueryConfig) -> Result<QueryOutput>;
}

/// SPARQL query engine using Oxigraph.
pub struct SparqlEngine {
    #[allow(dead_code)] // Reserved for future use with named graphs
    base_iri: String,
}

impl SparqlEngine {
    pub fn new(base_iri: String) -> Self {
        Self { base_iri }
    }
}

impl Default for SparqlEngine {
    fn default() -> Self {
        Self::new("http://example.org/".to_string())
    }
}

impl QueryEngine for SparqlEngine {
    fn execute(&self, files: Vec<ProcessedFile>, config: &QueryConfig) -> Result<QueryOutput> {
        info!("SPARQL engine: loading {} files", files.len());

        // Create store
        let store = Store::new().map_err(|e| PipelineError::Query(e.to_string()))?;

        // Load files
        let load_start = Instant::now();
        let mut files_loaded = 0;
        let mut triples_loaded = 0;

        for file in &files {
            // Determine format from extension
            let format = if file.filename.ends_with(".xml") || file.filename.ends_with(".rdf") {
                RdfFormat::RdfXml
            } else if file.filename.ends_with(".ttl") {
                RdfFormat::Turtle
            } else if file.filename.ends_with(".nt") {
                RdfFormat::NTriples
            } else if file.filename.ends_with(".nq") {
                RdfFormat::NQuads
            } else if file.filename.ends_with(".trig") {
                RdfFormat::TriG
            } else {
                // Default to RDF/XML for CIM data
                RdfFormat::RdfXml
            };

            debug!("Loading: {} ({:?})", file.filename, format);

            // Create base IRI from filename
            let file_base_iri = format!("{}{}", config.base_iri, file.filename);
            let parser = RdfParser::from_format(format)
                .with_base_iri(&file_base_iri)
                .map_err(|e| PipelineError::RdfParse(e.to_string()))?;

            let cursor = Cursor::new(&file.content[..]);
            let count_before = store
                .len()
                .map_err(|e| PipelineError::Query(e.to_string()))?;

            store
                .load_from_reader(parser, cursor)
                .map_err(|e| PipelineError::RdfParse(format!("{}: {}", file.filename, e)))?;

            let count_after = store
                .len()
                .map_err(|e| PipelineError::Query(e.to_string()))?;
            let loaded = count_after - count_before;

            debug!("  Loaded {} triples from {}", loaded, file.filename);
            triples_loaded += loaded;
            files_loaded += 1;
        }

        let load_time = load_start.elapsed();
        info!(
            "Loaded {} files with {} triples in {:?}",
            files_loaded, triples_loaded, load_time
        );

        // Get query
        let query_str = config.get_query()?;
        info!("Executing SPARQL query...");

        // Execute query
        let query_start = Instant::now();
        let results = store
            .query(&query_str)
            .map_err(|e| PipelineError::Sparql(e.to_string()))?;
        let query_time = query_start.elapsed();

        // Convert results
        let (results_data, count) = match results {
            QueryResults::Solutions(solutions) => {
                let variables: Vec<String> = solutions
                    .variables()
                    .iter()
                    .map(|v| v.as_str().to_string())
                    .collect();

                let mut rows = Vec::new();
                for solution in solutions {
                    let solution = solution.map_err(|e| PipelineError::Sparql(e.to_string()))?;
                    let row: Vec<String> = variables
                        .iter()
                        .map(|var| {
                            solution
                                .get(var.as_str())
                                .map(format_term)
                                .unwrap_or_default()
                        })
                        .collect();
                    rows.push(row);
                }

                let count = rows.len();
                (QueryResultsData::Solutions { variables, rows }, count)
            }
            QueryResults::Boolean(result) => (QueryResultsData::Boolean(result), 1),
            QueryResults::Graph(triples) => {
                let mut graph_triples = Vec::new();
                for triple in triples {
                    let triple = triple.map_err(|e| PipelineError::Sparql(e.to_string()))?;
                    graph_triples.push((
                        format_term(&triple.subject.into()),
                        format_term(&triple.predicate.into()),
                        format_term(&triple.object),
                    ));
                }
                let count = graph_triples.len();
                (
                    QueryResultsData::Graph {
                        triples: graph_triples,
                    },
                    count,
                )
            }
        };

        info!("Query returned {} results in {:?}", count, query_time);

        // Capture peak memory usage
        let peak_memory_bytes = memory_stats::memory_stats().map(|stats| stats.physical_mem);
        if let Some(mem) = peak_memory_bytes {
            info!("Peak memory usage: {:.2} MB", mem as f64 / 1024.0 / 1024.0);
        }

        Ok(QueryOutput {
            results: results_data,
            count,
            load_time,
            query_time,
            files_loaded,
            triples_loaded,
            peak_memory_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use oxigraph::model::{BlankNode, Literal, NamedNode};

    const TEST_RDF_XML: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:ex="http://example.org/">
  <rdf:Description rdf:about="http://example.org/thing1">
    <ex:name>Thing One</ex:name>
  </rdf:Description>
</rdf:RDF>"#;

    fn test_rdf_file() -> ProcessedFile {
        ProcessedFile {
            path: "test.rdf".to_string(),
            filename: "test.rdf".to_string(),
            content: Bytes::from(TEST_RDF_XML),
            source: "test".to_string(),
        }
    }

    fn query_config(query: &str) -> QueryConfig {
        QueryConfig {
            file: None,
            query: Some(query.to_string()),
            base_iri: "http://example.org/".to_string(),
        }
    }

    #[test]
    fn test_format_term_named_node() {
        let term = Term::NamedNode(NamedNode::new_unchecked("http://example.org/test"));
        assert_eq!(format_term(&term), "http://example.org/test");
    }

    #[test]
    fn test_format_term_blank_node() {
        let term = Term::BlankNode(BlankNode::new_unchecked("b0"));
        assert_eq!(format_term(&term), "_:b0");
    }

    #[test]
    fn test_format_term_literal() {
        let term = Term::Literal(Literal::new_simple_literal("hello"));
        assert_eq!(format_term(&term), "hello");
    }

    #[test]
    fn test_select_query() {
        let engine = SparqlEngine::default();
        let files = vec![test_rdf_file()];
        let config = query_config("SELECT ?s ?o WHERE { ?s <http://example.org/name> ?o }");

        let output = engine
            .execute(files, &config)
            .expect("query should succeed");

        match &output.results {
            QueryResultsData::Solutions { variables, rows: _ } => {
                assert!(variables.contains(&"s".to_string()));
                assert!(variables.contains(&"o".to_string()));
            }
            _ => panic!("Expected Solutions variant"),
        }
        assert!(output.count >= 1);
        assert_eq!(output.files_loaded, 1);
        assert!(output.triples_loaded >= 1);
    }

    #[test]
    fn test_ask_query() {
        let engine = SparqlEngine::default();
        let files = vec![test_rdf_file()];
        let config = query_config("ASK { ?s <http://example.org/name> ?o }");

        let output = engine
            .execute(files, &config)
            .expect("query should succeed");

        match &output.results {
            QueryResultsData::Boolean(val) => assert!(val, "ASK should return true"),
            _ => panic!("Expected Boolean variant"),
        }
    }

    #[test]
    fn test_construct_query() {
        let engine = SparqlEngine::default();
        let files = vec![test_rdf_file()];
        let config = query_config(
            "CONSTRUCT { ?s <http://example.org/name> ?o } WHERE { ?s <http://example.org/name> ?o }",
        );

        let output = engine
            .execute(files, &config)
            .expect("query should succeed");

        match &output.results {
            QueryResultsData::Graph { triples } => {
                assert!(!triples.is_empty(), "Graph should contain triples");
            }
            _ => panic!("Expected Graph variant"),
        }
    }

    #[test]
    fn test_invalid_sparql_query() {
        let engine = SparqlEngine::default();
        let files = vec![test_rdf_file()];
        let config = query_config("NOT VALID SPARQL");

        let result = engine.execute(files, &config);
        assert!(result.is_err(), "Invalid SPARQL should return Err");
    }

    #[test]
    fn test_invalid_rdf_data() {
        let engine = SparqlEngine::default();
        let files = vec![ProcessedFile {
            path: "bad.xml".to_string(),
            filename: "bad.xml".to_string(),
            content: Bytes::from("not xml at all"),
            source: "test".to_string(),
        }];
        let config = query_config("SELECT ?s ?p ?o WHERE { ?s ?p ?o }");

        let result = engine.execute(files, &config);
        assert!(result.is_err(), "Invalid RDF data should return Err");
    }
}
