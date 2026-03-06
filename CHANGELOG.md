# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Unit tests for all core modules (SPARQL engine, config parsing, processors, output writers, local source, error types)
- CI: cargo-deny license/advisory checks, MSRV verification, code coverage with cargo-llvm-cov
- Community files: SECURITY.md, CHANGELOG.md, CONTRIBUTING.md, issue templates, PR template
- Dependabot configuration for cargo and GitHub Actions
- `deny.toml` for license and advisory auditing
- `rust-version = "1.75"` MSRV in Cargo.toml

### Fixed
- README License section now links to LICENSE file instead of showing "TODO"

## [0.0.3] - 2026-03-04

### Added
- Blueprint support for downloading configs, queries, and data from manifest files
- Interactive menu and init wizard
- YAML and JSON config format support

## [0.0.2] - 2026-02-15

### Added
- S3, Azure Blob, and GCS storage backends via `object_store` crate
- ZIP and GZIP decompression (in-memory, including nested archives)
- Filter processor with glob-based include/exclude
- JSON output writer
- Metadata output with timing and triple counts
- Parallel source fetching

## [0.0.1] - 2026-01-20

### Added
- Initial release
- Local filesystem source
- SPARQL query engine via Oxigraph
- CSV output writer
- TOML configuration

[Unreleased]: https://github.com/TopSwagCode/cimishi/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/TopSwagCode/cimishi/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/TopSwagCode/cimishi/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/TopSwagCode/cimishi/releases/tag/v0.0.1
