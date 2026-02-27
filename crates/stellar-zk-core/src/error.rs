//! Unified error types for the stellar-zk toolkit.

use std::path::PathBuf;
use thiserror::Error;

/// All errors that can occur during stellar-zk operations.
#[derive(Error, Debug)]
pub enum StellarZkError {
    // --- Configuration ---

    /// The configuration file (`stellar-zk.config.json` or `backend.config.json`) was not found.
    #[error("config file not found at {path}")]
    ConfigNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// The configuration file exists but contains invalid JSON.
    #[error("failed to parse config at {path}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// The specified backend name is not one of: `groth16`, `ultrahonk`, `risc0`.
    #[error("unknown backend: {0} (supported: groth16, ultrahonk, risc0)")]
    UnknownBackend(String),

    /// The specified optimization profile is not one of: `development`, `testnet`, `stellar-production`.
    #[error("unknown profile: {0} (supported: development, testnet, stellar-production)")]
    UnknownProfile(String),

    // --- Prerequisites ---

    /// A required external tool (e.g., `circom`, `snarkjs`, `nargo`) is not installed.
    #[error("required tool '{name}' not found — install: {install}")]
    MissingTool { name: String, install: String },

    // --- Build ---

    /// The ZK circuit failed to compile (e.g., Circom syntax error, Noir type error).
    #[error("circuit compilation failed: {0}")]
    CircuitCompilation(String),

    /// The Soroban verifier contract failed to build.
    #[error("contract build failed: {0}")]
    ContractBuild(String),

    /// The WASM optimization step (`wasm-opt` or `wasm-strip`) failed.
    #[error("WASM optimization failed: {0}")]
    WasmOptFailed(String),

    /// The compiled WASM exceeds Soroban's size limit (64KB for production).
    #[error("WASM too large: {size} bytes (max {max}) at {path}")]
    WasmTooLarge {
        size: u64,
        max: u64,
        path: PathBuf,
    },

    // --- Proof ---

    /// Proof generation failed (witness computation, proving, or serialization).
    #[error("proof generation failed: {0}")]
    ProofGeneration(String),

    /// The input JSON file for proof generation was not found.
    #[error("input file not found: {0}")]
    InputNotFound(PathBuf),

    // --- Stellar CLI ---

    /// An error occurred while invoking the `stellar` CLI tool.
    #[error("stellar CLI error: {0}")]
    StellarCli(String),

    /// Contract deployment to the Stellar network failed.
    #[error("deployment failed: {0}")]
    DeployFailed(String),

    // --- Templates ---

    /// Handlebars template rendering failed (invalid template or missing variables).
    #[error("template rendering failed: {0}")]
    TemplateRender(String),

    // --- Project ---

    /// Attempted to create a project in a directory that already exists.
    #[error("project directory already exists: {0}")]
    ProjectExists(PathBuf),

    /// The current directory is not a stellar-zk project (missing config file).
    #[error("not a stellar-zk project (missing stellar-zk.config.json)")]
    NotAProject,

    // --- General ---

    /// A filesystem I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A catch-all for errors from dependencies.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Alias for `Result<T, StellarZkError>`.
pub type Result<T> = std::result::Result<T, StellarZkError>;
