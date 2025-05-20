use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use crate::uv_env::{UvEnv, UvEnvConfig};

/// Configuration for compiling a Python module to a pyd file
pub struct CompileConfig {
    /// Path to the Python interpreter to use
    pub python_path: Option<PathBuf>,

    /// Python version to use (e.g., "3.9")
    pub python_version: Option<String>,

    /// Optimization level (0-3)
    pub optimize_level: u8,

    /// Whether to keep temporary files
    pub keep_temp_files: bool,

    /// Target environment (for future use)
    pub target_dcc: Option<String>,

    /// Additional packages to install
    pub packages: Vec<String>,
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            python_path: None,
            python_version: None,
            optimize_level: 2,
            keep_temp_files: false,
            target_dcc: None,
            packages: vec![],
        }
    }
}

/// Compile a Python file to a pyd file using uv
pub fn compile_file(input_path: &Path, output_path: &Path, config: &CompileConfig) -> Result<()> {
    info!(
        "Compiling {} to {}",
        input_path.display(),
        output_path.display()
    );

    // Create a temporary directory for the build
    let temp_dir = TempDir::new().with_context(|| "Failed to create temporary directory")?;

    // If keep_temp_files is true, don't delete the temp directory when it's dropped
    let temp_dir_path = if config.keep_temp_files {
        let path = temp_dir.path().to_path_buf();
        temp_dir.into_path();
        path
    } else {
        temp_dir.path().to_path_buf()
    };

    debug!("Using temporary directory: {}", temp_dir_path.display());

    // Get the module name from the input file name
    let module_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid input file name"))?;

    // Read the Python source code
    let source_code = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read input file: {}", input_path.display()))?;

    // Create the setup.py file
    let setup_py_path = temp_dir_path.join("setup.py");
    let setup_py_content = generate_setup_py(module_name, &source_code, config)?;
    fs::write(&setup_py_path, setup_py_content)
        .with_context(|| format!("Failed to write setup.py to {}", setup_py_path.display()))?;

    // Copy the Python source file to the temp directory
    let source_path = temp_dir_path.join(format!("{}.py", module_name));
    fs::write(&source_path, source_code)
        .with_context(|| format!("Failed to write source file to {}", source_path.display()))?;

    // Create a uv virtual environment
    let mut packages = vec![
        "setuptools>=60.0.0".to_string(),
        "wheel>=0.37.0".to_string(),
        "cython>=3.0.0".to_string(),
    ];

    // Add user-specified packages
    packages.extend(config.packages.clone());

    let uv_config = UvEnvConfig {
        python_path: config.python_path.clone(),
        python_version: config.python_version.clone(),
        keep_venv: config.keep_temp_files,
        packages,
    };

    let uv_env =
        UvEnv::create(&uv_config).with_context(|| "Failed to create uv virtual environment")?;

    info!(
        "Created uv virtual environment at: {}",
        uv_env.venv_path.display()
    );
    info!("Using Python interpreter: {}", uv_env.python_path.display());

    // Build the extension module
    info!("Building extension module...");
    let status = Command::new(&uv_env.python_path)
        .current_dir(&temp_dir_path)
        .arg("setup.py")
        .arg("build_ext")
        .arg("--inplace")
        .status()
        .with_context(|| "Failed to execute Python setup.py build_ext")?;

    if !status.success() {
        return Err(anyhow!("Failed to build extension module"));
    }

    // Find the compiled extension module
    let extension = if cfg!(windows) { "pyd" } else { "so" };
    let mut extension_path = None;

    for entry in walkdir::WalkDir::new(&temp_dir_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == extension) {
            extension_path = Some(path.to_path_buf());
            break;
        }
    }

    let extension_path =
        extension_path.ok_or_else(|| anyhow!("Failed to find compiled extension module"))?;
    debug!(
        "Found compiled extension module: {}",
        extension_path.display()
    );

    // Create the output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Copy the compiled extension module to the output path
    fs::copy(&extension_path, output_path).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            extension_path.display(),
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
    config: &CompileConfig,
    recursive: bool,
) -> Result<()> {
    info!(
        "Batch compiling from {} to {}",
        input_pattern,
        output_dir.display()
    );

    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Collect all Python files matching the pattern
    let python_files = collect_python_files(input_pattern, recursive).with_context(|| {
        format!(
            "Failed to collect Python files from pattern: {}",
            input_pattern
        )
    })?;

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
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Compile the file
        match compile_file(&input_path, &output_path, config) {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                warn!("Failed to compile {}: {}", input_path.display(), e);
                failure_count += 1;
            }
        }
    }

    info!(
        "Batch compilation complete: {} succeeded, {} failed",
        success_count, failure_count
    );

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
        debug!("Pattern is a directory: {}", pattern);

        // Collect Python files from the directory
        if recursive {
            for entry in walkdir::WalkDir::new(pattern_path)
                .into_iter()
                .filter_map(|e| e.ok())
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
        debug!("Pattern is a glob pattern: {}", pattern);

        for entry in
            glob::glob(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?
        {
            let path = entry?;
            if path.is_file() && path.extension().map_or(false, |ext| ext == "py") {
                python_files.push(path);
            }
        }
    }

    debug!("Collected {} Python files", python_files.len());
    Ok(python_files)
}

/// Generate a setup.py file for building the extension module
fn generate_setup_py(
    module_name: &str,
    source_code: &str,
    config: &CompileConfig,
) -> Result<String> {
    let mut setup_py = String::new();

    setup_py.push_str("from setuptools import setup, Extension\n");
    setup_py.push_str("from setuptools.command.build_ext import build_ext\n");
    setup_py.push_str("import sys\n\n");

    // Add custom build_ext class to support ABI3
    setup_py.push_str("class ABI3BuildExt(build_ext):\n");
    setup_py.push_str("    def build_extension(self, ext):\n");
    setup_py.push_str("        ext.py_limited_api = True\n");
    setup_py.push_str("        super().build_extension(ext)\n\n");

    // Setup the extension module
    setup_py.push_str("setup(\n");
    setup_py.push_str(&format!("    name='{}',\n", module_name));
    setup_py.push_str("    version='0.1',\n");
    setup_py.push_str(&format!("    ext_modules=[Extension(\n"));
    setup_py.push_str(&format!("        '{}',\n", module_name));
    setup_py.push_str(&format!("        sources=['{}.py'],\n", module_name));

    // Add custom include paths if needed in the future
    // Currently not used

    // Enable ABI3 compatibility
    setup_py.push_str("        py_limited_api=True,\n");
    setup_py.push_str("        define_macros=[('Py_LIMITED_API', '0x03070000')],\n");
    setup_py.push_str("    )],\n");

    // Use custom build_ext class
    setup_py.push_str("    cmdclass={'build_ext': ABI3BuildExt},\n");

    setup_py.push_str(")\n");

    Ok(setup_py)
}
