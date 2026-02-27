use std::path::Path;

use anyhow::Result;

use stellar_zk_core::estimator;
use stellar_zk_core::project;
use stellar_zk_core::stellar::StellarCli;

use crate::output;
use crate::NetworkChoice;

/// Estimate on-chain verification costs.
///
/// Computes CPU instructions, memory usage, WASM size, and estimated fees
/// for verifying a proof on Soroban. Uses up to 3 tiers:
/// - **Tier 1** (static): instant offline estimate from backend cost models
/// - **Tier 2** (artifact): uses actual WASM file size from build output
/// - **Tier 3** (simulation): invokes `stellar --sim-only` against a deployed contract
pub async fn run(
    config_path: &Path,
    proof_path: Option<&Path>,
    num_public_inputs: u32,
    network: &NetworkChoice,
) -> Result<()> {
    output::print_header("stellar-zk estimate");

    let project_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    // Try to load project config; fall back to defaults
    let backend_name = if config_path.exists() {
        let (project_config, _) = project::load_project(&project_dir)?;
        project_config.backend
    } else {
        "groth16".to_string()
    };

    output::print_key_value("Backend", &backend_name);
    output::print_key_value("Public inputs", &num_public_inputs.to_string());

    // Tier 1: Static estimate
    let mut estimate = estimator::static_estimate(&backend_name, num_public_inputs);

    // Tier 2: If build artifacts exist, use actual WASM size
    if let Ok(artifacts) = stellar_zk_core::artifacts::load(&project_dir.join("target")) {
        if artifacts.verifier_wasm.exists() {
            if let Ok(meta) = std::fs::metadata(&artifacts.verifier_wasm) {
                let actual_size = meta.len();
                estimate.wasm_size = actual_size;
                output::print_key_value("WASM size (actual)", &format!("{actual_size} bytes"));
            }
        }
    }

    let report = estimator::format_estimate(&estimate, &backend_name);
    println!("{report}");

    // Tier 3: If a proof is provided, attempt RPC simulation
    if let Some(proof_path) = proof_path {
        if proof_path.exists() {
            output::print_step(1, 1, "Running on-chain simulation...");

            match StellarCli::new() {
                Ok(stellar) => {
                    let proof_bytes = std::fs::read(proof_path)?;
                    let proof_hex = hex::encode(&proof_bytes);

                    // We need a contract ID — check if one was recently deployed
                    // For now, simulation requires a deployed contract
                    output::print_key_value(
                        "Note",
                        "Tier 3 simulation requires a deployed contract (use --contract-id in future)",
                    );

                    // If we had a contract_id, we'd do:
                    // let sim = stellar.simulate(contract_id, "verify", &[("proof", &proof_hex)], network.as_str()).await?;
                    let _ = (stellar, proof_hex, network);
                }
                Err(_) => {
                    output::print_key_value(
                        "Note",
                        "stellar CLI not found — skipping Tier 3 simulation",
                    );
                }
            }
        }
    }

    Ok(())
}
