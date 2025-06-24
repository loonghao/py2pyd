# Testing Guide for py2pyd

This document describes the various tests available for py2pyd, including integration tests that download and compile real Python packages.

## Test Categories

### 1. Unit Tests
Standard unit tests that run quickly and don't require external dependencies.

```bash
# Run all unit tests
cargo test

# Run unit tests with verbose output
cargo test -- --nocapture
```

### 2. Integration Tests (Ignored by Default)
These tests download real Python packages from PyPI and attempt to compile them. They require:
- Internet connection
- `pip` command available
- Python build tools (MSVC on Windows, GCC on Linux/macOS)

```bash
# Run all integration tests (including ignored ones)
cargo test -- --ignored

# Run specific integration test
cargo test test_download_six_package -- --ignored

# Run with verbose output
cargo test -- --ignored --nocapture
```

## Available Integration Tests

### Simple Package Tests (`tests/simple_package_test.rs`)

Tests compilation of simple, hand-crafted Python modules:

- `test_compile_simple_python_module`: Creates and compiles a basic Python module
- `test_compile_python_with_imports`: Tests compilation of modules with standard library imports
- `test_batch_compile_multiple_files`: Tests batch compilation of multiple Python files

```bash
# Run simple package tests
cargo test simple_package_tests -- --ignored
```

### Pip Download Tests (`tests/pip_download_test.rs`)

Tests downloading and analyzing real PyPI packages:

- `test_download_six_package`: Downloads the 'six' package and analyzes its structure
- `test_download_and_compile_package`: Downloads and attempts to compile a package
- `test_analyze_package_structure`: Analyzes multiple packages for structure

```bash
# Run pip download tests
cargo test pip_download_tests -- --ignored
```

### Full Integration Tests (`tests/integration_pip_package_test.rs`)

Comprehensive tests that download packages and attempt full compilation:

- `test_download_and_compile_pure_python_package`: Tests with the 'six' package
- `test_download_and_compile_requests_package`: Tests with the more complex 'requests' package

```bash
# Run full integration tests
cargo test integration_pip_package_test -- --ignored
```

## Test Packages Used

### Simple Packages (Good for Testing)
- **six** (v1.16.0): Small, pure Python compatibility library
- **certifi** (v2023.7.22): Simple certificate bundle package

### Complex Packages (Challenging)
- **requests** (v2.31.0): Popular HTTP library with multiple modules

## Prerequisites

### Required Tools
1. **Rust toolchain**: `rustup` and `cargo`
2. **Python**: Python 3.7+ with `pip`
3. **Build tools**:
   - **Windows**: Visual Studio Build Tools or MinGW-w64
   - **Linux**: `build-essential` package
   - **macOS**: Xcode Command Line Tools

### Installation Commands

#### Windows
```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Or install MinGW-w64
# Download from: https://www.mingw-w64.org/downloads/
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install build-essential python3 python3-pip
```

#### macOS
```bash
xcode-select --install
```

## Running Tests

### Quick Test (No External Dependencies)
```bash
# Run only unit tests
cargo test --lib
```

### Full Test Suite
```bash
# Run all tests including integration tests
cargo test -- --ignored

# Run with detailed output
RUST_LOG=debug cargo test -- --ignored --nocapture
```

### Specific Test Examples

#### Test Simple Module Compilation
```bash
cargo test test_compile_simple_python_module -- --ignored --nocapture
```

#### Test Package Download
```bash
cargo test test_download_six_package -- --ignored --nocapture
```

#### Test Batch Compilation
```bash
cargo test test_batch_compile_multiple_files -- --ignored --nocapture
```

## Expected Results

### Successful Compilation
When tests pass, you should see output like:
```
✅ Successfully compiled Python module to pyd
Compiled file size: 1234 bytes
✅ Successfully downloaded six
📁 Extract path: /tmp/.../extracted_six
🐍 Python files: 1
```

### Partial Success
Some tests may partially succeed:
```
Compilation results:
  ✅ Successful: 2
  ❌ Failed: 1
📊 Success rate: 66.7%
```

This is normal - not all Python code can be compiled to pyd format.

### Common Issues

#### Build Tools Missing
```
❌ Compilation failed: No suitable build tools found
```
**Solution**: Install Visual Studio Build Tools (Windows) or build-essential (Linux)

#### Network Issues
```
❌ Failed to download package: Connection timeout
```
**Solution**: Check internet connection and try again

#### Python Environment Issues
```
❌ Failed to create virtual environment
```
**Solution**: Ensure Python and pip are properly installed

## Debugging Tests

### Enable Verbose Logging
```bash
RUST_LOG=debug cargo test test_name -- --ignored --nocapture
```

### Check Test Output Directory
Tests create temporary directories. To inspect them:

1. Modify test to use a fixed directory instead of `TempDir`
2. Add `std::thread::sleep(std::time::Duration::from_secs(60))` before cleanup
3. Inspect the directory contents during the sleep

### Manual Testing
You can also test compilation manually:
```bash
# Create a test Python file
echo 'def hello(): return "Hello, World!"' > test.py

# Compile it
cargo run -- compile --input test.py --output test.pyd --use-uv true

# Check the result
ls -la test.pyd
```

## Contributing Test Cases

When adding new test cases:

1. Use `#[ignore]` for tests that require external dependencies
2. Add descriptive test names and documentation
3. Handle failures gracefully (some failures are expected)
4. Clean up temporary files
5. Add the test to this documentation

## Performance Considerations

Integration tests can be slow because they:
- Download packages from PyPI
- Extract archives
- Compile Python code
- Set up virtual environments

For faster development, focus on unit tests and only run integration tests when needed.
