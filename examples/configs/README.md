# Pipeline Configuration Examples

Ready-made configs in different formats. They all do the same thing — pick whichever format you prefer.

| File | Format | Description |
|------|--------|-------------|
| `pipeline.toml` | TOML | Default, scans `/input` (Docker paths) |
| `pipeline.yaml` | YAML | Same config in YAML |
| `pipeline.json` | JSON | Same config in JSON |
| `pipeline-local.toml` | TOML | Same as pipeline.toml (local development) |
| `pipeline-zip.json` | JSON | With ZIP extraction enabled |
| `explicit-files.toml` | TOML | Lists specific files instead of scanning a directory |
| `files.json` | JSON | Example file list (network paths) |

## Usage

### Docker Compose

By default, `docker-compose.yml` mounts `pipeline.toml`. To use a different config:

```bash
docker compose run --rm \
  -v $(pwd)/examples/configs/pipeline.yaml:/app/pipeline.yaml:ro \
  cimishi --config pipeline.yaml
```

### Local Rust

```bash
# Development
cargo run -- --config examples/configs/pipeline-local.toml

# Release build (for deployment)
cargo build --release
./target/release/cimishi --config examples/configs/pipeline-local.toml
```

All configs use relative paths (`./examples/data`, `./examples/queries`, `./output`). When running in Docker, `docker-compose.yml` mounts host directories to match these paths.

## Creating Your Own

Copy any example and edit it:

```bash
cp examples/configs/pipeline.toml my-pipeline.toml
```

Format is auto-detected from the file extension (`.toml`, `.yaml`/`.yml`, `.json`).

See the main [README.md](../README.md) for the full configuration reference.
