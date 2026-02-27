use std::path::Path;

use anyhow::Result;

use stellar_zk_core::project;
use stellar_zk_core::stellar::StellarCli;

use crate::output;
use crate::NetworkChoice;

/// Deploy the verifier contract to a Stellar network.
///
/// Uploads the compiled WASM, deploys the contract, and initializes it with
/// the verification key via the `__constructor`. Returns the contract ID for
/// subsequent `call` invocations.
pub async fn run(
    config_path: &Path,
    network: &NetworkChoice,
    source: &str,
) -> Result<()> {
    output::print_header("stellar-zk deploy");

    let project_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let (project_config, _) = project::load_project(&project_dir)?;

    // Load build artifacts to find WASM and VK paths
    let build_artifacts =
        stellar_zk_core::artifacts::load(&project_dir.join("target"))
            .map_err(|_| anyhow::anyhow!("build artifacts not found — run `stellar-zk build` first"))?;

    let wasm_path = &build_artifacts.verifier_wasm;
    if !wasm_path.exists() {
        output::print_error("WASM not found. Run `stellar-zk build` first.");
        anyhow::bail!("WASM not found at {}", wasm_path.display());
    }

    // Load verification key for constructor initialization
    let vk_path = &build_artifacts.verification_key;
    if !vk_path.exists() {
        output::print_error("Verification key not found. Run `stellar-zk build` first.");
        anyhow::bail!("VK not found at {}", vk_path.display());
    }
    let vk_bytes = std::fs::read(vk_path)?;
    let vk_hex = hex::encode(&vk_bytes);

    output::print_key_value("Contract", &project_config.contract.name);
    output::print_key_value("Network", network.as_str());
    output::print_key_value("WASM", &wasm_path.display().to_string());
    output::print_key_value("VK size", &format!("{} bytes", vk_bytes.len()));

    let stellar = StellarCli::new()?;

    output::print_step(1, 1, "Deploying contract with VK initialization...");
    let contract_id = stellar
        .deploy(
            wasm_path,
            network.as_str(),
            source,
            &[("vk_bytes", &vk_hex)],
        )
        .await?;

    output::print_success(&format!("Contract deployed: {contract_id}"));
    println!();
    println!("  To verify a proof:");
    println!("    stellar-zk call --contract-id {contract_id} --proof proofs/proof.bin --source {source}");
    println!();

    Ok(())
}
