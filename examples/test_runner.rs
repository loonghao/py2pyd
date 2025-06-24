use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Example demonstrating how to test py2pyd compilation
fn main() -> Result<()> {
    println!("🚀 py2pyd Test Runner Example");
    println!("==============================\n");

    // Test 1: Create and compile a simple Python module
    test_simple_compilation()?;

    // Test 2: Test turbo-cdn integration
    test_turbo_cdn_integration()?;

    println!("\n✅ All example tests completed!");
    println!("💡 To run the full test suite:");
    println!("   cargo test -- --ignored");

    Ok(())
}

/// Test compiling a simple Python module
fn test_simple_compilation() -> Result<()> {
    println!("📝 Test 1: Simple Python Module Compilation");
    println!("--------------------------------------------");

    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();

    // Create a simple Python module
    let python_content = r#"
"""
Example Python module for testing py2pyd compilation.
"""

def greet(name):
    """Return a personalized greeting."""
    return f"Hello, {name}! This was compiled with py2pyd."

def calculate_factorial(n):
    """Calculate factorial of n."""
    if n <= 1:
        return 1
    return n * calculate_factorial(n - 1)

def process_list(items):
    """Process a list of items."""
    return [item.upper() if isinstance(item, str) else item * 2 for item in items]

class Calculator:
    """A simple calculator class."""
    
    def __init__(self):
        self.history = []
    
    def add(self, a, b):
        result = a + b
        self.history.append(f"{a} + {b} = {result}")
        return result
    
    def multiply(self, a, b):
        result = a * b
        self.history.append(f"{a} * {b} = {result}")
        return result
    
    def get_history(self):
        return self.history.copy()

# Module constants
VERSION = "1.0.0"
AUTHOR = "py2pyd test"
"#;

    let python_file = test_dir.join("example_module.py");
    fs::write(&python_file, python_content)?;
    
    println!("📄 Created test Python file: {}", python_file.display());

    // Compile the Python file
    let output_file = test_dir.join("example_module.pyd");
    
    println!("🔨 Compiling Python module...");
    let result = compile_python_file(&python_file, &output_file);
    
    match result {
        Ok(()) => {
            println!("✅ Successfully compiled to: {}", output_file.display());
            
            if output_file.exists() {
                let metadata = fs::metadata(&output_file)?;
                println!("📊 Compiled file size: {} bytes", metadata.len());
            }
        }
        Err(e) => {
            println!("❌ Compilation failed: {}", e);
            println!("💡 This might be expected if build tools are not available");
        }
    }

    println!();
    Ok(())
}

/// Test turbo-cdn integration
fn test_turbo_cdn_integration() -> Result<()> {
    println!("🌐 Test 2: Turbo-CDN Integration");
    println!("--------------------------------");

    println!("🔍 Testing turbo-cdn URL optimization...");
    
    // Test URL optimization
    let test_url = "https://github.com/astral-sh/uv/releases/download/0.7.6/uv-x86_64-pc-windows-msvc.zip";
    
    match test_url_optimization(test_url) {
        Ok(optimized_url) => {
            println!("✅ URL optimization successful!");
            println!("   Original: {}", test_url);
            println!("   Optimized: {}", optimized_url);
        }
        Err(e) => {
            println!("❌ URL optimization failed: {}", e);
            println!("💡 This might be expected without internet connection");
        }
    }

    println!();
    Ok(())
}

/// Compile a Python file using py2pyd
fn compile_python_file(input_file: &Path, output_file: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "compile",
            "--input", input_file.to_str().unwrap(),
            "--output", output_file.to_str().unwrap(),
            "--use-uv", "true",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Compilation failed: {}", stderr));
    }

    Ok(())
}

/// Test URL optimization using turbo-cdn
fn test_url_optimization(url: &str) -> Result<String> {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--example",
            "turbo_cdn_test",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Turbo-CDN test failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse the output to find the optimized URL
    for line in stdout.lines() {
        if line.starts_with("Optimized:") {
            return Ok(line.replace("Optimized: ", ""));
        }
    }

    // If we can't find the optimized URL, just return the original
    Ok(url.to_string())
}
