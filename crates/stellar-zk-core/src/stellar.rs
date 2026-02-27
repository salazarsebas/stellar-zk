use std::path::Path;
use std::process::Command;

use crate::error::{Result, StellarZkError};

/// Wrapper around the `stellar` CLI for deploy, call, and simulate operations.
pub struct StellarCli {
    binary: String,
}

/// Result of simulating a contract call.
#[derive(Debug, Clone)]
pub struct SimulateResult {
    pub cpu_instructions: u64,
    pub memory_bytes: u64,
    pub resource_fee_stroops: u64,
    pub ledger_reads: u32,
    pub ledger_writes: u32,
}

impl StellarCli {
    /// Create a new wrapper, verifying the stellar CLI is installed.
    pub fn new() -> Result<Self> {
        which::which("stellar").map_err(|_| StellarZkError::MissingTool {
            name: "stellar".into(),
            install: "https://developers.stellar.org/docs/tools/cli".into(),
        })?;
        Ok(Self {
            binary: "stellar".into(),
        })
    }

    /// Deploy a WASM contract to the network.
    ///
    /// If `constructor_args` is non-empty, they are passed after `--` so the
    /// contract's `__constructor` is invoked at deploy time.
    pub async fn deploy(
        &self,
        wasm_path: &Path,
        network: &str,
        source: &str,
        constructor_args: &[(&str, &str)],
    ) -> Result<String> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(["contract", "deploy"])
            .arg("--wasm")
            .arg(wasm_path)
            .arg("--network")
            .arg(network)
            .arg("--source")
            .arg(source);

        if !constructor_args.is_empty() {
            cmd.arg("--");
            for (key, value) in constructor_args {
                cmd.arg(format!("--{key}")).arg(value);
            }
        }

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(StellarZkError::DeployFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let contract_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(contract_id)
    }

    /// Invoke a contract function on the network.
    pub async fn invoke(
        &self,
        contract_id: &str,
        function: &str,
        args: &[(&str, &str)],
        network: &str,
        source: &str,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(["contract", "invoke"])
            .arg("--id")
            .arg(contract_id)
            .arg("--network")
            .arg(network)
            .arg("--source")
            .arg(source)
            .arg("--");

        cmd.arg(function);
        for (key, value) in args {
            cmd.arg(format!("--{key}")).arg(value);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(StellarZkError::StellarCli(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Simulate a contract call to estimate resource usage.
    pub async fn simulate(
        &self,
        contract_id: &str,
        function: &str,
        args: &[(&str, &str)],
        network: &str,
    ) -> Result<SimulateResult> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(["contract", "invoke"])
            .arg("--id")
            .arg(contract_id)
            .arg("--network")
            .arg(network)
            .arg("--sim-only")
            .arg("--");

        cmd.arg(function);
        for (key, value) in args {
            cmd.arg(format!("--{key}")).arg(value);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(StellarZkError::StellarCli(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Parse simulation output (best-effort JSON parsing)
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("simulate output: {stdout}");

        // The stellar CLI --sim-only outputs JSON with resource metrics.
        // We parse what we can and fall back to zeros for missing fields.
        let sim: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

        Ok(SimulateResult {
            cpu_instructions: sim["cpu_insns"].as_u64().unwrap_or(0),
            memory_bytes: sim["mem_bytes"].as_u64().unwrap_or(0),
            resource_fee_stroops: sim["resource_fee"].as_u64().unwrap_or(0),
            ledger_reads: sim["read_bytes"].as_u64().unwrap_or(0) as u32,
            ledger_writes: sim["write_bytes"].as_u64().unwrap_or(0) as u32,
        })
    }
}
