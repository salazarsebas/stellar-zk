# Roadmap

This document outlines the development roadmap for stellar-zk. Items are organized by phase, with completed phases marked accordingly.

---

## Completed

### Phase 1: Foundation

- [x] Cargo workspace with 5 crates (cli, core, groth16, ultrahonk, risc0)
- [x] `ZkBackend` trait with async methods for init, build, prove, estimate
- [x] CLI with 6 subcommands: `init`, `build`, `prove`, `deploy`, `call`, `estimate`
- [x] Configuration system (`stellar-zk.config.json` + `backend.config.json`)
- [x] Three optimization profiles (development, testnet, stellar-production)
- [x] Template engine (Handlebars) with embedded templates via `include_str!`
- [x] Project scaffolding with directory creation and template rendering
- [x] Static cost estimation models per backend

### Phase 2: Groth16 Backend

- [x] Circom circuit compilation via `circom` CLI
- [x] Powers of Tau generation for development
- [x] Trusted setup via `snarkjs groth16 setup`
- [x] Witness generation via `snarkjs wtns calculate`
- [x] Proof generation via `snarkjs groth16 prove`
- [x] BN254 serializer: snarkjs JSON (decimal strings) to Soroban binary (big-endian bytes)
- [x] VK serialization to binary format (alpha, beta, gamma, delta, IC points)
- [x] Soroban verifier contract with full pairing check
- [x] 9 unit tests for serialization

### Phase 3: Deploy, Call, and Estimate Pipeline

- [x] Build artifact persistence (`target/build_artifacts.json`)
- [x] WASM pipeline: `cargo build` -> `wasm-opt` -> `wasm-strip` -> size validation
- [x] Deploy with VK initialization via `__constructor(vk_bytes)`
- [x] Call with full verification args (proof, public_inputs, nullifier)
- [x] SHA256-based nullifier computation
- [x] Simulate output parsing for Tier 3 cost estimation
- [x] Stellar CLI wrapper (deploy, invoke, simulate)

### Phase 4: UltraHonk Backend

- [x] Noir circuit compilation via `nargo compile`
- [x] VK generation via `bb write_vk`
- [x] Witness generation via `nargo execute`
- [x] Proof generation via `bb prove_ultra_honk`
- [x] Off-chain verification via `bb verify_ultra_honk`
- [x] Oracle hash read from config (cached for prove)
- [x] `Nargo.toml` template for scaffolded projects
- [x] `public_inputs.json` output matching Groth16 pattern
- [x] KZG pairing-check on-chain verifier contract
- [x] 6 unit tests for proof format parsing

### Phase 5: RISC Zero Backend

- [x] `Cargo.toml` templates for guest and host programs
- [x] Host template with structured output (seal.bin, journal.bin, image_id.hex)
- [x] Guest and host binary compilation (`build_guest`, `build_host`)
- [x] Proof generation via shell-out to host binary
- [x] Seal validation (260 bytes, selector check)
- [x] Journal digest computation (SHA256)
- [x] `public_inputs.json` output matching other backends
- [x] Groth16 pairing-check on-chain verifier contract (with selector validation)
- [x] Build config caching for prove step
- [x] 5 unit tests for seal serialization and validation

---

### Phase 6: Testing and Hardening

- [x] Integration tests for the full `init -> build -> prove` pipeline (all 3 backends)
- [x] Edge case handling in serializers (overflow, malformed inputs)
- [x] Error message improvements with actionable recovery suggestions
- [x] CI pipeline (GitHub Actions) with build + test + clippy + fmt checks
- [x] Cross-platform testing (Linux, macOS)
- [x] Prerequisite version checking (minimum versions for circom, snarkjs, nargo, bb, cargo-risczero)
- [x] Graceful handling of missing external tools mid-pipeline (not just at init)

---

## Planned

### Phase 7: Developer Experience

Polish the CLI and make the tool easier to use for newcomers.

- [ ] Interactive `prove` command (prompt for inputs if not provided)
- [ ] `stellar-zk status` command showing project state (built? proved? deployed?)
- [ ] Progress bars for long-running operations (circuit compilation, proof generation)
- [ ] Colored diff output for cost estimation (show changes between runs)
- [ ] `stellar-zk verify` command for local off-chain proof verification
- [ ] `stellar-zk clean` command to remove build artifacts
- [ ] Better error messages when external tools produce unexpected output
- [ ] Shell completions (bash, zsh, fish)

### Phase 8: Production Readiness

Features required for mainnet deployments.

- [ ] Production trusted setup support (import community Powers of Tau files for Groth16)
- [ ] VK extraction from RISC Zero host at build time (replace placeholder with real universal VK)
- [ ] Contract upgrade support (versioned VKs, migration paths)
- [ ] Gas profiling: detailed breakdown of BN254 operation costs per verify() call
- [ ] WASM size optimization reports (which functions contribute most to size)
- [ ] Audit-ready contract templates (formal comments, invariant documentation)
- [ ] Deterministic builds (pinned tool versions, reproducible WASM output)

### Phase 9: Advanced Proving Features

Extend the backends with advanced capabilities.

- [ ] Recursive proof composition (prove verification of a proof)
- [ ] Batch verification (verify multiple proofs in a single transaction)
- [ ] Custom circuit support for UltraHonk (user-defined oracle functions)
- [ ] RISC Zero continuation support (segment proofs for long computations)
- [ ] Groth16 proof aggregation (SnarkPack or similar)
- [ ] Witness generation from on-chain data (Stellar ledger queries as circuit inputs)
- [ ] Private input management (encrypted input storage, key derivation)

### Phase 10: Ecosystem Integration

Connect stellar-zk with the broader Stellar and ZK ecosystems.

- [ ] npm/npx package for easy installation (`npx stellar-zk init myapp`)
- [ ] VS Code extension (syntax highlighting for circuit files, inline cost estimates)
- [ ] Soroban SDK integration (Rust helper library for calling verifier contracts)
- [ ] Proof relay service (submit proofs without running a Stellar node)
- [ ] Explorer integration (link verified proofs to Stellar transaction history)
- [ ] Template marketplace (community-contributed circuit templates)
- [ ] Multi-chain support (deploy verifiers to other chains with BN254 support)

### Phase 11: Performance and Optimization

Push the boundaries of what fits within Soroban's resource limits.

- [ ] Custom WASM optimization passes for verifier contracts
- [ ] Precomputed pairing values for fixed VKs (reduce on-chain CPU)
- [ ] Compressed proof formats (where protocol allows)
- [ ] Parallel proof generation (multi-threaded witness computation)
- [ ] Incremental compilation (only rebuild changed circuit components)
- [ ] Proof caching (skip re-proving if inputs haven't changed)

---

## Future Considerations

These are ideas being evaluated but not yet committed to:

- **New backends**: Plonky2, Halo2, SP1 — as Soroban adds new host functions or precompiles
- **zkLogin**: Stellar account abstraction using ZK proofs of OAuth/OIDC tokens
- **zkBridge**: Cross-chain light client verification using ZK proofs of block headers
- **Privacy primitives**: Shielded transfers, confidential assets built on the verifier infrastructure
- **Formal verification**: Machine-checked correctness proofs for the verifier contract templates
- **Hardware acceleration**: GPU/FPGA support for proof generation via backend plugins

---

## Versioning

stellar-zk follows [Semantic Versioning](https://semver.org/):

- **0.1.x**: Current development. API may change between minor versions.
- **0.2.0**: Target for Phase 7 completion (developer experience polish).
- **1.0.0**: Target for Phase 8 completion (production readiness). Stable API commitment.

---

## Contributing

Want to help? See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines. The most impactful areas for contribution right now are:

1. **Integration tests** (Phase 6) — help us test the full pipeline
2. **CI pipeline** (Phase 6) — GitHub Actions setup
3. **Developer experience** (Phase 7) — CLI polish, error messages, shell completions
4. **New backend exploration** — prototype implementations for Plonky2, Halo2, or SP1
