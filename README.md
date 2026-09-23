# py2pyd

[![CI](https://github.com/loonghao/py2pyd/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/py2pyd/actions/workflows/ci.yml)
[![Release](https://github.com/loonghao/py2pyd/actions/workflows/release.yml/badge.svg)](https://github.com/loonghao/py2pyd/actions/workflows/release.yml)

**A high-performance Rust-based tool to compile Python modules to extension files (.pyd on Windows, .so on Linux/macOS) with zero-dependency executables.**

> **Note**: This project is under active development. While core functionality is stable, APIs may evolve. We provide zero-dependency executables for maximum portability.

## Overview

py2pyd is a Rust-based command-line tool that compiles Python (.py) files to Python extension modules (.pyd on Windows, .so on Linux/macOS). It resolves a Python interpreter from PATH, `uv`, or an explicit path, drives Cython through a generated `setup.py` inside an isolated `uv` environment, and ships zero-dependency release binaries for Windows, Linux, and macOS.

### Key Highlights

- **Zero-Dependency Executables**: Windows MSVC and `x86_64-unknown-linux-musl` release archives are statically linked - download, extract, and run
- **Universal Compatibility**: Static binaries work on any Windows system or Linux distribution
- **Flexible Interpreter Discovery**: PATH lookup, `uv` version selection, or an explicit interpreter path
- **Cython Under the Hood**: Each build runs through a generated `setup.py` in an isolated `uv` environment

## Features

- Compile single Python files or entire directories to Python extension modules (.pyd on Windows, .so on Linux/macOS)
- Support for multiple Python interpreter discovery methods:
  - Default PATH lookup
  - uv integration with version selection (`--python-version 3.10`)
  - Explicit interpreter path specification (`--python-path`)
- Batch processing with recursive directory support
- Optimization level control
- Comprehensive logging and error reporting

## Installation

### Download Zero-Dependency Executables

Download the latest release from the [Releases page](https://github.com/loonghao/py2pyd/releases).

#### Windows (Zero Dependencies)
- **x86_64**: `py2pyd-x86_64-pc-windows-msvc.zip` - Runs on any Windows system
- **ARM64**: `py2pyd-aarch64-pc-windows-msvc.zip` - For ARM64 Windows systems

#### Linux (Static Binaries)
- **64-bit**: `py2pyd-x86_64-unknown-linux-musl.tar.gz` - Works on any Linux distribution

#### macOS
- **Apple Silicon**: `py2pyd-aarch64-apple-darwin.tar.gz`

> **Tip**: Windows MSVC and `x86_64-unknown-linux-musl` builds are completely self-contained with zero dependencies. Just download, extract, and run!

Only the archives that actually get published are listed here. `.github/workflows/release.yml` builds eight targets, but `aarch64-unknown-linux-musl` and `x86_64-apple-darwin` have not produced assets in any release so far; see [docs/RELEASE.md](docs/RELEASE.md) for the full matrix.

### Build from Source

```bash
git clone https://github.com/loonghao/py2pyd.git
cd py2pyd
cargo build --release
```

The binary will be available at `target/release/py2pyd` (or `py2pyd.exe` on Windows).

### Cross-Compilation Support

This project includes enhanced cross-compilation support based on [rust-actions-toolkit](https://github.com/loonghao/rust-actions-toolkit) best practices:

- **Windows targets**: Properly configured for `x86_64-pc-windows-gnu` and `i686-pc-windows-gnu`
- **Memory allocator compatibility**: Resolved `libmimalloc-sys` build errors in cross-compilation environments
- **Enhanced toolchain**: Pre-configured `Cross.toml` with proper environment variables

To test cross-compilation locally:
```bash
# Install cross-compilation tool
cargo install cross

# Test Windows targets
cross build --target x86_64-pc-windows-gnu --release
cross build --target i686-pc-windows-gnu --release
```

For troubleshooting cross-compilation issues, see the [rust-actions-toolkit documentation](https://github.com/loonghao/rust-actions-toolkit/blob/master/docs/CROSS_COMPILATION_ISSUES.md).

## Usage

```bash
# Compile a single file (uv selects the Python interpreter)
py2pyd compile -i input.py -o output.pyd

# Select a Python version
py2pyd --python-version 3.10 compile -i input.py -o output.pyd

# Use an explicit Python interpreter
py2pyd --python-path C:/Python310/python.exe compile -i input.py -o output.pyd

# Batch compile a directory recursively
py2pyd --python-version 3.10 batch -i src/ -o build/ --recursive
```

### Global Options

These are accepted before the subcommand.

| Option | Description |
|--------|-------------|
| `--python-path <PYTHON_PATH>` | Path to Python interpreter |
| `--python-version <PYTHON_VERSION>` | Python version to use (e.g. `3.9`, `3.10`) |
| `--keep-temp` | Keep temporary files after compilation |
| `--use-uv` | Use uv for Python environment management (default: true) |
| `--packages <PACKAGES>` | Additional Python packages to install (comma-separated) |
| `-v, --verbose...` | Sets the level of verbosity (`-v`, `-vv`, `-vvv`) |

### `compile`

| Option | Description |
|--------|-------------|
| `-i, --input <INPUT>` | Input Python file (required) |
| `-o, --output <OUTPUT>` | Output extension module (default: same as input with `.pyd` extension on Windows, `.so` on Linux/macOS) |
| `-O, --optimize <OPTIMIZE>` | Optimization level (0-3) (default: 2) |

### `batch`

| Option | Description |
|--------|-------------|
| `-i, --input <INPUT>` | Input directory or glob pattern (required) |
| `-o, --output <OUTPUT>` | Output directory (required) |
| `-O, --optimize <OPTIMIZE>` | Optimization level (0-3) (default: 2) |
| `-r, --recursive` | Recursive search |

#### Exit behavior

`batch` tolerates individual failures: a file that cannot be compiled is logged
at `warn` level and the remaining files are still built. The exit code reflects
the batch as a whole:

| Result | Exit code |
|--------|-----------|
| Every file compiled | `0` |
| Some files compiled, some failed | `0` (the counts are logged) |
| Every file failed | non-zero |
| No Python file matched the input pattern | `0` (logged at `warn`) |

### Limited API and target Python version

Every build targets the Python Limited API (`Py_LIMITED_API`) matching the
interpreter it builds with, so an extension compiled for Python 3.9 also loads
on later Python 3.x releases.

The minimum supported target is **Python 3.9**, because Cython does not build
Limited API extensions below that version. Selecting an older interpreter fails
before compilation starts with an explicit error instead of a compiler error:

```text
Python 3.7 is too old for a Limited API build: Cython requires Python 3.9 or newer.
Select a newer interpreter with --python-version or --python-path
```

The target is taken from `--python-version` when given, otherwise from
`--python-path`, otherwise from the interpreter that `uv` resolves.

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
- `uv` for Python environment management (installed automatically if not found), or a local Python interpreter

The tool will automatically check for required build tools and provide installation instructions if they are missing.

## Testing

py2pyd includes comprehensive tests, including integration tests that download and compile real Python packages from PyPI.

### Quick Tests
```bash
# Run the default suite: unit tests plus integration tests that need no build tools or network
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

## TODO List

- [x] Implement flexible Python interpreter discovery
  - [x] PATH-based discovery
  - [x] uv integration with version selection (`--python-version 3.10`)
  - [x] Explicit path specification (`--python-path`)
- [ ] Improve MSVC compiler detection and integration
  - [x] Auto-detection of installed MSVC
  - [x] Clear error messages and installation guidance
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

This project uses [release-plz](https://github.com/release-plz/release-plz) with GitHub Actions. Releases are automatically triggered based on [Conventional Commits](https://www.conventionalcommits.org/):

### Automatic Version Bumping

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `feat:` | Minor (0.1.0 → 0.2.0) | `feat: add Python 3.12 support` |
| `fix:` | Patch (0.1.0 → 0.1.1) | `fix: resolve memory leak in parser` |
| `feat!:` or `BREAKING CHANGE:` | Major (0.1.0 → 1.0.0) | `feat!: redesign command-line interface` |
| `docs:`, `chore:`, etc. | No version bump | `docs: update installation guide` |

### How to Release

Use conventional commit messages, then merge the release pull request that release-plz opens:

```bash
# Feature addition (minor version bump)
git commit -m "feat: add support for Python 3.12"

# Bug fix (patch version bump)
git commit -m "fix: handle edge case in file parsing"

# Breaking change (minor version bump while the version is 0.x)
git commit -m "feat!: redesign command-line interface

BREAKING CHANGE: The --input flag is now required"
```

The CI system will automatically:
1. **Analyze commit messages** to determine version bump
2. **Open a release PR** that updates the version in `Cargo.toml` and the changelog
3. **Publish to crates.io** and create the `v*` tag plus the GitHub release once that PR is merged
4. **Build binaries** for all supported platforms (`.github/workflows/release.yml`)
5. **Upload artifacts** with generated release notes

For detailed information, see [docs/VERSIONING.md](docs/VERSIONING.md) and [docs/RELEASE.md](docs/RELEASE.md).

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
