//! Groth16 backend for stellar-zk.
//!
//! Uses [Circom](https://docs.circom.io/) for circuit compilation and
//! [snarkjs](https://github.com/iden3/snarkjs) for trusted setup, witness generation,
//! and proof generation. Proof bytes are serialized into Soroban-compatible big-endian
//! format (256 bytes: A|B|C over BN254).
//!
//! On-chain verification costs ~12M CPU instructions via Soroban's BN254 pairing check.
//!
//! **Prerequisites**: `circom`, `snarkjs`, `node`

mod circuit;
mod prover;
pub mod serializer;

use std::path::Path;

use async_trait::async_trait;

use stellar_zk_core::backend::{
    BuildArtifacts, CostEstimate, PrerequisiteError, ProofArtifacts, ZkBackend,
};
use stellar_zk_core::config::{BackendConfig, ProjectConfig};
use stellar_zk_core::error::Result;
use stellar_zk_core::profile::OptimizationProfile;

/// Groth16 proving system backend using Circom circuits and snarkjs.
///
/// Produces 256-byte proofs (A|B|C over BN254) verified via 4-pairing check
/// on Soroban at ~12M CPU instructions. Requires a per-circuit trusted setup.
///
/// **External tools**: `circom`, `snarkjs`, `node`
pub struct Groth16Backend;

impl Default for Groth16Backend {
    fn default() -> Self {
        Self
    }
}

impl Groth16Backend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ZkBackend for Groth16Backend {
    fn name(&self) -> &'static str {
        "groth16"
    }

    fn display_name(&self) -> &'static str {
        "Groth16 (Circom + snarkjs)"
    }

    fn check_prerequisites(&self) -> std::result::Result<(), Vec<PrerequisiteError>> {
        let mut missing = Vec::new();

        if which::which("circom").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "circom".into(),
                install_instructions:
                    "https://docs.circom.io/getting-started/installation/".into(),
            });
        }

        if which::which("snarkjs").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "snarkjs".into(),
                install_instructions: "npm install -g snarkjs".into(),
            });
        }

        if which::which("node").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "node".into(),
                install_instructions: "https://nodejs.org/".into(),
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
        // Project scaffolding is handled by the CLI init command
        // using embedded templates. This method is for any
        // backend-specific post-init setup.
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

        // Step 1: Compile circom circuit
        let _circom_config = config.groth16.as_ref().ok_or_else(|| {
            stellar_zk_core::error::StellarZkError::CircuitCompilation(
                "missing groth16 config".into(),
            )
        })?;

        let circuit_path = project_dir.join("circuits/main.circom");
        tracing::info!("compiling circuit: {}", circuit_path.display());
        circuit::compile_circom(&circuit_path, &target_dir)?;

        let circuit_name = circuit_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("main");

        let r1cs = circuit::r1cs_path(&target_dir, circuit_name);

        // Step 2: Powers of Tau (generate dev ptau if none exists)
        let ptau_path = target_dir.join("pot12_final.ptau");
        if !ptau_path.exists() {
            tracing::info!("generating development Powers of Tau ceremony");
            prover::generate_dev_ptau(&ptau_path)?;
        }

        // Step 3: Generate proving and verification keys via snarkjs
        let zkey_path = target_dir.join("circuit.zkey");
        let vk_json_path = target_dir.join("verification_key.json");

        tracing::info!("running trusted setup (snarkjs groth16 setup)");
        prover::generate_keys(&r1cs, &ptau_path, &zkey_path, &vk_json_path)?;

        // Step 4: Convert VK to Soroban binary format
        let vk_bin_path = target_dir.join("verification.key");
        prover::convert_vk_to_soroban(&vk_json_path, &vk_bin_path)?;

        tracing::info!(
            "keys generated: zkey={}, VK={}",
            zkey_path.display(),
            vk_bin_path.display()
        );

        // Step 5: Build verifier contract WASM via pipeline
        let contract_dir = project_dir.join("contracts/verifier");
        let wasm_output =
            stellar_zk_core::pipeline::build_and_optimize(&contract_dir, profile).await?;
        let wasm_path = wasm_output.path;

        Ok(BuildArtifacts {
            circuit_artifact: r1cs,
            verifier_wasm: wasm_path,
            proving_key: Some(zkey_path),
            verification_key: vk_bin_path,
        })
    }

    async fn prove(
        &self,
        project_dir: &Path,
        build_artifacts: &BuildArtifacts,
        input_path: &Path,
    ) -> Result<ProofArtifacts> {
        if !input_path.exists() {
            return Err(stellar_zk_core::error::StellarZkError::InputNotFound(
                input_path.to_path_buf(),
            ));
        }

        let proof_dir = project_dir.join("proofs");
        std::fs::create_dir_all(&proof_dir)?;

        let target_dir = project_dir.join("target");

        // Locate witness generator WASM from circom output
        let circuit_name = build_artifacts
            .circuit_artifact
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("main");
        let witness_wasm = circuit::witness_wasm_path(&target_dir, circuit_name);

        if !witness_wasm.exists() {
            return Err(stellar_zk_core::error::StellarZkError::ProofGeneration(
                format!(
                    "witness generator WASM not found: {}. Run 'stellar-zk build' first.",
                    witness_wasm.display()
                ),
            ));
        }

        // Step 1: Generate witness
        let witness_path = target_dir.join("witness.wtns");
        tracing::info!("computing witness");
        prover::generate_witness(&witness_wasm, input_path, &witness_path)?;

        // Step 2: Generate proof
        let zkey_path = build_artifacts
            .proving_key
            .as_ref()
            .ok_or_else(|| {
                stellar_zk_core::error::StellarZkError::ProofGeneration(
                    "zkey path not set — run 'stellar-zk build' first".into(),
                )
            })?;

        let proof_json_path = proof_dir.join("proof.json");
        let public_json_path = proof_dir.join("public.json");

        tracing::info!("generating Groth16 proof");
        let (proof_bytes, public_inputs) = prover::generate_proof(
            zkey_path,
            &witness_path,
            &proof_json_path,
            &public_json_path,
        )?;

        // Write Soroban-format proof binary
        let proof_bin_path = proof_dir.join("proof.bin");
        std::fs::write(&proof_bin_path, &proof_bytes)?;

        // Write public inputs in hex format for reference
        let pi_hex: Vec<String> = public_inputs.iter().map(hex::encode).collect();
        let pi_info = serde_json::json!({
            "public_inputs_hex": pi_hex,
            "count": public_inputs.len(),
            "total_proof_size": proof_bytes.len(),
        });
        std::fs::write(
            proof_dir.join("public_inputs.json"),
            serde_json::to_string_pretty(&pi_info).unwrap(),
        )?;

        Ok(ProofArtifacts {
            proof: proof_bytes,
            public_inputs,
            proof_path: proof_bin_path,
        })
    }

    async fn estimate_cost(
        &self,
        _project_dir: &Path,
        proof_artifacts: &ProofArtifacts,
        _build_artifacts: &BuildArtifacts,
    ) -> Result<CostEstimate> {
        let num_inputs = proof_artifacts.public_inputs.len() as u32;
        Ok(stellar_zk_core::estimator::static_estimate(
            "groth16",
            num_inputs,
        ))
    }
}
