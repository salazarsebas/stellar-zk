use std::path::Path;
use std::process::Command;

use stellar_zk_core::error::{Result, StellarZkError};

/// Compile a Noir circuit using nargo.
pub fn compile(project_dir: &Path) -> Result<()> {
    let output = Command::new("nargo")
        .arg("compile")
        .current_dir(project_dir.join("circuits"))
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(StellarZkError::CircuitCompilation(
            String::from_utf8_lossy(&out.stderr).to_string(),
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "nargo".into(),
            install: "noirup".into(),
        }),
        Err(e) => Err(StellarZkError::CircuitCompilation(e.to_string())),
    }
}

/// Execute a Noir circuit to generate a witness.
pub fn execute(project_dir: &Path) -> Result<()> {
    let output = Command::new("nargo")
        .arg("execute")
        .current_dir(project_dir.join("circuits"))
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(StellarZkError::ProofGeneration(
            String::from_utf8_lossy(&out.stderr).to_string(),
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "nargo".into(),
            install: "noirup".into(),
        }),
        Err(e) => Err(StellarZkError::ProofGeneration(e.to_string())),
    }
}

/// Generate a verification key using bb.
pub fn write_vk(acir_path: &Path, output_path: &Path, oracle_hash: &str) -> Result<()> {
    let output = Command::new("bb")
        .arg("write_vk")
        .arg("--oracle_hash")
        .arg(oracle_hash)
        .arg("-b")
        .arg(acir_path)
        .arg("-o")
        .arg(output_path)
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(StellarZkError::CircuitCompilation(
            String::from_utf8_lossy(&out.stderr).to_string(),
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "bb".into(),
            install: "bbup".into(),
        }),
        Err(e) => Err(StellarZkError::CircuitCompilation(e.to_string())),
    }
}

/// Verify an UltraHonk proof using bb (off-chain).
pub fn verify_ultrahonk(proof_path: &Path, vk_path: &Path, oracle_hash: &str) -> Result<()> {
    let output = Command::new("bb")
        .arg("verify_ultra_honk")
        .arg("--oracle_hash")
        .arg(oracle_hash)
        .arg("-p")
        .arg(proof_path)
        .arg("-k")
        .arg(vk_path)
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(StellarZkError::ProofGeneration(format!(
            "proof verification failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "bb".into(),
            install: "bbup".into(),
        }),
        Err(e) => Err(StellarZkError::ProofGeneration(e.to_string())),
    }
}

/// Generate an UltraHonk proof using bb.
pub fn prove_ultrahonk(
    acir_path: &Path,
    witness_path: &Path,
    output_path: &Path,
    oracle_hash: &str,
) -> Result<()> {
    let output = Command::new("bb")
        .arg("prove_ultra_honk")
        .arg("--oracle_hash")
        .arg(oracle_hash)
        .arg("-b")
        .arg(acir_path)
        .arg("-w")
        .arg(witness_path)
        .arg("-o")
        .arg(output_path)
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(StellarZkError::ProofGeneration(
            String::from_utf8_lossy(&out.stderr).to_string(),
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "bb".into(),
            install: "bbup".into(),
        }),
        Err(e) => Err(StellarZkError::ProofGeneration(e.to_string())),
    }
}
