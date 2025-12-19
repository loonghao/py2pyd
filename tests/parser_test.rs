//! Unit tests for the parser module
//!
//! These tests verify Python source code parsing functionality.

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod parser_tests {
    use super::*;

    /// Test parsing a simple function
    #[test]
    fn test_parse_simple_function() -> Result<()> {
        let source = r#"
def add(a, b):
    return a + b
"#;

        let ast = py2pyd::parse_source(source)?;
        let functions = py2pyd::extract_functions(&ast);

        assert_eq!(functions.len(), 1);
        Ok(())
    }

    /// Test parsing multiple functions
    #[test]
    fn test_parse_multiple_functions() -> Result<()> {
        let source = r#"
def func1():
    pass

def func2():
    pass

def func3():
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let functions = py2pyd::extract_functions(&ast);

        assert_eq!(functions.len(), 3);
        Ok(())
    }

    /// Test parsing async functions
    #[test]
    fn test_parse_async_function() -> Result<()> {
        let source = r#"
async def async_fetch():
    return await some_operation()

def sync_function():
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        // Note: async functions are also FunctionDef in rustpython-parser
        let functions = py2pyd::extract_functions(&ast);

        // Should find both sync and async functions
        assert!(!functions.is_empty());
        Ok(())
    }

    /// Test parsing decorated functions
    #[test]
    fn test_parse_decorated_function() -> Result<()> {
        let source = r#"
@decorator
def decorated():
    pass

@decorator1
@decorator2
def multi_decorated():
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let functions = py2pyd::extract_functions(&ast);

        assert_eq!(functions.len(), 2);
        Ok(())
    }

    /// Test parsing a simple class
    #[test]
    fn test_parse_simple_class() -> Result<()> {
        let source = r#"
class MyClass:
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let classes = py2pyd::extract_classes(&ast);

        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing class with methods
    #[test]
    fn test_parse_class_with_methods() -> Result<()> {
        let source = r#"
class Calculator:
    def __init__(self):
        self.result = 0
    
    def add(self, value):
        self.result += value
        return self
    
    def subtract(self, value):
        self.result -= value
        return self
    
    def get_result(self):
        return self.result
"#;

        let ast = py2pyd::parse_source(source)?;
        let classes = py2pyd::extract_classes(&ast);

        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing class with inheritance
    #[test]
    fn test_parse_class_inheritance() -> Result<()> {
        let source = r#"
class Base:
    pass

class Derived(Base):
    pass

class MultiInherit(Base, object):
    pass
"#;

        let ast = py2pyd::parse_source(source)?;
        let classes = py2pyd::extract_classes(&ast);

        assert_eq!(classes.len(), 3);
        Ok(())
    }

    /// Test parsing imports
    #[test]
    fn test_parse_imports() -> Result<()> {
        let source = r#"
import os
import sys
import json
"#;

        let ast = py2pyd::parse_source(source)?;
        let imports = py2pyd::extract_imports(&ast);

        assert_eq!(imports.len(), 3);
        Ok(())
    }

    /// Test parsing from imports
    #[test]
    fn test_parse_from_imports() -> Result<()> {
        let source = r#"
from os import path
from sys import argv, exit
from typing import List, Dict, Optional
"#;

        let ast = py2pyd::parse_source(source)?;
        let from_imports = py2pyd::extract_from_imports(&ast);

        assert_eq!(from_imports.len(), 3);
        Ok(())
    }

    /// Test parsing relative imports
    #[test]
    fn test_parse_relative_imports() -> Result<()> {
        let source = r#"
from . import module
from .. import parent_module
from .sibling import something
"#;

        let ast = py2pyd::parse_source(source)?;
        let from_imports = py2pyd::extract_from_imports(&ast);

        assert_eq!(from_imports.len(), 3);
        Ok(())
    }

    /// Test parsing module variables
    #[test]
    fn test_parse_module_vars() -> Result<()> {
        let source = r#"
VERSION = "1.0.0"
DEBUG = True
CONFIG = {}
"#;

        let ast = py2pyd::parse_source(source)?;
        let vars = py2pyd::extract_module_vars(&ast);

        assert_eq!(vars.len(), 3);
        Ok(())
    }

    /// Test parsing complex assignments
    #[test]
    fn test_parse_complex_assignments() -> Result<()> {
        let source = r#"
a = 1
b = c = 2
x, y = 1, 2
data = {"key": "value"}
items = [1, 2, 3]
"#;

        let ast = py2pyd::parse_source(source)?;
        let vars = py2pyd::extract_module_vars(&ast);

        // Should capture all assignment statements
        assert!(vars.len() >= 4);
        Ok(())
    }

    /// Test parsing file from disk
    #[test]
    fn test_parse_file_from_disk() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.py");

        let content = r#"
"""Module docstring."""

import os

VERSION = "1.0"

def main():
    pass

class App:
    pass
"#;

        fs::write(&file_path, content)?;

        let ast = py2pyd::parse_file(&file_path)?;

        let functions = py2pyd::extract_functions(&ast);
        let classes = py2pyd::extract_classes(&ast);
        let imports = py2pyd::extract_imports(&ast);

        assert_eq!(functions.len(), 1);
        assert_eq!(classes.len(), 1);
        assert_eq!(imports.len(), 1);

        Ok(())
    }

    /// Test parsing empty file
    #[test]
    fn test_parse_empty_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.py");

        fs::write(&file_path, "")?;

        let ast = py2pyd::parse_file(&file_path)?;

        assert!(ast.is_empty());
        Ok(())
    }

    /// Test parsing file with only docstring
    #[test]
    fn test_parse_docstring_only() -> Result<()> {
        let source = r#"
"""
This is a module docstring.
It spans multiple lines.
"""
"#;

        let ast = py2pyd::parse_source(source)?;

        let functions = py2pyd::extract_functions(&ast);
        let classes = py2pyd::extract_classes(&ast);

        assert!(functions.is_empty());
        assert!(classes.is_empty());
        Ok(())
    }

    /// Test parsing syntax error
    #[test]
    fn test_parse_syntax_error() {
        let invalid_source = r#"
def broken(
    # Missing closing paren
"#;

        let result = py2pyd::parse_source(invalid_source);
        assert!(result.is_err());
    }

    /// Test parsing indentation error
    #[test]
    fn test_parse_indentation_error() {
        let invalid_source = r#"
def func():
pass  # Wrong indentation
"#;

        let result = py2pyd::parse_source(invalid_source);
        assert!(result.is_err());
    }

    /// Test parsing lambda expressions (not extracted as functions)
    #[test]
    fn test_parse_lambda() -> Result<()> {
        let source = r#"
add = lambda a, b: a + b
square = lambda x: x ** 2
"#;

        let ast = py2pyd::parse_source(source)?;

        // Lambdas are assigned to variables, not function definitions
        let functions = py2pyd::extract_functions(&ast);
        let vars = py2pyd::extract_module_vars(&ast);

        assert!(functions.is_empty());
        assert_eq!(vars.len(), 2);
        Ok(())
    }

    /// Test parsing nested functions
    #[test]
    fn test_parse_nested_functions() -> Result<()> {
        let source = r#"
def outer():
    def inner():
        pass
    return inner
"#;

        let ast = py2pyd::parse_source(source)?;

        // Only top-level functions are extracted
        let functions = py2pyd::extract_functions(&ast);
        assert_eq!(functions.len(), 1);
        Ok(())
    }

    /// Test parsing nested classes
    #[test]
    fn test_parse_nested_classes() -> Result<()> {
        let source = r#"
class Outer:
    class Inner:
        pass
"#;

        let ast = py2pyd::parse_source(source)?;

        // Only top-level classes are extracted
        let classes = py2pyd::extract_classes(&ast);
        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing type hints
    #[test]
    fn test_parse_type_hints() -> Result<()> {
        let source = r#"
from typing import List, Optional

def process(items: List[int]) -> Optional[int]:
    if items:
        return sum(items)
    return None

class Container:
    items: List[str]
    
    def __init__(self, items: List[str]) -> None:
        self.items = items
"#;

        let ast = py2pyd::parse_source(source)?;

        let functions = py2pyd::extract_functions(&ast);
        let classes = py2pyd::extract_classes(&ast);

        assert_eq!(functions.len(), 1);
        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing dataclass
    #[test]
    fn test_parse_dataclass() -> Result<()> {
        let source = r#"
from dataclasses import dataclass

@dataclass
class Point:
    x: float
    y: float
    
    def distance(self) -> float:
        return (self.x ** 2 + self.y ** 2) ** 0.5
"#;

        let ast = py2pyd::parse_source(source)?;

        let classes = py2pyd::extract_classes(&ast);
        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing context managers
    #[test]
    fn test_parse_context_manager() -> Result<()> {
        let source = r#"
class FileHandler:
    def __init__(self, filename):
        self.filename = filename
    
    def __enter__(self):
        self.file = open(self.filename)
        return self.file
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.file.close()
        return False
"#;

        let ast = py2pyd::parse_source(source)?;

        let classes = py2pyd::extract_classes(&ast);
        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing generator function
    #[test]
    fn test_parse_generator() -> Result<()> {
        let source = r#"
def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i += 1
"#;

        let ast = py2pyd::parse_source(source)?;

        let functions = py2pyd::extract_functions(&ast);
        assert_eq!(functions.len(), 1);
        Ok(())
    }

    /// Test parsing property decorator
    #[test]
    fn test_parse_property() -> Result<()> {
        let source = r#"
class Circle:
    def __init__(self, radius):
        self._radius = radius
    
    @property
    def radius(self):
        return self._radius
    
    @radius.setter
    def radius(self, value):
        self._radius = value
    
    @property
    def area(self):
        return 3.14159 * self._radius ** 2
"#;

        let ast = py2pyd::parse_source(source)?;

        let classes = py2pyd::extract_classes(&ast);
        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing static and class methods
    #[test]
    fn test_parse_static_class_methods() -> Result<()> {
        let source = r#"
class Utility:
    @staticmethod
    def static_method():
        return "static"
    
    @classmethod
    def class_method(cls):
        return cls.__name__
    
    def instance_method(self):
        return "instance"
"#;

        let ast = py2pyd::parse_source(source)?;

        let classes = py2pyd::extract_classes(&ast);
        assert_eq!(classes.len(), 1);
        Ok(())
    }

    /// Test parsing walrus operator (Python 3.8+)
    #[test]
    fn test_parse_walrus_operator() -> Result<()> {
        let source = r#"
def process(data):
    if (n := len(data)) > 10:
        return n
    return 0
"#;

        let ast = py2pyd::parse_source(source)?;

        let functions = py2pyd::extract_functions(&ast);
        assert_eq!(functions.len(), 1);
        Ok(())
    }

    /// Test parsing match statement (Python 3.10+)
    #[test]
    fn test_parse_match_statement() -> Result<()> {
        let source = r#"
def handle_command(command):
    match command:
        case "start":
            return "Starting..."
        case "stop":
            return "Stopping..."
        case _:
            return "Unknown command"
"#;

        let ast = py2pyd::parse_source(source)?;

        let functions = py2pyd::extract_functions(&ast);
        assert_eq!(functions.len(), 1);
        Ok(())
    }
}
