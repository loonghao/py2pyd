use anyhow::{Context, Result};
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

/// Clean up the virtual environment
pub fn cleanup_venv() -> Result<()> {
    let venv_dir = super::get_venv_dir()?;
    
    if venv_dir.exists() {
        info!("Cleaning up virtual environment at {}", venv_dir.display());
        fs::remove_dir_all(&venv_dir)
            .with_context(|| format!("Failed to remove virtual environment at {}", venv_dir.display()))?;
        debug!("Virtual environment cleaned up successfully");
    } else {
        debug!("No virtual environment to clean up");
    }
    
    Ok(())
}

/// Get the path to the virtual environment
pub fn get_venv_path() -> Result<PathBuf> {
    super::get_venv_dir()
}
