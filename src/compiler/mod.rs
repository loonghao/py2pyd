use anyhow::{Context, Result};
use glob::glob;
use log::{debug, error, info, warn};
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::transformer::TransformedModule;

/// Compile a single Python file to a pyd file
pub fn compile_file(
    input_path: &Path,
    output_path: &Path,
    _: &str, // Unused but kept for backward compatibility
    optimize_level: u8,
) -> Result<()> {
    info!(
        "Compiling {} to {}",
        input_path.display(),
        output_path.display()
    );

    // Target parameter is kept for backward compatibility
    debug!("Using generic target");

    // Transform the Python file to Rust
    let transformed = crate::transformer::transform_file(input_path, optimize_level)
        .with_context(|| format!("Failed to transform Python file: {}", input_path.display()))?;

    // Create the Rust project
    create_rust_project(&transformed).with_context(|| "Failed to create Rust project")?;

    // Build the Rust project
    build_rust_project(&transformed).with_context(|| "Failed to build Rust project")?;

    // Copy the compiled library to the output path
    copy_compiled_library(&transformed, output_path).with_context(|| {
        format!(
            "Failed to copy compiled library to {}",
            output_path.display()
        )
    })?;

    info!(
        "Successfully compiled {} to {}",
        input_path.display(),
        output_path.display()
    );
    Ok(())
}

/// Batch compile multiple Python files to pyd files
pub fn batch_compile(
    input_pattern: &str,
    output_dir: &Path,
    _: &str, // Unused but kept for backward compatibility
    optimize_level: u8,
    recursive: bool,
) -> Result<()> {
    info!(
        "Batch compiling from {} to {}",
        input_pattern,
        output_dir.display()
    );

    // Create the output directory if it doesn't exist
    create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Collect all Python files matching the pattern
    let python_files = collect_python_files(input_pattern, recursive)
        .with_context(|| format!("Failed to collect Python files from pattern: {input_pattern}"))?;

    info!("Found {} Python files to compile", python_files.len());

    // Compile each Python file
    let mut success_count = 0;
    let mut failure_count = 0;

    for input_path in python_files {
        // Determine the output path
        let relative_path = input_path
            .strip_prefix(Path::new(input_pattern))
            .unwrap_or(&input_path);
        let mut output_path = output_dir.join(relative_path);

        // Use the appropriate extension based on the platform
        if cfg!(windows) {
            output_path.set_extension("pyd");
        } else {
            output_path.set_extension("so");
        }

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Compile the file
        match compile_file(&input_path, &output_path, "", optimize_level) {
            Ok(()) => {
                success_count += 1;
            }
            Err(e) => {
                error!("Failed to compile {}: {}", input_path.display(), e);
                failure_count += 1;
            }
        }
    }

    info!("Batch compilation complete: {success_count} succeeded, {failure_count} failed");

    if failure_count > 0 {
        warn!("Some files failed to compile");
    }

    Ok(())
}

/// Collect Python files matching a pattern
fn collect_python_files(pattern: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut python_files = Vec::new();

    // Check if the pattern is a directory
    let pattern_path = Path::new(pattern);
    if pattern_path.is_dir() {
        debug!("Pattern is a directory: {pattern}");

        // Collect Python files from the directory
        if recursive {
            for entry in WalkDir::new(pattern_path)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "py") {
                    python_files.push(path.to_path_buf());
                }
            }
        } else {
            for entry in fs::read_dir(pattern_path)
                .with_context(|| format!("Failed to read directory: {}", pattern_path.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "py") {
                    python_files.push(path);
                }
            }
        }
    } else {
        // Treat the pattern as a glob pattern
        debug!("Pattern is a glob pattern: {pattern}");

        for entry in glob(pattern).with_context(|| format!("Invalid glob pattern: {pattern}"))? {
            let path = entry?;
            if path.is_file() && path.extension().map_or(false, |ext| ext == "py") {
                python_files.push(path);
            }
        }
    }

    debug!("Collected {} Python files", python_files.len());
    Ok(python_files)
}

/// Create a Rust project from a transformed module
fn create_rust_project(transformed: &TransformedModule) -> Result<()> {
    info!(
        "Creating Rust project in {}",
        transformed.build_dir.display()
    );

    // Create the src directory
    let src_dir = transformed.build_dir.join("src");
    create_dir_all(&src_dir)
        .with_context(|| format!("Failed to create src directory: {}", src_dir.display()))?;

    // Write the Cargo.toml file
    fs::write(
        transformed.build_dir.join("Cargo.toml"),
        &transformed.cargo_toml,
    )
    .with_context(|| "Failed to write Cargo.toml")?;

    // Write the lib.rs file
    fs::write(src_dir.join("lib.rs"), &transformed.rust_code)
        .with_context(|| "Failed to write lib.rs")?;

    debug!("Created Rust project files");
    Ok(())
}

/// Build a Rust project
fn build_rust_project(transformed: &TransformedModule) -> Result<()> {
    info!(
        "Building Rust project in {}",
        transformed.build_dir.display()
    );

    // Instead of using Python to build, we'll use cargo directly
    // This is more reliable and doesn't require Python environment setup

    // First, create a pyproject.toml file for maturin
    let pyproject_toml = r#"[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "extension_module"
requires-python = ">=3.7"

[tool.maturin]
features = ["pyo3/extension-module"]
"#;

    fs::write(transformed.build_dir.join("pyproject.toml"), pyproject_toml)
        .with_context(|| "Failed to write pyproject.toml")?;

    // Use cargo directly to build the extension
    info!("Building with cargo...");
    let status = Command::new("cargo")
        .current_dir(&transformed.build_dir)
        .arg("build")
        .arg("--release")
        .status()
        .with_context(|| "Failed to execute cargo build")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Cargo build failed with status: {}",
            status
        ));
    }

    debug!("Built Rust project successfully with cargo");
    Ok(())
}

/// Copy the compiled library to the output path
fn copy_compiled_library(transformed: &TransformedModule, output_path: &Path) -> Result<()> {
    info!("Copying compiled library to {}", output_path.display());

    // Determine the compiled library path
    // Cargo puts the compiled library in target/release
    let lib_name = if cfg!(windows) {
        format!(
            "{}.dll",
            transformed.build_dir.file_name().unwrap().to_string_lossy()
        )
    } else if cfg!(target_os = "macos") {
        format!(
            "lib{}.dylib",
            transformed.build_dir.file_name().unwrap().to_string_lossy()
        )
    } else {
        format!(
            "lib{}.so",
            transformed.build_dir.file_name().unwrap().to_string_lossy()
        )
    };

    let compiled_lib_path = transformed
        .build_dir
        .join("target")
        .join("release")
        .join(&lib_name);

    if !compiled_lib_path.exists() {
        // Try to find the library by searching in the release directory
        let release_dir = transformed.build_dir.join("target").join("release");
        let mut found_lib = None;

        if release_dir.exists() {
            for entry in fs::read_dir(&release_dir)
                .with_context(|| format!("Failed to read directory: {}", release_dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if (cfg!(windows) && ext == "dll")
                        || (cfg!(target_os = "macos") && ext == "dylib")
                        || (!cfg!(windows) && !cfg!(target_os = "macos") && ext == "so")
                    {
                        found_lib = Some(path);
                        break;
                    }
                }
            }
        }

        if let Some(path) = found_lib {
            debug!("Found library at: {}", path.display());

            // Create the output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // Copy the compiled library
            fs::copy(&path, output_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    path.display(),
                    output_path.display()
                )
            })?;

            debug!("Copied compiled library to {}", output_path.display());
            return Ok(());
        }

        return Err(anyhow::anyhow!(
            "No compiled library found in {}",
            release_dir.display()
        ));
    }

    debug!("Found compiled library at: {}", compiled_lib_path.display());

    // Create the output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Copy the compiled library
    fs::copy(&compiled_lib_path, output_path).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            compiled_lib_path.display(),
            output_path.display()
        )
    })?;

    debug!("Copied compiled library to {}", output_path.display());
    Ok(())
}
