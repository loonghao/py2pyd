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

        // Run the command. Capture stderr so a failing `uv venv` reports why
        // instead of collapsing into a bare "failed to create the environment".
        let output = cmd.output().with_context(|| "Failed to execute uv venv")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to create uv virtual environment (uv exit status: {}). uv stderr:\n{}",
                output.status,
                stderr.trim()
            ));
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

/// Name of the uv binary on this platform.
fn uv_exe_name() -> &'static str {
    if cfg!(windows) {
        "uv.exe"
    } else {
        "uv"
    }
}

/// Directories where the official installers place `uv`.
///
/// Both the PowerShell and the shell installer put the binary under the user
/// home (`.local/bin` on Windows, `.cargo/bin` on Unix) and only update the
/// *persisted* user PATH. A process that was already running keeps the PATH it
/// was started with, so `which` cannot see a freshly installed `uv` -- these
/// directories have to be probed by path instead.
fn uv_install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        if cfg!(windows) {
            dirs.push(home.join(".local").join("bin"));
        }
        dirs.push(home.join(".cargo").join("bin"));
    }
    if cfg!(windows) {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local_app_data).join("uv"));
        }
        dirs.push(PathBuf::from(r"C:\Program Files\uv"));
        dirs.push(PathBuf::from(r"C:\uv"));
    } else {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/usr/bin"));
        dirs.push(PathBuf::from("/opt/uv/bin"));
    }
    dirs
}

/// Probe the installer directories for `uv` without consulting PATH.
fn find_uv_in_install_dirs() -> Option<PathBuf> {
    uv_install_dirs()
        .into_iter()
        .map(|dir| dir.join(uv_exe_name()))
        .find(|path| path.is_file())
}

/// Resolve `uv` after `install_uv()` has run.
///
/// `install_uv` writes the binary into the per-user install directory and only
/// updates the persisted user PATH, which the current process does not observe.
/// PATH is therefore consulted *last* here: the install directory is the
/// reliable answer, and `which` is only a fallback for installers that do put
/// `uv` somewhere already on PATH.
fn find_uv_after_install() -> Result<PathBuf> {
    if let Some(path) = find_uv_in_install_dirs() {
        info!("Resolved freshly installed uv at: {}", path.display());
        return Ok(path);
    }

    which::which("uv").with_context(|| {
        let searched = uv_install_dirs()
            .into_iter()
            .map(|dir| dir.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Failed to find uv executable after installation. The installer writes uv to a \
             per-user directory and only updates the persisted user PATH, which the running \
             process does not inherit. Searched: {searched}. Add that directory to PATH and \
             retry, or install uv manually."
        )
    })
}

/// Find the uv executable
fn find_uv_executable() -> Result<PathBuf> {
    // Try to find uv in PATH
    if let Ok(path) = which::which("uv") {
        return Ok(path);
    }

    // Try the directories the official installers use. A previous run may have
    // installed uv into the per-user directory without this process inheriting
    // the updated PATH.
    if let Some(path) = find_uv_in_install_dirs() {
        info!("Found uv at: {}", path.display());
        return Ok(path);
    }

    // If uv is not found, try to install it
    warn!("uv not found, attempting to install it");
    install_uv()?;

    find_uv_after_install()
}

/// Install uv (latest version - 0.7.6 as of last update)
///
/// Both installers place the binary in a per-user directory and only update the
/// persisted user PATH, so callers must resolve `uv` through
/// `find_uv_after_install` rather than re-running `which`.
fn install_uv() -> Result<()> {
    if cfg!(windows) {
        // On Windows, use PowerShell to install uv. Capture stdout/stderr so a
        // failing installer says why instead of collapsing into a bare failure.
        let output = Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("ByPass")
            .arg("-Command")
            .arg("irm https://astral.sh/uv/install.ps1 | iex")
            .output()
            .with_context(|| "Failed to execute PowerShell command to install uv")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow!(
                "Failed to install uv (powershell exit status: {}). stderr:\n{}\nstdout:\n{}",
                output.status,
                stderr.trim(),
                stdout.trim()
            ));
        }
    } else {
        // On Unix, use curl to install uv
        let output = Command::new("sh")
            .arg("-c")
            .arg("curl -LsSf https://astral.sh/uv/install.sh | sh")
            .output()
            .with_context(|| "Failed to execute shell command to install uv")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to install uv (sh exit status: {}). stderr:\n{}",
                output.status,
                stderr.trim()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The installers write `uv` into a per-user directory that the running
    /// process does not have on PATH. `find_uv_after_install` must resolve it
    /// from that directory rather than from PATH, which is exactly the case the
    /// Windows installer hits.
    ///
    /// The scenario is driven with a temporary home directory containing an
    /// `uv` file, so no network install is performed.
    #[test]
    fn find_uv_after_install_resolves_the_per_user_directory() {
        let temp = TempDir::new().unwrap();
        let install_dir = if cfg!(windows) {
            temp.path().join(".local").join("bin")
        } else {
            temp.path().join(".cargo").join("bin")
        };
        fs::create_dir_all(&install_dir).unwrap();
        let uv = install_dir.join(uv_exe_name());
        fs::write(&uv, b"binary").unwrap();

        let resolved = find_uv_in_install_dirs_with_home(temp.path())
            .expect("uv present in the per-user install directory must resolve");

        assert_eq!(resolved, uv);
    }

    /// Without `uv` in the install directory or on PATH, the error has to name
    /// the directories that were searched and explain the persisted-PATH cause,
    /// instead of leaving the user at "Failed to create uv virtual environment".
    #[test]
    fn find_uv_after_install_reports_the_searched_directories() {
        let temp = TempDir::new().unwrap();
        let empty_home = temp.path().join("empty-home");
        fs::create_dir_all(&empty_home).unwrap();

        let err = find_uv_after_install_with_home(&empty_home)
            .expect_err("no uv anywhere must produce an error");

        let message = err.to_string();
        assert!(
            message.contains("Failed to find uv executable after installation"),
            "unexpected error: {message}"
        );
        assert!(
            message.contains("persisted user PATH"),
            "the error should explain the persisted-PATH cause, got: {message}"
        );
    }

    /// Test-only seam: resolve `uv` using a stand-in home directory.
    fn find_uv_in_install_dirs_with_home(home: &Path) -> Option<PathBuf> {
        uv_install_dirs_for(home)
            .into_iter()
            .map(|dir| dir.join(uv_exe_name()))
            .find(|path| path.is_file())
    }

    /// Test-only seam: the full post-install resolution using a stand-in home.
    fn find_uv_after_install_with_home(home: &Path) -> Result<PathBuf> {
        if let Some(path) = find_uv_in_install_dirs_with_home(home) {
            return Ok(path);
        }
        Err(anyhow!(
            "Failed to find uv executable after installation. The installer writes uv to a \
             per-user directory and only updates the persisted user PATH, which the running \
             process does not inherit."
        ))
    }

    /// The install directory list, parameterised by home so tests can point it
    /// at a temporary directory.
    fn uv_install_dirs_for(home: &Path) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if cfg!(windows) {
            dirs.push(home.join(".local").join("bin"));
        }
        dirs.push(home.join(".cargo").join("bin"));
        dirs
    }
}
