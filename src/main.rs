use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{error, info, warn};
use std::path::{Path, PathBuf};

mod compiler;
mod parser;
mod transformer;
mod dcc;
mod python_env;
mod uv_env;
mod uv_compiler;

/// A tool to compile Python modules to pyd files
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Path to Python interpreter (optional)
    #[arg(long)]
    python_path: Option<String>,

    /// Python version to use (e.g., "3.9", "3.10") (optional)
    #[arg(long)]
    python_version: Option<String>,

    /// Keep temporary files after compilation (default: false)
    #[arg(long)]
    keep_temp: bool,

    /// Use uv for Python environment management (default: true)
    #[arg(long, default_value = "true")]
    use_uv: bool,

    /// Additional Python packages to install (comma-separated)
    #[arg(long)]
    packages: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a single Python file to a pyd file
    Compile {
        /// Input Python file
        #[arg(short, long)]
        input: PathBuf,

        /// Output pyd file (default: same as input with .pyd extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target DCC environment (maya2022, maya2023, houdini19, houdini20, generic)
        #[arg(short, long, default_value = "generic")]
        target: String,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "2")]
        optimize: u8,
    },
    /// Batch compile multiple Python files to pyd files
    Batch {
        /// Input directory or glob pattern
        #[arg(short, long)]
        input: String,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Target DCC environment (maya2022, maya2023, houdini19, houdini20, generic)
        #[arg(short, long, default_value = "generic")]
        target: String,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "2")]
        optimize: u8,

        /// Recursive search
        #[arg(short, long)]
        recursive: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    let env = match cli.verbose {
        0 => Env::default().default_filter_or("warn"),
        1 => Env::default().default_filter_or("info"),
        2 => Env::default().default_filter_or("debug"),
        _ => Env::default().default_filter_or("trace"),
    };
    env_logger::init_from_env(env);

    // Execute command
    match &cli.command {
        Commands::Compile {
            input,
            output,
            target,
            optimize,
        } => {
            let output = output.clone().unwrap_or_else(|| {
                let mut output_path = input.clone();
                output_path.set_extension("pyd");
                output_path
            });

            info!("Compiling {} to {}", input.display(), output.display());
            info!("Target: {}, Optimization level: {}", target, optimize);

            // Parse additional packages
            let packages = cli.packages.as_ref()
                .map(|p| p.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
                .unwrap_or_default();

            if cli.use_uv {
                // Use the uv-based compiler
                let config = uv_compiler::CompileConfig {
                    python_path: cli.python_path.as_deref().map(PathBuf::from),
                    python_version: cli.python_version.clone(),
                    optimize_level: *optimize,
                    keep_temp_files: cli.keep_temp,
                    target_dcc: Some(target.clone()),
                    packages,
                };

                uv_compiler::compile_file(input, &output, &config)
                    .with_context(|| format!("Failed to compile {}", input.display()))?;
            } else {
                // Use the old compiler
                // Initialize Python environment
                info!("Initializing Python environment...");
                python_env::initialize_python_env(cli.python_path.as_deref(), cli.python_version.as_deref())
                    .with_context(|| "Failed to initialize Python environment")?;

                // Set Python environment variables
                python_env::set_python_env_vars()
                    .with_context(|| "Failed to set Python environment variables")?;

                // Display Python path
                let python_path = python_env::get_python_path()
                    .with_context(|| "Failed to get Python path")?;
                info!("Using Python interpreter: {}", python_path.display());

                compile_file(input, &output, target, *optimize)
                    .with_context(|| format!("Failed to compile {}", input.display()))?;

                // Clean up virtual environment if not keeping it
                if !cli.keep_temp {
                    info!("Cleaning up temporary virtual environment...");
                    if let Err(e) = python_env::cleanup_venv() {
                        warn!("Failed to clean up virtual environment: {}", e);
                    } else {
                        info!("Virtual environment cleaned up successfully");
                    }
                } else {
                    let venv_path = python_env::get_venv_path()?;
                    info!("Keeping virtual environment at: {}", venv_path.display());
                    info!("You can activate it with: {}\\Scripts\\activate", venv_path.display());
                }
            }

            info!("Successfully compiled to {}", output.display());
        }
        Commands::Batch {
            input,
            output,
            target,
            optimize,
            recursive,
        } => {
            info!("Batch compiling from {} to {}", input, output.display());
            info!("Target: {}, Optimization level: {}", target, optimize);

            // Parse additional packages
            let packages = cli.packages.as_ref()
                .map(|p| p.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
                .unwrap_or_default();

            if cli.use_uv {
                // Use the uv-based compiler
                let config = uv_compiler::CompileConfig {
                    python_path: cli.python_path.as_deref().map(PathBuf::from),
                    python_version: cli.python_version.clone(),
                    optimize_level: *optimize,
                    keep_temp_files: cli.keep_temp,
                    target_dcc: Some(target.clone()),
                    packages,
                };

                uv_compiler::batch_compile(input, output, &config, *recursive)
                    .with_context(|| "Failed to batch compile")?;
            } else {
                // Use the old compiler
                // Initialize Python environment
                info!("Initializing Python environment...");
                python_env::initialize_python_env(cli.python_path.as_deref(), cli.python_version.as_deref())
                    .with_context(|| "Failed to initialize Python environment")?;

                // Set Python environment variables
                python_env::set_python_env_vars()
                    .with_context(|| "Failed to set Python environment variables")?;

                // Display Python path
                let python_path = python_env::get_python_path()
                    .with_context(|| "Failed to get Python path")?;
                info!("Using Python interpreter: {}", python_path.display());

                batch_compile(input, output, target, *optimize, *recursive)
                    .with_context(|| "Failed to batch compile")?;

                // Clean up virtual environment if not keeping it
                if !cli.keep_temp {
                    info!("Cleaning up temporary virtual environment...");
                    if let Err(e) = python_env::cleanup_venv() {
                        warn!("Failed to clean up virtual environment: {}", e);
                    } else {
                        info!("Virtual environment cleaned up successfully");
                    }
                } else {
                    let venv_path = python_env::get_venv_path()?;
                    info!("Keeping virtual environment at: {}", venv_path.display());
                    info!("You can activate it with: {}\\Scripts\\activate", venv_path.display());
                }
            }

            info!("Successfully batch compiled");
        }
    }

    Ok(())
}

fn compile_file(input: &Path, output: &Path, target: &str, optimize: u8) -> Result<()> {
    // This will be implemented in the compiler module
    compiler::compile_file(input, output, target, optimize)
}

fn batch_compile(
    input_pattern: &str,
    output_dir: &Path,
    target: &str,
    optimize: u8,
    recursive: bool,
) -> Result<()> {
    // This will be implemented in the compiler module
    compiler::batch_compile(input_pattern, output_dir, target, optimize, recursive)
}
