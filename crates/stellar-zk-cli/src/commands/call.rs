use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

use stellar_zk_core::project;
use stellar_zk_core::stellar::StellarCli;

use crate::output;
use crate::NetworkChoice;

/// Call the deployed verifier contract with a proof.
///
/// Reads the proof binary, loads public inputs, computes a SHA256 nullifier,
/// and invokes the contract's `verify(proof, public_inputs, nullifier)` function
/// via the `stellar` CLI.
pub async fn run(
    config_path: &Path,
    contract_id: &str,
    proof_path: &Path,
    public_inputs_path: Option<&Path>,
    network: &NetworkChoice,
    source: &str,
) -> Result<()> {
    output::print_header("stellar-zk call");

    let project_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    let (project_config, _) = project::load_project(&project_dir)?;

    output::print_key_value("Contract", contract_id);
    output::print_key_value("Backend", &project_config.backend);
    output::print_key_value("Network", network.as_str());
    output::print_key_value("Proof", &proof_path.display().to_string());

    // Load proof data
    if !proof_path.exists() {
        anyhow::bail!("proof file not found: {}", proof_path.display());
    }
    let proof_bytes = std::fs::read(proof_path)?;
    let proof_hex = hex::encode(&proof_bytes);

    // Load public inputs
    let pi_bytes = load_public_inputs(public_inputs_path, &project_dir)?;
    let pi_hex = hex::encode(&pi_bytes);
    let num_inputs = pi_bytes.len() / 32;

    output::print_key_value("Public inputs", &format!("{num_inputs} field elements"));

    // Compute nullifier: SHA256(proof || public_inputs)
    let nullifier = compute_nullifier(&proof_bytes, &pi_bytes);
    let nullifier_hex = hex::encode(nullifier);

    let stellar = StellarCli::new()?;

    output::print_step(1, 1, "Calling verify on contract...");
    let result = stellar
        .invoke(
            contract_id,
            "verify",
            &[
                ("proof", &proof_hex),
                ("public_inputs", &pi_hex),
                ("nullifier", &nullifier_hex),
            ],
            network.as_str(),
            source,
        )
        .await?;

    output::print_success(&format!("Verification result: {result}"));

    Ok(())
}

/// Load public inputs as concatenated 32-byte field elements.
///
/// If an explicit path is given, reads it as raw bytes. Otherwise looks for
/// `proofs/public_inputs.json` (written by the prove step) and decodes the
/// hex-encoded field elements.
fn load_public_inputs(explicit_path: Option<&Path>, project_dir: &Path) -> Result<Vec<u8>> {
    if let Some(path) = explicit_path {
        if !path.exists() {
            anyhow::bail!("public inputs file not found: {}", path.display());
        }
        return Ok(std::fs::read(path)?);
    }

    // Fall back to proofs/public_inputs.json from the prove step
    let pi_json_path = project_dir.join("proofs/public_inputs.json");
    if !pi_json_path.exists() {
        anyhow::bail!(
            "public inputs not found — run `stellar-zk prove` first or pass --public-inputs"
        );
    }

    let pi_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&pi_json_path)?)?;
    let hex_array = pi_json["public_inputs_hex"].as_array().ok_or_else(|| {
        anyhow::anyhow!("invalid public_inputs.json: missing public_inputs_hex array")
    })?;

    let mut result = Vec::new();
    for hex_val in hex_array {
        let hex_str = hex_val
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("invalid hex value in public_inputs.json"))?;
        let bytes = hex::decode(hex_str)?;
        result.extend_from_slice(&bytes);
    }

    Ok(result)
}

/// Compute a deterministic nullifier: SHA256(proof || public_inputs).
fn compute_nullifier(proof: &[u8], public_inputs: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(proof);
    hasher.update(public_inputs);
    hasher.finalize().into()
}
