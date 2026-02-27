//! Project directory creation and config I/O.
//!
//! Provides helpers for the `init` command to scaffold a new project directory,
//! and for other commands to load an existing project's configuration.
//!
//! ## Directory layout
//!
//! All backends share a common base structure:
//! ```text
//! <project>/
//! ├── stellar-zk.config.json    # ProjectConfig
//! ├── backend.config.json       # BackendConfig
//! ├── inputs/                   # Proof input files
//! ├── proofs/                   # Generated proofs (after prove)
//! └── contracts/verifier/src/   # Soroban verifier contract
//! ```
//!
//! Backend-specific directories are added on top:
//! - **Groth16**: `circuits/`
//! - **UltraHonk**: `circuits/src/`
//! - **RISC Zero**: `programs/host/src/`, `programs/guest/src/`

use std::path::Path;

use crate::config::{BackendConfig, ProjectConfig};
use crate::error::{Result, StellarZkError};

/// Create the base project directory structure shared by all backends.
pub fn create_project_dirs(project_dir: &Path, backend: &str) -> Result<()> {
    if project_dir.exists() {
        return Err(StellarZkError::ProjectExists(project_dir.to_path_buf()));
    }

    std::fs::create_dir_all(project_dir)?;

    // Common directories
    std::fs::create_dir_all(project_dir.join("inputs"))?;
    std::fs::create_dir_all(project_dir.join("proofs"))?;
    std::fs::create_dir_all(project_dir.join("contracts/verifier/src"))?;

    // Backend-specific directories
    match backend {
        "groth16" => {
            std::fs::create_dir_all(project_dir.join("circuits"))?;
        }
        "ultrahonk" => {
            std::fs::create_dir_all(project_dir.join("circuits/src"))?;
        }
        "risc0" => {
            std::fs::create_dir_all(project_dir.join("programs/host/src"))?;
            std::fs::create_dir_all(project_dir.join("programs/guest/src"))?;
        }
        _ => {}
    }

    Ok(())
}

/// Write the config files to the project directory.
pub fn write_configs(project_dir: &Path, project: &ProjectConfig, backend: &BackendConfig) -> Result<()> {
    project.save(&project_dir.join("stellar-zk.config.json"))?;
    backend.save(&project_dir.join("backend.config.json"))?;
    Ok(())
}

/// Load configs from an existing project directory.
pub fn load_project(project_dir: &Path) -> Result<(ProjectConfig, BackendConfig)> {
    let config_path = project_dir.join("stellar-zk.config.json");
    if !config_path.exists() {
        return Err(StellarZkError::NotAProject);
    }

    let project = ProjectConfig::load(&config_path)?;
    let backend = BackendConfig::load(&project_dir.join("backend.config.json"))?;
    Ok((project, backend))
}
