use anyhow::{anyhow, Result};
use log::debug;
use std::env;
use std::path::PathBuf;

/// Supported DCC environments
#[derive(Debug, Clone, PartialEq)]
pub enum DCCEnvironment {
    Maya2022,
    Maya2023,
    Houdini19,
    Houdini20,
    Generic,
}

impl DCCEnvironment {
    /// Parse a string into a DCCEnvironment
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "maya2022" => Ok(DCCEnvironment::Maya2022),
            "maya2023" => Ok(DCCEnvironment::Maya2023),
            "houdini19" => Ok(DCCEnvironment::Houdini19),
            "houdini20" => Ok(DCCEnvironment::Houdini20),
            "generic" => Ok(DCCEnvironment::Generic),
            _ => Err(anyhow!("Unknown DCC environment: {}", s)),
        }
    }
}

/// Configuration for a DCC environment
#[derive(Debug, Clone)]
pub struct DCCConfig {
    pub python_version: (u8, u8),
    pub include_paths: Vec<PathBuf>,
    pub library_paths: Vec<PathBuf>,
    pub required_libs: Vec<String>,
}

/// Detect the current DCC environment
pub fn detect_dcc_environment() -> DCCEnvironment {
    // Check for Maya environment
    if let Ok(maya_location) = env::var("MAYA_LOCATION") {
        debug!("Found MAYA_LOCATION: {}", maya_location);
        if maya_location.contains("2022") {
            return DCCEnvironment::Maya2022;
        } else if maya_location.contains("2023") {
            return DCCEnvironment::Maya2023;
        }
    }
    
    // Check for Houdini environment
    if let Ok(hfs) = env::var("HFS") {
        debug!("Found HFS: {}", hfs);
        if hfs.contains("19.") {
            return DCCEnvironment::Houdini19;
        } else if hfs.contains("20.") {
            return DCCEnvironment::Houdini20;
        }
    }
    
    DCCEnvironment::Generic
}

/// Get configuration for a DCC environment
pub fn get_dcc_config(env: &DCCEnvironment) -> DCCConfig {
    match env {
        DCCEnvironment::Maya2022 => {
            let maya_location = env::var("MAYA_LOCATION").unwrap_or_default();
            DCCConfig {
                python_version: (3, 7),
                include_paths: vec![
                    PathBuf::from(&maya_location).join("include").join("python3.7m")
                ],
                library_paths: vec![
                    PathBuf::from(&maya_location).join("lib")
                ],
                required_libs: vec!["python37".to_string()],
            }
        },
        DCCEnvironment::Maya2023 => {
            let maya_location = env::var("MAYA_LOCATION").unwrap_or_default();
            DCCConfig {
                python_version: (3, 9),
                include_paths: vec![
                    PathBuf::from(&maya_location).join("include").join("python3.9")
                ],
                library_paths: vec![
                    PathBuf::from(&maya_location).join("lib")
                ],
                required_libs: vec!["python39".to_string()],
            }
        },
        DCCEnvironment::Houdini19 => {
            let hfs = env::var("HFS").unwrap_or_default();
            DCCConfig {
                python_version: (3, 9),
                include_paths: vec![
                    PathBuf::from(&hfs).join("python").join("include").join("python3.9")
                ],
                library_paths: vec![
                    PathBuf::from(&hfs).join("python").join("lib")
                ],
                required_libs: vec!["python3.9".to_string()],
            }
        },
        DCCEnvironment::Houdini20 => {
            let hfs = env::var("HFS").unwrap_or_default();
            DCCConfig {
                python_version: (3, 10),
                include_paths: vec![
                    PathBuf::from(&hfs).join("python").join("include").join("python3.10")
                ],
                library_paths: vec![
                    PathBuf::from(&hfs).join("python").join("lib")
                ],
                required_libs: vec!["python3.10".to_string()],
            }
        },
        DCCEnvironment::Generic => {
            // Try to detect Python from the system
            DCCConfig {
                python_version: (3, 10), // Default to Python 3.10
                include_paths: vec![],
                library_paths: vec![],
                required_libs: vec![],
            }
        }
    }
}
