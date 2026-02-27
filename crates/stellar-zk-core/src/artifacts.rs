//! Persistence for build artifacts between CLI commands.
//!
//! Saves [`BuildArtifacts`] to `target/build_artifacts.json` after `build`,
//! and loads them in `prove`, `deploy`, `call`, and `estimate`.

use std::path::Path;

use crate::backend::BuildArtifacts;
use crate::error::{Result, StellarZkError};

const ARTIFACTS_FILE: &str = "build_artifacts.json";

/// Save build artifacts to `<target_dir>/build_artifacts.json`.
pub fn save(artifacts: &BuildArtifacts, target_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(target_dir)?;
    let path = target_dir.join(ARTIFACTS_FILE);
    let json = serde_json::to_string_pretty(artifacts).map_err(|e| {
        StellarZkError::ConfigParse {
            path: path.clone(),
            source: e,
        }
    })?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Load build artifacts from `<target_dir>/build_artifacts.json`.
pub fn load(target_dir: &Path) -> Result<BuildArtifacts> {
    let path = target_dir.join(ARTIFACTS_FILE);
    let contents = std::fs::read_to_string(&path).map_err(|e| StellarZkError::ConfigNotFound {
        path: path.clone(),
        source: e,
    })?;
    let artifacts: BuildArtifacts =
        serde_json::from_str(&contents).map_err(|e| StellarZkError::ConfigParse {
            path: path.clone(),
            source: e,
        })?;
    Ok(artifacts)
}
