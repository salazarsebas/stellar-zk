use std::path::{Path, PathBuf};
use std::process::Command;

use stellar_zk_core::error::{Result, StellarZkError};

/// Compile a Circom circuit to R1CS and WASM.
pub fn compile_circom(circuit_path: &Path, output_dir: &Path) -> Result<()> {
    let output = Command::new("circom")
        .arg(circuit_path)
        .arg("--r1cs")
        .arg("--wasm")
        .arg("--sym")
        .arg("-o")
        .arg(output_dir)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            tracing::info!("circom compilation succeeded");
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(StellarZkError::CircuitCompilation(stderr.to_string()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StellarZkError::MissingTool {
            name: "circom".into(),
            install: "npm install -g circom".into(),
        }),
        Err(e) => Err(StellarZkError::CircuitCompilation(e.to_string())),
    }
}

/// Get the path to the witness-generator WASM produced by circom.
///
/// Circom outputs the WASM to `<output_dir>/<circuit_name>_js/<circuit_name>.wasm`.
pub fn witness_wasm_path(output_dir: &Path, circuit_name: &str) -> PathBuf {
    output_dir
        .join(format!("{circuit_name}_js"))
        .join(format!("{circuit_name}.wasm"))
}

/// Get the path to the R1CS file produced by circom.
pub fn r1cs_path(output_dir: &Path, circuit_name: &str) -> PathBuf {
    output_dir.join(format!("{circuit_name}.r1cs"))
}
