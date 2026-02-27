use std::path::Path;

use anyhow::Result;

use stellar_zk_core::project;

use crate::output;

/// Generate a ZK proof from input data.
///
/// Reads the input JSON, computes the witness, and generates a proof using
/// the build artifacts (proving key, circuit). Outputs a Soroban-compatible
/// proof binary and public inputs file.
pub async fn run(
    config_path: &Path,
    input_path: &Path,
    _output_path: Option<&Path>,
) -> Result<()> {
    output::print_header("stellar-zk prove");

    let project_dir = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let (project_config, _backend_config) = project::load_project(&project_dir)?;

    output::print_key_value("Backend", &project_config.backend);
    output::print_key_value("Input", &input_path.display().to_string());

    let backend = super::init::create_backend(&project_config.backend)?;

    // Load build artifacts from the previous build step
    output::print_step(1, 2, "Loading build artifacts...");
    let build_artifacts = stellar_zk_core::artifacts::load(&project_dir.join("target"))
        .map_err(|_| anyhow::anyhow!("build artifacts not found — run `stellar-zk build` first"))?;

    output::print_step(2, 2, "Generating proof...");
    let proof_artifacts = backend.prove(&project_dir, &build_artifacts, input_path).await?;

    output::print_success("Proof generated");
    output::print_key_value("Proof file", &proof_artifacts.proof_path.display().to_string());
    output::print_key_value(
        "Proof size",
        &format!("{} bytes", proof_artifacts.proof.len()),
    );
    output::print_key_value(
        "Public inputs",
        &format!("{}", proof_artifacts.public_inputs.len()),
    );

    Ok(())
}
