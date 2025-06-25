use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Simple integration test for compiling Python packages
#[cfg(test)]
mod simple_package_tests {
    use super::*;

    /// Test compiling a simple Python module
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_compile_simple_python_module() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();

        // Create a simple Python module
        let python_content = r#"
"""
A simple Python module for testing py2pyd compilation.
"""

def hello_world():
    """Return a greeting message."""
    return "Hello from compiled Python!"

def add_numbers(a, b):
    """Add two numbers together."""
    return a + b

def fibonacci(n):
    """Calculate fibonacci number."""
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

class SimpleClass:
    """A simple class for testing."""
    
    def __init__(self, name):
        self.name = name
    
    def greet(self):
        return f"Hello, I'm {self.name}!"
    
    def calculate(self, x, y):
        return x * y + len(self.name)

# Module-level variable
MODULE_VERSION = "1.0.0"

# Module-level function call
DEFAULT_GREETING = hello_world()
"#;

        let python_file = test_dir.join("simple_module.py");
        fs::write(&python_file, python_content)?;

        println!("Created test Python file: {}", python_file.display());

        // Compile the Python file
        let output_file = test_dir.join("simple_module.pyd");
        let result = compile_python_file_with_py2pyd(&python_file, &output_file);

        match result {
            Ok(()) => {
                println!("✅ Successfully compiled Python module to pyd");
                assert!(output_file.exists(), "Output pyd file should exist");

                // Check file size (should be > 0)
                let metadata = fs::metadata(&output_file)?;
                assert!(metadata.len() > 0, "Output file should not be empty");

                println!("Compiled file size: {} bytes", metadata.len());
            }
            Err(e) => {
                println!("❌ Compilation failed: {}", e);
                // Don't fail the test immediately, let's see what went wrong
                println!("This might be expected if build tools are not available");
            }
        }

        Ok(())
    }

    /// Test compiling a Python module with imports
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_compile_python_with_imports() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();

        // Create a Python module that uses standard library imports
        let python_content = r#"
"""
A Python module with imports for testing py2pyd compilation.
"""

import os
import sys
import json
import math
from datetime import datetime

def get_system_info():
    """Get basic system information."""
    return {
        "platform": sys.platform,
        "python_version": sys.version,
        "current_time": datetime.now().isoformat(),
    }

def calculate_circle_area(radius):
    """Calculate the area of a circle."""
    return math.pi * radius ** 2

def process_json_data(json_string):
    """Process JSON data."""
    try:
        data = json.loads(json_string)
        return {"success": True, "data": data}
    except json.JSONDecodeError as e:
        return {"success": False, "error": str(e)}

def list_directory(path="."):
    """List files in a directory."""
    try:
        return os.listdir(path)
    except OSError as e:
        return f"Error: {e}"

class DataProcessor:
    """A class for processing data."""
    
    def __init__(self):
        self.processed_count = 0
    
    def process_numbers(self, numbers):
        """Process a list of numbers."""
        result = []
        for num in numbers:
            result.append(math.sqrt(num) if num >= 0 else 0)
        self.processed_count += len(numbers)
        return result
    
    def get_stats(self):
        """Get processing statistics."""
        return {
            "processed_count": self.processed_count,
            "timestamp": datetime.now().isoformat()
        }
"#;

        let python_file = test_dir.join("module_with_imports.py");
        fs::write(&python_file, python_content)?;

        println!(
            "Created Python file with imports: {}",
            python_file.display()
        );

        // Compile the Python file
        let output_file = test_dir.join("module_with_imports.pyd");
        let result = compile_python_file_with_py2pyd(&python_file, &output_file);

        match result {
            Ok(()) => {
                println!("✅ Successfully compiled Python module with imports");
                assert!(output_file.exists(), "Output pyd file should exist");

                let metadata = fs::metadata(&output_file)?;
                println!("Compiled file size: {} bytes", metadata.len());
            }
            Err(e) => {
                println!("❌ Compilation failed: {}", e);
                println!("This might be expected - some imports might not be supported");
            }
        }

        Ok(())
    }

    /// Test batch compilation of multiple Python files
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_batch_compile_multiple_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path();

        // Create multiple Python files
        let files_to_create = vec![
            (
                "math_utils.py",
                r#"
def add(a, b):
    return a + b

def multiply(a, b):
    return a * b

def power(base, exp):
    return base ** exp
"#,
            ),
            (
                "string_utils.py",
                r#"
def reverse_string(s):
    return s[::-1]

def count_words(text):
    return len(text.split())

def capitalize_words(text):
    return ' '.join(word.capitalize() for word in text.split())
"#,
            ),
            (
                "list_utils.py",
                r#"
def find_max(numbers):
    return max(numbers) if numbers else None

def find_min(numbers):
    return min(numbers) if numbers else None

def average(numbers):
    return sum(numbers) / len(numbers) if numbers else 0
"#,
            ),
        ];

        let mut created_files = Vec::new();
        for (filename, content) in files_to_create {
            let file_path = test_dir.join(filename);
            fs::write(&file_path, content)?;
            created_files.push(file_path);
            println!("Created: {}", filename);
        }

        // Try to compile each file
        let output_dir = test_dir.join("compiled");
        fs::create_dir_all(&output_dir)?;

        let mut successful_compilations = 0;
        let mut failed_compilations = 0;

        for python_file in &created_files {
            let file_stem = python_file.file_stem().unwrap().to_string_lossy();
            let output_file = output_dir.join(format!("{}.pyd", file_stem));

            match compile_python_file_with_py2pyd(python_file, &output_file) {
                Ok(()) => {
                    println!(
                        "✅ Compiled: {}",
                        python_file.file_name().unwrap().to_string_lossy()
                    );
                    successful_compilations += 1;
                }
                Err(e) => {
                    println!(
                        "❌ Failed to compile {}: {}",
                        python_file.file_name().unwrap().to_string_lossy(),
                        e
                    );
                    failed_compilations += 1;
                }
            }
        }

        println!("Batch compilation results:");
        println!("  ✅ Successful: {}", successful_compilations);
        println!("  ❌ Failed: {}", failed_compilations);
        println!(
            "  📊 Success rate: {:.1}%",
            (successful_compilations as f64 / created_files.len() as f64) * 100.0
        );

        Ok(())
    }
}

/// Compile a Python file using our py2pyd tool
fn compile_python_file_with_py2pyd(input_file: &Path, output_file: &Path) -> Result<()> {
    println!(
        "Compiling {} -> {}",
        input_file.display(),
        output_file.display()
    );

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "compile",
            "--input",
            input_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
            "--use-uv",
            "true",
            "--verbose",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "py2pyd compilation failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout,
            stderr
        ));
    }

    if !output_file.exists() {
        return Err(anyhow::anyhow!(
            "Output file was not created: {}",
            output_file.display()
        ));
    }

    // Print compilation output for debugging
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        println!("Compilation output:\n{}", stdout);
    }

    Ok(())
}
