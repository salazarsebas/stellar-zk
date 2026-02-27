use std::path::Path;
use std::process::Command;

use stellar_zk_core::error::{Result, StellarZkError};

/// Structured output from running the host binary.
pub struct ReceiptOutput {
    /// The Groth16 seal (4-byte selector + 256-byte proof = 260 bytes).
    pub seal: Vec<u8>,
    /// The journal bytes (public output of the zkVM execution).
    pub journal: Vec<u8>,
    /// The 32-byte image ID identifying the guest program.
    pub image_id: [u8; 32],
}

/// Run the host binary to generate a RISC Zero Groth16 proof.
///
/// The host binary reads the guest ELF, executes it in the zkVM, generates
/// a Groth16-wrapped proof, and writes structured output files:
/// - `proofs/seal.bin` — the Groth16 seal
/// - `proofs/journal.bin` — the execution journal
/// - `proofs/image_id.hex` — hex-encoded image ID
pub fn run_host(project_dir: &Path, input_path: &Path) -> Result<ReceiptOutput> {
    let host_bin = project_dir.join("programs/host/target/release/host");
    if !host_bin.exists() {
        return Err(StellarZkError::ProofGeneration(
            "host binary not found — run `stellar-zk build` first".into(),
        ));
    }

    let output = Command::new(&host_bin)
        .env("RISC0_INPUT", input_path.as_os_str())
        .current_dir(project_dir)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            tracing::info!("host binary completed successfully");
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(StellarZkError::ProofGeneration(format!(
                "host binary failed: {stderr}"
            )));
        }
        Err(e) => {
            return Err(StellarZkError::ProofGeneration(format!(
                "failed to run host binary: {e}"
            )));
        }
    }

    // Read output files
    let proof_dir = project_dir.join("proofs");

    let seal = std::fs::read(proof_dir.join("seal.bin")).map_err(|e| {
        StellarZkError::ProofGeneration(format!("failed to read seal.bin: {e}"))
    })?;

    let journal = std::fs::read(proof_dir.join("journal.bin")).map_err(|e| {
        StellarZkError::ProofGeneration(format!("failed to read journal.bin: {e}"))
    })?;

    let image_id_hex = std::fs::read_to_string(proof_dir.join("image_id.hex"))
        .map_err(|e| {
            StellarZkError::ProofGeneration(format!("failed to read image_id.hex: {e}"))
        })?;
    let image_id_bytes = hex::decode(image_id_hex.trim()).map_err(|e| {
        StellarZkError::ProofGeneration(format!("invalid image_id hex: {e}"))
    })?;
    if image_id_bytes.len() != 32 {
        return Err(StellarZkError::ProofGeneration(format!(
            "image_id must be 32 bytes, got {}",
            image_id_bytes.len()
        )));
    }

    let mut image_id = [0u8; 32];
    image_id.copy_from_slice(&image_id_bytes);

    Ok(ReceiptOutput {
        seal,
        journal,
        image_id,
    })
}
