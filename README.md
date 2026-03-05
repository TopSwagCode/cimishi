# cimishi

![image.png](image.png)

CIM + 意志 (will) = cimishi — open-source CIM/RDF tooling in Rust.

Fetches RDF/XML from local disk or cloud storage (S3, Azure Blob, GCS), decompresses archives in-memory, runs SPARQL queries via Oxigraph, and writes results to CSV/JSON.

## Quick Start

### Docker Compose

```bash
docker compose up --build
```

### Docker Run

Build the image:

```bash
docker build -t cimishi .
```

Run with a config file:

```bash
docker run --rm \
  -v $(pwd)/examples/configs/pipeline.toml:/app/pipeline.toml:ro \
  -v $(pwd)/examples/data:/app/examples/data:ro \
  -v $(pwd)/examples/queries:/app/examples/queries:ro \
  -v $(pwd)/output:/app/output \
  cimishi query --config /app/pipeline.toml
```

The `-v` flags mount your config, input files, and output directory into the container. Add `--verbose` for debug logging.

YAML and JSON configs work the same way — just mount a different file.

### With Cargo

Requires [Rust 1.75+](https://rustup.rs/).

#### Development

```bash
# Run directly (compiles + runs in one step)
cargo run -- query --config examples/configs/pipeline-local.toml

# With verbose logging
cargo run -- query --config examples/configs/pipeline-local.toml --verbose
```

The `--` separates Cargo flags from application arguments.

#### Release Build (for deployment)

```bash
# Build optimized binary
cargo build --release

# Run the binary
./target/release/cimishi --config examples/configs/pipeline-local.toml
```

All configs use relative paths (`./examples/data`, `./examples/queries`, `./output`). When running in Docker, the docker-compose.yml mounts host directories to match these paths.

### Example Output

```
$ cargo run -- --config examples/configs/pipeline-local.toml
2026-03-04T12:00:00Z  INFO Starting pipeline: cim-query
2026-03-04T12:00:00Z  INFO Stage 1: Fetching data from 1 sources
2026-03-04T12:00:00Z  INFO Fetched 14 files total
2026-03-04T12:00:01Z  INFO Loaded 14 files with 341304 triples in 850ms
2026-03-04T12:00:01Z  INFO Query returned 105 results in 7ms
2026-03-04T12:00:01Z  INFO Peak memory usage: 206.83 MB
2026-03-04T12:00:01Z  INFO CSV written to: output/results_20260304_120001.csv
```

---

## Configuration

Configs can be TOML, YAML, or JSON — format is auto-detected from the file extension. See `examples/configs/` for ready-made configs.

| File | Description |
|------|-------------|
| `pipeline.toml` | Default TOML config |
| `pipeline.yaml` | Same thing in YAML |
| `pipeline.json` | Same thing in JSON |
| `pipeline-local.toml` | Same as pipeline.toml (local development) |
| `pipeline-zip.json` | With ZIP extraction enabled |
| `explicit-files.toml` | Lists specific files instead of scanning |

### Minimal Config (TOML)

```toml
[pipeline]
name = "my-query"

[[sources]]
type = "local"
path = "./examples/data"
patterns = ["*.xml"]

[query]
file = "./examples/queries/query.sparql"

[output]
dir = "./output"
formats = ["csv"]
```

### Full Configuration Reference

```toml
[pipeline]
name = "cim-igm-query"            # Pipeline name (for logging)
parallel = true                   # Run sources in parallel
max_concurrent = 10               # Max concurrent operations

# --- SOURCES ---

# Local filesystem (directory scan)
[[sources]]
type = "local"
path = "./examples/data"          # Directory or file path
patterns = ["*.xml", "*.rdf"]     # Glob patterns to match
recursive = true                  # Search subdirectories

# Local filesystem (explicit files)
[[sources]]
type = "local"
files = [
    "/data/model_EQ.xml",
    "/shared/network/topology.rdf",
    "/archive/2026/full_model.zip"
]

# AWS S3
[[sources]]
type = "s3"
bucket = "my-bucket"
prefix = "data/2026/"
region = "eu-west-1"
endpoint = "http://minio:9000"    # Optional, for MinIO etc.
patterns = ["*.xml", "*.zip"]

# Azure Blob Storage
[[sources]]
type = "azure"
account = "mystorageaccount"
container = "rdf-data"
prefix = "igm/"
patterns = ["*.xml"]

# Google Cloud Storage
[[sources]]
type = "gcs"
bucket = "my-gcs-bucket"
prefix = "cim/"
patterns = ["*.xml", "*.gz"]

# --- PROCESSORS ---

# Decompress archives in-memory
[[processors]]
type = "unzip"
archive_patterns = ["*.zip"]
gzip_patterns = ["*.gz", "*.gzip"]
patterns = ["*.xml", "*.rdf"]     # Keep only these after extraction

# Filter by filename pattern
[[processors]]
type = "filter"
include = ["*_EQ_*.xml", "*_TP_*.xml"]
exclude = ["*_BD_*.xml"]

# --- QUERY ---

[query]
file = "./examples/queries/query.sparql"  # External query file
base_iri = "http://example.org/"

# Or inline:
# query = """
# PREFIX cim: <http://iec.ch/TC57/CIM100#>
# SELECT ?name WHERE { ?sub a cim:Substation ; cim:IdentifiedObject.name ?name . }
# """

# --- OUTPUT ---

[output]
dir = "./output"
formats = ["csv", "json"]
metadata = true                   # Write .metadata file with timing info
prefix = "results"                # Filename prefix
```

You can combine `path`/`prefix` scanning with explicit `files` in the same source — the pipeline fetches both.

---

## Architecture

```
Sources --> Processors --> Query --> Output
```

Four stages, each defined by a trait:

1. **Sources** — Fetch files from storage backends. `LocalSource` for the filesystem, `ObjectStoreSource` for S3/Azure/GCS (via the `object_store` crate). Multiple sources run in parallel when `parallel = true`.

2. **Processors** — Transform files in sequence. `UnzipProcessor` handles ZIP and GZIP decompression entirely in memory, including nested archives (e.g. `.xml.gz` inside a `.zip`). `FilterProcessor` does glob-based include/exclude.

3. **Query** — Loads all processed files as RDF/XML into an in-memory Oxigraph store and executes a SPARQL 1.1 query.

4. **Output** — Writes results. CSV, JSON, and a metadata file with timing/triple counts.

### Compression

All decompression happens in memory. The unzip processor handles:

- ZIP archives (with nested GZIP support)
- GZIP files (`.gz`, `.gzip`)

Files inside archives that don't match the configured patterns are discarded.

### Output Files

Each run produces timestamped files:

```
output/
  results_20260304_120000.csv
  results_20260304_120000.json
  results_20260304_120000.metadata
```

The metadata file includes source count, triple count, query time, and peak memory.

---

## Cloud Storage Setup

### AWS S3

```bash
export AWS_ACCESS_KEY_ID=AKIA...
export AWS_SECRET_ACCESS_KEY=secret...
export AWS_REGION=eu-west-1
```

Also works with `~/.aws/credentials` or IAM roles.

### Azure Blob Storage

```bash
export AZURE_STORAGE_ACCOUNT_NAME=myaccount
export AZURE_STORAGE_ACCOUNT_KEY=base64key...
```

### Google Cloud Storage

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
# or
gcloud auth application-default login
```

### S3-Compatible (MinIO, etc.)

Set `endpoint` in the source config:

```toml
[[sources]]
type = "s3"
bucket = "my-bucket"
region = "us-east-1"
endpoint = "http://localhost:9000"
```

---

## SPARQL Examples

List all substations:

```sparql
PREFIX cim: <http://iec.ch/TC57/CIM100#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?substation ?name ?region
WHERE {
    ?substation rdf:type cim:Substation ;
                cim:IdentifiedObject.name ?name .
    OPTIONAL {
        ?substation cim:Substation.Region ?regionRef .
        ?regionRef cim:IdentifiedObject.name ?region .
    }
}
ORDER BY ?name
```

Find high-voltage lines (220kV+):

```sparql
PREFIX cim: <http://iec.ch/TC57/CIM100#>

SELECT ?line ?name ?voltage
WHERE {
    ?line a cim:ACLineSegment ;
          cim:IdentifiedObject.name ?name ;
          cim:ConductingEquipment.BaseVoltage ?bv .
    ?bv cim:BaseVoltage.nominalVoltage ?voltage .
    FILTER(?voltage >= 220)
}
ORDER BY DESC(?voltage)
```

Count entities by type:

```sparql
SELECT ?type (COUNT(?s) AS ?count)
WHERE {
    ?s a ?type .
}
GROUP BY ?type
ORDER BY DESC(?count)
LIMIT 20
```

---

## Development

```bash
# Run during development
cargo run -- --config examples/configs/pipeline-local.toml

# Run tests
cargo test
```

### Adding a New Source

1. Create `src/sources/my_source.rs`
2. Implement the `Source` trait
3. Add a variant to `SourceConfig` in `src/config/pipeline.rs`
4. Update `create_source()` in `src/sources/mod.rs`

### Adding a New Processor

Same pattern — implement `Processor`, add to `ProcessorConfig`, update `create_processor()`.

---

## License

TODO