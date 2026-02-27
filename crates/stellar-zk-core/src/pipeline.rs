use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, StellarZkError};
use crate::profile::{OptimizationProfile, WasmOptLevel};

/// Result of the WASM optimization pipeline.
#[derive(Debug, Clone)]
pub struct WasmOutput {
    /// Path to the final optimized WASM file.
    pub path: PathBuf,
    /// Final size in bytes.
    pub size_bytes: u64,
    /// Whether wasm-opt was applied.
    pub optimized: bool,
    /// Size at each pipeline stage for reporting.
    pub stage_sizes: Vec<(String, u64)>,
}

/// Run the WASM build and optimization pipeline for a Soroban contract.
pub async fn build_and_optimize(
    contract_dir: &Path,
    profile: &OptimizationProfile,
) -> Result<WasmOutput> {
    let mut stages = Vec::new();

    // Stage 1: cargo build
    let raw_wasm = cargo_build(contract_dir, profile).await?;
    let raw_size = std::fs::metadata(&raw_wasm)?.len();
    stages.push(("cargo build".into(), raw_size));

    // Stage 2: wasm-opt (if profile requires it)
    let optimized_wasm = match &profile.wasm_opt_level {
        WasmOptLevel::None => raw_wasm.clone(),
        level => {
            let output = run_wasm_opt(&raw_wasm, level)?;
            let opt_size = std::fs::metadata(&output)?.len();
            stages.push(("wasm-opt".into(), opt_size));
            output
        }
    };

    // Stage 3: strip (if profile requires it)
    let final_wasm = if profile.strip_symbols {
        let stripped = strip_wasm(&optimized_wasm)?;
        let strip_size = std::fs::metadata(&stripped)?.len();
        stages.push(("strip".into(), strip_size));
        stripped
    } else {
        optimized_wasm
    };

    let final_size = std::fs::metadata(&final_wasm)?.len();

    // Stage 4: size validation
    if profile.enforce_size_limit && final_size > OptimizationProfile::MAX_WASM_SIZE {
        return Err(StellarZkError::WasmTooLarge {
            size: final_size,
            max: OptimizationProfile::MAX_WASM_SIZE,
            path: final_wasm,
        });
    }

    Ok(WasmOutput {
        path: final_wasm,
        size_bytes: final_size,
        optimized: !matches!(profile.wasm_opt_level, WasmOptLevel::None),
        stage_sizes: stages,
    })
}

/// Build the contract WASM using cargo.
async fn cargo_build(contract_dir: &Path, profile: &OptimizationProfile) -> Result<PathBuf> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .current_dir(contract_dir);

    match profile.cargo_profile.as_str() {
        "dev" => {}
        "release" => {
            cmd.arg("--release");
        }
        custom => {
            cmd.arg("--profile").arg(custom);
        }
    }

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(StellarZkError::ContractBuild(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // Find the WASM output
    let target_subdir = match profile.cargo_profile.as_str() {
        "dev" => "debug",
        _ => "release",
    };

    // Look for .wasm files in the target directory
    let target_dir = contract_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join(target_subdir);

    find_wasm_file(&target_dir)
}

/// Find the first .wasm file in a directory.
fn find_wasm_file(dir: &Path) -> Result<PathBuf> {
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wasm") {
                return Ok(path);
            }
        }
    }

    Err(StellarZkError::ContractBuild(format!(
        "no .wasm file found in {}",
        dir.display()
    )))
}

/// Run wasm-opt on a WASM file.
fn run_wasm_opt(input: &Path, level: &WasmOptLevel) -> Result<PathBuf> {
    let output_path = input.with_extension("opt.wasm");
    let opt_flag = match level {
        WasmOptLevel::None => return Ok(input.to_path_buf()),
        WasmOptLevel::Os => "-Os",
        WasmOptLevel::Oz => "-Oz",
    };

    let result = Command::new("wasm-opt")
        .arg(opt_flag)
        .arg(input)
        .arg("-o")
        .arg(&output_path)
        .output();

    match result {
        Ok(output) if output.status.success() => Ok(output_path),
        Ok(output) => Err(StellarZkError::WasmOptFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )),
        Err(_) => {
            // wasm-opt not installed; skip optimization with a warning
            tracing::warn!("wasm-opt not found, skipping WASM optimization");
            Ok(input.to_path_buf())
        }
    }
}

/// Strip debug symbols and custom sections from a WASM file.
fn strip_wasm(input: &Path) -> Result<PathBuf> {
    let output_path = input.with_extension("stripped.wasm");

    let result = Command::new("wasm-strip")
        .arg(input)
        .arg("-o")
        .arg(&output_path)
        .output();

    match result {
        Ok(output) if output.status.success() => Ok(output_path),
        _ => {
            // wasm-strip not available; copy as-is
            tracing::warn!("wasm-strip not found, skipping symbol stripping");
            std::fs::copy(input, &output_path)?;
            Ok(output_path)
        }
    }
}
