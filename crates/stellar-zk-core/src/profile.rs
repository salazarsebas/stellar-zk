use serde::{Deserialize, Serialize};

/// WASM optimization level applied after cargo build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmOptLevel {
    /// No wasm-opt pass (development).
    None,
    /// Size optimization -Os (testnet).
    Os,
    /// Aggressive size optimization -Oz (production).
    Oz,
}

/// Build optimization profile for Stellar/Soroban contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationProfile {
    pub name: String,
    /// Cargo profile name: "dev", "release-testnet", or "release".
    pub cargo_profile: String,
    /// Rust opt-level: "0", "s", "z".
    pub opt_level: String,
    /// Enable link-time optimization.
    pub lto: bool,
    /// Strip symbols from WASM.
    pub strip_symbols: bool,
    /// Number of codegen units (1 = maximum LTO effectiveness).
    pub codegen_units: u32,
    /// WASM-opt optimization level.
    pub wasm_opt_level: WasmOptLevel,
    /// Keep overflow checks (always true for ZK — security critical).
    pub overflow_checks: bool,
    /// Enforce 64KB WASM size limit.
    pub enforce_size_limit: bool,
    /// Enforce 100M CPU instruction limit.
    pub enforce_cpu_limit: bool,
}

impl OptimizationProfile {
    /// Fast builds, no optimization. For local testing only.
    pub fn development() -> Self {
        Self {
            name: "development".into(),
            cargo_profile: "dev".into(),
            opt_level: "0".into(),
            lto: false,
            strip_symbols: false,
            codegen_units: 256,
            wasm_opt_level: WasmOptLevel::None,
            overflow_checks: true,
            enforce_size_limit: false,
            enforce_cpu_limit: false,
        }
    }

    /// Moderate optimization for testnet deployment.
    pub fn testnet() -> Self {
        Self {
            name: "testnet".into(),
            cargo_profile: "release".into(),
            opt_level: "s".into(),
            lto: true,
            strip_symbols: false,
            codegen_units: 1,
            wasm_opt_level: WasmOptLevel::Os,
            overflow_checks: true,
            enforce_size_limit: true,
            enforce_cpu_limit: false,
        }
    }

    /// Maximum optimization for mainnet. All limits enforced.
    pub fn stellar_production() -> Self {
        Self {
            name: "stellar-production".into(),
            cargo_profile: "release".into(),
            opt_level: "z".into(),
            lto: true,
            strip_symbols: true,
            codegen_units: 1,
            wasm_opt_level: WasmOptLevel::Oz,
            overflow_checks: true,
            enforce_size_limit: true,
            enforce_cpu_limit: true,
        }
    }

    /// Resolve a profile by name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "development" => Some(Self::development()),
            "testnet" => Some(Self::testnet()),
            "stellar-production" => Some(Self::stellar_production()),
            _ => None,
        }
    }

    /// Maximum WASM size in bytes (Soroban limit).
    pub const MAX_WASM_SIZE: u64 = 65_536;

    /// Maximum CPU instructions per transaction (Soroban limit).
    pub const MAX_CPU_INSTRUCTIONS: u64 = 100_000_000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name_valid_profiles() {
        assert!(OptimizationProfile::from_name("development").is_some());
        assert!(OptimizationProfile::from_name("testnet").is_some());
        assert!(OptimizationProfile::from_name("stellar-production").is_some());
    }

    #[test]
    fn test_from_name_invalid() {
        assert!(OptimizationProfile::from_name("invalid").is_none());
        assert!(OptimizationProfile::from_name("").is_none());
    }

    #[test]
    fn test_development_no_enforce_limits() {
        let dev = OptimizationProfile::development();
        assert!(!dev.enforce_size_limit);
        assert!(!dev.enforce_cpu_limit);
        assert!(dev.overflow_checks);
    }

    #[test]
    fn test_production_enforces_all() {
        let prod = OptimizationProfile::stellar_production();
        assert!(prod.enforce_size_limit);
        assert!(prod.enforce_cpu_limit);
        assert!(prod.overflow_checks);
        assert!(prod.lto);
        assert!(prod.strip_symbols);
    }
}
