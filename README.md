# py2pyd

A Rust-based tool to compile Python modules to pyd files, with special support for DCC environments like Maya and Houdini.

> **Warning**: This project is currently under active development and not ready for production use. APIs and functionality may change significantly between versions.

## Overview

py2pyd is a command-line tool that simplifies the process of compiling Python (.py) files to Python extension modules (.pyd) for Windows. It's particularly useful for developers working with Digital Content Creation (DCC) applications like Maya, Houdini, and others.

## Features (Planned)

- Compile single Python files or entire directories to pyd files
- Support for multiple Python interpreter discovery methods:
  - Default PATH lookup
  - uv integration with version selection (`--uv-python 3.10`)
  - Explicit interpreter path specification (`--python-path`)
- Batch processing with recursive directory support
- Optimization level control
- Special handling for DCC-specific Python environments
- Comprehensive logging and error reporting

## Installation

*Installation instructions will be provided once the project reaches a stable release.*

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

- Windows operating system
- Compatible C/C++ compiler (MSVC) for the target Python version
- Python interpreter (if not using embedded mode)

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
  - [ ] CI/CD pipeline setup
- [ ] Improve documentation
  - [ ] Detailed usage examples
  - [ ] Troubleshooting guide
  - [ ] API documentation
- [ ] Future enhancements
  - [ ] Investigate embedded Python interpreter option
  - [ ] Support for additional platforms (Linux, macOS)
  - [ ] Performance optimizations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
