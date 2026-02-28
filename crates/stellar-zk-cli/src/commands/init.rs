use std::path::Path;

use anyhow::Result;
use dialoguer::Select;

use stellar_zk_core::config::{BackendConfig, ProjectConfig};
use stellar_zk_core::project;
use stellar_zk_core::templates::embedded;
use stellar_zk_core::templates::renderer::TemplateRenderer;

use crate::output;
use crate::{BackendChoice, ProfileChoice};

/// Initialize a new stellar-zk project.
///
/// Creates the directory structure, writes configuration files, renders circuit
/// and contract templates for the selected backend, and checks for required
/// external tools. If no backend is specified, prompts interactively.
pub async fn run(
    name: &str,
    backend: Option<BackendChoice>,
    profile: &ProfileChoice,
) -> Result<()> {
    output::print_header(&format!("stellar-zk init: {name}"));

    // Select backend (interactive if not provided)
    let backend_name = match backend {
        Some(b) => b.as_str().to_string(),
        None => {
            let options = &["groth16", "ultrahonk", "risc0"];
            let descriptions = &[
                "Groth16 (Circom) — low cost, simple proofs, trusted setup",
                "Noir + UltraHonk — modern zk apps, no trusted setup",
                "RISC Zero (zkVM) — arbitrary Rust computation, no trusted setup",
            ];

            let selection = Select::new()
                .with_prompt("Select ZK backend")
                .items(descriptions)
                .default(0)
                .interact()?;

            options[selection].to_string()
        }
    };

    let project_dir = Path::new(name);
    output::print_step(1, 4, &format!("Creating project directory: {name}/"));

    // Create directory structure
    project::create_project_dirs(project_dir, &backend_name)?;

    // Generate configs
    output::print_step(2, 4, "Writing configuration files");
    let project_config = ProjectConfig::default_for_backend(name, &backend_name);
    let backend_config = BackendConfig::default_for_backend(&backend_name);
    project::write_configs(project_dir, &project_config, &backend_config)?;

    // Render and write templates
    output::print_step(3, 4, "Scaffolding circuit and contract templates");
    let renderer = TemplateRenderer::new();
    let data = serde_json::json!({
        "project_name": name,
        "backend": backend_name,
        "contract_name": project_config.contract.name,
        "profile": profile.as_str(),
    });

    write_circuit_templates(project_dir, &backend_name)?;
    write_contract_templates(project_dir, &backend_name, &renderer, &data)?;
    write_input_template(project_dir)?;

    // Check prerequisites
    output::print_step(4, 4, "Checking prerequisites");
    let backend_impl = create_backend(&backend_name)?;
    match backend_impl.check_prerequisites() {
        Ok(()) => output::print_success("All required tools found"),
        Err(missing) => {
            for m in &missing {
                output::print_warning(&format!(
                    "Missing: {} — install: {}",
                    m.tool_name, m.install_instructions
                ));
            }
        }
    }

    output::print_success(&format!(
        "Project '{name}' created with {backend_name} backend"
    ));
    println!();
    println!("  Next steps:");
    println!("    cd {name}");
    println!("    stellar-zk build");
    println!("    stellar-zk prove --input inputs/input.json");
    println!("    stellar-zk estimate");
    println!();

    Ok(())
}

fn write_circuit_templates(project_dir: &Path, backend: &str) -> Result<()> {
    match backend {
        "groth16" => {
            std::fs::write(
                project_dir.join("circuits/main.circom"),
                embedded::GROTH16_CIRCUIT,
            )?;
        }
        "ultrahonk" => {
            std::fs::write(
                project_dir.join("circuits/Nargo.toml"),
                embedded::ULTRAHONK_NARGO_TOML,
            )?;
            std::fs::write(
                project_dir.join("circuits/src/main.nr"),
                embedded::ULTRAHONK_CIRCUIT,
            )?;
        }
        "risc0" => {
            std::fs::write(
                project_dir.join("programs/guest/Cargo.toml"),
                embedded::RISC0_GUEST_CARGO_TOML,
            )?;
            std::fs::write(
                project_dir.join("programs/guest/src/main.rs"),
                embedded::RISC0_GUEST,
            )?;
            std::fs::write(
                project_dir.join("programs/host/Cargo.toml"),
                embedded::RISC0_HOST_CARGO_TOML,
            )?;
            std::fs::write(
                project_dir.join("programs/host/src/main.rs"),
                embedded::RISC0_HOST,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn write_contract_templates(
    project_dir: &Path,
    backend: &str,
    renderer: &TemplateRenderer,
    data: &serde_json::Value,
) -> Result<()> {
    let (cargo_tmpl, lib_tmpl) = match backend {
        "groth16" => (
            embedded::GROTH16_CONTRACT_CARGO,
            embedded::GROTH16_CONTRACT_LIB,
        ),
        "ultrahonk" => (
            embedded::ULTRAHONK_CONTRACT_CARGO,
            embedded::ULTRAHONK_CONTRACT_LIB,
        ),
        "risc0" => (embedded::RISC0_CONTRACT_CARGO, embedded::RISC0_CONTRACT_LIB),
        _ => return Ok(()),
    };

    let cargo_content = renderer.render(cargo_tmpl, data)?;
    let lib_content = renderer.render(lib_tmpl, data)?;

    std::fs::write(
        project_dir.join("contracts/verifier/Cargo.toml"),
        cargo_content,
    )?;
    std::fs::write(
        project_dir.join("contracts/verifier/src/lib.rs"),
        lib_content,
    )?;

    Ok(())
}

fn write_input_template(project_dir: &Path) -> Result<()> {
    std::fs::write(project_dir.join("inputs/input.json"), embedded::INPUT_JSON)?;
    Ok(())
}

/// Create the appropriate backend implementation.
pub fn create_backend(name: &str) -> Result<Box<dyn stellar_zk_core::backend::ZkBackend>> {
    match name {
        "groth16" => Ok(Box::new(stellar_zk_groth16::Groth16Backend::new())),
        "ultrahonk" => Ok(Box::new(stellar_zk_ultrahonk::UltraHonkBackend::new())),
        "risc0" => Ok(Box::new(stellar_zk_risc0::Risc0Backend::new())),
        _ => anyhow::bail!("unknown backend: {name}"),
    }
}
