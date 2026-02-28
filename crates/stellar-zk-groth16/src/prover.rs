use std::path::Path;
use std::process::Command;

use stellar_zk_core::error::{Result, StellarZkError};

use crate::serializer;

/// Generate a Powers of Tau file for development use.
///
/// This creates a small ptau file (2^12 = 4096 constraints) suitable for
/// development and testing. For production, use a proper ceremony file.
pub fn generate_dev_ptau(output_path: &Path) -> Result<()> {
    let ptau_new = output_path.with_extension("ptau.tmp");

    // Step 1: new ceremony
    run_snarkjs(&[
        "powersoftau",
        "new",
        "bn128",
        "12",
        &ptau_new.display().to_string(),
    ])?;

    // Step 2: contribute (with random entropy for dev)
    let ptau_contributed = output_path.with_extension("ptau.contributed");
    run_snarkjs(&[
        "powersoftau",
        "contribute",
        &ptau_new.display().to_string(),
        &ptau_contributed.display().to_string(),
        "--name=dev",
        "-e=stellar-zk-dev-entropy",
    ])?;

    // Step 3: prepare phase 2
    run_snarkjs(&[
        "powersoftau",
        "prepare",
        "phase2",
        &ptau_contributed.display().to_string(),
        &output_path.display().to_string(),
    ])?;

    // Cleanup temp files
    let _ = std::fs::remove_file(&ptau_new);
    let _ = std::fs::remove_file(&ptau_contributed);

    Ok(())
}

/// Generate proving and verification keys using snarkjs trusted setup.
///
/// Requires: R1CS file from circom compilation and a Powers of Tau (.ptau) file.
/// Outputs: .zkey file (proving key) and verification_key.json.
pub fn generate_keys(
    r1cs_path: &Path,
    ptau_path: &Path,
    zkey_output: &Path,
    vk_json_output: &Path,
) -> Result<()> {
    // groth16 setup
    run_snarkjs(&[
        "groth16",
        "setup",
        &r1cs_path.display().to_string(),
        &ptau_path.display().to_string(),
        &zkey_output.display().to_string(),
    ])?;

    // Export verification key
    run_snarkjs(&[
        "zkey",
        "export",
        "verificationkey",
        &zkey_output.display().to_string(),
        &vk_json_output.display().to_string(),
    ])?;

    Ok(())
}

/// Generate witness from circuit WASM and input JSON.
///
/// Uses Node.js to run the circom-generated witness calculator.
pub fn generate_witness(wasm_path: &Path, input_path: &Path, witness_output: &Path) -> Result<()> {
    // snarkjs wtns calculate
    run_snarkjs(&[
        "wtns",
        "calculate",
        &wasm_path.display().to_string(),
        &input_path.display().to_string(),
        &witness_output.display().to_string(),
    ])?;
    Ok(())
}

/// Generate a Groth16 proof using snarkjs.
///
/// Returns the serialized proof bytes (256 bytes, Soroban format) and
/// public inputs as 32-byte big-endian field elements.
pub fn generate_proof(
    zkey_path: &Path,
    witness_path: &Path,
    proof_json_output: &Path,
    public_json_output: &Path,
) -> Result<(Vec<u8>, Vec<[u8; 32]>)> {
    // Generate proof
    run_snarkjs(&[
        "groth16",
        "prove",
        &zkey_path.display().to_string(),
        &witness_path.display().to_string(),
        &proof_json_output.display().to_string(),
        &public_json_output.display().to_string(),
    ])?;

    // Parse and serialize proof
    let proof_str = std::fs::read_to_string(proof_json_output)?;
    let proof_json: serde_json::Value = serde_json::from_str(&proof_str)
        .map_err(|e| StellarZkError::ProofGeneration(format!("failed to parse proof.json: {e}")))?;

    let proof_bytes = serializer::serialize_proof_from_snarkjs(&proof_json)
        .map_err(|e| StellarZkError::ProofGeneration(format!("proof serialization: {e}")))?;

    // Parse and serialize public inputs
    let public_str = std::fs::read_to_string(public_json_output)?;
    let public_json: serde_json::Value = serde_json::from_str(&public_str).map_err(|e| {
        StellarZkError::ProofGeneration(format!("failed to parse public.json: {e}"))
    })?;

    let public_inputs = serializer::serialize_public_inputs_from_snarkjs(&public_json)
        .map_err(|e| StellarZkError::ProofGeneration(format!("public input serialization: {e}")))?;

    Ok((proof_bytes, public_inputs))
}

/// Convert a snarkjs verification_key.json to Soroban binary format.
pub fn convert_vk_to_soroban(vk_json_path: &Path, vk_bin_output: &Path) -> Result<()> {
    let vk_str = std::fs::read_to_string(vk_json_path)?;
    let vk_json: serde_json::Value = serde_json::from_str(&vk_str).map_err(|e| {
        StellarZkError::ProofGeneration(format!("failed to parse verification_key.json: {e}"))
    })?;

    let vk_bytes = serializer::serialize_vk_from_snarkjs(&vk_json)
        .map_err(|e| StellarZkError::ProofGeneration(format!("VK serialization: {e}")))?;

    std::fs::write(vk_bin_output, &vk_bytes)?;
    Ok(())
}

/// Run a snarkjs command and check for success.
fn run_snarkjs(args: &[&str]) -> Result<String> {
    tracing::debug!("snarkjs {}", args.join(" "));

    let output = Command::new("snarkjs").args(args).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(stdout)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            Err(StellarZkError::ProofGeneration(format!(
                "snarkjs {} failed:\nstdout: {stdout}\nstderr: {stderr}",
                args.first().unwrap_or(&"")
            )))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "snarkjs".into(),
            install: "npm install -g snarkjs".into(),
        }),
        Err(e) => Err(StellarZkError::ProofGeneration(format!(
            "failed to run snarkjs: {e}"
        ))),
    }
}
