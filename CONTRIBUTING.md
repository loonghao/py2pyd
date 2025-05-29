# Contributing to py2pyd

Thank you for your interest in contributing to py2pyd! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- A C/C++ compiler:
  - **Windows**: Visual Studio Build Tools or MinGW-w64
  - **Linux**: GCC (`sudo apt-get install build-essential`)
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)

### Development Setup

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/py2pyd.git
   cd py2pyd
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Run the tool**:
   ```bash
   cargo run -- --help
   ```

## 📝 Commit Message Guidelines

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automatic semantic versioning and release generation.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description | Version Bump |
|------|-------------|--------------|
| `feat` | A new feature | Minor |
| `fix` | A bug fix | Patch |
| `docs` | Documentation only changes | Patch |
| `style` | Changes that do not affect the meaning of the code | Patch |
| `refactor` | A code change that neither fixes a bug nor adds a feature | Patch |
| `perf` | A code change that improves performance | Patch |
| `test` | Adding missing tests or correcting existing tests | Patch |
| `chore` | Changes to the build process or auxiliary tools | Patch |

### Breaking Changes

For breaking changes, add `!` after the type or include `BREAKING CHANGE:` in the footer:

```bash
feat!: redesign command-line interface
# or
feat(api): redesign command-line interface

BREAKING CHANGE: The --input flag is now required
```

### Examples

```bash
# Feature additions
feat(parser): add support for Python 3.12
feat(cli): add --recursive flag for batch processing

# Bug fixes
fix(compiler): resolve segmentation fault on Windows
fix(parser): handle edge case with empty files

# Documentation
docs(readme): add installation instructions
docs(api): update function documentation

# Refactoring
refactor(core): simplify error handling logic
refactor(parser): extract common functionality

# Performance improvements
perf(build): optimize compilation speed
perf(memory): reduce memory usage in parser

# Tests
test(parser): add unit tests for edge cases
test(integration): add end-to-end compilation tests

# Chores
chore(deps): update rustpython-parser to 0.4.1
chore(ci): update GitHub Actions versions
```

## 🔄 Pull Request Process

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Your Changes

- Write clear, readable code
- Add tests for new functionality
- Update documentation as needed
- Follow Rust best practices and idioms

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run with different features
cargo test --all-features

# Check formatting
cargo fmt --check

# Run linter (if available)
cargo clippy
```

### 4. Commit Your Changes

Use conventional commit messages:

```bash
git add .
git commit -m "feat(parser): add support for async functions"
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title following conventional commits
- Detailed description of changes
- Reference to any related issues

## 🧪 Testing Guidelines

### Unit Tests

- Write unit tests for all new functions
- Test edge cases and error conditions
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        // Test implementation
    }

    #[test]
    fn test_parse_empty_file_returns_error() {
        // Test implementation
    }
}
```

### Integration Tests

- Add integration tests in `tests/` directory
- Test complete workflows
- Test with real Python files

## 📚 Documentation

### Code Documentation

- Add rustdoc comments for public APIs
- Include examples in documentation
- Document error conditions

```rust
/// Parse a Python file into an AST
/// 
/// # Arguments
/// 
/// * `path` - Path to the Python file
/// 
/// # Returns
/// 
/// Returns `Ok(ast::Suite)` on success, or an error if parsing fails
/// 
/// # Examples
/// 
/// ```
/// use py2pyd::parser::parse_file;
/// let ast = parse_file("example.py")?;
/// ```
pub fn parse_file(path: &Path) -> Result<ast::Suite> {
    // Implementation
}
```

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update troubleshooting guides

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment information**:
   - Operating system and version
   - Rust version (`rustc --version`)
   - py2pyd version

2. **Steps to reproduce**:
   - Minimal example that reproduces the issue
   - Expected vs actual behavior

3. **Additional context**:
   - Error messages or logs
   - Relevant configuration

## 💡 Feature Requests

For feature requests:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** and motivation
3. **Propose a solution** if you have ideas
4. **Consider implementation complexity**

## 🔧 Development Tips

### Useful Commands

```bash
# Watch for changes and rebuild
cargo watch -x build

# Run with debug logging
RUST_LOG=debug cargo run -- compile input.py

# Profile performance
cargo build --release
perf record target/release/py2pyd compile input.py
```

### IDE Setup

Recommended VS Code extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Better TOML

## 📋 Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use meaningful variable and function names
- Keep functions focused and small
- Add comments for complex logic
- Use `Result<T>` for error handling

## 🚀 Release Process

Releases are automated based on conventional commits:

- `feat:` commits trigger minor version bumps
- `fix:` commits trigger patch version bumps  
- `feat!:` or `BREAKING CHANGE:` trigger major version bumps

The CI system automatically:
1. Analyzes commit messages
2. Updates version in `Cargo.toml`
3. Creates Git tags
4. Builds and publishes releases

## ❓ Questions?

If you have questions:

1. Check existing [issues](https://github.com/loonghao/py2pyd/issues)
2. Read the [documentation](docs/)
3. Create a new issue with the `question` label

Thank you for contributing to py2pyd! 🎉
