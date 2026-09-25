use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use crate::batch_outcome::batch_outcome;
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

/// Minimum Python minor version that Cython supports for the Limited API.
///
/// Cython emits `#error "Cython <version> requires the Python Limited API
/// version to be 3.9 or greater."` into the generated C source when
/// `Py_LIMITED_API` is set below this version, which surfaces as
/// `fatal error C1189` on MSVC. py2pyd rejects the target up front instead of
/// letting the C compiler fail with an opaque message.
///
/// This is the floor py2pyd assumes when the installed Cython version cannot
/// be determined. See [`cython_limited_api_minor`] for the version-aware floor.
pub const MIN_LIMITED_API_MINOR: u32 = 9;

/// `Py_LIMITED_API` floors introduced by Cython releases, newest last.
///
/// This table is only a **fallback**. The authoritative floor is read from the
/// installed Cython's `ModuleSetupCode.c` by
/// [`detect_cython_limited_api_minor`]; this table is consulted when that file
/// cannot be read. It therefore carries a maintenance obligation: add an entry
/// whenever a Cython release raises its floor, or the fallback silently
/// under-reports the requirement for that release.
const CYTHON_LIMITED_API_FLOORS: &[((u32, u32), u32)] = &[((3, 3), MIN_LIMITED_API_MINOR)];

/// Oldest `Py_LIMITED_API` minor version the given Cython release supports.
///
/// Falls back to [`MIN_LIMITED_API_MINOR`] when `cython_version` is `None` or
/// is not a `major.minor[.patch]` release covered by the known Cython floors:
/// a missing version must never fail a build that would otherwise work.
pub fn cython_limited_api_minor(cython_version: Option<&str>) -> u32 {
    let Some((major, minor)) = cython_version.and_then(parse_cython_release) else {
        return MIN_LIMITED_API_MINOR;
    };

    CYTHON_LIMITED_API_FLOORS
        .iter()
        .filter(|((cython_major, cython_minor), _)| {
            (major, minor) >= (*cython_major, *cython_minor)
        })
        .map(|(_, floor)| *floor)
        .max()
        .unwrap_or(MIN_LIMITED_API_MINOR)
}

/// Build the `Py_LIMITED_API` macro value for a Python `major.minor` version.
///
/// # Errors
///
/// Returns an error when the target predates the Limited API support that
/// Cython requires, or when it is not a Python 3.x version.
pub fn limited_api_macro(major: u32, minor: u32) -> Result<String> {
    limited_api_macro_for_cython(major, minor, None)
}

/// Build the `Py_LIMITED_API` macro value, reporting the Cython version that
/// imposes the floor.
///
/// Cython is installed without an upper version bound, so the floor is looked
/// up from the version that actually ended up in the build environment.
/// `cython_version` is `None` when that version could not be determined, in
/// which case [`MIN_LIMITED_API_MINOR`] applies and the error names no Cython
/// release.
///
/// # Errors
///
/// Returns an error when the target predates the Limited API support that
/// Cython requires, or when it is not a Python 3.x version.
pub fn limited_api_macro_for_cython(
    major: u32,
    minor: u32,
    cython_version: Option<&str>,
) -> Result<String> {
    limited_api_macro_for_floor(
        major,
        minor,
        cython_limited_api_minor(cython_version),
        cython_version,
    )
}

/// Build the `Py_LIMITED_API` macro value against a known Cython floor.
///
/// Prefer this over [`limited_api_macro_for_cython`] when the floor was read
/// from the installed Cython by [`detect_cython_limited_api_minor`]: that value
/// is authoritative, whereas the table behind the other function only covers
/// releases measured by hand.
///
/// # Errors
///
/// Returns an error when the target predates the Limited API support that
/// Cython requires, or when it is not a Python 3.x version.
pub fn limited_api_macro_for_floor(
    major: u32,
    minor: u32,
    floor: u32,
    cython_version: Option<&str>,
) -> Result<String> {
    if major != 3 {
        return Err(anyhow!(
            "Unsupported Python version {major}.{minor}: py2pyd can only build Limited API extensions for Python 3"
        ));
    }

    if minor < floor {
        return Err(match cython_version {
            Some(version) => anyhow!(
                "Python 3.{minor} is too old for a Limited API build: Cython {version} requires Python 3.{floor} or newer. \
                 Select a newer interpreter with --python-version or --python-path"
            ),
            None => anyhow!(
                "Python 3.{minor} is too old for a Limited API build: Cython requires Python 3.{floor} or newer. \
                 Select a newer interpreter with --python-version or --python-path"
            ),
        });
    }

    Ok(format!("0x{major:02X}{minor:02X}0000"))
}

/// Read a `major.minor` release from a Cython version string such as `3.3.0`.
fn parse_cython_release(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.trim().trim_start_matches('v').split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    Some((major, minor))
}

/// Pick the version token out of `cython --version` output.
///
/// The command prints `Cython version <version>`; the version is returned
/// untouched so it can be quoted in diagnostics.
///
/// The parse and the validity check are folded into a single pass so a leading
/// numeric token that is not a version (a date, a log counter) is skipped
/// instead of masking the real version further along the line.
fn parse_cython_version(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .map(|token| token.trim().trim_start_matches('v'))
        .filter_map(|token| {
            let version: String = token
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();

            parse_cython_release(&version).map(|_| version)
        })
        .next()
}

/// Ask a Python environment which Cython version it has installed.
///
/// `python -m cython --version` is tried first because it uses the
/// interpreter the build will actually run; importing `Cython` is the fallback
/// for environments without that entry point. Returns `None` when neither
/// works: the Limited API floor then falls back to
/// [`MIN_LIMITED_API_MINOR`] rather than failing the build.
pub fn detect_cython_version(python: &Path) -> Option<String> {
    let commands: [(&str, &[&str]); 2] = [
        ("-m", &["cython", "--version"]),
        ("-c", &["import Cython; print(Cython.__version__)"]),
    ];

    for (flag, args) in commands {
        let Ok(output) = Command::new(python).arg(flag).args(args).output() else {
            continue;
        };

        if output.status.success() {
            if let Some(version) = parse_cython_version(&String::from_utf8_lossy(&output.stdout)) {
                return Some(version);
            }
        }
    }

    None
}

/// Read the `Py_LIMITED_API` floor Cython enforces in its generated C code.
///
/// Cython ships the requirement in `Cython/Utility/ModuleSetupCode.c` as
/// `#if Py_LIMITED_API < 0x03090000`, and the C compiler turns a violation into
/// `fatal error C1189`. Reading it from the installed Cython keeps the floor
/// honest: a future Cython that raises it is picked up automatically instead of
/// waiting for someone to notice the bare compiler error and update the
/// [`CYTHON_LIMITED_API_FLOORS`] fallback by hand.
///
/// Returns `None` when no `major.minor.0` bound is present, so callers keep
/// their fallback instead of acting on a misread value.
pub fn cython_limited_api_minor_from_source(setup_code: &str) -> Option<u32> {
    setup_code.lines().find_map(|line| {
        let bound = line.split_once("Py_LIMITED_API <")?.1.trim();
        let hex = bound.split_whitespace().next()?;
        let value =
            u32::from_str_radix(hex.trim_start_matches("0x").trim_end_matches('U'), 16).ok()?;

        // `0x03090000`: major in the top byte, minor next, micro zero.
        let (major, minor, micro) = (value >> 24, (value >> 16) & 0xFF, value & 0xFFFF);
        (major == 3 && micro == 0).then_some(minor)
    })
}

/// Ask a Python environment where its Cython package lives.
fn cython_package_root(python: &Path) -> Option<PathBuf> {
    let output = Command::new(python)
        .arg("-c")
        .arg("import Cython, os; print(os.path.dirname(Cython.__file__))")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
    (!path.as_os_str().is_empty()).then_some(path)
}

/// Read the Limited API floor from the Cython installed in `python`.
///
/// Returns `None` when Cython is missing or its source is unreadable (for
/// example a wheel that ships only bytecode), in which case the caller falls
/// back to [`cython_limited_api_minor`].
pub fn detect_cython_limited_api_minor(python: &Path) -> Option<u32> {
    let setup_code_path = cython_package_root(python)?
        .join("Utility")
        .join("ModuleSetupCode.c");
    let setup_code = fs::read_to_string(&setup_code_path).ok()?;

    let minor = cython_limited_api_minor_from_source(&setup_code);
    if minor.is_none() {
        warn!(
            "Could not find a Py_LIMITED_API bound in {}",
            setup_code_path.display()
        );
    }

    minor
}

/// Parse a `major.minor` Python version string such as `3.12`.
pub fn parse_python_version(version: &str) -> Result<(u32, u32)> {
    let trimmed = version.trim();
    let mut parts = trimmed.split('.');
    let major = parts.next().unwrap_or_default();
    let minor = parts.next().unwrap_or_default();

    let parse_part = |part: &str, label: &str| -> Result<u32> {
        part.parse::<u32>()
            .with_context(|| format!("Invalid {label} in Python version '{trimmed}'"))
    };

    Ok((
        parse_part(major, "major version")?,
        parse_part(minor, "minor version")?,
    ))
}

/// Ask a Python interpreter for its `major.minor` version.
pub fn detect_python_version(python: &Path) -> Result<(u32, u32)> {
    let output = Command::new(python)
        .arg("-c")
        .arg("import sys; print('%d.%d' % sys.version_info[:2])")
        .output()
        .with_context(|| format!("Failed to query the Python version of {}", python.display()))?;

    if !output.status.success() {
        return Err(anyhow!(
            "Python interpreter {} failed to report its version: {}",
            python.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    parse_python_version(&String::from_utf8_lossy(&output.stdout))
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
        // keep() returns a Result that we need to use
        let _ = temp_dir.keep();
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

    // Resolve the Limited API target from an explicitly requested Python
    // version before spending time on the uv environment. Without an explicit
    // version the target is only known once uv picked an interpreter.
    let requested_target = match &config.python_version {
        Some(requested) => {
            let (major, minor) = parse_python_version(requested)
                .with_context(|| format!("Invalid --python-version: {requested}"))?;
            // Reject a target below the default floor up front: it cannot
            // succeed with any Cython version, so the check must not wait for
            // uv. A newer Cython can still raise the floor, so the target is
            // checked again once the environment reports its Cython version.
            limited_api_macro(major, minor)?;
            Some((major, minor))
        }
        None => None,
    };

    // Copy the Python source file to the temp directory
    let source_path = temp_dir_path.join(format!("{module_name}.py"));
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

    // Cython is installed without an upper version bound, so ask the
    // environment which one it picked instead of trusting the constant floor.
    let cython_version = detect_cython_version(&uv_env.python_path);
    match &cython_version {
        Some(version) => info!("Building with Cython {version}"),
        None => warn!(
            "Could not determine the installed Cython version; assuming the Python 3.{MIN_LIMITED_API_MINOR} Limited API floor"
        ),
    }

    // Read the floor out of the Cython that actually got installed rather than
    // trusting a table of hand-measured releases: a Cython that raises its
    // floor would otherwise keep reporting the old number and fail later with a
    // bare `fatal error C1189`. The table is the fallback for Cython installs
    // whose source is not available.
    let floor = match detect_cython_limited_api_minor(&uv_env.python_path) {
        Some(minor) => {
            debug!("Cython enforces a Python 3.{minor} Limited API floor");
            minor
        }
        None => cython_limited_api_minor(cython_version.as_deref()),
    };

    // `Py_LIMITED_API` has to match the interpreter we build with. A mismatch
    // makes Cython abort the build with `fatal error C1189`.
    let (major, minor) = match requested_target {
        Some(target) => target,
        None => detect_python_version(&uv_env.python_path).with_context(|| {
            format!(
                "Failed to determine the Python version of {}",
                uv_env.python_path.display()
            )
        })?,
    };
    let limited_api = limited_api_macro_for_floor(major, minor, floor, cython_version.as_deref())?;
    info!("Building with Py_LIMITED_API={limited_api}");

    // Create the setup.py file
    let setup_py_path = temp_dir_path.join("setup.py");
    let setup_py_content = generate_setup_py(module_name, &limited_api);
    fs::write(&setup_py_path, setup_py_content)
        .with_context(|| format!("Failed to write setup.py to {}", setup_py_path.display()))?;

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
    let python_files = collect_python_files(input_pattern, recursive)
        .with_context(|| format!("Failed to collect Python files from pattern: {input_pattern}"))?;

    if python_files.is_empty() {
        warn!("No Python files matched '{input_pattern}': nothing to compile");
        return Ok(());
    }

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
            Ok(()) => {
                success_count += 1;
            }
            Err(e) => {
                warn!("Failed to compile {}: {}", input_path.display(), e);
                failure_count += 1;
            }
        }
    }

    batch_outcome(success_count, failure_count)
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
            for entry in walkdir::WalkDir::new(pattern_path)
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

        for entry in
            glob::glob(pattern).with_context(|| format!("Invalid glob pattern: {pattern}"))?
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
///
/// `limited_api` is the `Py_LIMITED_API` macro value matching the interpreter
/// the extension is built against.
fn generate_setup_py(module_name: &str, limited_api: &str) -> String {
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
    setup_py.push_str(&format!("    name='{module_name}',\n"));
    setup_py.push_str("    version='0.1',\n");
    setup_py.push_str("    ext_modules=[Extension(\n");
    setup_py.push_str(&format!("        '{module_name}',\n"));
    setup_py.push_str(&format!("        sources=['{module_name}.py'],\n"));

    // Add custom include paths if needed in the future
    // Currently not used

    // Enable ABI3 compatibility against the resolved target version
    setup_py.push_str("        py_limited_api=True,\n");
    setup_py.push_str(&format!(
        "        define_macros=[('Py_LIMITED_API', '{limited_api}')],\n"
    ));
    setup_py.push_str("    )],\n");

    // Use custom build_ext class
    setup_py.push_str("    cmdclass={'build_ext': ABI3BuildExt},\n");

    setup_py.push_str(")\n");

    setup_py
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Locate an interpreter for the tests that shell out to Python.
    ///
    /// `python3` is tried before `python` because Windows ships a Store alias
    /// named `python3` that exits without running the script; falling through
    /// to `python` keeps the search working there too.
    fn find_interpreter() -> Option<PathBuf> {
        ["python3", "python"]
            .iter()
            .find_map(|name| which::which(name).ok())
            .filter(|path| detect_python_version(path).is_ok())
    }

    #[test]
    fn test_limited_api_macro_matches_target_version() {
        assert_eq!(limited_api_macro(3, 9).unwrap(), "0x03090000");
        assert_eq!(limited_api_macro(3, 10).unwrap(), "0x030A0000");
        assert_eq!(limited_api_macro(3, 12).unwrap(), "0x030C0000");
        assert_eq!(limited_api_macro(3, 13).unwrap(), "0x030D0000");
    }

    /// The hardcoded `0x03070000` made every build fail with Cython 3.x.
    #[test]
    fn test_limited_api_macro_rejects_versions_below_cython_minimum() {
        for minor in [0, 7, 8] {
            let err = limited_api_macro(3, minor).unwrap_err().to_string();
            assert!(
                err.contains("too old for a Limited API build"),
                "unexpected error for 3.{minor}: {err}"
            );
            assert!(
                err.contains("3.9 or newer"),
                "error should name the minimum: {err}"
            );
        }
    }

    #[test]
    fn test_limited_api_macro_rejects_python_2() {
        let err = limited_api_macro(2, 7).unwrap_err().to_string();
        assert!(err.contains("Unsupported Python version 2.7"), "{err}");
    }

    /// A future Cython can raise the Limited API floor. The error has to name
    /// the Cython version that set it, otherwise the failure looks like the
    /// interpreter being too old and sends users down the wrong path.
    #[test]
    fn test_limited_api_error_names_the_installed_cython_version() {
        let err = limited_api_macro_for_cython(3, 7, Some("3.3.0"))
            .unwrap_err()
            .to_string();

        assert!(err.contains("Cython 3.3.0"), "{err}");
        assert!(err.contains("too old for a Limited API build"), "{err}");
        assert!(err.contains("3.9 or newer"), "{err}");
    }

    /// Without a Cython version the constant floor still applies and the
    /// message stays the one the README quotes.
    #[test]
    fn test_limited_api_error_falls_back_without_a_cython_version() {
        let err = limited_api_macro_for_cython(3, 8, None)
            .unwrap_err()
            .to_string();

        assert_eq!(
            err,
            "Python 3.8 is too old for a Limited API build: Cython requires Python 3.9 or newer. \
             Select a newer interpreter with --python-version or --python-path"
        );
        assert!(!err.contains("Cython 3."), "{err}");
        assert_eq!(limited_api_macro(3, 8).unwrap_err().to_string(), err);
    }

    #[test]
    fn test_cython_limited_api_minor_uses_the_known_floor() {
        assert_eq!(cython_limited_api_minor(Some("3.3.0")), 9);
        assert_eq!(cython_limited_api_minor(Some("3.12.1")), 9);
    }

    /// An unknown or missing version must not fail the build, so the floor
    /// falls back to the constant instead.
    #[test]
    fn test_cython_limited_api_minor_falls_back_to_the_constant() {
        assert_eq!(cython_limited_api_minor(None), MIN_LIMITED_API_MINOR);
        assert_eq!(
            cython_limited_api_minor(Some("not-a-version")),
            MIN_LIMITED_API_MINOR
        );
        assert_eq!(cython_limited_api_minor(Some("")), MIN_LIMITED_API_MINOR);
    }

    #[test]
    fn test_parse_cython_version_reads_the_command_output() {
        assert_eq!(
            parse_cython_version("Cython version 3.3.0\n").as_deref(),
            Some("3.3.0")
        );
        assert_eq!(parse_cython_version("3.0.11").as_deref(), Some("3.0.11"));
        assert_eq!(
            parse_cython_version("Cython version 4.0.0a1").as_deref(),
            Some("4.0.0")
        );
        assert_eq!(parse_cython_version("3"), None);
        assert_eq!(parse_cython_version("no version here"), None);
    }

    /// A leading token that starts with a digit but is not a version used to
    /// win the `find` and then fail the `filter`, hiding the real version.
    #[test]
    fn test_parse_cython_version_skips_leading_numeric_noise() {
        assert_eq!(
            parse_cython_version("2024-01-01 Cython version 3.3.0").as_deref(),
            Some("3.3.0")
        );
        assert_eq!(
            parse_cython_version("12 warnings Cython version 3.0.11").as_deref(),
            Some("3.0.11")
        );
        assert_eq!(parse_cython_version("2024-01-01 00:00:00"), None);
    }

    /// The floor is read from the guard Cython actually compiles, so a Cython
    /// that raises it is picked up without a table edit.
    #[test]
    fn test_cython_limited_api_minor_from_source_reads_the_guard() {
        let source = "\
            #ifdef Py_LIMITED_API
              #define __PYX_LIMITED_VERSION_HEX Py_LIMITED_API
              #if Py_LIMITED_API < 0x03090000
                #error \"Cython 3.3 requires the Python Limited API version to be 3.9 or greater.\"
              #endif
            #endif
        ";

        assert_eq!(cython_limited_api_minor_from_source(source), Some(9));
    }

    /// A hypothetical Cython that raises the floor has to be reported at the
    /// new value; this is the regression the static table could not cover.
    #[test]
    fn test_cython_limited_api_minor_from_source_tracks_a_raised_floor() {
        let source = "#if Py_LIMITED_API < 0x030B0000";

        assert_eq!(cython_limited_api_minor_from_source(source), Some(11));
    }

    /// A bound that is not a `3.minor.0` target is not a floor py2pyd can act
    /// on, so it degrades to the fallback instead of inventing a number.
    #[test]
    fn test_cython_limited_api_minor_from_source_ignores_unusable_bounds() {
        assert_eq!(
            cython_limited_api_minor_from_source("#if Py_LIMITED_API < 0x02070000"),
            None
        );
        assert_eq!(
            cython_limited_api_minor_from_source("#if Py_LIMITED_API < 0x03090001"),
            None
        );
        assert_eq!(
            cython_limited_api_minor_from_source("#if Py_LIMITED_API < PY_VERSION"),
            None
        );
        assert_eq!(cython_limited_api_minor_from_source(""), None);
    }

    /// The detected floor drives the error, so the message stays correct even
    /// when it disagrees with the hand-measured fallback table.
    #[test]
    fn test_limited_api_error_uses_the_detected_floor() {
        let err = limited_api_macro_for_floor(3, 9, 11, Some("4.0.0"))
            .unwrap_err()
            .to_string();

        assert!(err.contains("Cython 4.0.0"), "{err}");
        assert!(err.contains("3.11 or newer"), "{err}");
        assert!(!err.contains("3.9 or newer"), "{err}");
    }

    /// Guards against the table silently drifting below the constant it is
    /// supposed to be a refinement of.
    #[test]
    fn test_cython_floor_table_is_consistent_with_the_constant() {
        for (_, floor) in CYTHON_LIMITED_API_FLOORS {
            assert!(
                *floor >= MIN_LIMITED_API_MINOR,
                "a recorded Cython floor ({floor}) is below the assumed minimum ({MIN_LIMITED_API_MINOR})"
            );
        }
    }

    /// End-to-end: the floor read from a real Cython install must match what
    /// that Cython enforces. Skipped where Cython is not installed.
    #[test]
    fn test_detect_cython_limited_api_minor_reads_the_installed_cython() {
        let Some(python) = find_interpreter() else {
            eprintln!("skipping: no python3/python on PATH");
            return;
        };

        let Some(version) = detect_cython_version(&python) else {
            eprintln!("skipping: no Cython in {}", python.display());
            return;
        };

        let detected = detect_cython_limited_api_minor(&python)
            .expect("Cython is installed, so ModuleSetupCode.c should be readable");

        assert_eq!(
            detected,
            cython_limited_api_minor(Some(&version)),
            "Cython {version} enforces a Python 3.{detected} floor but the table says otherwise"
        );
    }

    /// A Cython version that cannot be queried degrades to the constant floor
    /// rather than failing the compilation.
    #[test]
    fn test_detect_cython_version_is_none_for_a_missing_interpreter() {
        let missing = std::env::temp_dir().join("py2pyd-no-such-cython");

        assert_eq!(detect_cython_version(&missing), None);
    }

    #[test]
    fn test_parse_python_version() {
        assert_eq!(parse_python_version("3.12").unwrap(), (3, 12));
        assert_eq!(parse_python_version("3.9").unwrap(), (3, 9));
        assert_eq!(parse_python_version(" 3.10 ").unwrap(), (3, 10));
        assert!(parse_python_version("3").is_err());
        assert!(parse_python_version("3.x").is_err());
        assert!(parse_python_version("").is_err());
    }

    /// `detect_python_version` is the default path when neither
    /// `--python-version` nor `--python-path` is given, so a broken detector
    /// silently produces a `Py_LIMITED_API` value for the wrong interpreter.
    #[test]
    fn test_detect_python_version_reports_the_interpreter_version() {
        let Some(python) = find_interpreter() else {
            eprintln!("skipping: no python3/python on PATH");
            return;
        };

        let (major, minor) =
            detect_python_version(&python).expect("the interpreter should report its version");

        assert_eq!(major, 3, "{} reported {major}.{minor}", python.display());
        assert!(
            minor >= MIN_LIMITED_API_MINOR,
            "py2pyd cannot build against 3.{minor}; the Limited API floor is 3.{MIN_LIMITED_API_MINOR}"
        );
        assert!(
            limited_api_macro(major, minor).is_ok(),
            "a detected interpreter must produce a usable Py_LIMITED_API value"
        );
    }

    #[test]
    fn test_detect_python_version_reports_a_missing_interpreter() {
        let missing = std::env::temp_dir().join("py2pyd-no-such-interpreter");

        let err = detect_python_version(&missing).unwrap_err().to_string();

        assert!(
            err.contains("Failed to query the Python version of"),
            "{err}"
        );
    }

    #[test]
    fn test_generate_setup_py_uses_resolved_limited_api() {
        let setup_py = generate_setup_py("demo", "0x030C0000");
        assert!(
            setup_py.contains("('Py_LIMITED_API', '0x030C0000')"),
            "{setup_py}"
        );
        assert!(!setup_py.contains("0x03070000"), "{setup_py}");
        assert!(setup_py.contains("py_limited_api=True"), "{setup_py}");
        assert!(setup_py.contains("sources=['demo.py']"), "{setup_py}");
    }

    #[test]
    fn test_batch_compile_reports_failure_when_every_file_fails() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("in");
        let output_dir = temp_dir.path().join("out");
        fs::create_dir_all(&input_dir).unwrap();

        for name in ["broken_a.py", "broken_b.py"] {
            fs::write(input_dir.join(name), "def broken(:\n    not python\n").unwrap();
        }

        // A target below the Cython minimum is rejected before uv or a C
        // compiler is involved, so this stays a fast, hermetic test.
        let config = CompileConfig {
            python_version: Some("3.7".to_string()),
            ..Default::default()
        };

        let err = batch_compile(input_dir.to_str().unwrap(), &output_dir, &config, false)
            .unwrap_err()
            .to_string();

        assert!(err.contains("all 2 file(s) failed to compile"), "{err}");
    }

    #[test]
    fn test_batch_compile_no_matching_files_is_not_an_error() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let input_dir = temp_dir.path().join("empty");
        let output_dir = temp_dir.path().join("out");
        fs::create_dir_all(&input_dir).unwrap();

        let config = CompileConfig::default();
        assert!(batch_compile(input_dir.to_str().unwrap(), &output_dir, &config, false).is_ok());
    }
}
