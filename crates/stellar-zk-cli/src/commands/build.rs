use std::path::Path;

use anyhow::Result;

use stellar_zk_core::profile::OptimizationProfile;
use stellar_zk_core::project;

use crate::output;
use crate::ProfileChoice;

/// Build the ZK circuit and Soroban verifier contract.
///
/// Compiles the circuit, generates proving/verification keys, and builds
/// the verifier contract WASM. Uses the backend and profile from the project config,
/// optionally overridden by CLI flags.
pub async fn run(
    config_path: &Path,
    profile_override: Option<ProfileChoice>,
    _circuit_only: bool,
    _contract_only: bool,
) -> Result<()> {
    output::print_header("stellar-zk build");

    let project_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    let (project_config, backend_config) = project::load_project(&project_dir)?;

    // Resolve profile
    let profile_name = profile_override
        .map(|p| p.as_str().to_string())
        .unwrap_or(project_config.profile.clone());

    let profile = OptimizationProfile::from_name(&profile_name)
        .ok_or_else(|| anyhow::anyhow!("unknown profile: {profile_name}"))?;

    output::print_key_value("Backend", &project_config.backend);
    output::print_key_value("Profile", &profile.name);

    // Create backend
    let backend = super::init::create_backend(&project_config.backend)?;

    // Check prerequisites
    if let Err(missing) = backend.check_prerequisites() {
        for m in &missing {
            output::print_error(&format!(
                "Missing tool: {} — {}",
                m.tool_name, m.install_instructions
            ));
        }
        anyhow::bail!("missing prerequisites");
    }

    // Check versions
    let version_warnings = backend.check_versions();
    for w in &version_warnings {
        output::print_warning(&format!(
            "{}: found v{}, minimum v{} recommended",
            w.tool_name, w.found_version, w.minimum_version
        ));
    }

    // Build
    output::print_step(1, 2, "Building circuit and contract...");
    let artifacts = backend
        .build(&project_dir, &backend_config, &profile)
        .await?;

    // Persist artifacts for prove/deploy/call/estimate
    output::print_step(2, 2, "Saving build artifacts...");
    stellar_zk_core::artifacts::save(&artifacts, &project_dir.join("target"))?;

    output::print_success("Build complete");
    output::print_key_value("Circuit", &artifacts.circuit_artifact.display().to_string());
    output::print_key_value("WASM", &artifacts.verifier_wasm.display().to_string());
    output::print_key_value("VK", &artifacts.verification_key.display().to_string());

    if let Some(pk) = &artifacts.proving_key {
        output::print_key_value("PK", &pk.display().to_string());
    }

    Ok(())
}
