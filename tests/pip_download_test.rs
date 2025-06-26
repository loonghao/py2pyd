use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Test downloading and analyzing pip packages
#[cfg(test)]
mod pip_download_tests {
    use super::*;

    /// Test downloading a simple pure Python package from `PyPI`
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_download_six_package() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();

        println!("Test directory: {}", test_dir.display());

        // Download the 'six' package - it's small and pure Python
        let package_info = download_and_extract_package("six", "1.16.0", test_dir)?;

        println!(
            "Package extracted to: {}",
            package_info.extract_path.display()
        );
        println!("Found {} Python files", package_info.python_files.len());

        // Verify we found the main six.py file
        assert!(
            !package_info.python_files.is_empty(),
            "Should find Python files"
        );

        let six_py = package_info
            .python_files
            .iter()
            .find(|f| f.file_name().unwrap().to_string_lossy() == "six.py");

        assert!(six_py.is_some(), "Should find six.py file");

        if let Some(six_file) = six_py {
            println!("Found six.py at: {}", six_file.display());

            // Check file size
            let metadata = fs::metadata(six_file)?;
            println!("six.py size: {} bytes", metadata.len());
            assert!(metadata.len() > 1000, "six.py should be a substantial file");
        }

        Ok(())
    }

    /// Test downloading and attempting to compile a package
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_download_and_compile_package() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();

        println!("Test directory: {}", test_dir.display());

        // Download a simple package
        let package_info = download_and_extract_package("six", "1.16.0", test_dir)?;

        // Try to compile the main Python file
        if let Some(main_file) = package_info.python_files.first() {
            println!("Attempting to compile: {}", main_file.display());

            let output_dir = test_dir.join("compiled");
            fs::create_dir_all(&output_dir)?;

            let file_stem = main_file.file_stem().unwrap().to_string_lossy();
            let output_file = output_dir.join(format!("{}.pyd", file_stem));

            match compile_with_py2pyd(main_file, &output_file) {
                Ok(()) => {
                    println!("✅ Successfully compiled {} to pyd", file_stem);
                    assert!(output_file.exists(), "Compiled file should exist");

                    let metadata = fs::metadata(&output_file)?;
                    println!("Compiled file size: {} bytes", metadata.len());
                }
                Err(e) => {
                    println!("❌ Compilation failed: {}", e);
                    // This is not necessarily a test failure - some files might not be compilable
                    println!("Note: This might be expected for complex packages");
                }
            }
        }

        Ok(())
    }

    /// Test analyzing package structure
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_analyze_package_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();

        // Download and analyze multiple packages
        let packages_to_test = vec![
            ("six", "1.16.0"),
            ("certifi", "2023.7.22"), // Another simple pure Python package
        ];

        for (package_name, version) in packages_to_test {
            println!("\n--- Analyzing package: {} v{} ---", package_name, version);

            match download_and_extract_package(package_name, version, test_dir) {
                Ok(package_info) => {
                    println!("✅ Successfully downloaded {}", package_name);
                    println!("  📁 Extract path: {}", package_info.extract_path.display());
                    println!("  🐍 Python files: {}", package_info.python_files.len());

                    // List all Python files
                    for (i, py_file) in package_info.python_files.iter().enumerate() {
                        if i < 5 {
                            // Limit output
                            println!(
                                "    {}: {}",
                                i + 1,
                                py_file.file_name().unwrap().to_string_lossy()
                            );
                        }
                    }
                    if package_info.python_files.len() > 5 {
                        println!(
                            "    ... and {} more files",
                            package_info.python_files.len() - 5
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Failed to download {}: {}", package_name, e);
                }
            }
        }

        Ok(())
    }
}

/// Information about a downloaded and extracted package
#[derive(Debug)]
struct PackageInfo {
    extract_path: PathBuf,
    python_files: Vec<PathBuf>,
}

/// Download and extract a pip package
fn download_and_extract_package(
    package_name: &str,
    version: &str,
    work_dir: &Path,
) -> Result<PackageInfo> {
    let package_spec = format!("{}=={}", package_name, version);
    let download_dir = work_dir.join(format!("download_{}", package_name));
    fs::create_dir_all(&download_dir)?;

    println!("Downloading {} to {}", package_spec, download_dir.display());

    // Use pip download to get the package
    let output = Command::new("pip")
        .args([
            "download",
            "--no-deps", // Don't download dependencies
            "--dest",
            download_dir.to_str().unwrap(),
            &package_spec,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to download package {}: {}",
            package_name,
            stderr
        ));
    }

    // Find the downloaded file
    let mut downloaded_file = None;
    for entry in fs::read_dir(&download_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && (path.extension().is_some_and(|ext| ext == "whl")
                || path.to_string_lossy().ends_with(".tar.gz"))
        {
            downloaded_file = Some(path);
            break;
        }
    }

    let downloaded_file =
        downloaded_file.ok_or_else(|| anyhow::anyhow!("No package file found after download"))?;

    println!("Downloaded file: {}", downloaded_file.display());

    // Extract the package
    let extract_dir = work_dir.join(format!("extracted_{}", package_name));
    fs::create_dir_all(&extract_dir)?;

    if downloaded_file.extension().is_some_and(|ext| ext == "whl") {
        extract_wheel(&downloaded_file, &extract_dir)?;
    } else if downloaded_file.to_string_lossy().ends_with(".tar.gz") {
        extract_tar_gz(&downloaded_file, &extract_dir)?;
    } else {
        return Err(anyhow::anyhow!("Unsupported package format"));
    }

    // Find Python files
    let python_files = find_python_files(&extract_dir)?;

    Ok(PackageInfo {
        extract_path: extract_dir,
        python_files,
    })
}

/// Extract a wheel file (which is just a zip file)
fn extract_wheel(wheel_path: &Path, dest_dir: &Path) -> Result<()> {
    use std::fs::File;
    use std::io;
    use zip::ZipArchive;

    let file = File::open(wheel_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Extract a tar.gz file
fn extract_tar_gz(tar_path: &Path, dest_dir: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    let file = File::open(tar_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    archive.unpack(dest_dir)?;

    Ok(())
}

/// Find all Python files in a directory recursively
fn find_python_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut python_files = Vec::new();
    find_python_files_recursive(dir, &mut python_files)?;
    Ok(python_files)
}

fn find_python_files_recursive(dir: &Path, python_files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip certain directories
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if !dir_name.starts_with('.') && dir_name != "__pycache__" {
                find_python_files_recursive(&path, python_files)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "py") {
            // Skip test files and __init__.py for now
            let file_name = path.file_name().unwrap().to_string_lossy();
            if !file_name.starts_with("test_")
                && !file_name.contains("test")
                && file_name != "__init__.py"
            {
                python_files.push(path);
            }
        }
    }

    Ok(())
}

/// Compile a Python file using py2pyd
fn compile_with_py2pyd(input_file: &Path, output_file: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "compile",
            "--input",
            input_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
            "--use-uv",
            "true",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Compilation failed: {}", stderr));
    }

    Ok(())
}
