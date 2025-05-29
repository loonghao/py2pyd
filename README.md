# py2pyd

[![CI](https://github.com/loonghao/py2pyd/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/py2pyd/actions/workflows/ci.yml)
[![Code Quality](https://github.com/loonghao/py2pyd/actions/workflows/code-quality.yml/badge.svg)](https://github.com/loonghao/py2pyd/actions/workflows/code-quality.yml)
[![Release](https://github.com/loonghao/py2pyd/actions/workflows/release.yml/badge.svg)](https://github.com/loonghao/py2pyd/actions/workflows/release.yml)

A Rust-based tool to compile Python modules to extension files (.pyd on Windows, .so on Linux/macOS).

> **Warning**: This project is currently under active development and not ready for production use. APIs and functionality may change significantly between versions.

## Overview

py2pyd is a command-line tool that simplifies the process of compiling Python (.py) files to Python extension modules (.pyd on Windows, .so on Linux/macOS).

## Features (Planned)

- Compile single Python files or entire directories to Python extension modules (.pyd on Windows, .so on Linux/macOS)
- Support for multiple Python interpreter discovery methods:
  - Default PATH lookup
  - uv integration with version selection (`--uv-python 3.10`)
  - Explicit interpreter path specification (`--python-path`)
- Batch processing with recursive directory support
- Optimization level control

- Comprehensive logging and error reporting

## Installation

### Download Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/loonghao/py2pyd/releases).

Available platforms:
- **Windows**: `py2pyd-windows-x86_64.zip`
- **Linux**: `py2pyd-linux-x86_64.tar.gz`
- **macOS (Intel)**: `py2pyd-macos-x86_64.tar.gz`
- **macOS (ARM)**: `py2pyd-macos-aarch64.tar.gz`

### Build from Source

```bash
git clone https://github.com/loonghao/py2pyd.git
cd py2pyd
cargo build --release
```

The binary will be available at `target/release/py2pyd` (or `py2pyd.exe` on Windows).

## Usage

```bash
# Basic usage (uses Python from PATH)
py2pyd -i input.py -o output.pyd

# Using uv with specific Python version
py2pyd --uv-python 3.10 -i input.py -o output.pyd

# Using explicit Python interpreter path
py2pyd --python-path C:/Python310/python.exe -i input.py -o output.pyd

# Batch processing
py2pyd --uv-python 3.10 -i src/ -o build/ --recursive
```

## Requirements

- Operating system:
  - Windows (primary target)
  - Linux and macOS (experimental support)
- Compatible C/C++ compiler for the target Python version:
  - **Windows**:
    - **MSVC (Recommended)**: Install Visual Studio Build Tools from [here](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
    - **MinGW-w64 (Alternative)**: Install from [here](https://www.mingw-w64.org/downloads/)
  - **Linux**: GCC (install via `sudo apt-get install build-essential` on Debian/Ubuntu)
  - **macOS**: Xcode Command Line Tools (install via `xcode-select --install`)
- Python interpreter (if not using embedded mode)

The tool will automatically check for required build tools and provide installation instructions if they are missing.

## TODO List

- [ ] Implement flexible Python interpreter discovery
  - [ ] PATH-based discovery
  - [ ] uv integration with version selection
  - [ ] Explicit path specification
- [ ] Improve MSVC compiler detection and integration
  - [ ] Auto-detection of installed MSVC
  - [ ] Clear error messages and installation guidance
  - [ ] Investigate minimal MSVC toolchain options
- [ ] Enhance compilation process
  - [ ] Optimize Cython usage
  - [ ] Improve error handling and reporting
  - [ ] Add support for compilation configuration
- [ ] Add comprehensive testing
  - [ ] Unit tests for different Python versions
  - [ ] Integration tests for various compilation scenarios
  - [x] CI/CD pipeline setup
- [ ] Improve documentation
  - [ ] Detailed usage examples
  - [ ] Troubleshooting guide
  - [ ] API documentation
- [ ] Future enhancements
  - [ ] Investigate embedded Python interpreter option
  - [x] Support for additional platforms (Linux, macOS)
  - [ ] Performance optimizations

## Release Process

This project uses automated CI/CD for releases. When the version in `Cargo.toml` is updated and pushed to the main branch, the system automatically:

1. **Detects version changes** and creates a Git tag
2. **Builds binaries** for all supported platforms
3. **Creates a GitHub release** with binary artifacts
4. **Generates release notes** from commit messages

### For Maintainers

Use the provided helper scripts to create releases:

```bash
# Windows
.\scripts\release.ps1 1.0.0 "Release message"

# Linux/macOS
./scripts/release.sh 1.0.0 "Release message"
```

For detailed information, see [docs/RELEASE.md](docs/RELEASE.md).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
