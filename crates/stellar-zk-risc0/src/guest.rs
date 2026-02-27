use std::path::Path;
use std::process::Command;

use stellar_zk_core::error::{Result, StellarZkError};

/// Build a RISC Zero guest program.
///
/// This shells out to `cargo build` with the RISC-V target.
/// Requires the RISC Zero toolchain installed via `rzup`.
pub fn build_guest(guest_dir: &Path, target: &str) -> Result<()> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(target)
        .current_dir(guest_dir)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            tracing::info!("guest program compiled successfully");
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(StellarZkError::CircuitCompilation(format!(
                "RISC Zero guest build failed: {stderr}"
            )))
        }
        Err(e) => Err(StellarZkError::CircuitCompilation(format!(
            "failed to run cargo for guest build: {e}"
        ))),
    }
}

/// Build the RISC Zero host binary.
///
/// Shells out to `cargo build --release` in the host directory.
/// The resulting binary is used by `prove()` to generate proofs.
pub fn build_host(host_dir: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(host_dir)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            tracing::info!("host binary compiled successfully");
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(StellarZkError::CircuitCompilation(format!(
                "RISC Zero host build failed: {stderr}"
            )))
        }
        Err(e) => Err(StellarZkError::CircuitCompilation(format!(
            "failed to run cargo for host build: {e}"
        ))),
    }
}
