use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Integration test for downloading and compiling a pure Python pip package
#[cfg(test)]
mod pip_package_tests {
    use super::*;

    /// Test downloading and compiling a simple pure Python package
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_download_and_compile_pure_python_package() -> Result<()> {
        // Create a temporary directory for our test
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();
        
        println!("Test directory: {}", test_dir.display());

        // Test with a simple, pure Python package
        let package_name = "six"; // A simple, stable pure Python package
        let package_version = "1.16.0";
        
        // Step 1: Download the package using pip
        let download_result = download_pip_package(package_name, package_version, test_dir)?;
        println!("Downloaded package to: {}", download_result.display());

        // Step 2: Find Python files in the downloaded package
        let python_files = find_python_files(&download_result)?;
        println!("Found {} Python files", python_files.len());
        
        assert!(!python_files.is_empty(), "Should find at least one Python file");

        // Step 3: Try to compile each Python file to pyd
        let output_dir = test_dir.join("compiled");
        fs::create_dir_all(&output_dir)?;

        let mut successful_compilations = 0;
        let mut failed_compilations = 0;

        for python_file in &python_files {
            println!("Attempting to compile: {}", python_file.display());
            
            match compile_python_file(python_file, &output_dir) {
                Ok(pyd_path) => {
                    println!("✅ Successfully compiled to: {}", pyd_path.display());
                    assert!(pyd_path.exists(), "Compiled pyd file should exist");
                    successful_compilations += 1;
                }
                Err(e) => {
                    println!("❌ Failed to compile {}: {}", python_file.display(), e);
                    failed_compilations += 1;
                }
            }
        }

        println!("Compilation results:");
        println!("  ✅ Successful: {}", successful_compilations);
        println!("  ❌ Failed: {}", failed_compilations);

        // We expect at least some files to compile successfully
        assert!(successful_compilations > 0, "At least one file should compile successfully");

        Ok(())
    }

    /// Test with a more complex package that has multiple modules
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_download_and_compile_requests_package() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();
        
        println!("Test directory: {}", test_dir.display());

        // Test with requests package (more complex, but still pure Python)
        let package_name = "requests";
        let package_version = "2.31.0";
        
        let download_result = download_pip_package(package_name, package_version, test_dir)?;
        println!("Downloaded package to: {}", download_result.display());

        let python_files = find_python_files(&download_result)?;
        println!("Found {} Python files in requests package", python_files.len());
        
        assert!(!python_files.is_empty(), "Should find Python files in requests package");

        // Try to compile a few key files
        let output_dir = test_dir.join("compiled");
        fs::create_dir_all(&output_dir)?;

        let mut compiled_count = 0;
        let max_files_to_test = 5; // Limit to avoid long test times

        for python_file in python_files.iter().take(max_files_to_test) {
            if let Ok(pyd_path) = compile_python_file(python_file, &output_dir) {
                println!("✅ Compiled: {} -> {}", 
                    python_file.file_name().unwrap().to_string_lossy(),
                    pyd_path.file_name().unwrap().to_string_lossy()
                );
                compiled_count += 1;
            }
        }

        println!("Successfully compiled {}/{} files", compiled_count, max_files_to_test.min(python_files.len()));
        
        Ok(())
    }
}

/// Download a pip package to a specified directory
fn download_pip_package(package_name: &str, version: &str, target_dir: &Path) -> Result<PathBuf> {
    let package_spec = format!("{}=={}", package_name, version);
    let download_dir = target_dir.join("downloads");
    fs::create_dir_all(&download_dir)?;

    println!("Downloading {} to {}", package_spec, download_dir.display());

    // Use pip download to get the package
    let output = Command::new("pip")
        .args(&[
            "download",
            "--no-deps", // Don't download dependencies
            "--dest", download_dir.to_str().unwrap(),
            &package_spec,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to download package: {}", stderr));
    }

    // Find the downloaded file
    let entries = fs::read_dir(&download_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "whl" || ext == "gz") {
            // Extract the package
            return extract_package(&path, &download_dir);
        }
    }

    Err(anyhow::anyhow!("No package file found after download"))
}

/// Extract a downloaded package (wheel or tar.gz)
fn extract_package(package_path: &Path, extract_dir: &Path) -> Result<PathBuf> {
    let extract_target = extract_dir.join("extracted");
    fs::create_dir_all(&extract_target)?;

    if package_path.extension().map_or(false, |ext| ext == "whl") {
        // Extract wheel file (it's just a zip)
        extract_zip(package_path, &extract_target)?;
    } else if package_path.to_string_lossy().ends_with(".tar.gz") {
        // Extract tar.gz file
        extract_tar_gz(package_path, &extract_target)?;
    }

    Ok(extract_target)
}

/// Extract a zip file (for .whl files)
fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    use zip::ZipArchive;
    use std::fs::File;
    use std::io;

    let file = File::open(zip_path)?;
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
    use tar::Archive;
    use std::fs::File;

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
            find_python_files_recursive(&path, python_files)?;
        } else if path.extension().map_or(false, |ext| ext == "py") {
            // Skip __pycache__ and test files for now
            if !path.to_string_lossy().contains("__pycache__") 
                && !path.to_string_lossy().contains("test") {
                python_files.push(path);
            }
        }
    }

    Ok(())
}

/// Compile a Python file to pyd using our py2pyd tool
fn compile_python_file(python_file: &Path, output_dir: &Path) -> Result<PathBuf> {
    let file_stem = python_file.file_stem().unwrap().to_string_lossy();
    let output_file = output_dir.join(format!("{}.pyd", file_stem));

    // Use our py2pyd binary to compile
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "compile",
            "--input", python_file.to_str().unwrap(),
            "--output", output_file.to_str().unwrap(),
            "--use-uv", "true",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Failed to compile {}: stdout: {}, stderr: {}", 
            python_file.display(), 
            stdout, 
            stderr
        ));
    }

    if !output_file.exists() {
        return Err(anyhow::anyhow!("Output file was not created: {}", output_file.display()));
    }

    Ok(output_file)
}
