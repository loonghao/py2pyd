use anyhow::{anyhow, Result};
use log::debug;
use std::path::PathBuf;
use std::process::Command;
use which::which;

/// Represents the build tools available on the system
pub struct BuildTools {
    /// Path to MSVC compiler (cl.exe)
    pub msvc_path: Option<PathBuf>,
    /// Path to MinGW compiler (gcc.exe)
    pub mingw_path: Option<PathBuf>,
    /// Path to dlltool.exe (part of MinGW)
    pub dlltool_path: Option<PathBuf>,
    /// Path to Visual Studio installation
    pub vs_path: Option<PathBuf>,
    /// Path to GCC compiler (for Unix systems)
    pub gcc_path: Option<PathBuf>,
    /// Path to Xcode Command Line Tools (for macOS)
    pub xcode_path: Option<PathBuf>,
}

impl BuildTools {
    /// Check if MSVC is available
    pub fn has_msvc(&self) -> bool {
        self.msvc_path.is_some()
    }

    /// Check if MinGW is available
    pub fn has_mingw(&self) -> bool {
        self.mingw_path.is_some() && self.dlltool_path.is_some()
    }

    /// Check if GCC is available (for Unix systems)
    pub fn has_gcc(&self) -> bool {
        self.gcc_path.is_some()
    }

    /// Check if Xcode Command Line Tools are available (for macOS)
    pub fn has_xcode(&self) -> bool {
        self.xcode_path.is_some()
    }

    /// Check if any build tools are available
    pub fn has_any_tools(&self) -> bool {
        self.has_msvc() || self.has_mingw() || self.has_gcc() || self.has_xcode()
    }

    /// Get a string representation of the available build tools
    pub fn get_tools_info(&self) -> String {
        let mut info = String::new();

        if let Some(msvc) = &self.msvc_path {
            info.push_str(&format!("MSVC: {}\n", msvc.display()));
        }

        if let Some(mingw) = &self.mingw_path {
            info.push_str(&format!("MinGW: {}\n", mingw.display()));
        }

        if let Some(dlltool) = &self.dlltool_path {
            info.push_str(&format!("dlltool: {}\n", dlltool.display()));
        }

        if let Some(vs) = &self.vs_path {
            info.push_str(&format!("Visual Studio: {}\n", vs.display()));
        }

        if let Some(gcc) = &self.gcc_path {
            info.push_str(&format!("GCC: {}\n", gcc.display()));
        }

        if let Some(xcode) = &self.xcode_path {
            info.push_str(&format!("Xcode Command Line Tools: {}\n", xcode.display()));
        }

        if info.is_empty() {
            info.push_str("No build tools found");
        }

        info
    }
}

/// Detect build tools available on the system
pub fn detect_build_tools() -> Result<BuildTools> {
    let mut tools = BuildTools {
        msvc_path: None,
        mingw_path: None,
        dlltool_path: None,
        vs_path: None,
        gcc_path: None,
        xcode_path: None,
    };

    // Detect MSVC
    match which("cl") {
        Ok(path) => {
            debug!("Found MSVC compiler: {}", path.display());
            tools.msvc_path = Some(path);

            // Try to find Visual Studio installation
            if let Ok(output) = Command::new("cl").arg("/?").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
                    if line.contains("Microsoft") {
                        debug!("MSVC version info: {}", line);
                    }
                }
            }

            // Try to find VS installation path
            if let Ok(output) = Command::new("where").arg("devenv.exe").output() {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(line) = output_str.lines().next() {
                        let path = PathBuf::from(line);
                        if let Some(parent) = path.parent() {
                            if let Some(parent) = parent.parent() {
                                tools.vs_path = Some(parent.to_path_buf());
                                debug!("Found Visual Studio at: {}", parent.display());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            debug!("MSVC compiler not found in PATH: {}", e);
        }
    }

    // Detect MinGW
    match which("gcc") {
        Ok(path) => {
            debug!("Found MinGW compiler: {}", path.display());
            tools.mingw_path = Some(path);
        }
        Err(e) => {
            debug!("MinGW compiler not found in PATH: {}", e);
        }
    }

    // Detect dlltool
    match which("dlltool") {
        Ok(path) => {
            debug!("Found dlltool: {}", path.display());
            tools.dlltool_path = Some(path);
        }
        Err(e) => {
            debug!("dlltool not found in PATH: {}", e);
        }
    }

    // For Unix systems (Linux and macOS), detect GCC
    if cfg!(unix) {
        match which("gcc") {
            Ok(path) => {
                debug!("Found GCC compiler: {}", path.display());
                tools.gcc_path = Some(path);
            }
            Err(e) => {
                debug!("GCC compiler not found in PATH: {}", e);
            }
        }
    }

    // For macOS, detect Xcode Command Line Tools
    if cfg!(target_os = "macos") {
        // Check for xcode-select
        match which("xcode-select") {
            Ok(path) => {
                debug!("Found xcode-select: {}", path.display());

                // Try to get the path to the developer directory
                if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
                    if output.status.success() {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        let path = PathBuf::from(output_str.trim());
                        if path.exists() {
                            debug!("Found Xcode Command Line Tools at: {}", path.display());
                            tools.xcode_path = Some(path);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("xcode-select not found in PATH: {}", e);
            }
        }
    }

    Ok(tools)
}

/// Get installation instructions for build tools
pub fn get_build_tools_installation_instructions() -> String {
    if cfg!(windows) {
        r#"
To install the required build tools on Windows, you have two options:

1. Install Visual Studio Build Tools (Recommended):
   - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - During installation, select "C++ build tools" workload
   - This will install MSVC compiler and necessary tools

2. Install MinGW-w64:
   - Download from: https://www.mingw-w64.org/downloads/
   - Add the bin directory to your PATH environment variable
   - Restart your terminal after installation

After installation, try running py2pyd again.
"#
        .to_string()
    } else {
        r#"
To install the required build tools on Linux/macOS:

1. On Ubuntu/Debian:
   sudo apt-get update
   sudo apt-get install build-essential

2. On macOS:
   xcode-select --install

After installation, try running py2pyd again.
"#
        .to_string()
    }
}

/// Check if build tools are available and provide helpful error messages
pub fn check_build_tools() -> Result<BuildTools> {
    let tools = detect_build_tools()?;

    if !tools.has_any_tools() {
        let instructions = get_build_tools_installation_instructions();

        if cfg!(windows) {
            return Err(anyhow!(
                "No suitable build tools found. You need either MSVC or MinGW to compile Python extensions.\n\n{}",
                instructions
            ));
        } else if cfg!(target_os = "macos") {
            return Err(anyhow!(
                "No suitable build tools found. You need Xcode Command Line Tools to compile Python extensions.\n\n{}",
                instructions
            ));
        } else {
            return Err(anyhow!(
                "No suitable build tools found. You need GCC and development tools to compile Python extensions.\n\n{}",
                instructions
            ));
        }
    }

    Ok(tools)
}
