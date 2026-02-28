# Copilot Instructions — stellar-zk

## Project Overview

stellar-zk is a Rust CLI toolkit for zero-knowledge proofs on Stellar/Soroban. It orchestrates three ZK backends (Groth16, UltraHonk, RISC Zero) through a unified CLI with 6 subcommands: init, build, prove, deploy, call, estimate.

## Workspace Structure

5-crate Cargo workspace:

- `crates/stellar-zk-cli/` — Binary. Clap CLI, factory pattern for backend selection.
- `crates/stellar-zk-core/` — Shared library. `ZkBackend` trait, config types, error enum, WASM pipeline, estimator, template engine.
- `crates/stellar-zk-groth16/` — Groth16 backend. Shells out to `circom` + `snarkjs`. BN254 serializer.
- `crates/stellar-zk-ultrahonk/` — UltraHonk backend. Shells out to `nargo` + `bb` (Barretenberg).
- `crates/stellar-zk-risc0/` — RISC Zero backend. Shells out to `cargo-risczero`. Seal validation.

## Build & Test

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

No external services or Docker needed for unit tests.

## Key Patterns

### Trait-based backends

All backends implement `ZkBackend` (defined in `core/src/backend.rs`). The trait is async and uses `async-trait`.

### Shell-out strategy (CRITICAL)

Backends call external tools via `std::process::Command` instead of linking as Rust dependencies. This is intentional — do NOT add heavy dependencies like `risc0-zkvm`, `ark-circom`, or `wasmer`.

### Template system

Templates live in `crates/stellar-zk-core/templates/` and are embedded at compile-time via `include_str!` in `core/src/templates/embedded.rs`. They use Handlebars syntax (`{{contract_name}}`, `{{project_name}}`).

### Factory pattern

`create_backend()` in `cli/src/commands/init.rs` selects the backend by name string.

### Artifact persistence

`target/build_artifacts.json` links build -> prove -> deploy -> call. Do not break this chain.

## BN254 Serialization Rules

- snarkjs outputs decimal strings; Soroban expects big-endian 32-byte field elements
- G2 points: snarkjs gives `[c0, c1]` but Soroban needs `c1 | c0` (higher-degree first)
- Proof layout: Groth16 = `A(G1:64) | B(G2:128) | C(G1:64)` = 256 bytes
- VK layout: `alpha(64) | beta(128) | gamma(128) | delta(128) | ic_count(4 BE) | IC[](64 each)`

## Do NOT

- Do NOT add heavy dependencies (`risc0-zkvm`, `ark-circom`, `wasmer`)
- Do NOT modify templates in `crates/stellar-zk-core/templates/` without updating `embedded.rs`
- Do NOT change byte order in serializers without updating BOTH the serializer AND the contract template
- Do NOT use `ark-circom` (depends on `wasmer-wasix`, broken on Rust 1.84+)
- Do NOT break the artifact chain (`build_artifacts.json`)
- Do NOT remove `overflow_checks` from profiles (security-critical)

## File Coupling

These files are tightly coupled and must be updated together:

- `crates/stellar-zk-core/templates/**/*.tmpl` <-> `crates/stellar-zk-core/src/templates/embedded.rs`
- `groth16/src/serializer.rs` <-> `crates/stellar-zk-core/templates/contracts/groth16_verifier/src/lib.rs.tmpl`
- `ultrahonk/src/serializer.rs` <-> `crates/stellar-zk-core/templates/contracts/ultrahonk_verifier/src/lib.rs.tmpl`
- `risc0/src/serializer.rs` <-> `crates/stellar-zk-core/templates/contracts/risc0_verifier/src/lib.rs.tmpl`
- `cli/src/main.rs` (BackendChoice) <-> `cli/src/commands/init.rs` (create_backend) <-> `core/src/config.rs` (BackendConfig)

## Code Style

- Error handling: `thiserror` for `StellarZkError`, `anyhow` at CLI boundary
- Async: `async-trait` for `ZkBackend` trait
- Serde: derive `Serialize`/`Deserialize` on all config and artifact types
- Doc comments: `///` for items, `//!` for module-level
