use crate::backend::CostEstimate;
use crate::profile::OptimizationProfile;

/// Static cost estimates based on known backend characteristics.
/// These are Tier 1 (instant, offline) estimates.
pub fn static_estimate(backend: &str, num_public_inputs: u32) -> CostEstimate {
    match backend {
        "groth16" => groth16_estimate(num_public_inputs),
        "ultrahonk" => ultrahonk_estimate(num_public_inputs),
        "risc0" => risc0_estimate(),
        _ => CostEstimate {
            cpu_instructions: 0,
            memory_bytes: 0,
            wasm_size: 0,
            ledger_reads: 0,
            ledger_writes: 0,
            estimated_fee_stroops: 0,
            warnings: vec!["unknown backend".into()],
        },
    }
}

fn groth16_estimate(num_public_inputs: u32) -> CostEstimate {
    // BN254 Groth16 with host functions:
    // - 4 pairings (fixed)
    // - num_public_inputs g1_mul + g1_add operations
    // Base: ~10M instructions for pairing check
    // Per public input: ~0.5M instructions for g1_mul
    let base_cpu = 10_000_000u64;
    let per_input_cpu = 500_000u64;
    let cpu = base_cpu + (num_public_inputs as u64 * per_input_cpu);

    let mut warnings = Vec::new();
    if cpu > (OptimizationProfile::MAX_CPU_INSTRUCTIONS as f64 * 0.7) as u64 {
        warnings.push(format!(
            "CPU usage {cpu} is above 70% of the {}M limit",
            OptimizationProfile::MAX_CPU_INSTRUCTIONS / 1_000_000
        ));
    }
    if num_public_inputs > 20 {
        warnings.push(
            "many public inputs increase g1_mul cost — consider hashing inputs off-chain".into(),
        );
    }

    CostEstimate {
        cpu_instructions: cpu,
        memory_bytes: 500_000,
        wasm_size: 45_000,
        ledger_reads: 2,  // VK + nullifier check
        ledger_writes: 2, // nullifier + counter
        estimated_fee_stroops: estimate_fee(cpu),
        warnings,
    }
}

fn ultrahonk_estimate(num_public_inputs: u32) -> CostEstimate {
    // UltraHonk with host functions:
    // More complex than Groth16 (sumcheck + multiple MSMs)
    let cpu = 35_000_000u64 + (num_public_inputs as u64 * 200_000);

    let mut warnings = Vec::new();
    if cpu > (OptimizationProfile::MAX_CPU_INSTRUCTIONS as f64 * 0.7) as u64 {
        warnings.push(format!(
            "CPU usage {cpu} is above 70% of the {}M limit",
            OptimizationProfile::MAX_CPU_INSTRUCTIONS / 1_000_000
        ));
    }
    warnings.push(
        "UltraHonk verification is the most CPU-intensive backend — monitor limits closely".into(),
    );

    CostEstimate {
        cpu_instructions: cpu,
        memory_bytes: 2_000_000,
        wasm_size: 55_000,
        ledger_reads: 2,
        ledger_writes: 1,
        estimated_fee_stroops: estimate_fee(cpu),
        warnings,
    }
}

fn risc0_estimate() -> CostEstimate {
    // RISC Zero uses a Groth16 wrapper with fixed VK
    // Similar to Groth16 but with 2 fixed public inputs (image_id + journal_hash)
    let cpu = 15_000_000u64;

    CostEstimate {
        cpu_instructions: cpu,
        memory_bytes: 600_000,
        wasm_size: 48_000,
        ledger_reads: 2,
        ledger_writes: 2,
        estimated_fee_stroops: estimate_fee(cpu),
        warnings: vec![],
    }
}

/// Rough fee estimate in stroops based on CPU instructions.
fn estimate_fee(cpu_instructions: u64) -> u64 {
    // Approximation: resource fee scales roughly with CPU usage
    // Base fee ~100 stroops, plus ~1 stroop per 10K instructions
    100 + cpu_instructions / 10_000
}

/// Format a cost estimate as a human-readable report.
pub fn format_estimate(estimate: &CostEstimate, backend: &str) -> String {
    let cpu_pct =
        estimate.cpu_instructions as f64 / OptimizationProfile::MAX_CPU_INSTRUCTIONS as f64 * 100.0;
    let wasm_pct = estimate.wasm_size as f64 / OptimizationProfile::MAX_WASM_SIZE as f64 * 100.0;

    let cpu_status = if cpu_pct > 100.0 {
        "FAIL"
    } else if cpu_pct > 70.0 {
        "WARN"
    } else {
        "OK"
    };

    let wasm_status = if wasm_pct > 100.0 {
        "FAIL"
    } else if wasm_pct > 75.0 {
        "WARN"
    } else {
        "OK"
    };

    let fee_xlm = estimate.estimated_fee_stroops as f64 / 10_000_000.0;

    let mut report = format!(
        r#"
Cost Estimate: {backend}
============================================

Resource                Estimated        Limit         Usage
------------------------------------------------------------
CPU Instructions        {:<16} {:<13} {:.1}%  [{cpu_status}]
Memory                  {:<16} {:<13} -
WASM Size               {:<16} {:<13} {:.1}%  [{wasm_status}]
Ledger Reads            {:<16} {:<13} -
Ledger Writes           {:<16} {:<13} -

Estimated Fee: {fee_xlm:.4} XLM
"#,
        format_number(estimate.cpu_instructions),
        "100,000,000",
        cpu_pct,
        format_bytes(estimate.memory_bytes),
        "40 MB",
        format_bytes(estimate.wasm_size),
        "65,536",
        wasm_pct,
        estimate.ledger_reads,
        "40",
        estimate.ledger_writes,
        "20",
    );

    if !estimate.warnings.is_empty() {
        report.push_str("\nWarnings:\n");
        for w in &estimate.warnings {
            report.push_str(&format!("  * {w}\n"));
        }
    }

    report
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_bytes(n: u64) -> String {
    if n >= 1_048_576 {
        format!("{:.1} MB", n as f64 / 1_048_576.0)
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groth16_base_cost() {
        let est = static_estimate("groth16", 0);
        assert_eq!(est.cpu_instructions, 10_000_000);
    }

    #[test]
    fn test_groth16_scaling_with_inputs() {
        let est0 = static_estimate("groth16", 0);
        let est5 = static_estimate("groth16", 5);
        assert_eq!(est5.cpu_instructions, est0.cpu_instructions + 5 * 500_000);
    }

    #[test]
    fn test_groth16_warning_above_70_percent() {
        // Need enough inputs to push above 70M CPU (70% of 100M)
        // 10M base + N*0.5M > 70M → N > 120
        let est = static_estimate("groth16", 130);
        assert!(est.warnings.iter().any(|w| w.contains("70%")));
    }

    #[test]
    fn test_unknown_backend() {
        let est = static_estimate("plonky2", 0);
        assert_eq!(est.cpu_instructions, 0);
        assert!(est.warnings.iter().any(|w| w.contains("unknown backend")));
    }

    #[test]
    fn test_format_estimate_contains_backend() {
        let est = static_estimate("groth16", 1);
        let report = format_estimate(&est, "groth16");
        assert!(report.contains("groth16"));
    }
}
