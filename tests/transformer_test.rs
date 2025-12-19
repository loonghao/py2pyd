//! Unit tests for the transformer module
//!
//! These tests verify Python AST to Rust code transformation.

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod transformer_tests {
    use super::*;

    /// Test transforming a simple function
    #[test]
    fn test_transform_simple_function() -> Result<()> {
        let source = r#"
def hello():
    return "Hello!"
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "simple", 2);

        assert!(rust_code.contains("use pyo3::prelude::*"));
        assert!(rust_code.contains("#[pymodule]"));
        assert!(rust_code.contains("fn simple"));
        assert!(rust_code.contains("#[pyfunction]"));
        assert!(rust_code.contains("fn hello"));

        Ok(())
    }

    /// Test transforming multiple functions
    #[test]
    fn test_transform_multiple_functions() -> Result<()> {
        let source = r#"
def add(a, b):
    return a + b

def subtract(a, b):
    return a - b

def multiply(a, b):
    return a * b
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "math_ops", 2);

        assert!(rust_code.contains("fn add"));
        assert!(rust_code.contains("fn subtract"));
        assert!(rust_code.contains("fn multiply"));
        assert!(rust_code.contains("wrap_pyfunction!(add"));
        assert!(rust_code.contains("wrap_pyfunction!(subtract"));
        assert!(rust_code.contains("wrap_pyfunction!(multiply"));

        Ok(())
    }

    /// Test transforming a class
    #[test]
    fn test_transform_class() -> Result<()> {
        let source = r#"
class MyClass:
    def __init__(self):
        pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "class_module", 2);

        assert!(rust_code.contains("#[pyclass]"));
        assert!(rust_code.contains("struct MyClass"));
        assert!(rust_code.contains("#[pymethods]"));
        assert!(rust_code.contains("impl MyClass"));
        assert!(rust_code.contains("#[new]"));

        Ok(())
    }

    /// Test transforming multiple classes
    #[test]
    fn test_transform_multiple_classes() -> Result<()> {
        let source = r#"
class Point:
    pass

class Rectangle:
    pass

class Circle:
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "shapes", 2);

        assert!(rust_code.contains("struct Point"));
        assert!(rust_code.contains("struct Rectangle"));
        assert!(rust_code.contains("struct Circle"));
        assert!(rust_code.contains("m.add_class::<Point>()"));
        assert!(rust_code.contains("m.add_class::<Rectangle>()"));
        assert!(rust_code.contains("m.add_class::<Circle>()"));

        Ok(())
    }

    /// Test transforming mixed functions and classes
    #[test]
    fn test_transform_mixed() -> Result<()> {
        let source = r#"
def create_point(x, y):
    return Point(x, y)

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "mixed", 2);

        assert!(rust_code.contains("#[pyfunction]"));
        assert!(rust_code.contains("fn create_point"));
        assert!(rust_code.contains("#[pyclass]"));
        assert!(rust_code.contains("struct Point"));

        Ok(())
    }

    /// Test Cargo.toml generation with default optimization
    #[test]
    fn test_generate_cargo_toml_default() {
        let cargo_toml = py2pyd::generate_cargo_toml("my_module", 2);

        assert!(cargo_toml.contains("[package]"));
        assert!(cargo_toml.contains("name = \"my_module\""));
        assert!(cargo_toml.contains("version = \"0.1.0\""));
        assert!(cargo_toml.contains("edition = \"2021\""));
        assert!(cargo_toml.contains("[lib]"));
        assert!(cargo_toml.contains("crate-type = [\"cdylib\"]"));
        assert!(cargo_toml.contains("[dependencies]"));
        assert!(cargo_toml.contains("pyo3"));
        assert!(cargo_toml.contains("[profile.release]"));
        assert!(cargo_toml.contains("opt-level = 2"));
    }

    /// Test Cargo.toml generation with optimization level 0
    #[test]
    fn test_generate_cargo_toml_opt_0() {
        let cargo_toml = py2pyd::generate_cargo_toml("module", 0);

        assert!(cargo_toml.contains("opt-level = 0"));
        assert!(!cargo_toml.contains("lto"));
    }

    /// Test Cargo.toml generation with optimization level 1
    #[test]
    fn test_generate_cargo_toml_opt_1() {
        let cargo_toml = py2pyd::generate_cargo_toml("module", 1);

        assert!(cargo_toml.contains("opt-level = 1"));
    }

    /// Test Cargo.toml generation with optimization level 3 (with LTO)
    #[test]
    fn test_generate_cargo_toml_opt_3() {
        let cargo_toml = py2pyd::generate_cargo_toml("module", 3);

        assert!(cargo_toml.contains("opt-level = 3"));
        assert!(cargo_toml.contains("lto = true"));
        assert!(cargo_toml.contains("codegen-units = 1"));
    }

    /// Test Cargo.toml generation with high optimization level (>3)
    #[test]
    fn test_generate_cargo_toml_opt_high() {
        let cargo_toml = py2pyd::generate_cargo_toml("module", 5);

        // Should be treated as level 3
        assert!(cargo_toml.contains("opt-level = 3"));
        assert!(cargo_toml.contains("lto = true"));
    }

    /// Test transform_file function
    #[test]
    fn test_transform_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("test_transform.py");

        let content = r#"
"""Test module."""

def greet(name):
    return f"Hello, {name}!"

class Greeter:
    def __init__(self, prefix):
        self.prefix = prefix
    
    def greet(self, name):
        return f"{self.prefix}, {name}!"
"#;

        fs::write(&python_file, content)?;

        let transformed = py2pyd::transform_file(&python_file, 2)?;

        assert_eq!(transformed.module_name, "test_transform");
        assert!(!transformed.rust_code.is_empty());
        assert!(!transformed.cargo_toml.is_empty());
        assert!(!transformed.build_script.is_empty());

        // Check rust code content
        assert!(transformed.rust_code.contains("fn greet"));
        assert!(transformed.rust_code.contains("struct Greeter"));

        // Check cargo toml content
        assert!(transformed.cargo_toml.contains("name = \"test_transform\""));

        Ok(())
    }

    /// Test transform_file with underscore in name
    #[test]
    fn test_transform_file_with_underscore() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("my_cool_module.py");

        let content = r#"
def func():
    pass
"#;

        fs::write(&python_file, content)?;

        let transformed = py2pyd::transform_file(&python_file, 2)?;

        assert_eq!(transformed.module_name, "my_cool_module");
        assert!(transformed.rust_code.contains("fn my_cool_module"));
        assert!(transformed.cargo_toml.contains("name = \"my_cool_module\""));

        Ok(())
    }

    /// Test transforming empty module
    #[test]
    fn test_transform_empty_module() -> Result<()> {
        let source = "";

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "empty", 2);

        // Should still generate valid module structure
        assert!(rust_code.contains("#[pymodule]"));
        assert!(rust_code.contains("fn empty"));
        assert!(rust_code.contains("Ok(())"));

        Ok(())
    }

    /// Test transforming module with only imports (no functions/classes)
    #[test]
    fn test_transform_imports_only() -> Result<()> {
        let source = r#"
import os
import sys
from pathlib import Path
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "imports_only", 2);

        // Should still generate valid module structure
        assert!(rust_code.contains("#[pymodule]"));
        assert!(rust_code.contains("fn imports_only"));

        Ok(())
    }

    /// Test transforming module with module-level variables
    #[test]
    fn test_transform_with_vars() -> Result<()> {
        let source = r#"
VERSION = "1.0.0"
DEBUG = True

def get_version():
    return VERSION
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "with_vars", 2);

        // Function should be transformed
        assert!(rust_code.contains("fn get_version"));

        Ok(())
    }

    /// Test that generated code has proper structure
    #[test]
    fn test_generated_code_structure() -> Result<()> {
        let source = r#"
def func():
    pass

class MyClass:
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "structured", 2);

        // Check imports come first
        let import_pos = rust_code.find("use pyo3::prelude::*").unwrap();
        let module_pos = rust_code.find("#[pymodule]").unwrap();

        assert!(import_pos < module_pos, "Imports should come before module");

        // Check module contains function and class registrations
        assert!(rust_code.contains("m.add_function"));
        assert!(rust_code.contains("m.add_class"));

        Ok(())
    }

    /// Test Cargo.toml contains maturin metadata
    #[test]
    fn test_cargo_toml_maturin_metadata() {
        let cargo_toml = py2pyd::generate_cargo_toml("maturin_test", 2);

        assert!(cargo_toml.contains("[package.metadata.maturin]"));
        assert!(cargo_toml.contains("binding = \"pyo3\""));
    }

    /// Test module name sanitization (if any special chars)
    #[test]
    fn test_transform_module_name() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Test with various file names
        let test_cases = vec![
            ("simple.py", "simple"),
            ("with_underscore.py", "with_underscore"),
            ("CamelCase.py", "CamelCase"),
            ("numbers123.py", "numbers123"),
        ];

        for (filename, expected_name) in test_cases {
            let python_file = temp_dir.path().join(filename);
            fs::write(&python_file, "def func(): pass")?;

            let transformed = py2pyd::transform_file(&python_file, 2)?;
            assert_eq!(
                transformed.module_name, expected_name,
                "Module name mismatch for {filename}"
            );
        }

        Ok(())
    }

    /// Test TransformedModule struct fields
    #[test]
    fn test_transformed_module_fields() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let python_file = temp_dir.path().join("fields_test.py");

        fs::write(&python_file, "def test(): pass")?;

        let transformed = py2pyd::transform_file(&python_file, 2)?;

        // Check all fields are populated
        assert!(!transformed.module_name.is_empty());
        assert!(!transformed.rust_code.is_empty());
        assert!(!transformed.cargo_toml.is_empty());
        assert!(!transformed.build_script.is_empty());
        // build_dir is a PathBuf, just check it exists as a field
        let _ = &transformed.build_dir;

        Ok(())
    }

    /// Test that generated Rust code is syntactically plausible
    #[test]
    fn test_generated_rust_syntax() -> Result<()> {
        let source = r#"
def add(a, b):
    return a + b

class Calculator:
    def __init__(self):
        self.value = 0
"#;

        let ast = py2pyd::parse_source(source)?;
        let rust_code = py2pyd::transform_ast(&ast, "syntax_test", 2);

        // Check for balanced braces (simple check)
        let open_braces = rust_code.matches('{').count();
        let close_braces = rust_code.matches('}').count();
        assert_eq!(
            open_braces, close_braces,
            "Braces should be balanced"
        );

        // Check for proper function signatures
        assert!(rust_code.contains("fn "));
        assert!(rust_code.contains("-> PyResult"));

        // Check for proper struct definition
        assert!(rust_code.contains("struct Calculator"));

        Ok(())
    }
}
