# Changelog

All notable changes to stellar-zk will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - Unreleased

### Added

- **Workspace**: 5-crate architecture (cli, core, groth16, ultrahonk, risc0)
- **CLI**: 6 subcommands — `init`, `build`, `prove`, `deploy`, `call`, `estimate`
- **ZkBackend trait**: extensible backend system for pluggable proving systems
- **Groth16 backend**: full integration with Circom + snarkjs
  - Circuit compilation via `circom`
  - Trusted setup via `snarkjs groth16 setup`
  - Witness generation via `snarkjs wtns calculate`
  - Proof generation via `snarkjs groth16 prove`
  - BN254 serialization to Soroban-compatible big-endian format (256-byte proofs)
- **UltraHonk backend** (stub): nargo + Barretenberg orchestration scaffolding
- **RISC Zero backend** (stub): cargo-risczero orchestration scaffolding
- **Contract templates**: Soroban verifier contracts for all 3 backends with nullifier tracking, verification counter, and event emission
- **Circuit templates**: example circuits for Circom, Noir, and RISC Zero guest programs
- **Configuration system**: `stellar-zk.config.json` + `backend.config.json`
- **Optimization profiles**: development, testnet, stellar-production
- **Cost estimator**: static cost models per backend (CPU, memory, WASM size, fees)
- **WASM pipeline**: build + optimize + strip + size validation
- **Template system**: Handlebars rendering with `include_str!` embedding
- **Stellar CLI wrapper**: deploy, invoke, and simulate via `stellar` CLI
- **Build artifact persistence**: `build_artifacts.json` links build/prove/deploy/call commands
- **Deploy with VK initialization**: `__constructor(vk_bytes)` called at deploy time
- **Call with full verification args**: proof + public_inputs + SHA256-based nullifier
- **WASM pipeline integration**: all 3 backends use `pipeline::build_and_optimize()` instead of placeholders
- **Cost estimator Tier 2**: uses actual WASM file size from build artifacts
- **Simulate output parsing**: best-effort JSON parsing of `stellar --sim-only` response
- **UltraHonk backend**: full integration with nargo + Barretenberg
  - `Nargo.toml` template for scaffolded projects
  - Off-chain proof verification via `bb verify_ultra_honk`
  - Oracle hash read from config (not hardcoded)
  - `public_inputs.json` output matching Groth16 pattern
  - KZG pairing-check on-chain verifier contract via BN254 host functions
  - 6 unit tests for proof format parsing
- **RISC Zero backend**: full integration with cargo-risczero + host binary orchestration
  - `Cargo.toml` templates for guest and host programs (scaffolded projects can now build)
  - Host template writes structured output (seal.bin, journal.bin, image_id.hex)
  - `build()` compiles guest ELF + host binary, caches config for prove()
  - `prove()` shells out to host binary, validates seal, computes journal digest
  - `public_inputs.json` output matching Groth16/UltraHonk pattern
  - Real Groth16 pairing-check on-chain verifier contract via BN254 host functions
  - Seal serializer with selector validation
  - 5 unit tests for seal serialization and validation
