# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/loonghao/py2pyd/releases/tag/v0.1.1) - 2025-06-26

### Added

- switch to Docker containers with updated Rust 1.83+ images
- upgrade to rust-actions-toolkit v2.0.2 with Rust 1.83.0
- optimize CI configuration with caching and concurrency control
- update README with Docker revolution features and upgrade to v2.0.1
- upgrade to rust-actions-toolkit@v2.0.0 with Docker revolution
- upgrade to rust-actions-toolkit@v1.2.0 with enhanced features
- migrate reqwest to rustls-tls with async implementation
- upgrade to turbo-cdn 0.4.1 with new API and use precise CI version
- migrate CI to rust-actions-toolkit for standardized workflows
- add test runner example and improve testing documentation
- add comprehensive integration tests for PyPI package compilation
- integrate turbo-cdn 0.3.0 for enhanced download performance
- [**breaking**] switch to semantic release workflow
- add semantic release workflow option
- add release helper scripts
- add automated release CI/CD workflow

### Fixed

- resolve used_underscore_binding clippy warning
- remove redundant else block in build_tools.rs
- correct rust-toolchain parameter and disable Docker temporarily
- resolve sudo permission issues in remaining Docker workflows
- resolve code formatting issues for CI compliance
- resolve clippy warnings and improve code quality
- resolve sudo permission issues by using direct cargo commands in Docker
- resolve Docker container sudo permission issues in CI
- upgrade rust-actions-toolkit to v2.0.2 to resolve CI issues
- *(deps)* update rust crate uuid to v1.17.0
- *(deps)* update rust crate tokio to v1.45.1
- *(deps)* update rust crate zip to v4.2.0
- *(deps)* update rust crate reqwest to v0.12.20
- *(deps)* update rust crate clap to v4.5.40
- correct YAML formatting in ci.yml workflow
- resolve dead_code warnings in test files
- resolve all clippy warnings and errors
- resolve compilation warnings and errors
- add required permissions to CI workflows for rust-actions-toolkit
- use minimal CI configuration with master branch
- simplify CI configuration to resolve startup failures
- update CI workflows to use rust-actions-toolkit@v1 with correct configuration
- *(deps)* update rust crate which to v8
- resolve cargo-audit JSON parsing error in CI
- update rustpython-parser API calls for version 0.4
- resolve proc-macro compilation issues on Linux
- update sccache configuration to resolve timeout issues
- disable clippy checks to resolve build issues

### Other

- fix all clippy warnings and improve code quality
- add Rust version check to CI
- *(deps)* update loonghao/rust-actions-toolkit action to v1.1.6
- *(deps)* update rust crate flate2 to v1.1.2
- version 0.1.1
- remove test files from repository
- Merge branch 'remove-dcc-references' into main
- optimize CI configuration and improve build performance
- Fix clippy warnings
- Fix macOS and Linux build tool detection
- Fix code formatting issues
- Update GitHub Actions to latest versions
- Resolve merge conflicts in Cargo.toml
- Remove DCC references and add CI configuration
- Update Rust crate env_logger to 0.11
- Update Rust crate reqwest to 0.12
- Add renovate.json
- Initial commit: Basic project structure and documentation
