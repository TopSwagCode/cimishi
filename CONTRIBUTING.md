# Contributing to Cimishi

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

1. Install [Rust 1.75+](https://rustup.rs/)
2. Clone the repo and build:

```bash
git clone https://github.com/TopSwagCode/cimishi.git
cd cimishi
cargo build
```

## Running Tests

```bash
cargo test                    # Unit tests
cargo fmt --all -- --check    # Check formatting
cargo clippy -- -D warnings   # Lint
```

### Integration Tests (require Docker)

```bash
cd tests && ./run-tests.sh
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` and fix any warnings
- Follow existing patterns in the codebase

## Adding New Components

### Adding a New Source

1. Create `src/sources/my_source.rs`
2. Implement the `Source` trait
3. Add a variant to `SourceConfig` in `src/config/pipeline.rs`
4. Update `create_source()` in `src/sources/mod.rs`
5. Add tests

### Adding a New Processor

Same pattern: implement `Processor`, add to `ProcessorConfig`, update `create_processor()`.

### Adding a New Output Format

Implement `OutputWriter`, add to `OutputFormat` enum, update `create_writers()`.

## Pull Request Process

1. Fork the repo and create a feature branch
2. Make your changes with tests
3. Ensure `cargo test`, `cargo fmt --check`, and `cargo clippy` all pass
4. Open a PR against `master` with a clear description
5. Wait for CI and review

## Reporting Issues

Use [GitHub Issues](https://github.com/TopSwagCode/cimishi/issues) with the provided templates.
