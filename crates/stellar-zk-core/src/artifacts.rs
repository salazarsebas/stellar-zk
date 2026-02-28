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
    let json =
        serde_json::to_string_pretty(artifacts).map_err(|e| StellarZkError::ConfigParse {
            path: path.clone(),
            source: e,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_artifacts_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let artifacts = BuildArtifacts {
            circuit_artifact: PathBuf::from("target/main.r1cs"),
            verifier_wasm: PathBuf::from("target/verifier.wasm"),
            proving_key: Some(PathBuf::from("target/circuit.zkey")),
            verification_key: PathBuf::from("target/verification.key"),
        };
        save(&artifacts, dir.path()).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded.circuit_artifact, artifacts.circuit_artifact);
        assert_eq!(loaded.verifier_wasm, artifacts.verifier_wasm);
        assert_eq!(loaded.proving_key, artifacts.proving_key);
        assert_eq!(loaded.verification_key, artifacts.verification_key);
    }

    #[test]
    fn test_artifacts_roundtrip_no_proving_key() {
        let dir = tempfile::tempdir().unwrap();
        let artifacts = BuildArtifacts {
            circuit_artifact: PathBuf::from("target/circuits.json"),
            verifier_wasm: PathBuf::from("target/verifier.wasm"),
            proving_key: None,
            verification_key: PathBuf::from("target/vk"),
        };
        save(&artifacts, dir.path()).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert!(loaded.proving_key.is_none());
    }

    #[test]
    fn test_artifacts_load_nonexistent() {
        let result = load(Path::new("/tmp/nonexistent_stellar_zk_artifacts"));
        assert!(result.is_err());
    }
}
