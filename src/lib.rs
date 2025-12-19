// Allow certain warnings to pass CI
#![allow(unknown_lints)]
#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::single_match)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::useless_format)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::needless_return)]

//! # py2pyd
//!
//! A Rust library for compiling Python modules to pyd/so extension files.
//!
//! This library provides programmatic access to py2pyd's compilation capabilities,
//! allowing you to integrate Python-to-pyd compilation into your own Rust projects.
//!
//! ## Features
//!
//! - Compile single Python files to pyd/so extensions
//! - Batch compile multiple Python files
//! - Support for uv-based Python environment management
//! - Automatic build tools detection (MSVC, MinGW, GCC, Xcode)
//! - Python AST parsing and transformation
//!
//! ## Example
//!
//! ```rust,no_run
//! use py2pyd::{compile_file, CompileConfig};
//! use std::path::Path;
//!
//! fn main() -> anyhow::Result<()> {
//!     let config = CompileConfig::default();
//!     compile_file(
//!         Path::new("input.py"),
//!         Path::new("output.pyd"),
//!         &config,
//!     )?;
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use std::path::Path;

// Re-export modules for library usage
pub mod build_tools;
pub mod compiler;
pub mod parser;
pub mod python_env;
pub mod transformer;
pub mod turbo_downloader;
pub mod uv_compiler;
pub mod uv_env;

// Re-export commonly used types
pub use build_tools::{check_build_tools, detect_build_tools, BuildTools};
pub use compiler::{
    batch_compile as compiler_batch_compile, compile_file as compiler_compile_file,
};
pub use parser::{
    extract_classes, extract_from_imports, extract_functions, extract_imports, extract_module_vars,
    parse_file, parse_source,
};
pub use transformer::{generate_cargo_toml, transform_ast, transform_file, TransformedModule};
pub use uv_compiler::CompileConfig;
pub use uv_env::{UvEnv, UvEnvConfig};

/// Compile a single Python file to a pyd/so extension using uv-based compilation.
///
/// This is the recommended way to compile Python files as it uses uv for
/// Python environment management, which is faster and more reliable.
///
/// # Arguments
///
/// * `input` - Path to the input Python file
/// * `output` - Path to the output pyd/so file
/// * `config` - Compilation configuration
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if compilation fails.
///
/// # Example
///
/// ```rust,no_run
/// use py2pyd::{compile_file, CompileConfig};
/// use std::path::Path;
///
/// let config = CompileConfig::default();
/// compile_file(
///     Path::new("my_module.py"),
///     Path::new("my_module.pyd"),
///     &config,
/// ).expect("Compilation failed");
/// ```
pub fn compile_file(input: &Path, output: &Path, config: &CompileConfig) -> Result<()> {
    uv_compiler::compile_file(input, output, config)
}

/// Batch compile multiple Python files to pyd/so extensions.
///
/// This function compiles all Python files matching the input pattern
/// to the specified output directory.
///
/// # Arguments
///
/// * `input_pattern` - A directory path or glob pattern for input files
/// * `output_dir` - Directory where compiled files will be placed
/// * `config` - Compilation configuration
/// * `recursive` - Whether to search for Python files recursively
///
/// # Returns
///
/// Returns `Ok(())` on success. Note that individual file compilation
/// failures are logged but don't cause the entire batch to fail.
///
/// # Example
///
/// ```rust,no_run
/// use py2pyd::{batch_compile, CompileConfig};
/// use std::path::Path;
///
/// let config = CompileConfig::default();
/// batch_compile(
///     "src/python",
///     Path::new("dist"),
///     &config,
///     true,  // recursive
/// ).expect("Batch compilation failed");
/// ```
pub fn batch_compile(
    input_pattern: &str,
    output_dir: &Path,
    config: &CompileConfig,
    recursive: bool,
) -> Result<()> {
    uv_compiler::batch_compile(input_pattern, output_dir, config, recursive)
}

/// Compile a Python file using the legacy compiler (without uv).
///
/// This function uses the traditional compilation approach without uv.
/// It's provided for backward compatibility but `compile_file` is recommended.
///
/// # Arguments
///
/// * `input` - Path to the input Python file
/// * `output` - Path to the output pyd/so file
/// * `optimize_level` - Optimization level (0-3)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if compilation fails.
pub fn compile_file_legacy(input: &Path, output: &Path, optimize_level: u8) -> Result<()> {
    compiler::compile_file(input, output, "generic", optimize_level)
}

/// Batch compile using the legacy compiler (without uv).
///
/// This function uses the traditional compilation approach without uv.
/// It's provided for backward compatibility but `batch_compile` is recommended.
///
/// # Arguments
///
/// * `input_pattern` - A directory path or glob pattern for input files
/// * `output_dir` - Directory where compiled files will be placed
/// * `optimize_level` - Optimization level (0-3)
/// * `recursive` - Whether to search for Python files recursively
///
/// # Returns
///
/// Returns `Ok(())` on success.
pub fn batch_compile_legacy(
    input_pattern: &str,
    output_dir: &Path,
    optimize_level: u8,
    recursive: bool,
) -> Result<()> {
    compiler::batch_compile(
        input_pattern,
        output_dir,
        "generic",
        optimize_level,
        recursive,
    )
}

/// Get the appropriate extension for compiled Python modules on the current platform.
///
/// Returns "pyd" on Windows and "so" on other platforms.
///
/// # Example
///
/// ```rust
/// use py2pyd::get_extension;
///
/// let ext = get_extension();
/// #[cfg(windows)]
/// assert_eq!(ext, "pyd");
/// #[cfg(not(windows))]
/// assert_eq!(ext, "so");
/// ```
#[must_use]
pub const fn get_extension() -> &'static str {
    if cfg!(windows) {
        "pyd"
    } else {
        "so"
    }
}

/// Check if the required build tools are available on the system.
///
/// This function detects available compilers (MSVC, MinGW, GCC, Xcode)
/// and returns an error if no suitable tools are found.
///
/// # Returns
///
/// Returns `Ok(BuildTools)` with information about available tools,
/// or an error with installation instructions if no tools are found.
///
/// # Example
///
/// ```rust,no_run
/// use py2pyd::verify_build_tools;
///
/// match verify_build_tools() {
///     Ok(tools) => println!("Build tools available:\n{}", tools.get_tools_info()),
///     Err(e) => eprintln!("No build tools found: {}", e),
/// }
/// ```
pub fn verify_build_tools() -> Result<BuildTools> {
    check_build_tools()
}

/// Create a new uv-based Python virtual environment.
///
/// This function creates a virtual environment using uv and optionally
/// installs specified packages.
///
/// # Arguments
///
/// * `config` - Configuration for the virtual environment
///
/// # Returns
///
/// Returns `Ok(UvEnv)` on success, which provides access to the
/// Python interpreter and other environment utilities.
///
/// # Example
///
/// ```rust,no_run
/// use py2pyd::{create_uv_env, UvEnvConfig};
///
/// let config = UvEnvConfig {
///     python_version: Some("3.10".to_string()),
///     packages: vec!["numpy".to_string(), "pandas".to_string()],
///     ..Default::default()
/// };
///
/// let env = create_uv_env(&config).expect("Failed to create environment");
/// println!("Python path: {}", env.python_path.display());
/// ```
pub fn create_uv_env(config: &UvEnvConfig) -> Result<UvEnv> {
    UvEnv::create(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension() {
        let ext = get_extension();
        #[cfg(windows)]
        assert_eq!(ext, "pyd");
        #[cfg(not(windows))]
        assert_eq!(ext, "so");
    }

    #[test]
    fn test_compile_config_default() {
        let config = CompileConfig::default();
        assert!(config.python_path.is_none());
        assert!(config.python_version.is_none());
        assert_eq!(config.optimize_level, 2);
        assert!(!config.keep_temp_files);
        assert!(config.target_dcc.is_none());
        assert!(config.packages.is_empty());
    }

    #[test]
    fn test_uv_env_config_default() {
        let config = UvEnvConfig::default();
        assert!(config.python_path.is_none());
        assert!(config.python_version.is_none());
        assert!(!config.keep_venv);
        assert!(config.packages.is_empty());
    }
}
