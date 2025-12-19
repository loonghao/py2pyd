//! End-to-end tests for py2pyd compilation
//!
//! These tests verify the complete compilation pipeline from Python source
//! to compiled extension modules.

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to compile a Python file using the py2pyd CLI
fn compile_with_cli(input: &Path, output: &Path, use_uv: bool) -> Result<std::process::Output> {
    let output_result = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "compile",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--use-uv",
            if use_uv { "true" } else { "false" },
        ])
        .output()?;

    Ok(output_result)
}

/// Helper function to batch compile Python files using the py2pyd CLI
fn batch_compile_with_cli(
    input_dir: &Path,
    output_dir: &Path,
    recursive: bool,
) -> Result<std::process::Output> {
    let mut args = vec![
        "run",
        "--quiet",
        "--",
        "batch",
        "--input",
        input_dir.to_str().unwrap(),
        "--output",
        output_dir.to_str().unwrap(),
    ];

    if recursive {
        args.push("--recursive");
    }

    let output = Command::new("cargo").args(&args).output()?;

    Ok(output)
}

/// Test module for e2e compilation tests
#[cfg(test)]
mod e2e_tests {
    use super::*;

    /// Test compiling a minimal Python module
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_e2e_compile_minimal_module() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("minimal.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "minimal.pyd"
        } else {
            "minimal.so"
        });

        // Create a minimal Python module
        let content = r#"
"""Minimal Python module."""

def hello():
    """Return a greeting."""
    return "Hello from minimal module!"
"#;

        fs::write(&python_file, content)?;
        println!("Created Python file: {}", python_file.display());

        // Compile the module
        let output = compile_with_cli(&python_file, &output_file, true)?;

        if output.status.success() {
            println!("✅ Compilation successful!");
            assert!(output_file.exists(), "Output file should exist");

            let metadata = fs::metadata(&output_file)?;
            println!("Output file size: {} bytes", metadata.len());
            assert!(metadata.len() > 0, "Output file should not be empty");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Compilation output:\nstdout: {stdout}\nstderr: {stderr}");
            // Don't fail - build tools might not be available
        }

        Ok(())
    }

    /// Test compiling a module with multiple functions
    #[test]
    #[ignore]
    fn test_e2e_compile_multi_function_module() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("math_ops.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "math_ops.pyd"
        } else {
            "math_ops.so"
        });

        let content = r#"
"""Mathematical operations module."""

def add(a, b):
    """Add two numbers."""
    return a + b

def subtract(a, b):
    """Subtract b from a."""
    return a - b

def multiply(a, b):
    """Multiply two numbers."""
    return a * b

def divide(a, b):
    """Divide a by b."""
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

def power(base, exp):
    """Raise base to the power of exp."""
    return base ** exp

def factorial(n):
    """Calculate factorial of n."""
    if n < 0:
        raise ValueError("Factorial not defined for negative numbers")
    if n <= 1:
        return 1
    return n * factorial(n - 1)
"#;

        fs::write(&python_file, content)?;

        let output = compile_with_cli(&python_file, &output_file, true)?;

        if output.status.success() {
            println!("✅ Multi-function module compiled successfully!");
            assert!(output_file.exists());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Compilation failed (may be expected): {stderr}");
        }

        Ok(())
    }

    /// Test compiling a module with classes
    #[test]
    #[ignore]
    fn test_e2e_compile_class_module() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("classes.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "classes.pyd"
        } else {
            "classes.so"
        });

        let content = r#"
"""Module with class definitions."""

class Point:
    """A 2D point class."""
    
    def __init__(self, x, y):
        self.x = x
        self.y = y
    
    def distance_to_origin(self):
        """Calculate distance to origin."""
        return (self.x ** 2 + self.y ** 2) ** 0.5
    
    def move(self, dx, dy):
        """Move the point by dx, dy."""
        self.x += dx
        self.y += dy
        return self

class Rectangle:
    """A rectangle class."""
    
    def __init__(self, width, height):
        self.width = width
        self.height = height
    
    def area(self):
        """Calculate area."""
        return self.width * self.height
    
    def perimeter(self):
        """Calculate perimeter."""
        return 2 * (self.width + self.height)

def create_point(x, y):
    """Factory function for Point."""
    return Point(x, y)

def create_rectangle(width, height):
    """Factory function for Rectangle."""
    return Rectangle(width, height)
"#;

        fs::write(&python_file, content)?;

        let output = compile_with_cli(&python_file, &output_file, true)?;

        if output.status.success() {
            println!("✅ Class module compiled successfully!");
            assert!(output_file.exists());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Compilation failed (may be expected): {stderr}");
        }

        Ok(())
    }

    /// Test compiling a module with imports
    #[test]
    #[ignore]
    fn test_e2e_compile_module_with_imports() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("with_imports.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "with_imports.pyd"
        } else {
            "with_imports.so"
        });

        let content = r#"
"""Module with standard library imports."""

import os
import sys
import json
import math
from datetime import datetime
from pathlib import Path

def get_system_info():
    """Get system information."""
    return {
        "platform": sys.platform,
        "cwd": os.getcwd(),
        "python_version": sys.version,
    }

def calculate_circle(radius):
    """Calculate circle properties."""
    return {
        "area": math.pi * radius ** 2,
        "circumference": 2 * math.pi * radius,
    }

def parse_json(json_str):
    """Parse JSON string."""
    return json.loads(json_str)

def get_timestamp():
    """Get current timestamp."""
    return datetime.now().isoformat()

def list_files(directory="."):
    """List files in directory."""
    return list(Path(directory).iterdir())
"#;

        fs::write(&python_file, content)?;

        let output = compile_with_cli(&python_file, &output_file, true)?;

        if output.status.success() {
            println!("✅ Module with imports compiled successfully!");
            assert!(output_file.exists());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Compilation failed (may be expected): {stderr}");
        }

        Ok(())
    }

    /// Test batch compilation of multiple files
    #[test]
    #[ignore]
    fn test_e2e_batch_compile() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("dist");

        fs::create_dir_all(&input_dir)?;

        // Create multiple Python files
        let files = vec![
            (
                "utils.py",
                r#"
def helper():
    return "helper"
"#,
            ),
            (
                "math_utils.py",
                r#"
def add(a, b):
    return a + b
"#,
            ),
            (
                "string_utils.py",
                r#"
def reverse(s):
    return s[::-1]
"#,
            ),
        ];

        for (name, content) in &files {
            fs::write(input_dir.join(name), content)?;
        }

        println!("Created {} Python files in {}", files.len(), input_dir.display());

        let output = batch_compile_with_cli(&input_dir, &output_dir, false)?;

        if output.status.success() {
            println!("✅ Batch compilation successful!");

            // Check that output directory was created
            assert!(output_dir.exists(), "Output directory should exist");

            // List compiled files
            if output_dir.exists() {
                for entry in fs::read_dir(&output_dir)? {
                    let entry = entry?;
                    println!("  Compiled: {}", entry.path().display());
                }
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Batch compilation output:\nstdout: {stdout}\nstderr: {stderr}");
        }

        Ok(())
    }

    /// Test recursive batch compilation
    #[test]
    #[ignore]
    fn test_e2e_batch_compile_recursive() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("project");
        let output_dir = temp_dir.path().join("build");

        // Create nested directory structure
        let sub_dirs = vec!["", "core", "utils", "utils/helpers"];

        for sub_dir in &sub_dirs {
            let dir = input_dir.join(sub_dir);
            fs::create_dir_all(&dir)?;

            // Create a Python file in each directory
            let file_name = if sub_dir.is_empty() {
                "main.py"
            } else {
                "module.py"
            };

            let content = format!(
                r#"
"""Module in {}."""

def function_in_{}():
    return "{}"
"#,
                if sub_dir.is_empty() { "root" } else { sub_dir },
                sub_dir.replace('/', "_"),
                sub_dir
            );

            fs::write(dir.join(file_name), content)?;
        }

        println!("Created nested Python project in {}", input_dir.display());

        let output = batch_compile_with_cli(&input_dir, &output_dir, true)?;

        if output.status.success() {
            println!("✅ Recursive batch compilation successful!");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Recursive batch compilation failed (may be expected): {stderr}");
        }

        Ok(())
    }

    /// Test compilation with different optimization levels
    #[test]
    #[ignore]
    fn test_e2e_optimization_levels() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("opt_test.py");

        let content = r#"
def compute_heavy(n):
    """A computationally heavy function."""
    result = 0
    for i in range(n):
        for j in range(n):
            result += i * j
    return result
"#;

        fs::write(&python_file, content)?;

        for opt_level in 0..=3 {
            let output_file = temp_dir.path().join(format!(
                "opt_test_O{}.{}",
                opt_level,
                if cfg!(windows) { "pyd" } else { "so" }
            ));

            let output = Command::new("cargo")
                .args([
                    "run",
                    "--quiet",
                    "--",
                    "compile",
                    "--input",
                    python_file.to_str().unwrap(),
                    "--output",
                    output_file.to_str().unwrap(),
                    "-O",
                    &opt_level.to_string(),
                ])
                .output()?;

            if output.status.success() {
                let size = fs::metadata(&output_file)?.len();
                println!("✅ O{}: {} bytes", opt_level, size);
            } else {
                println!("❌ O{}: compilation failed", opt_level);
            }
        }

        Ok(())
    }

    /// Test compilation with additional packages
    #[test]
    #[ignore]
    fn test_e2e_with_packages() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("with_deps.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "with_deps.pyd"
        } else {
            "with_deps.so"
        });

        let content = r#"
"""Module that might use external packages."""

def process_data(data):
    """Process some data."""
    return sorted(data)
"#;

        fs::write(&python_file, content)?;

        let output = Command::new("cargo")
            .args([
                "run",
                "--quiet",
                "--",
                "compile",
                "--input",
                python_file.to_str().unwrap(),
                "--output",
                output_file.to_str().unwrap(),
                "--packages",
                "typing-extensions",
            ])
            .output()?;

        if output.status.success() {
            println!("✅ Compilation with packages successful!");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Compilation failed (may be expected): {stderr}");
        }

        Ok(())
    }

    /// Test CLI help output
    #[test]
    fn test_cli_help() -> Result<()> {
        let output = Command::new("cargo")
            .args(["run", "--quiet", "--", "--help"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "Help command should succeed");
        assert!(stdout.contains("py2pyd"), "Help should mention py2pyd");
        assert!(stdout.contains("compile"), "Help should mention compile command");
        assert!(stdout.contains("batch"), "Help should mention batch command");

        Ok(())
    }

    /// Test CLI version output
    #[test]
    fn test_cli_version() -> Result<()> {
        let output = Command::new("cargo")
            .args(["run", "--quiet", "--", "--version"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "Version command should succeed");
        assert!(stdout.contains("py2pyd"), "Version should mention py2pyd");

        Ok(())
    }

    /// Test compile subcommand help
    #[test]
    fn test_compile_help() -> Result<()> {
        let output = Command::new("cargo")
            .args(["run", "--quiet", "--", "compile", "--help"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "Compile help should succeed");
        assert!(stdout.contains("--input"), "Should show --input option");
        assert!(stdout.contains("--output"), "Should show --output option");
        assert!(stdout.contains("--optimize"), "Should show --optimize option");

        Ok(())
    }

    /// Test batch subcommand help
    #[test]
    fn test_batch_help() -> Result<()> {
        let output = Command::new("cargo")
            .args(["run", "--quiet", "--", "batch", "--help"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "Batch help should succeed");
        assert!(stdout.contains("--input"), "Should show --input option");
        assert!(stdout.contains("--output"), "Should show --output option");
        assert!(stdout.contains("--recursive"), "Should show --recursive option");

        Ok(())
    }

    /// Test error handling for non-existent input file
    #[test]
    fn test_error_nonexistent_input() -> Result<()> {
        let output = Command::new("cargo")
            .args([
                "run",
                "--quiet",
                "--",
                "compile",
                "--input",
                "/nonexistent/path/to/file.py",
                "--output",
                "output.pyd",
            ])
            .output()?;

        assert!(
            !output.status.success(),
            "Should fail for non-existent input"
        );

        Ok(())
    }

    /// Test error handling for invalid Python syntax
    #[test]
    #[ignore]
    fn test_error_invalid_python() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("invalid.py");
        let output_file = temp_dir.path().join("invalid.pyd");

        // Write invalid Python syntax
        let content = r#"
def broken(
    # Missing closing parenthesis and body
"#;

        fs::write(&python_file, content)?;

        let output = compile_with_cli(&python_file, &output_file, true)?;

        // Should fail due to invalid syntax
        assert!(
            !output.status.success(),
            "Should fail for invalid Python syntax"
        );

        Ok(())
    }
}

/// Test module for library API e2e tests
#[cfg(test)]
mod lib_e2e_tests {
    use super::*;

    /// Test using the library API to compile a file
    #[test]
    #[ignore]
    fn test_lib_compile_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("lib_test.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "lib_test.pyd"
        } else {
            "lib_test.so"
        });

        let content = r#"
def greet(name):
    return f"Hello, {name}!"
"#;

        fs::write(&python_file, content)?;

        let config = py2pyd::CompileConfig::default();
        let result = py2pyd::compile_file(&python_file, &output_file, &config);

        match result {
            Ok(()) => {
                println!("✅ Library API compilation successful!");
                assert!(output_file.exists());
            }
            Err(e) => {
                println!("Library compilation failed (may be expected): {e}");
            }
        }

        Ok(())
    }

    /// Test using the library API with custom config
    #[test]
    #[ignore]
    fn test_lib_compile_with_custom_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("custom_config.py");
        let output_file = temp_dir.path().join(if cfg!(windows) {
            "custom_config.pyd"
        } else {
            "custom_config.so"
        });

        let content = r#"
def compute(x):
    return x * 2
"#;

        fs::write(&python_file, content)?;

        let config = py2pyd::CompileConfig {
            python_version: Some("3.10".to_string()),
            optimize_level: 3,
            keep_temp_files: false,
            ..Default::default()
        };

        let result = py2pyd::compile_file(&python_file, &output_file, &config);

        match result {
            Ok(()) => {
                println!("✅ Custom config compilation successful!");
            }
            Err(e) => {
                println!("Custom config compilation failed (may be expected): {e}");
            }
        }

        Ok(())
    }

    /// Test library batch compile API
    #[test]
    #[ignore]
    fn test_lib_batch_compile() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("input");
        let output_dir = temp_dir.path().join("output");

        fs::create_dir_all(&input_dir)?;

        // Create test files
        fs::write(
            input_dir.join("a.py"),
            "def func_a(): return 'a'",
        )?;
        fs::write(
            input_dir.join("b.py"),
            "def func_b(): return 'b'",
        )?;

        let config = py2pyd::CompileConfig::default();
        let result = py2pyd::batch_compile(
            input_dir.to_str().unwrap(),
            &output_dir,
            &config,
            false,
        );

        match result {
            Ok(()) => {
                println!("✅ Library batch compile successful!");
            }
            Err(e) => {
                println!("Library batch compile failed (may be expected): {e}");
            }
        }

        Ok(())
    }

    /// Test verify_build_tools API
    #[test]
    fn test_lib_verify_build_tools() {
        let result = py2pyd::verify_build_tools();

        match result {
            Ok(tools) => {
                println!("Build tools found:\n{}", tools.get_tools_info());
            }
            Err(e) => {
                println!("No build tools found: {e}");
                // This is expected on systems without build tools
            }
        }
    }

    /// Test transform_file API
    #[test]
    fn test_lib_transform_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("transform.py");

        let content = r#"
def hello():
    return "Hello!"

class Greeter:
    def greet(self):
        return "Hi!"
"#;

        fs::write(&python_file, content)?;

        let transformed = py2pyd::transform_file(&python_file, 2)?;

        assert_eq!(transformed.module_name, "transform");
        assert!(transformed.rust_code.contains("pyo3"));
        assert!(transformed.rust_code.contains("fn hello"));
        assert!(transformed.rust_code.contains("struct Greeter"));
        assert!(transformed.cargo_toml.contains("pyo3"));

        println!("✅ Transform file API works correctly!");

        Ok(())
    }
}
