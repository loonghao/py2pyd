use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Create a virtual environment with a specific Python version using uv
pub fn create_venv_with_uv_and_version(uv_path: &Path, python_version: &str) -> Result<PathBuf> {
    let venv_dir = super::get_venv_dir()?;

    // Check if the virtual environment already exists
    if venv_dir.exists() {
        debug!(
            "Virtual environment already exists at {}",
            venv_dir.display()
        );
        return Ok(venv_dir);
    }

    // Create the parent directory if it doesn't exist
    if let Some(parent) = venv_dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Create the virtual environment with specific Python version
    info!(
        "Creating virtual environment with Python {} at {}",
        python_version,
        venv_dir.display()
    );

    // In uv 0.7.6, the command is "uv venv create --python X.Y [path]"
    let status = Command::new(uv_path)
        .arg("venv")
        .arg("create")
        .arg("--python")
        .arg(python_version)
        .arg(venv_dir.to_str().unwrap())
        .status()
        .with_context(|| {
            format!("Failed to execute uv venv create with Python {python_version}")
        })?;

    if !status.success() {
        return Err(anyhow!(
            "Failed to create virtual environment with Python {}",
            python_version
        ));
    }

    info!(
        "Virtual environment created successfully with Python {} at {}",
        python_version,
        venv_dir.display()
    );
    Ok(venv_dir)
}
