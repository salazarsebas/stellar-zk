//! Noir + UltraHonk backend for stellar-zk.
//!
//! Orchestrates [nargo](https://noir-lang.org/) for circuit compilation and execution,
//! and Barretenberg's `bb` tool for verification key generation and UltraHonk proving.
//! No trusted setup required (universal SRS).
//!
//! Proof size is ~14KB (~440 field elements). On-chain verification costs ~35M CPU instructions.
//!
//! **Prerequisites**: `nargo`, `bb` (Barretenberg)

mod nargo;
mod proof_convert;
#[allow(dead_code)]
mod serializer;

use std::path::Path;

use async_trait::async_trait;

use stellar_zk_core::backend::{
    BuildArtifacts, CostEstimate, PrerequisiteError, ProofArtifacts, VersionWarning, ZkBackend,
};
use stellar_zk_core::config::{BackendConfig, ProjectConfig};
use stellar_zk_core::error::Result;
use stellar_zk_core::profile::OptimizationProfile;

/// Noir + UltraHonk proving system backend using nargo and Barretenberg.
///
/// Produces ~14KB proofs verified via sumcheck + MSM on Soroban at ~35M CPU instructions.
/// Uses a universal SRS — no per-circuit trusted setup required.
///
/// **External tools**: `nargo`, `bb` (Barretenberg)
pub struct UltraHonkBackend;

impl Default for UltraHonkBackend {
    fn default() -> Self {
        Self
    }
}

impl UltraHonkBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ZkBackend for UltraHonkBackend {
    fn name(&self) -> &'static str {
        "ultrahonk"
    }

    fn display_name(&self) -> &'static str {
        "Noir + UltraHonk (Barretenberg)"
    }

    fn check_versions(&self) -> Vec<VersionWarning> {
        use stellar_zk_core::version::{detect_version, Version};

        let checks: &[(&str, Version)] = &[
            (
                "nargo",
                Version {
                    major: 0,
                    minor: 36,
                    patch: 0,
                },
            ),
            (
                "bb",
                Version {
                    major: 0,
                    minor: 56,
                    patch: 0,
                },
            ),
        ];

        let mut warnings = Vec::new();
        for &(tool, min) in checks {
            if let Some(found) = detect_version(tool) {
                if found < min {
                    warnings.push(VersionWarning {
                        tool_name: tool.into(),
                        found_version: found.to_string(),
                        minimum_version: min.to_string(),
                    });
                }
            }
        }
        warnings
    }

    fn check_prerequisites(&self) -> std::result::Result<(), Vec<PrerequisiteError>> {
        let mut missing = Vec::new();

        if which::which("nargo").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "nargo".into(),
                install_instructions: "curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup".into(),
            });
        }

        if which::which("bb").is_err() {
            missing.push(PrerequisiteError {
                tool_name: "bb (Barretenberg)".into(),
                install_instructions: "curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash && bbup".into(),
            });
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    async fn init_project(&self, _project_dir: &Path, _config: &ProjectConfig) -> Result<()> {
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

        let uh_config = config.ultrahonk.as_ref().ok_or_else(|| {
            stellar_zk_core::error::StellarZkError::CircuitCompilation(
                "missing ultrahonk config".into(),
            )
        })?;

        // Step 1: nargo compile
        tracing::info!("compiling Noir circuit with nargo");
        nargo::compile(project_dir)?;

        // Step 2: bb write_vk
        let acir_path = target_dir.join("circuits.json");
        let vk_path = target_dir.join("vk");
        tracing::info!("generating verification key with bb");
        nargo::write_vk(&acir_path, &vk_path, &uh_config.oracle_hash)?;

        // Persist oracle hash for prove() which doesn't have BackendConfig access
        let cached_config = serde_json::json!({
            "oracle_hash": &uh_config.oracle_hash,
        });
        std::fs::write(
            target_dir.join("ultrahonk_config.json"),
            serde_json::to_string(&cached_config).unwrap(),
        )?;

        // Step 3: Build verifier contract WASM via pipeline
        let contract_dir = project_dir.join("contracts/verifier");
        let wasm_output =
            stellar_zk_core::pipeline::build_and_optimize(&contract_dir, profile).await?;
        let wasm_path = wasm_output.path;

        Ok(BuildArtifacts {
            circuit_artifact: acir_path,
            verifier_wasm: wasm_path,
            proving_key: None, // UltraHonk has no separate proving key
            verification_key: vk_path,
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

        // Load oracle hash from cached build config
        let oracle_hash = load_oracle_hash(&target_dir);

        // Step 1: nargo execute (generate witness)
        tracing::info!("executing Noir circuit to generate witness");
        nargo::execute(project_dir)?;

        // Step 2: bb prove_ultra_honk
        let acir_path = &build_artifacts.circuit_artifact;
        let witness_path = target_dir.join("witness");
        let proof_path = proof_dir.join("proof.bin");
        tracing::info!("generating UltraHonk proof with bb");
        nargo::prove_ultrahonk(acir_path, &witness_path, &proof_path, &oracle_hash)?;

        // Step 3: Verify proof off-chain
        tracing::info!("verifying UltraHonk proof off-chain");
        nargo::verify_ultrahonk(&proof_path, &build_artifacts.verification_key, &oracle_hash)?;
        tracing::info!("off-chain verification passed");

        let proof_bytes = std::fs::read(&proof_path)?;
        let public_inputs = proof_convert::extract_public_inputs(&proof_bytes);

        // Write public_inputs.json for the call command
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
        Ok(stellar_zk_core::estimator::static_estimate(
            "ultrahonk",
            num_inputs,
        ))
    }
}

/// Load cached oracle hash from target/ultrahonk_config.json.
/// Falls back to "keccak" if the file is missing.
fn load_oracle_hash(target_dir: &Path) -> String {
    let config_path = target_dir.join("ultrahonk_config.json");
    if let Ok(contents) = std::fs::read_to_string(&config_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(hash) = v["oracle_hash"].as_str() {
                return hash.to_string();
            }
        }
    }
    "keccak".to_string()
}
