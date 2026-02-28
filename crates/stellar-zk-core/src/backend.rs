use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::{BackendConfig, ProjectConfig};
use crate::error::Result;
use crate::profile::OptimizationProfile;

/// Artifacts produced by the build step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifacts {
    /// Path to the compiled circuit/program artifact.
    pub circuit_artifact: PathBuf,
    /// Path to the generated Soroban verifier contract WASM.
    pub verifier_wasm: PathBuf,
    /// Path to proving key (if applicable; None for UltraHonk/RISC0).
    pub proving_key: Option<PathBuf>,
    /// Path to verification key.
    pub verification_key: PathBuf,
}

/// Artifacts produced by the prove step.
#[derive(Debug, Clone)]
pub struct ProofArtifacts {
    /// Serialized proof bytes in Soroban-compatible format.
    pub proof: Vec<u8>,
    /// Public inputs as 32-byte big-endian field elements.
    pub public_inputs: Vec<[u8; 32]>,
    /// Path to the proof file on disk.
    pub proof_path: PathBuf,
}

/// Cost estimation result.
#[derive(Debug, Clone)]
pub struct CostEstimate {
    /// Estimated CPU instructions consumed.
    pub cpu_instructions: u64,
    /// Estimated memory usage in bytes.
    pub memory_bytes: u64,
    /// WASM binary size in bytes.
    pub wasm_size: u64,
    /// Number of ledger entry reads.
    pub ledger_reads: u32,
    /// Number of ledger entry writes.
    pub ledger_writes: u32,
    /// Estimated fee in stroops.
    pub estimated_fee_stroops: u64,
    /// Warnings about approaching limits.
    pub warnings: Vec<String>,
}

/// Information about a missing prerequisite tool.
#[derive(Debug, Clone)]
pub struct PrerequisiteError {
    pub tool_name: String,
    pub install_instructions: String,
}

/// Warning about a tool version being below the recommended minimum.
#[derive(Debug, Clone)]
pub struct VersionWarning {
    pub tool_name: String,
    pub found_version: String,
    pub minimum_version: String,
}

/// Every ZK backend must implement this trait.
#[async_trait]
pub trait ZkBackend: Send + Sync {
    /// Human-readable name: "groth16", "ultrahonk", "risc0".
    fn name(&self) -> &'static str;

    /// Display name for user-facing output.
    fn display_name(&self) -> &'static str;

    /// Check that all required external tools are installed.
    fn check_prerequisites(&self) -> std::result::Result<(), Vec<PrerequisiteError>>;

    /// Check installed tool versions against recommended minimums.
    ///
    /// Returns warnings for tools whose versions are below the minimum.
    /// If version detection fails (tool doesn't support `--version`, unexpected output),
    /// the tool is silently skipped — no warning emitted.
    fn check_versions(&self) -> Vec<VersionWarning> {
        vec![]
    }

    /// Initialize a new project: scaffold circuit/program files and
    /// the verifier contract template into the project directory.
    async fn init_project(&self, project_dir: &Path, config: &ProjectConfig) -> Result<()>;

    /// Compile the circuit/program and generate the verifier contract WASM.
    async fn build(
        &self,
        project_dir: &Path,
        config: &BackendConfig,
        profile: &OptimizationProfile,
    ) -> Result<BuildArtifacts>;

    /// Generate a proof from the compiled circuit and input data.
    async fn prove(
        &self,
        project_dir: &Path,
        build_artifacts: &BuildArtifacts,
        input_path: &Path,
    ) -> Result<ProofArtifacts>;

    /// Estimate the on-chain verification cost.
    async fn estimate_cost(
        &self,
        project_dir: &Path,
        proof_artifacts: &ProofArtifacts,
        build_artifacts: &BuildArtifacts,
    ) -> Result<CostEstimate>;
}
