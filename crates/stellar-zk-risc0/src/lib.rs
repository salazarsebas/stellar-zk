//! RISC Zero backend for stellar-zk.
//!
//! Compiles Rust guest programs via `cargo-risczero`, generates STARK proofs,
//! and wraps them in a Groth16 seal (~260 bytes) for on-chain verification on Soroban.
//! Uses a fixed universal verification key — no per-circuit trusted setup.
//!
//! On-chain verification costs ~15M CPU instructions via BN254 pairing check.
//!
//! **Prerequisites**: `cargo-risczero`, `docker`

mod guest;
mod prover;
mod serializer;

use std::path::Path;

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use stellar_zk_core::backend::{
    BuildArtifacts, CostEstimate, PrerequisiteError, ProofArtifacts, ZkBackend,
};
use stellar_zk_core::config::{BackendConfig, ProjectConfig};
use stellar_zk_core::error::Result;
use stellar_zk_core::profile::OptimizationProfile;

/// RISC Zero zkVM backend for proving arbitrary Rust computations.
///
/// Compiles Rust guest programs, generates STARK proofs, and wraps them in a
/// ~260-byte Groth16 seal. Uses a fixed universal verification key — verified
/// on Soroban at ~15M CPU instructions.
///
/// **External tools**: `cargo-risczero`, `docker`
pub struct Risc0Backend;

impl Default for Risc0Backend {
    fn default() -> Self {
        Self
    }
}

impl Risc0Backend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ZkBackend for Risc0Backend {
    fn name(&self) -> &'static str {
        "risc0"
    }

    fn display_name(&self) -> &'static str {
        "RISC Zero (zkVM)"
    }

    fn check_prerequisites(&self) -> std::result::Result<(), Vec<PrerequisiteError>> {
        let mut missing = Vec::new();

        // Check for cargo-risczero
        if which::which("cargo-risczero").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "cargo-risczero".into(),
                install_instructions:
                    "curl -L https://risczero.com/install | bash && rzup install".into(),
            });
        }

        // Docker is needed for Groth16 wrapping
        if which::which("docker").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "docker".into(),
                install_instructions:
                    "https://docs.docker.com/get-docker/ (needed for Groth16 proof wrapping)"
                        .into(),
            });
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    async fn init_project(
        &self,
        _project_dir: &Path,
        _config: &ProjectConfig,
    ) -> Result<()> {
        Ok(())
    }

    async fn build(
        &self,
        project_dir: &Path,
        config: &BackendConfig,
        profile: &OptimizationProfile,
    ) -> Result<BuildArtifacts> {
        let target_dir = project_dir.join("target");
        std::fs::create_dir_all(&target_dir)?;

        let r0_config = config.risc0.as_ref().ok_or_else(|| {
            stellar_zk_core::error::StellarZkError::CircuitCompilation(
                "missing risc0 config".into(),
            )
        })?;

        // Step 1: Build guest program
        let guest_dir = project_dir.join("programs/guest");
        tracing::info!("building RISC Zero guest program");
        guest::build_guest(&guest_dir, &r0_config.guest_target)?;

        // Step 2: Locate the real ELF and copy to target/
        let guest_elf_src = guest_dir
            .join("target")
            .join(&r0_config.guest_target)
            .join("release/guest");
        let elf_path = target_dir.join("guest.elf");
        if guest_elf_src.exists() {
            std::fs::copy(&guest_elf_src, &elf_path)?;
            tracing::info!("copied guest ELF to {}", elf_path.display());
        } else {
            tracing::warn!(
                "guest ELF not found at {}; writing placeholder",
                guest_elf_src.display()
            );
            std::fs::write(&elf_path, b"[elf placeholder]")?;
        }

        // Step 3: Build host binary (used by prove to generate proofs)
        let host_dir = project_dir.join("programs/host");
        tracing::info!("building RISC Zero host binary");
        guest::build_host(&host_dir)?;

        // Step 4: Cache config for prove() which doesn't have BackendConfig access
        let cached_config = serde_json::json!({
            "guest_target": &r0_config.guest_target,
            "segment_limit_po2": r0_config.segment_limit_po2,
            "groth16_wrap": r0_config.groth16_wrap,
        });
        std::fs::write(
            target_dir.join("risc0_config.json"),
            serde_json::to_string(&cached_config).unwrap(),
        )?;

        // Step 5: Build verifier contract WASM via pipeline
        let contract_dir = project_dir.join("contracts/verifier");
        let wasm_output =
            stellar_zk_core::pipeline::build_and_optimize(&contract_dir, profile).await?;
        let wasm_path = wasm_output.path;

        // Step 6: VK placeholder — RISC Zero uses a universal VK.
        // The real VK is extracted at prove time by the host binary.
        // For deploy, the VK will be the standard Groth16 VK format
        // (alpha + beta + gamma + delta + IC points).
        let vk_path = target_dir.join("risc0.vk");
        if !vk_path.exists() {
            std::fs::write(&vk_path, b"[risc0 universal vk]")?;
        }

        Ok(BuildArtifacts {
            circuit_artifact: elf_path,
            verifier_wasm: wasm_path,
            proving_key: None,
            verification_key: vk_path,
        })
    }

    async fn prove(
        &self,
        project_dir: &Path,
        _build_artifacts: &BuildArtifacts,
        input_path: &Path,
    ) -> Result<ProofArtifacts> {
        if !input_path.exists() {
            return Err(stellar_zk_core::error::StellarZkError::InputNotFound(
                input_path.to_path_buf(),
            ));
        }

        let proof_dir = project_dir.join("proofs");
        std::fs::create_dir_all(&proof_dir)?;

        // Step 1: Run the host binary to generate the proof
        tracing::info!("generating RISC Zero proof via host binary");
        let receipt = prover::run_host(project_dir, input_path)?;

        // Step 2: Validate the seal format
        if !serializer::validate_seal(&receipt.seal) {
            return Err(stellar_zk_core::error::StellarZkError::ProofGeneration(
                format!(
                    "invalid seal: expected 260 bytes with correct selector, got {} bytes",
                    receipt.seal.len()
                ),
            ));
        }

        // Step 3: Write proof file (the raw seal)
        let proof_path = proof_dir.join("receipt.bin");
        std::fs::write(&proof_path, &receipt.seal)?;

        // Step 4: Compute journal digest = SHA256(journal_bytes)
        let journal_digest: [u8; 32] = {
            let mut hasher = Sha256::new();
            hasher.update(&receipt.journal);
            hasher.finalize().into()
        };

        // Public inputs: [image_id(32 bytes), journal_digest(32 bytes)]
        let public_inputs: Vec<[u8; 32]> = vec![receipt.image_id, journal_digest];

        // Step 5: Write public_inputs.json for the call command
        let pi_hex: Vec<String> = public_inputs.iter().map(hex::encode).collect();
        let pi_info = serde_json::json!({
            "public_inputs_hex": pi_hex,
            "count": public_inputs.len(),
            "total_proof_size": receipt.seal.len(),
        });
        std::fs::write(
            proof_dir.join("public_inputs.json"),
            serde_json::to_string_pretty(&pi_info).unwrap(),
        )?;

        Ok(ProofArtifacts {
            proof: receipt.seal,
            public_inputs,
            proof_path,
        })
    }

    async fn estimate_cost(
        &self,
        _project_dir: &Path,
        proof_artifacts: &ProofArtifacts,
        _build_artifacts: &BuildArtifacts,
    ) -> Result<CostEstimate> {
        let num_inputs = proof_artifacts.public_inputs.len() as u32;
        Ok(stellar_zk_core::estimator::static_estimate("risc0", num_inputs))
    }
}
