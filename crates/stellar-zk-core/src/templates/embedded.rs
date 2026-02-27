//! Compile-time embedded templates for project scaffolding.
//!
//! Each constant loads a template file from `templates/` via [`include_str!`]. The paths
//! are relative to this source file (`crates/stellar-zk-core/src/templates/embedded.rs`).
//!
//! ## Adding a new template
//!
//! 1. Place the template file under the appropriate `templates/` subdirectory
//! 2. Add a `pub const` here with `include_str!("../../../../templates/<path>")`
//! 3. Use the constant in the backend's `init_project()` implementation
//! 4. Run `cargo build` — if the path is wrong, compilation will fail
//!
//! ## Warning
//!
//! Do NOT rename or move template files without updating the `include_str!` path here.
//! Do NOT modify template files without checking that the Handlebars variables still match
//! what the renderer passes in.

// -------------------------------------------------------
// Circuit / program starter templates
// -------------------------------------------------------

pub const GROTH16_CIRCUIT: &str = include_str!("../../../../templates/circuits/groth16/example.circom");
pub const ULTRAHONK_CIRCUIT: &str = include_str!("../../../../templates/circuits/ultrahonk/src/main.nr");
pub const ULTRAHONK_NARGO_TOML: &str = include_str!("../../../../templates/circuits/ultrahonk/Nargo.toml");
pub const RISC0_HOST: &str = include_str!("../../../../templates/circuits/risc0/host/src/main.rs");
pub const RISC0_GUEST: &str = include_str!("../../../../templates/circuits/risc0/guest/src/main.rs");
pub const RISC0_GUEST_CARGO_TOML: &str = include_str!("../../../../templates/circuits/risc0/guest/Cargo.toml");
pub const RISC0_HOST_CARGO_TOML: &str = include_str!("../../../../templates/circuits/risc0/host/Cargo.toml");

// -------------------------------------------------------
// Verifier contract templates
// -------------------------------------------------------

pub const GROTH16_CONTRACT_CARGO: &str = include_str!("../../../../templates/contracts/groth16_verifier/Cargo.toml.tmpl");
pub const GROTH16_CONTRACT_LIB: &str = include_str!("../../../../templates/contracts/groth16_verifier/src/lib.rs.tmpl");

pub const ULTRAHONK_CONTRACT_CARGO: &str = include_str!("../../../../templates/contracts/ultrahonk_verifier/Cargo.toml.tmpl");
pub const ULTRAHONK_CONTRACT_LIB: &str = include_str!("../../../../templates/contracts/ultrahonk_verifier/src/lib.rs.tmpl");

pub const RISC0_CONTRACT_CARGO: &str = include_str!("../../../../templates/contracts/risc0_verifier/Cargo.toml.tmpl");
pub const RISC0_CONTRACT_LIB: &str = include_str!("../../../../templates/contracts/risc0_verifier/src/lib.rs.tmpl");

// -------------------------------------------------------
// Config templates
// -------------------------------------------------------

pub const INPUT_JSON: &str = include_str!("../../../../templates/config/input.json.tmpl");
