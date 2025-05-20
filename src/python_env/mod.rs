use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;
use std::env;
use std::fs::{self, File};
use std::io::{self, copy, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use which::which;
use zip::ZipArchive;

mod version;
mod cleanup;
pub use version::create_venv_with_uv_and_version;
pub use cleanup::{cleanup_venv, get_venv_path};

// UV tool URLs and versions
const UV_VERSION: &str = "0.7.6";
const UV_WINDOWS_URL: &str = "https://github.com/astral-sh/uv/releases/download/0.7.6/uv-x86_64-pc-windows-msvc.zip";

// Global state for Python environment
static PYTHON_ENV: Lazy<Mutex<PythonEnvironment>> = Lazy::new(|| Mutex::new(PythonEnvironment::new()));

/// Represents a Python environment configuration
pub struct PythonEnvironment {
    python_path: Option<PathBuf>,
    uv_path: Option<PathBuf>,
    venv_path: Option<PathBuf>,
    initialized: bool,
}

impl PythonEnvironment {
    /// Create a new Python environment configuration
    fn new() -> Self {
        PythonEnvironment {
            python_path: None,
            uv_path: None,
            venv_path: None,
            initialized: false,
        }
    }
}

/// Initialize the Python environment with the given configuration
pub fn initialize_python_env(python_path: Option<&str>, python_version: Option<&str>) -> Result<()> {
    let mut env = PYTHON_ENV.lock().unwrap();

    if env.initialized {
        debug!("Python environment already initialized");
        return Ok(());
    }

    // 1. Try to use explicitly provided Python path
    if let Some(path) = python_path {
        let path = PathBuf::from(path);
        if path.exists() {
            info!("Using provided Python interpreter: {}", path.display());
            env.python_path = Some(path);
            env.initialized = true;
            return Ok(());
        } else {
            warn!("Provided Python interpreter not found: {}", path.display());
        }
    }

    // 2. Try to find Python in PATH (if no specific version is requested)
    if python_version.is_none() {
        match find_python_in_path() {
            Ok(path) => {
                info!("Found Python interpreter in PATH: {}", path.display());
                env.python_path = Some(path);
                env.initialized = true;
                return Ok(());
            }
            Err(e) => {
                debug!("Failed to find Python in PATH: {}", e);
            }
        }
    }

    // 3. Use uv to create a Python environment
    info!("Setting up uv...");
    let uv_path = setup_uv()?;
    env.uv_path = Some(uv_path.clone());

    // Create a virtual environment with specified Python version
    let venv_path = if let Some(version) = python_version {
        info!("Creating virtual environment with Python {}", version);
        create_venv_with_uv_and_version(&uv_path, version)?
    } else {
        info!("Creating virtual environment with default Python");
        create_venv_with_uv(&uv_path)?
    };

    env.venv_path = Some(venv_path.clone());

    // Get Python path from the virtual environment
    let python_path = get_python_from_venv(&venv_path)?;
    env.python_path = Some(python_path);

    env.initialized = true;
    info!("Python environment initialized successfully");
    Ok(())
}

/// Get the path to the Python interpreter
pub fn get_python_path() -> Result<PathBuf> {
    let env = PYTHON_ENV.lock().unwrap();

    if !env.initialized {
        return Err(anyhow!("Python environment not initialized"));
    }

    env.python_path
        .clone()
        .ok_or_else(|| anyhow!("Python interpreter not found"))
}

/// Find a Python interpreter in the system PATH
fn find_python_in_path() -> Result<PathBuf> {
    // Try different Python executable names
    for name in &["python", "python3", "py"] {
        match which(name) {
            Ok(path) => {
                // Verify it's Python 3.x
                if is_python3(&path)? {
                    return Ok(path);
                }
            }
            Err(_) => continue,
        }
    }

    Err(anyhow!("No Python 3.x interpreter found in PATH"))
}

/// Check if the given path points to a Python 3.x interpreter
fn is_python3(path: &Path) -> Result<bool> {
    let output = Command::new(path)
        .args(&["--version"])
        .output()
        .with_context(|| format!("Failed to execute Python at {}", path.display()))?;

    if !output.status.success() {
        return Ok(false);
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    if version_str.is_empty() {
        let version_str = String::from_utf8_lossy(&output.stderr);
        is_python3_version(&version_str)
    } else {
        is_python3_version(&version_str)
    }
}

/// Check if the version string indicates Python 3.x
fn is_python3_version(version_str: &str) -> Result<bool> {
    static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"Python (\d+)\.").unwrap());

    if let Some(captures) = VERSION_REGEX.captures(version_str) {
        if let Some(major_version) = captures.get(1) {
            let major_version = major_version.as_str().parse::<u32>()
                .with_context(|| format!("Failed to parse Python version: {}", version_str))?;
            return Ok(major_version >= 3);
        }
    }

    Ok(false)
}

/// Set up the uv tool
fn setup_uv() -> Result<PathBuf> {
    // First, try to find uv in PATH
    match which("uv") {
        Ok(path) => {
            debug!("Found uv in PATH: {}", path.display());
            return Ok(path);
        }
        Err(_) => {
            debug!("uv not found in PATH, will download and install");
        }
    }

    let uv_dir = get_uv_dir()?;
    let uv_exe = uv_dir.join("uv.exe");

    // Check if uv is already installed
    if uv_exe.exists() {
        debug!("uv already installed at {}", uv_exe.display());
        return Ok(uv_exe);
    }

    // Create the directory if it doesn't exist
    fs::create_dir_all(&uv_dir)
        .with_context(|| format!("Failed to create directory: {}", uv_dir.display()))?;

    // Download uv
    info!("Downloading uv v{} from {}", UV_VERSION, UV_WINDOWS_URL);
    let zip_path = uv_dir.join("uv.zip");
    download_file(UV_WINDOWS_URL, &zip_path)
        .with_context(|| format!("Failed to download uv from {}", UV_WINDOWS_URL))?;

    // Extract uv
    info!("Extracting uv to {}", uv_dir.display());
    extract_zip(&zip_path, &uv_dir)
        .with_context(|| format!("Failed to extract uv to {}", uv_dir.display()))?;

    // Clean up
    fs::remove_file(&zip_path)
        .with_context(|| format!("Failed to remove temporary file: {}", zip_path.display()))?;

    info!("uv installed successfully at {}", uv_exe.display());
    Ok(uv_exe)
}

/// Get the directory where uv should be installed
fn get_uv_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow!("Failed to determine data directory"))?;

    Ok(data_dir.join("py2pyd").join("uv").join(UV_VERSION))
}

/// Download a file from a URL
fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = Client::new();
    let mut response = client.get(url)
        .send()
        .with_context(|| format!("Failed to download from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download from {}: {}", url, response.status()));
    }

    let mut file = File::create(dest)
        .with_context(|| format!("Failed to create file: {}", dest.display()))?;

    copy(&mut response, &mut file)
        .with_context(|| format!("Failed to write to file: {}", dest.display()))?;

    Ok(())
}

/// Extract a zip file
fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = File::open(zip_path)
        .with_context(|| format!("Failed to open zip file: {}", zip_path.display()))?;

    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("Failed to read zip file: {}", zip_path.display()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .with_context(|| format!("Failed to read file {} in zip", i))?;

        let outpath = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .with_context(|| format!("Failed to create directory: {}", outpath.display()))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
                }
            }

            let mut outfile = File::create(&outpath)
                .with_context(|| format!("Failed to create file: {}", outpath.display()))?;

            io::copy(&mut file, &mut outfile)
                .with_context(|| format!("Failed to write to file: {}", outpath.display()))?;
        }
    }

    Ok(())
}

/// Create a virtual environment using uv
fn create_venv_with_uv(uv_path: &Path) -> Result<PathBuf> {
    let venv_dir = get_venv_dir()?;

    // Check if the virtual environment already exists
    if venv_dir.exists() {
        debug!("Virtual environment already exists at {}", venv_dir.display());
        return Ok(venv_dir);
    }

    // Create the parent directory if it doesn't exist
    if let Some(parent) = venv_dir.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Create the virtual environment
    info!("Creating virtual environment at {}", venv_dir.display());

    // In uv 0.7.6, the command is "uv venv create [path]"
    let status = Command::new(uv_path)
        .arg("venv")
        .arg("create")
        .arg(venv_dir.to_str().unwrap())
        .status()
        .with_context(|| "Failed to execute uv venv create command")?;

    if !status.success() {
        // Try the older syntax as fallback
        let status = Command::new(uv_path)
            .arg("venv")
            .arg(venv_dir.to_str().unwrap())
            .status()
            .with_context(|| "Failed to execute uv venv command")?;

        if !status.success() {
            return Err(anyhow!("Failed to create virtual environment"));
        }
    }

    info!("Virtual environment created successfully at {}", venv_dir.display());
    Ok(venv_dir)
}

/// Get the directory where the virtual environment should be created
fn get_venv_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow!("Failed to determine data directory"))?;

    Ok(data_dir.join("py2pyd").join("venv"))
}

/// Get the Python interpreter path from a virtual environment
fn get_python_from_venv(venv_dir: &Path) -> Result<PathBuf> {
    let python_path = if cfg!(windows) {
        venv_dir.join("Scripts").join("python.exe")
    } else {
        venv_dir.join("bin").join("python")
    };

    if !python_path.exists() {
        return Err(anyhow!("Python interpreter not found in virtual environment"));
    }

    Ok(python_path)
}

/// Install a Python package in the current environment
pub fn install_package(package: &str) -> Result<()> {
    // Get uv path
    let uv_path = setup_uv()?;

    // Get Python path for environment variables
    let python_path = get_python_path()?;
    let python_dir = python_path.parent()
        .ok_or_else(|| anyhow!("Failed to determine Python directory"))?;

    info!("Installing package: {}", package);

    // Use uv to install the package
    let mut command = Command::new(&uv_path);
    command
        .arg("pip")
        .arg("install")
        .arg(package)
        .env("PYO3_PYTHON", &python_path);

    // Add Python to PATH
    if let Ok(path) = env::var("PATH") {
        let mut paths = env::split_paths(&path).collect::<Vec<_>>();
        paths.push(python_dir.to_path_buf());
        if let Ok(new_path) = env::join_paths(paths) {
            command.env("PATH", new_path);
        }
    }

    let status = command
        .status()
        .with_context(|| format!("Failed to execute uv pip install for {}", package))?;

    if !status.success() {
        return Err(anyhow!("Failed to install package: {}", package));
    }

    info!("Package installed successfully: {}", package);
    Ok(())
}

/// Set environment variables for Python
pub fn set_python_env_vars() -> Result<()> {
    // We won't try to set any environment variables ourselves
    // as it's causing more problems than it solves

    // Instead, we'll just check if Python is working correctly
    let python_path = get_python_path()?;

    // Test if Python can import basic modules
    let output = Command::new(&python_path)
        .args(&["-c", "import sys; import os; print('Python is working correctly')"])
        .output()
        .with_context(|| "Failed to test Python")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Python is not working correctly: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    debug!("Python test: {}", stdout);

    Ok(())
}
