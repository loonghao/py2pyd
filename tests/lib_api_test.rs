//! Integration tests for the py2pyd library API
//!
//! These tests verify that the library API works correctly when used
//! as a dependency in other projects.

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Test that CompileConfig can be created with default values
#[test]
fn test_compile_config_default() {
    let config = py2pyd::CompileConfig::default();

    assert!(config.python_path.is_none());
    assert!(config.python_version.is_none());
    assert_eq!(config.optimize_level, 2);
    assert!(!config.keep_temp_files);
    assert!(config.target_dcc.is_none());
    assert!(config.packages.is_empty());
}

/// Test that CompileConfig can be customized
#[test]
fn test_compile_config_custom() {
    let config = py2pyd::CompileConfig {
        python_path: Some(std::path::PathBuf::from("/usr/bin/python3")),
        python_version: Some("3.10".to_string()),
        optimize_level: 3,
        keep_temp_files: true,
        target_dcc: Some("maya".to_string()),
        packages: vec!["numpy".to_string(), "scipy".to_string()],
    };

    assert_eq!(
        config.python_path,
        Some(std::path::PathBuf::from("/usr/bin/python3"))
    );
    assert_eq!(config.python_version, Some("3.10".to_string()));
    assert_eq!(config.optimize_level, 3);
    assert!(config.keep_temp_files);
    assert_eq!(config.target_dcc, Some("maya".to_string()));
    assert_eq!(config.packages.len(), 2);
}

/// Test that UvEnvConfig can be created with default values
#[test]
fn test_uv_env_config_default() {
    let config = py2pyd::UvEnvConfig::default();

    assert!(config.python_path.is_none());
    assert!(config.python_version.is_none());
    assert!(!config.keep_venv);
    assert!(config.packages.is_empty());
}

/// Test that UvEnvConfig can be customized
#[test]
fn test_uv_env_config_custom() {
    let config = py2pyd::UvEnvConfig {
        python_path: Some(std::path::PathBuf::from("/usr/bin/python3")),
        python_version: Some("3.11".to_string()),
        keep_venv: true,
        packages: vec!["requests".to_string()],
    };

    assert_eq!(
        config.python_path,
        Some(std::path::PathBuf::from("/usr/bin/python3"))
    );
    assert_eq!(config.python_version, Some("3.11".to_string()));
    assert!(config.keep_venv);
    assert_eq!(config.packages, vec!["requests".to_string()]);
}

/// Test get_extension returns correct value for platform
#[test]
fn test_get_extension() {
    let ext = py2pyd::get_extension();

    #[cfg(windows)]
    assert_eq!(ext, "pyd");

    #[cfg(not(windows))]
    assert_eq!(ext, "so");
}

/// Test Python source parsing
#[test]
fn test_parse_source_simple() -> Result<()> {
    let source = r#"
def hello():
    return "Hello, World!"

def add(a, b):
    return a + b
"#;

    let ast = py2pyd::parse_source(source)?;
    assert!(!ast.is_empty());

    let functions = py2pyd::extract_functions(&ast);
    assert_eq!(functions.len(), 2);

    Ok(())
}

/// Test Python source parsing with classes
#[test]
fn test_parse_source_with_class() -> Result<()> {
    let source = r#"
class MyClass:
    def __init__(self):
        self.value = 0
    
    def increment(self):
        self.value += 1

def standalone_function():
    pass
"#;

    let ast = py2pyd::parse_source(source)?;
    assert!(!ast.is_empty());

    let classes = py2pyd::extract_classes(&ast);
    assert_eq!(classes.len(), 1);

    let functions = py2pyd::extract_functions(&ast);
    assert_eq!(functions.len(), 1);

    Ok(())
}

/// Test Python source parsing with imports
#[test]
fn test_parse_source_with_imports() -> Result<()> {
    let source = r#"
import os
import sys
from pathlib import Path
from typing import List, Dict

def get_path():
    return Path.cwd()
"#;

    let ast = py2pyd::parse_source(source)?;
    assert!(!ast.is_empty());

    let imports = py2pyd::extract_imports(&ast);
    assert_eq!(imports.len(), 2); // os, sys

    let from_imports = py2pyd::extract_from_imports(&ast);
    assert_eq!(from_imports.len(), 2); // pathlib, typing

    Ok(())
}

/// Test Python source parsing with module-level variables
#[test]
fn test_parse_source_with_module_vars() -> Result<()> {
    let source = r#"
VERSION = "1.0.0"
DEBUG = True
CONFIG = {"key": "value"}

def get_version():
    return VERSION
"#;

    let ast = py2pyd::parse_source(source)?;
    assert!(!ast.is_empty());

    let vars = py2pyd::extract_module_vars(&ast);
    assert_eq!(vars.len(), 3);

    Ok(())
}

/// Test parsing a Python file from disk
#[test]
fn test_parse_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let python_file = temp_dir.path().join("test_module.py");

    let content = r#"
"""Test module docstring."""

def greet(name):
    """Greet someone by name."""
    return f"Hello, {name}!"

class Greeter:
    """A class for greeting."""
    
    def __init__(self, prefix="Hello"):
        self.prefix = prefix
    
    def greet(self, name):
        return f"{self.prefix}, {name}!"
"#;

    fs::write(&python_file, content)?;

    let ast = py2pyd::parse_file(&python_file)?;
    assert!(!ast.is_empty());

    let functions = py2pyd::extract_functions(&ast);
    assert_eq!(functions.len(), 1);

    let classes = py2pyd::extract_classes(&ast);
    assert_eq!(classes.len(), 1);

    Ok(())
}

/// Test AST transformation
#[test]
fn test_transform_ast() -> Result<()> {
    let source = r#"
def hello():
    return "Hello!"

class MyClass:
    pass
"#;

    let ast = py2pyd::parse_source(source)?;
    let rust_code = py2pyd::transform_ast(&ast, "test_module", 2);

    // Check that the generated code contains expected elements
    assert!(rust_code.contains("use pyo3::prelude::*"));
    assert!(rust_code.contains("#[pymodule]"));
    assert!(rust_code.contains("fn test_module"));
    assert!(rust_code.contains("#[pyfunction]"));
    assert!(rust_code.contains("fn hello"));
    assert!(rust_code.contains("#[pyclass]"));
    assert!(rust_code.contains("struct MyClass"));

    Ok(())
}

/// Test Cargo.toml generation
#[test]
fn test_generate_cargo_toml() {
    let cargo_toml = py2pyd::generate_cargo_toml("my_module", 2);

    assert!(cargo_toml.contains("name = \"my_module\""));
    assert!(cargo_toml.contains("crate-type = [\"cdylib\"]"));
    assert!(cargo_toml.contains("pyo3"));
    assert!(cargo_toml.contains("[profile.release]"));
    assert!(cargo_toml.contains("opt-level = 2"));
}

/// Test Cargo.toml generation with different optimization levels
#[test]
fn test_generate_cargo_toml_optimization_levels() {
    // Level 0
    let cargo_toml_0 = py2pyd::generate_cargo_toml("module", 0);
    assert!(cargo_toml_0.contains("opt-level = 0"));

    // Level 1
    let cargo_toml_1 = py2pyd::generate_cargo_toml("module", 1);
    assert!(cargo_toml_1.contains("opt-level = 1"));

    // Level 2
    let cargo_toml_2 = py2pyd::generate_cargo_toml("module", 2);
    assert!(cargo_toml_2.contains("opt-level = 2"));

    // Level 3 (should include LTO)
    let cargo_toml_3 = py2pyd::generate_cargo_toml("module", 3);
    assert!(cargo_toml_3.contains("opt-level = 3"));
    assert!(cargo_toml_3.contains("lto = true"));
    assert!(cargo_toml_3.contains("codegen-units = 1"));
}

/// Test file transformation
#[test]
fn test_transform_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let python_file = temp_dir.path().join("transform_test.py");

    let content = r#"
def add(a, b):
    return a + b

def multiply(a, b):
    return a * b
"#;

    fs::write(&python_file, content)?;

    let transformed = py2pyd::transform_file(&python_file, 2)?;

    assert_eq!(transformed.module_name, "transform_test");
    assert!(!transformed.rust_code.is_empty());
    assert!(!transformed.cargo_toml.is_empty());
    assert!(transformed.rust_code.contains("fn add"));
    assert!(transformed.rust_code.contains("fn multiply"));

    Ok(())
}

/// Test build tools detection (doesn't require actual tools)
#[test]
fn test_detect_build_tools() {
    let tools = py2pyd::detect_build_tools();

    // Just verify it doesn't panic and returns a valid struct
    let info = tools.get_tools_info();
    assert!(!info.is_empty());

    // Check boolean methods work
    let _ = tools.has_msvc();
    let _ = tools.has_mingw();
    let _ = tools.has_gcc();
    let _ = tools.has_xcode();
    let _ = tools.has_any_tools();
}

/// Test BuildTools info formatting
#[test]
fn test_build_tools_info() {
    let tools = py2pyd::detect_build_tools();
    let info = tools.get_tools_info();

    // Should return some string (either tool info or "No build tools found")
    assert!(!info.is_empty());
}

/// Test parsing invalid Python source
#[test]
fn test_parse_invalid_source() {
    let invalid_source = r#"
def broken(
    # Missing closing parenthesis
"#;

    let result = py2pyd::parse_source(invalid_source);
    assert!(result.is_err());
}

/// Test parsing empty source
#[test]
fn test_parse_empty_source() -> Result<()> {
    let empty_source = "";
    let ast = py2pyd::parse_source(empty_source)?;
    assert!(ast.is_empty());
    Ok(())
}

/// Test parsing source with only comments
#[test]
fn test_parse_comments_only() -> Result<()> {
    let source = r#"
# This is a comment
# Another comment
"""
This is a docstring
"""
"#;

    let ast = py2pyd::parse_source(source)?;
    // Should parse successfully even with only comments
    let functions = py2pyd::extract_functions(&ast);
    assert!(functions.is_empty());

    Ok(())
}

/// Test complex Python module parsing
#[test]
fn test_parse_complex_module() -> Result<()> {
    let source = r#"
"""Complex module for testing."""

import os
import sys
from typing import Optional, List, Dict
from dataclasses import dataclass

VERSION = "1.0.0"
DEBUG = False

@dataclass
class Config:
    name: str
    value: int

class BaseProcessor:
    """Base class for processors."""
    
    def __init__(self):
        self.initialized = True
    
    def process(self, data):
        raise NotImplementedError

class DataProcessor(BaseProcessor):
    """Concrete processor implementation."""
    
    def __init__(self, config: Config):
        super().__init__()
        self.config = config
    
    def process(self, data):
        return data * self.config.value

def create_processor(name: str, value: int = 1) -> DataProcessor:
    """Factory function for creating processors."""
    config = Config(name=name, value=value)
    return DataProcessor(config)

async def async_operation():
    """An async function."""
    pass

def _private_function():
    """A private function."""
    pass
"#;

    let ast = py2pyd::parse_source(source)?;
    assert!(!ast.is_empty());

    let functions = py2pyd::extract_functions(&ast);
    assert_eq!(functions.len(), 3); // create_processor, async_operation, _private_function

    let classes = py2pyd::extract_classes(&ast);
    assert_eq!(classes.len(), 3); // Config, BaseProcessor, DataProcessor

    let imports = py2pyd::extract_imports(&ast);
    assert_eq!(imports.len(), 2); // os, sys

    let from_imports = py2pyd::extract_from_imports(&ast);
    assert_eq!(from_imports.len(), 2); // typing, dataclasses

    Ok(())
}
