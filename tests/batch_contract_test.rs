//! Exit-code contract tests for `py2pyd batch`.
//!
//! `batch` is the only command whose exit code carries meaning beyond "the
//! process ran": a batch in which every file failed has to exit non-zero so
//! that CI can tell it apart from a successful run. These tests drive the real
//! binary instead of the library so that the `?` in `main.rs` — the step that
//! turns a `batch_outcome` error into a process exit code — is covered too.
//!
//! `main` refuses to run at all when no C toolchain is present, so the CLI
//! tests below skip on machines without one. The library-level test always
//! runs and guards the shared outcome decision itself.
//!
//! One case cannot be hermetic: constructing a *partially* successful batch
//! needs a module that genuinely compiles, which means a real Cython build
//! through `uv`. `uv` is missing on the windows-latest runner and
//! `uv_env::install_uv` does not put it on PATH for the running process, so
//! that one test is ignored on Windows only. The contract is still covered
//! there by the other cases: the two CLI tests pin both halves of the exit
//! code mapping (`Err` => non-zero, `Ok` => zero) and
//! `batch_outcome_decides_the_exit_code` pins every branch of the decision,
//! including partial success.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_py2pyd");

/// Extension of a compiled module on the host platform.
fn module_ext() -> &'static str {
    if cfg!(windows) {
        "pyd"
    } else {
        "so"
    }
}

/// `main` calls `check_build_tools` before it dispatches any subcommand, so
/// CLI-level assertions only hold where a C toolchain exists.
fn build_toolchain_available() -> bool {
    py2pyd::detect_build_tools().has_any_tools()
}

/// Run `py2pyd [global args] batch --input <input> --output <output>`.
///
/// `--python-version` and friends are declared on the top-level command and
/// are not marked `global`, so they have to appear *before* the subcommand.
fn run_batch(input: &Path, output: &Path, global_args: &[&str]) -> Output {
    Command::new(BIN)
        .args(global_args)
        .arg("batch")
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .output()
        .expect("failed to launch the py2pyd binary")
}

fn write_broken_module(dir: &Path, name: &str) {
    fs::write(dir.join(name), "def broken(:\n    not python\n").unwrap();
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A batch where nothing compiled must exit non-zero, otherwise a broken build
/// is indistinguishable from a clean one in CI.
#[test]
fn batch_exits_non_zero_when_every_file_fails() {
    if !build_toolchain_available() {
        eprintln!("skipping: no C toolchain detected");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("in");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir_all(&input_dir).unwrap();
    write_broken_module(&input_dir, "broken_a.py");
    write_broken_module(&input_dir, "broken_b.py");

    // `--python-version 3.7` is rejected before uv or a C compiler is used, so
    // the failure is deterministic and fast.
    let output = run_batch(&input_dir, &output_dir, &["--python-version", "3.7"]);

    assert!(
        !output.status.success(),
        "a batch where every file failed must exit non-zero"
    );

    // The per-file failures have to be reported too, not just swallowed into
    // a summary: that warning is how a user finds out which file broke.
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("all 2 file(s) failed to compile"),
        "stderr should report the total batch failure, got:\n{stderr}"
    );
    for name in ["broken_a.py", "broken_b.py"] {
        assert!(
            stderr.contains(name),
            "{name} should be reported by name, got:\n{stderr}"
        );
    }
}

/// One broken file must not discard the rest of the batch: the run still
/// exits zero, the broken file is reported as a warning, and the modules that
/// did compile are still written out.
///
/// This one really invokes Cython, so it is written to survive a single flaky
/// file: two good modules go in so that one failing for an environmental
/// reason (uv download, compiler hiccup) still leaves a partial batch to
/// assert on. The contract under test is the exit code, the warning and the
/// surviving artifacts, not the exact failure count.
///
/// Windows carries an explicit ignore rather than a runtime skip: `uv` is not
/// on the windows-latest image, and `uv_env::install_uv` installs it outside
/// the PATH of the running process, so every module fails there and a partial
/// batch cannot be constructed at all. Leaving it un-ignored would make this
/// test fail for an environmental reason on every Windows run while proving
/// nothing about the contract. The other cases in this file keep the contract
/// covered on Windows; see the module docs.
#[cfg_attr(
    target_os = "windows",
    ignore = "needs a working uv toolchain; uv is absent from the windows image"
)]
#[test]
fn batch_exits_zero_and_warns_when_only_some_files_fail() {
    if !build_toolchain_available() {
        eprintln!("skipping: no C toolchain detected");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("in");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(
        input_dir.join("good_a.py"),
        "def double(x):\n    return x * 2\n",
    )
    .unwrap();
    fs::write(
        input_dir.join("good_b.py"),
        "def triple(x):\n    return x * 3\n",
    )
    .unwrap();
    write_broken_module(&input_dir, "bad.py");

    let output = run_batch(&input_dir, &output_dir, &[]);
    let stderr = stderr_of(&output);

    assert!(
        output.status.success(),
        "a partially successful batch must exit zero, stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("bad.py"),
        "the broken file should be reported by name, got:\n{stderr}"
    );
    assert!(
        stderr.contains("file(s) failed to compile"),
        "the warning should summarise the partial failure, got:\n{stderr}"
    );

    let artifacts: Vec<PathBuf> = ["good_a", "good_b"]
        .iter()
        .map(|name| output_dir.join(format!("{name}.{}", module_ext())))
        .collect();
    assert!(
        artifacts.iter().any(|path| path.exists()),
        "a module that compiled should reach the output directory, stderr:\n{stderr}"
    );

    let broken_artifact = output_dir.join(format!("bad.{}", module_ext()));
    assert!(
        !broken_artifact.exists(),
        "the broken module must not be produced"
    );
}

/// An input pattern matching no Python file is not a failure.
#[test]
fn batch_exits_zero_when_no_python_file_matches() {
    if !build_toolchain_available() {
        eprintln!("skipping: no C toolchain detected");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("in");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir_all(&input_dir).unwrap();

    let output = run_batch(&input_dir, &output_dir, &[]);

    assert!(
        output.status.success(),
        "an empty batch is not a failure, stderr:\n{}",
        stderr_of(&output)
    );
}

/// Pin every branch of the decision that both backends share, so the mapping
/// from counters to exit code is asserted on every platform -- including
/// Windows, where the partial-success case above cannot build a real module.
///
/// `main` turns `Err` into a non-zero exit code and `Ok` into zero, so these
/// four rows are the whole contract.
#[test]
fn batch_outcome_decides_the_exit_code() {
    // Nothing failed: success.
    assert!(py2pyd::batch_outcome(3, 0).is_ok());
    // Nothing was attempted: success, not a failure.
    assert!(py2pyd::batch_outcome(0, 0).is_ok());
    // Partial failure: tolerated, the batch still counts as successful.
    assert!(py2pyd::batch_outcome(2, 1).is_ok());
    // Total failure: an error, so the process exits non-zero.
    let err = py2pyd::batch_outcome(0, 2).expect_err("a fully failed batch must fail");
    assert!(err.to_string().contains("all 2 file(s) failed to compile"));
}

/// The legacy (non-uv) backend shares the same outcome contract as the
/// uv-backed one. This one needs no toolchain because parsing fails first.
#[test]
fn legacy_batch_returns_an_error_when_every_file_fails() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("in");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir_all(&input_dir).unwrap();
    write_broken_module(&input_dir, "broken_a.py");
    write_broken_module(&input_dir, "broken_b.py");

    let err = py2pyd::batch_compile_legacy(input_dir.to_str().unwrap(), &output_dir, 2, false)
        .expect_err("every file failed, so the batch must fail");

    assert!(err.to_string().contains("all 2 file(s) failed to compile"));
}
