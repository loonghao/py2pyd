use anyhow::{anyhow, Context, Result};
use log::{info, warn};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;

/// Configuration for a uv virtual environment
pub struct UvEnvConfig {
    /// Path to the Python interpreter to use
    pub python_path: Option<PathBuf>,

    /// Python version to use (e.g., "3.9")
    pub python_version: Option<String>,

    /// Whether to keep the virtual environment after use
    pub keep_venv: bool,

    /// Additional packages to install
    pub packages: Vec<String>,
}

impl Default for UvEnvConfig {
    fn default() -> Self {
        Self {
            python_path: None,
            python_version: None,
            keep_venv: false,
            packages: vec![],
        }
    }
}

/// A uv virtual environment
pub struct UvEnv {
    /// Path to the virtual environment
    pub venv_path: PathBuf,

    /// Path to the Python interpreter in the virtual environment
    pub python_path: PathBuf,

    /// Temporary directory holding the virtual environment (if any)
    temp_dir: Option<TempDir>,
}

impl UvEnv {
    /// Create a new uv virtual environment
    pub fn create(config: &UvEnvConfig) -> Result<Self> {
        // Check if uv is installed
        let uv_path = find_uv_executable()?;
        info!("Found uv at: {}", uv_path.display());

        // Create a temporary directory for the virtual environment
        let temp_dir = if config.keep_venv {
            // Create a directory in the user's home directory
            let home_dir =
                dirs::home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
            let venv_dir = home_dir
                .join(".py2pyd")
                .join("venvs")
                .join(Uuid::new_v4().to_string());
            fs::create_dir_all(&venv_dir)
                .with_context(|| format!("Failed to create directory: {}", venv_dir.display()))?;
            None
        } else {
            // Create a temporary directory
            Some(TempDir::new().with_context(|| "Failed to create temporary directory")?)
        };

        // Get the path to the virtual environment
        let venv_path = if let Some(ref temp_dir) = temp_dir {
            temp_dir.path().to_path_buf()
        } else {
            let home_dir =
                dirs::home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
            home_dir
                .join(".py2pyd")
                .join("venvs")
                .join(Uuid::new_v4().to_string())
        };

        info!(
            "Creating uv virtual environment at: {}",
            venv_path.display()
        );

        // Build the command to create the virtual environment
        let mut cmd = Command::new(&uv_path);
        cmd.arg("venv");

        // Add Python version if specified
        if let Some(ref version) = config.python_version {
            cmd.arg("--python");
            cmd.arg(version);
        } else if let Some(ref python_path) = config.python_path {
            cmd.arg("--python");
            cmd.arg(python_path);
        }

        // Add the path to the virtual environment
        cmd.arg(&venv_path);

        // Run the command
        let status = cmd.status().with_context(|| "Failed to execute uv venv")?;

        if !status.success() {
            return Err(anyhow!("Failed to create uv virtual environment"));
        }

        // Get the path to the Python interpreter in the virtual environment
        let python_path = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };

        if !python_path.exists() {
            return Err(anyhow!(
                "Python interpreter not found in virtual environment"
            ));
        }

        // Install required packages
        if !config.packages.is_empty() {
            info!("Installing packages: {:?}", config.packages);

            let mut cmd = Command::new(&uv_path);
            cmd.arg("pip");
            cmd.arg("install");

            // Add packages
            for package in &config.packages {
                cmd.arg(package);
            }

            // Set the virtual environment
            cmd.env("VIRTUAL_ENV", &venv_path);

            // Add the virtual environment's bin directory to PATH
            let path_var = if cfg!(windows) { "Path" } else { "PATH" };
            let mut paths = env::var(path_var).unwrap_or_default();
            let bin_dir = if cfg!(windows) {
                venv_path.join("Scripts")
            } else {
                venv_path.join("bin")
            };
            paths = format!(
                "{}{}{}",
                bin_dir.to_string_lossy(),
                if cfg!(windows) { ";" } else { ":" },
                paths
            );
            cmd.env(path_var, paths);

            // Run the command
            let status = cmd
                .status()
                .with_context(|| "Failed to execute uv pip install")?;

            if !status.success() {
                return Err(anyhow!("Failed to install packages"));
            }
        }

        Ok(Self {
            venv_path,
            python_path,
            temp_dir,
        })
    }

    /// Run a Python script in the virtual environment
    pub fn run_script(&self, script: &str) -> Result<String> {
        let output = Command::new(&self.python_path)
            .arg("-c")
            .arg(script)
            .output()
            .with_context(|| "Failed to execute Python script")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Python script failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Run a Python module in the virtual environment
    pub fn run_module(&self, module: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(&self.python_path)
            .arg("-m")
            .arg(module)
            .args(args)
            .status()
            .with_context(|| format!("Failed to execute Python module: {module}"))?;

        if !status.success() {
            return Err(anyhow!("Python module failed: {}", module));
        }

        Ok(())
    }

    /// Install a package in the virtual environment
    pub fn install_package(&self, package: &str) -> Result<()> {
        let uv_path = find_uv_executable()?;

        let status = Command::new(&uv_path)
            .arg("pip")
            .arg("install")
            .arg(package)
            .env("VIRTUAL_ENV", &self.venv_path)
            .status()
            .with_context(|| format!("Failed to install package: {package}"))?;

        if !status.success() {
            return Err(anyhow!("Failed to install package: {}", package));
        }

        Ok(())
    }
}

/// Find the uv executable
fn find_uv_executable() -> Result<PathBuf> {
    // Try to find uv in PATH
    if let Ok(path) = which::which("uv") {
        return Ok(path);
    }

    // Try common installation locations
    let common_paths = if cfg!(windows) {
        vec![
            r"C:\Users\hallo\.cargo\bin\uv.exe",
            r"C:\Program Files\uv\uv.exe",
            r"C:\uv\uv.exe",
        ]
    } else {
        vec![
            "/usr/bin/uv",
            "/usr/local/bin/uv",
            "/opt/uv/bin/uv",
            "/home/hallo/.cargo/bin/uv",
        ]
    };

    for path_str in common_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    // If uv is not found, try to install it
    warn!("uv not found, attempting to install it");
    install_uv()?;

    // Try to find uv again
    which::which("uv").with_context(|| "Failed to find uv executable after installation")
}

/// Install uv (latest version - 0.7.6 as of last update)
fn install_uv() -> Result<()> {
    if cfg!(windows) {
        // On Windows, use PowerShell to install uv
        let status = Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("ByPass")
            .arg("-Command")
            .arg("irm https://astral.sh/uv/install.ps1 | iex")
            .status()
            .with_context(|| "Failed to execute PowerShell command to install uv")?;

        if !status.success() {
            return Err(anyhow!("Failed to install uv"));
        }
    } else {
        // On Unix, use curl to install uv
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
            .status()
            .with_context(|| "Failed to execute shell command to install uv")?;

        if !status.success() {
            return Err(anyhow!("Failed to install uv"));
        }
    }

    Ok(())
}
