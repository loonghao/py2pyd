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

## Testing

py2pyd includes comprehensive tests, including integration tests that download and compile real Python packages from PyPI.

### Quick Tests
```bash
# Run unit tests only
cargo test
```

### Integration Tests
```bash
# Run all tests including integration tests (requires internet and build tools)
cargo test -- --ignored

# Test compiling a simple Python module
cargo test test_compile_simple_python_module -- --ignored

# Test downloading and compiling a real PyPI package
cargo test test_download_six_package -- --ignored
```

For detailed testing information, see [TESTING_GUIDE.md](TESTING_GUIDE.md).

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

This project uses **Semantic Release** with automated CI/CD. Releases are automatically triggered based on [Conventional Commits](https://www.conventionalcommits.org/):

### Automatic Version Bumping

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `feat:` | Minor (0.1.0 → 0.2.0) | `feat: add Python 3.12 support` |
| `fix:` | Patch (0.1.0 → 0.1.1) | `fix: resolve memory leak in parser` |
| `feat!:` or `BREAKING CHANGE:` | Major (0.1.0 → 1.0.0) | `feat!: redesign command-line interface` |
| `docs:`, `chore:`, etc. | Patch (0.1.0 → 0.1.1) | `docs: update installation guide` |

### How to Release

Simply use conventional commit messages when pushing to main:

```bash
# Feature addition (minor version bump)
git commit -m "feat: add support for Python 3.12"

# Bug fix (patch version bump)
git commit -m "fix: handle edge case in file parsing"

# Breaking change (major version bump)
git commit -m "feat!: redesign command-line interface

BREAKING CHANGE: The --input flag is now required"
```

The CI system will automatically:
1. **Analyze commit messages** to determine version bump
2. **Update version** in `Cargo.toml`
3. **Build binaries** for all supported platforms
4. **Create Git tag** and GitHub release
5. **Upload artifacts** with generated release notes

For detailed information, see [docs/VERSIONING.md](docs/VERSIONING.md).

## Contributing

Contributions are welcome! Please follow these guidelines:

### Commit Message Format

This project uses [Conventional Commits](https://www.conventionalcommits.org/). Please format your commit messages as:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools

**Examples:**
```bash
feat(parser): add support for async functions
fix(compiler): resolve segmentation fault on Windows
docs(readme): add installation instructions
refactor(core): simplify error handling logic
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes using conventional commits
4. Submit a pull request

Please feel free to submit a Pull Request!

## License

This project is licensed under the MIT License - see the LICENSE file for details.
