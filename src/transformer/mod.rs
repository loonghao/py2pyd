use anyhow::{Context, Result};
use log::{debug, info};
use rustpython_parser::ast;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// Represents a transformed Python module
pub struct TransformedModule {
    pub module_name: String,
    pub rust_code: String,
    pub build_script: String,
    pub cargo_toml: String,
    pub build_dir: PathBuf,
}

/// Transform a Python AST into Rust code using `PyO3`
pub fn transform_ast(ast: &ast::Suite, module_name: &str, optimize_level: u8) -> String {
    info!("Transforming Python AST to Rust code");
    debug!("Module name: {module_name}, Optimization level: {optimize_level}");

    // This is a simplified implementation
    // In a real implementation, we would analyze the AST and generate
    // appropriate Rust code with PyO3 bindings

    let mut rust_code = String::new();

    // Add standard imports
    rust_code.push_str("use pyo3::prelude::*;\n");
    rust_code.push_str("use pyo3::wrap_pyfunction;\n\n");

    // Generate module
    writeln!(rust_code, "#[pymodule]\nfn {module_name}(_py: Python, m: &PyModule) -> PyResult<()> {{").unwrap();

    // Transform functions
    for func in crate::parser::extract_functions(ast) {
        if let ast::Stmt::FunctionDef(func_def) = func {
            writeln!(rust_code, "    m.add_function(wrap_pyfunction!({}, m)?)?;", func_def.name).unwrap();
        }
    }

    // Transform classes
    for class in crate::parser::extract_classes(ast) {
        if let ast::Stmt::ClassDef(class_def) = class {
            writeln!(rust_code, "    m.add_class::<{}>()?;", class_def.name).unwrap();
        }
    }

    rust_code.push_str("    Ok(())\n");
    rust_code.push_str("}\n\n");

    // Generate function implementations
    for func in crate::parser::extract_functions(ast) {
        if let ast::Stmt::FunctionDef(func_def) = func {
            writeln!(rust_code, "#[pyfunction]\nfn {}(py: Python) -> PyResult<PyObject> {{", func_def.name).unwrap();
            rust_code.push_str("    // Auto-generated function implementation\n");
            rust_code.push_str("    Ok(py.None())\n");
            rust_code.push_str("}\n\n");
        }
    }

    // Generate class implementations
    for class in crate::parser::extract_classes(ast) {
        if let ast::Stmt::ClassDef(class_def) = class {
            writeln!(rust_code, "#[pyclass]\nstruct {} {{", class_def.name).unwrap();
            rust_code.push_str("    // Auto-generated class implementation\n");
            rust_code.push_str("}\n\n");

            writeln!(rust_code, "#[pymethods]\nimpl {} {{", class_def.name).unwrap();
            rust_code.push_str("    #[new]\n");
            rust_code.push_str("    fn new() -> Self {\n");
            writeln!(rust_code, "        {}{{ }}", class_def.name).unwrap();
            rust_code.push_str("    }\n");
            rust_code.push_str("}\n\n");
        }
    }

    debug!("Generated {} lines of Rust code", rust_code.lines().count());
    rust_code
}

/// Generate a Cargo.toml file for the transformed module
pub fn generate_cargo_toml(module_name: &str, optimize_level: u8) -> String {
    let mut cargo_toml = String::new();

    cargo_toml.push_str("[package]\n");
    writeln!(cargo_toml, "name = \"{module_name}\"").unwrap();
    cargo_toml.push_str("version = \"0.1.0\"\n");
    cargo_toml.push_str("edition = \"2021\"\n\n");

    cargo_toml.push_str("[lib]\n");
    writeln!(cargo_toml, "name = \"{module_name}\"").unwrap();
    cargo_toml.push_str("crate-type = [\"cdylib\"]\n\n");

    // Add maturin configuration
    cargo_toml.push_str("[package.metadata.maturin]\n");
    writeln!(cargo_toml, "name = \"{module_name}\"").unwrap();
    cargo_toml.push_str("binding = \"pyo3\"\n");
    cargo_toml.push_str("strip = true\n\n");

    cargo_toml.push_str("[dependencies]\n");
    cargo_toml.push_str("pyo3 = { version = \"0.19\", features = [\"extension-module\"] }\n");

    // Add optimization flags
    cargo_toml.push_str("\n[profile.release]\n");
    match optimize_level {
        0 => {
            cargo_toml.push_str("opt-level = 0\n");
        }
        1 => {
            cargo_toml.push_str("opt-level = 1\n");
        }
        2 => {
            cargo_toml.push_str("opt-level = 2\n");
        }
        _ => {
            cargo_toml.push_str("opt-level = 3\n");
            cargo_toml.push_str("lto = true\n");
            cargo_toml.push_str("codegen-units = 1\n");
        }
    }

    cargo_toml
}

/// Transform a Python file into a Rust project
pub fn transform_file(input_path: &Path, optimize_level: u8) -> Result<TransformedModule> {
    info!("Transforming Python file: {}", input_path.display());

    // Parse the Python file
    let ast = crate::parser::parse_file(input_path)
        .with_context(|| format!("Failed to parse Python file: {}", input_path.display()))?;

    // Get the module name from the file name
    let module_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

    // Transform the AST to Rust code
    let rust_code = transform_ast(&ast, module_name, optimize_level);

    // Generate Cargo.toml
    let cargo_toml = generate_cargo_toml(module_name, optimize_level);

    // Create a temporary directory for the build
    let temp_dir = tempfile::tempdir().with_context(|| "Failed to create temporary directory")?;
    let build_dir = temp_dir.path().to_path_buf();
    // We'll let the temp_dir be dropped, which will clean up the directory
    // In a real implementation, we might want to keep it for debugging

    // Create the build script
    let build_script = "cargo build --release".to_string();

    Ok(TransformedModule {
        module_name: module_name.to_string(),
        rust_code,
        build_script,
        cargo_toml,
        build_dir,
    })
}
