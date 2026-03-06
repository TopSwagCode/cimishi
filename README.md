# cimishi

![image.png](image.png)

**CIM + 意志 (will) = cimishi** — a fast, open-source CLI tool for querying CIM/RDF data with SPARQL.

Fetches RDF/XML from local disk or cloud storage (S3, Azure Blob, GCS), decompresses archives in-memory, runs SPARQL queries via [Oxigraph](https://github.com/oxigraph/oxigraph), and writes results to CSV or JSON.

---

## Installation

### Download a binary

Pre-built binaries for **Windows**, **Linux**, and **macOS** are available as zip archives on the [Releases page](https://github.com/TopSwagCode/cimishi/releases).

1. Download the zip for your platform
2. Extract it
3. Run `cimishi` (or `cimishi.exe` on Windows)

<details>
<summary>Linux / macOS</summary>

```bash
# Example for Linux x86_64 — adjust the URL for your platform
curl -LO https://github.com/TopSwagCode/cimishi/releases/latest/download/cimishi-linux-x86_64.zip
unzip cimishi-linux-x86_64.zip
chmod +x cimishi
./cimishi --help
```

Optionally move it to your PATH:

```bash
sudo mv cimishi /usr/local/bin/
```

</details>

<details>
<summary>Windows</summary>

Download the `.zip` from the [Releases page](https://github.com/TopSwagCode/cimishi/releases), extract it, and run `cimishi.exe` from a terminal.

</details>

### Build from source

Requires [Rust 1.75+](https://rustup.rs/).

```bash
git clone https://github.com/TopSwagCode/cimishi.git
cd cimishi
cargo build --release
./target/release/cimishi --help
```

### Docker

```bash
docker compose up --build
```

Or run directly:

```bash
docker run --rm \
  -v $(pwd)/examples/configs/pipeline.toml:/app/pipeline.toml:ro \
  -v $(pwd)/examples/data:/app/examples/data:ro \
  -v $(pwd)/examples/queries:/app/examples/queries:ro \
  -v $(pwd)/output:/app/output \
  cimishi query --config /app/pipeline.toml
```

---

## Quick Start

### 1. Write a config file

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

### 2. Run it

```bash
cimishi query --config pipeline.toml
```

### 3. Check the output

```
2026-03-04T12:00:00Z  INFO Starting pipeline: my-query
2026-03-04T12:00:00Z  INFO Fetched 14 files total
2026-03-04T12:00:01Z  INFO Loaded 14 files with 341304 triples in 850ms
2026-03-04T12:00:01Z  INFO Query returned 105 results in 7ms
2026-03-04T12:00:01Z  INFO Peak memory usage: 206.83 MB
2026-03-04T12:00:01Z  INFO CSV written to: output/results_20260304_120001.csv
```

Add `--verbose` for debug logging. Configs can be TOML, YAML, or JSON — format is auto-detected from the file extension.

---

## Blueprints

A blueprint is a manifest file that lists configs, queries, and data files to download. Point at a blueprint and everything gets installed to `.cimishi/` — ready to run.

```bash
# From a local file
cimishi blueprint --source examples/blueprints/example-blueprint.toml

# From a URL
cimishi blueprint --source https://example.com/my-blueprint.toml
```

Blueprints are also available from the interactive menu (`cimishi` with no arguments) and the init wizard (`cimishi init`).

<details>
<summary>Blueprint format</summary>

```toml
[blueprint]
name = "my-setup"
description = "Optional description"

[[configs]]
url = "https://example.com/pipeline.toml"
filename = "my-pipeline.toml"  # optional, defaults to URL filename

[[queries]]
url = "https://example.com/query.sparql"

[[data]]
url = "https://example.com/data.zip"
```

All three sections are optional. Files download to:

| Section   | Directory          |
|-----------|--------------------|
| `configs` | `.cimishi/config/` |
| `queries` | `.cimishi/query/`  |
| `data`    | `.cimishi/data/`   |

See `examples/blueprints/` for ready-made examples.

</details>

---

## Configuration Reference

See `examples/configs/` for ready-made configs.

| File                   | Description                          |
|------------------------|--------------------------------------|
| `pipeline.toml`        | Default TOML config                  |
| `pipeline.yaml`        | Same thing in YAML                   |
| `pipeline.json`        | Same thing in JSON                   |
| `pipeline-local.toml`  | Local development config             |
| `pipeline-zip.json`    | With ZIP extraction enabled          |
| `explicit-files.toml`  | Lists specific files instead of scanning |

<details>
<summary>Full configuration reference</summary>

```toml
[pipeline]
name = "cim-igm-query"            # Pipeline name (for logging)
parallel = true                   # Run sources in parallel (default: true)
max_concurrent = 10               # Max concurrent operations (default: 10)

# --- SOURCES ---

# Local filesystem (directory scan)
[[sources]]
type = "local"
path = "./examples/data"          # Directory or file path
patterns = ["*.xml", "*.rdf"]     # Glob patterns to match
recursive = true                  # Search subdirectories (default: true)

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

You can combine `path`/`prefix` scanning with explicit `files` in the same source.

</details>

---

## Cloud Storage

<details>
<summary>AWS S3</summary>

```bash
export AWS_ACCESS_KEY_ID=AKIA...
export AWS_SECRET_ACCESS_KEY=secret...
export AWS_REGION=eu-west-1
```

Also works with `~/.aws/credentials` or IAM roles.

</details>

<details>
<summary>Azure Blob Storage</summary>

```bash
export AZURE_STORAGE_ACCOUNT_NAME=myaccount
export AZURE_STORAGE_ACCOUNT_KEY=base64key...
```

</details>

<details>
<summary>Google Cloud Storage</summary>

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
# or
gcloud auth application-default login
```

</details>

<details>
<summary>S3-Compatible (MinIO, etc.)</summary>

Set `endpoint` in the source config:

```toml
[[sources]]
type = "s3"
bucket = "my-bucket"
region = "us-east-1"
endpoint = "http://localhost:9000"
```

</details>

---

## SPARQL Examples

<details>
<summary>List all substations</summary>

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

</details>

<details>
<summary>Find high-voltage lines (220kV+)</summary>

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

</details>

<details>
<summary>Count entities by type</summary>

```sparql
SELECT ?type (COUNT(?s) AS ?count)
WHERE {
    ?s a ?type .
}
GROUP BY ?type
ORDER BY DESC(?count)
LIMIT 20
```

</details>

---

## Architecture

```
Sources --> Processors --> Query --> Output
```

| Stage          | What it does                                                                 |
|----------------|-----------------------------------------------------------------------------|
| **Sources**    | Fetch files from local disk, S3, Azure, or GCS. Runs in parallel.           |
| **Processors** | Decompress ZIP/GZIP in-memory, filter files by glob patterns.               |
| **Query**      | Load RDF/XML into Oxigraph, execute a SPARQL 1.1 query.                     |
| **Output**     | Write results as CSV, JSON, and/or a metadata file with timing info.        |

Each run produces timestamped output files:

```
output/
  results_20260304_120000.csv
  results_20260304_120000.json
  results_20260304_120000.metadata
```

---

## Development

```bash
cargo run -- query --config examples/configs/pipeline-local.toml    # Run
cargo test                                                          # Test
cargo fmt --all -- --check                                          # Check formatting
cargo clippy --all-targets -- -D warnings                           # Lint
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on adding new sources, processors, or output formats.

---

## License

This project is licensed under the [MIT License](LICENSE).
