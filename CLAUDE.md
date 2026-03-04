# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cimishi (`cimishi`) — an open-source CIM/RDF Rust CLI tool that fetches RDF/XML data from multiple storage backends, processes it (decompression, filtering), loads it into an in-memory Oxigraph store, runs a SPARQL query, and writes results to CSV/JSON.

## Build & Run Commands

```bash
# Development (debug build - faster compile, slower execution)
cargo run -- --config examples/configs/pipeline-local.toml
cargo run -- --config examples/configs/pipeline-local.toml --verbose

# Release build (for deployment or performance testing)
cargo build --release
./target/release/cimishi --config examples/configs/pipeline-local.toml

# Tests
cargo test                                               # Run unit tests
cargo test -- --test-name test_parse_config              # Run a specific test
```

### Integration Tests (require Docker)

```bash
cd tests && ./run-tests.sh           # Run all integration tests (local, S3 via MinIO, Azure via Azurite)
cd tests && ./run-tests.sh --cleanup # Clean up test containers
```

### Docker

```bash
docker compose up --build            # Build and run with default config
docker build -t cimishi .             # Build image only
```

## Architecture

Four-stage pipeline defined in `src/pipeline/mod.rs`:

1. **Sources** (`src/sources/`) — Fetch files. Trait: `Source::fetch() -> Vec<FetchedFile>`. Implementations: `LocalSource` (filesystem), `ObjectStoreSource` (S3/Azure/GCS via `object_store` crate). Sources run in parallel when `pipeline.parallel = true`.

2. **Processors** (`src/processors/`) — Transform files in sequence. Trait: `Processor::process(Vec<ProcessedFile>) -> Vec<ProcessedFile>`. Implementations: `UnzipProcessor` (ZIP/GZIP decompression), `FilterProcessor` (glob-based include/exclude).

3. **Query** (`src/query/sparql.rs`) — Loads all processed files as RDF/XML into an in-memory Oxigraph `Store`, then executes a SPARQL query. Trait: `QueryEngine::execute()`.

4. **Output** (`src/output/`) — Writes results. Trait: `OutputWriter::write()`. Implementations: `CsvWriter`, `JsonWriter`, `MetadataWriter`.

Each stage uses a factory function (`create_source`, `create_processor`, `create_writers`) that maps config enum variants to trait objects.

## Configuration

`PipelineConfig` in `src/config/pipeline.rs` is the root config struct. Supports TOML, YAML, and JSON (auto-detected from file extension). Source types are discriminated via `#[serde(tag = "type")]` on `SourceConfig` enum. Example configs live in `examples/configs/`, with sample data in `examples/data/` and queries in `examples/queries/`.

## Adding New Components

To add a new source or processor:
1. Create the implementation file in the appropriate `src/` subdirectory
2. Implement the `Source` or `Processor` trait
3. Add a variant to `SourceConfig` or `ProcessorConfig` enum in `src/config/pipeline.rs`
4. Update the factory function in the module's `mod.rs`

## Key Dependencies

- **oxigraph** — In-memory RDF store and SPARQL engine
- **object_store** — Unified cloud storage access (S3, Azure Blob, GCS)
- **tokio** — Async runtime
- **async_zip / async-compression** — In-memory archive handling
- **clap** — CLI argument parsing
- **serde + toml/serde_yaml/serde_json** — Multi-format config deserialization
