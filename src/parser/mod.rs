use anyhow::{Context, Result};
use log::{debug, info};
use rustpython_parser::{ast, parser};
use std::fs;
use std::path::Path;

/// Parse a Python file into an AST
pub fn parse_file(path: &Path) -> Result<ast::Suite> {
    info!("Parsing Python file: {}", path.display());

    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read Python file: {}", path.display()))?;

    parse_source(&source)
        .with_context(|| format!("Failed to parse Python file: {}", path.display()))
}

/// Parse Python source code into an AST
pub fn parse_source(source: &str) -> Result<ast::Suite> {
    debug!("Parsing Python source code");

    // Use a dummy file name for the source path
    let ast = parser::parse_program(source, "<string>")
        .map_err(|e| anyhow::anyhow!("Python parsing error: {}", e))?;

    debug!("Successfully parsed Python source code");
    Ok(ast)
}

/// Extract function definitions from an AST
pub fn extract_functions(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    let mut functions = Vec::new();

    for stmt in ast {
        match &stmt.node {
            ast::StmtKind::FunctionDef { .. } => {
                functions.push(stmt);
            },
            _ => {}
        }
    }

    debug!("Extracted {} functions", functions.len());
    functions
}

/// Extract class definitions from an AST
pub fn extract_classes(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    let mut classes = Vec::new();

    for stmt in ast {
        match &stmt.node {
            ast::StmtKind::ClassDef { .. } => {
                classes.push(stmt);
            },
            _ => {}
        }
    }

    debug!("Extracted {} classes", classes.len());
    classes
}

/// Extract imports from an AST
pub fn extract_imports(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    let mut imports = Vec::new();

    for stmt in ast {
        match &stmt.node {
            ast::StmtKind::Import { .. } => {
                imports.push(stmt);
            },
            _ => {}
        }
    }

    debug!("Extracted {} imports", imports.len());
    imports
}

/// Extract from imports from an AST
pub fn extract_from_imports(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    let mut imports = Vec::new();

    for stmt in ast {
        match &stmt.node {
            ast::StmtKind::ImportFrom { .. } => {
                imports.push(stmt);
            },
            _ => {}
        }
    }

    debug!("Extracted {} from imports", imports.len());
    imports
}

/// Extract module-level variables from an AST
pub fn extract_module_vars(ast: &ast::Suite) -> Vec<&ast::Stmt> {
    let mut vars = Vec::new();

    for stmt in ast {
        match &stmt.node {
            ast::StmtKind::Assign { .. } => {
                vars.push(stmt);
            },
            _ => {}
        }
    }

    debug!("Extracted {} module variables", vars.len());
    vars
}
