# stellar-zk

**ZK DevKit for Stellar/Soroban** — unified CLI for Groth16, UltraHonk, and RISC Zero.

[![CI](https://github.com/stellar-zk/stellar-zk/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-zk/stellar-zk/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.84%2B-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

The easiest way to build, prove, and verify zero-knowledge proofs on the [Stellar](https://stellar.org/) network. stellar-zk orchestrates ZK proof systems for [Soroban](https://soroban.stellar.org/) smart contracts — handling circuit compilation, trusted setup, proof generation, contract deployment, and on-chain verification so you can focus on your ZK application logic.

Built for Stellar Protocol 25, which introduced native BN254 host functions (`g1_add`, `g1_mul`, `g1_neg`, `fr_from_bytes`, `pairing_check`) enabling on-chain ZK verification within the 100M CPU instruction budget. Groth16 verification costs ~12M CPU instructions (~1,100 stroops) — just 12% of the budget.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Backends](#backends)
- [CLI Reference](#cli-reference)
- [Prerequisites](#prerequisites)
- [Project Structure](#project-structure)
- [Workflow](#workflow)
- [Optimization Profiles](#optimization-profiles)
- [On-Chain Verification](#on-chain-verification)
- [Architecture](#architecture)
- [Configuration](#configuration)
- [Cost Estimation](#cost-estimation)
- [Security Model](#security-model)
- [License](#license)

---

## Features

- **Three backends**: Groth16 (Circom), UltraHonk (Noir), RISC Zero (Rust zkVM)
- **Full lifecycle CLI**: `init` -> `build` -> `prove` -> `deploy` -> `call` -> `estimate`
- **Soroban-optimized**: enforces 64 KB WASM limit and 100M CPU instruction budget
- **Contract templates**: generated Soroban verifier contracts with BN254 pairing checks, nullifier tracking, and structured events
- **Three-tier cost estimation**: static models, artifact-based, and on-chain simulation
- **Optimization profiles**: development (fast builds), testnet (balanced), stellar-production (aggressive)
- **Artifact persistence**: build artifacts link all CLI commands without manual path wiring
- **Anti-replay protection**: SHA256-based nullifiers prevent double verification

---

## Quick Start

```bash
# Install from source
cargo install --path crates/stellar-zk-cli

# Create a new project with Groth16
stellar-zk init myapp --backend groth16
cd myapp

# Build circuit + verifier contract
stellar-zk build

# Generate a proof
stellar-zk prove --input inputs/input.json

# Estimate on-chain costs
stellar-zk estimate

# Deploy to testnet
stellar-zk deploy --network testnet --source alice

# Verify the proof on-chain
stellar-zk call --contract-id CXYZ... --proof proofs/proof.bin --source alice
```

---

## Installation

### From source

```bash
git clone https://github.com/stellar-zk/stellar-zk.git
cd stellar-zk
cargo install --path crates/stellar-zk-cli
```

### Verify installation

```bash
stellar-zk --help
```

### Requirements

- **Rust 1.84.0+** — install via [rustup](https://rustup.rs/)
- **Stellar CLI** — for deploy/call commands ([installation guide](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli))

Backend-specific prerequisites are listed in the [Prerequisites](#prerequisites) section below.

---

## Backends

stellar-zk supports three proving systems, each with different trade-offs:

| | Groth16 | UltraHonk | RISC Zero |
|---|---------|-----------|-----------|
| **Circuit language** | Circom | Noir | Rust |
| **Proof size** | 256 bytes | ~14 KB | ~260 bytes |
| **On-chain CPU** | ~12M instructions | ~35M instructions | ~15M instructions |
| **Trusted setup** | Yes (per-circuit) | No (universal SRS) | No (universal) |
| **WASM size** | ~10 KB | ~50 KB | ~10 KB |
| **Verification** | BN254 pairing check | Sumcheck + MSM | BN254 pairing check (Groth16 seal) |
| **Best for** | Simple proofs, lowest cost | Modern ZK apps, complex logic | Arbitrary Rust computation |

### Groth16 (Circom)

The most cost-efficient option for simple circuits. Proof generation uses [Circom](https://docs.circom.io/) for circuit definition and [snarkjs](https://github.com/iden3/snarkjs) for key generation, witness computation, and proving. Requires a per-circuit trusted setup (Powers of Tau ceremony), which is generated automatically in development mode.

**Proof format**: 256 bytes — `A(64) | B(128) | C(64)` (G1, G2, G1 points on BN254)

### UltraHonk (Noir)

A modern proving system with no trusted setup, using a universal Structured Reference String (SRS). Circuits are written in [Noir](https://noir-lang.org/), a Rust-inspired DSL. Proof generation uses [nargo](https://noir-lang.org/docs/getting_started/noir_installation/) for compilation/execution and [Barretenberg](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg) (`bb`) for proving/verification.

**Proof format**: ~14 KB (sumcheck proofs + commitments)

### RISC Zero (zkVM)

Proves execution of arbitrary Rust programs. A guest program runs inside the RISC-V zkVM, producing a STARK proof that is wrapped into a ~260-byte Groth16 seal for on-chain verification. Uses [cargo-risczero](https://risczero.com/docs) for building guest programs and Docker for Groth16 wrapping.

**Proof format**: 260 bytes — `selector(4) | Groth16_proof(256)` (selector identifies the circuit version)

---

## CLI Reference

### `stellar-zk init`

Scaffold a new ZK project with circuit and contract templates.

```bash
stellar-zk init <name> [--backend <backend>] [--profile <profile>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--backend` | *(interactive)* | `groth16`, `ultrahonk`, or `risc0` |
| `--profile` | `development` | `development`, `testnet`, or `stellar-production` |

Creates the project directory with:
- Circuit/program source files
- Soroban verifier contract template
- Configuration files (`stellar-zk.config.json`, `backend.config.json`)
- Input template (`inputs/input.json`)
- Cargo.toml files (for Rust-based backends)

### `stellar-zk build`

Compile the circuit and generate the verifier contract WASM.

```bash
stellar-zk build [--profile <profile>] [--circuit-only] [--contract-only]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--profile` | from config | Override optimization profile |
| `--circuit-only` | `false` | Only compile the circuit |
| `--contract-only` | `false` | Only build the WASM contract |

What happens per backend:
- **Groth16**: compiles Circom circuit, generates R1CS, runs trusted setup, serializes VK, builds WASM
- **UltraHonk**: compiles Noir circuit with nargo, generates VK with bb, builds WASM
- **RISC Zero**: compiles guest ELF, builds host binary, caches config, builds WASM

Outputs are saved to `target/build_artifacts.json` for use by subsequent commands.

### `stellar-zk prove`

Generate a proof from input data.

```bash
stellar-zk prove --input <file> [--output <file>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--input` | *(required)* | Path to input JSON file |
| `--output` | auto | Path for proof output |

Outputs:
- `proofs/proof.bin` or `proofs/receipt.bin` — the proof bytes
- `proofs/public_inputs.json` — hex-encoded public inputs for the `call` command

### `stellar-zk deploy`

Deploy the verifier contract to a Stellar network.

```bash
stellar-zk deploy [--network <network>] --source <identity>
```

| Flag | Default | Description |
|------|---------|-------------|
| `--network` | `testnet` | `local`, `testnet`, or `mainnet` |
| `--source` | *(required)* | Stellar identity for signing |

The contract is deployed with the verification key passed to `__constructor(vk_bytes)`, initializing it for on-chain verification.

### `stellar-zk call`

Invoke the deployed verifier contract with a proof.

```bash
stellar-zk call --contract-id <id> --proof <file> [--public-inputs <file>] [--network <network>] --source <identity>
```

| Flag | Default | Description |
|------|---------|-------------|
| `--contract-id` | *(required)* | Deployed contract address |
| `--proof` | *(required)* | Path to proof binary |
| `--public-inputs` | auto | Path to public inputs (auto-detected from `proofs/public_inputs.json`) |
| `--network` | `testnet` | Target network |
| `--source` | *(required)* | Stellar identity |

Computes `nullifier = SHA256(proof || public_inputs)` and calls `verify(proof, public_inputs, nullifier)`.

### `stellar-zk estimate`

Estimate on-chain verification costs.

```bash
stellar-zk estimate [--network <network>] [--source <identity>] [--contract-id <id>]
```

Three tiers of estimation:
1. **Static**: offline cost models per backend (always available)
2. **Artifact**: uses actual WASM size from build output (after `build`)
3. **Simulation**: runs `stellar --sim-only` on the deployed contract (after `deploy`)

### Global flags

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to config file (default: `stellar-zk.config.json`) |
| `-v`, `-vv`, `-vvv` | Increase log verbosity (info, debug, trace) |

---

## Prerequisites

### All backends

- [Rust](https://rustup.rs/) 1.84.0+
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (for deploy and call commands)

### Groth16

- [Circom](https://docs.circom.io/getting-started/installation/) — circuit compiler
- [snarkjs](https://github.com/iden3/snarkjs) — `npm install -g snarkjs`
- [Node.js](https://nodejs.org/) — required by snarkjs

### UltraHonk

- [nargo](https://noir-lang.org/docs/getting_started/noir_installation/) — Noir toolchain (`noirup`)
- [bb](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg) — Barretenberg backend (`bbup`)

### RISC Zero

- [cargo-risczero](https://risczero.com/docs) — `curl -L https://risczero.com/install | bash && rzup install`
- [Docker](https://docs.docker.com/get-docker/) — needed for Groth16 proof wrapping

---

## Project Structure

### Workspace layout

```
stellar-zk/
├── Cargo.toml                          # Workspace manifest
├── crates/
│   ├── stellar-zk-cli/                 # Binary entry point (clap)
│   │   └── src/
│   │       ├── main.rs                 # CLI definition + subcommand dispatch
│   │       ├── output.rs               # Terminal formatting (colors, progress)
│   │       └── commands/               # Subcommand implementations
│   │           ├── init.rs             # Project scaffolding + backend factory
│   │           ├── build.rs            # Circuit compile + WASM build
│   │           ├── prove.rs            # Proof generation
│   │           ├── deploy.rs           # Contract deployment via stellar CLI
│   │           ├── call.rs             # Contract invocation + nullifier
│   │           └── estimate.rs         # Cost estimation (3 tiers)
│   │
│   ├── stellar-zk-core/                # Shared library
│   │   └── src/
│   │       ├── backend.rs              # ZkBackend trait + types
│   │       ├── config.rs               # ProjectConfig + BackendConfig
│   │       ├── error.rs                # StellarZkError enum
│   │       ├── estimator.rs            # Static cost models per backend
│   │       ├── pipeline.rs             # WASM build pipeline (cargo -> wasm-opt -> strip)
│   │       ├── profile.rs              # OptimizationProfile (3 presets)
│   │       ├── project.rs              # Directory creation, config I/O
│   │       ├── artifacts.rs            # BuildArtifacts persistence (JSON)
│   │       ├── stellar.rs              # Stellar CLI wrapper (deploy, invoke, simulate)
│   │       └── templates/
│   │           ├── embedded.rs         # include_str! constants for all templates
│   │           └── renderer.rs         # Handlebars template engine
│   │
│   ├── stellar-zk-groth16/             # Groth16 backend
│   │   └── src/
│   │       ├── lib.rs                  # Groth16Backend (ZkBackend impl)
│   │       ├── circuit.rs              # Circom compilation wrapper
│   │       ├── prover.rs               # snarkjs keygen + prove
│   │       └── serializer.rs           # JSON decimal strings -> big-endian bytes
│   │
│   ├── stellar-zk-ultrahonk/           # UltraHonk backend
│   │   └── src/
│   │       ├── lib.rs                  # UltraHonkBackend (ZkBackend impl)
│   │       ├── nargo.rs                # nargo + bb CLI wrappers
│   │       ├── proof_convert.rs        # Extract public inputs from proof
│   │       └── serializer.rs           # Proof format utilities
│   │
│   └── stellar-zk-risc0/               # RISC Zero backend
│       └── src/
│           ├── lib.rs                  # Risc0Backend (ZkBackend impl)
│           ├── guest.rs                # Build guest ELF + host binary
│           ├── prover.rs               # Shell out to host binary
│           └── serializer.rs           # Seal validation (selector + length)
│
└── templates/                           # Embedded via include_str!
    ├── circuits/
    │   ├── groth16/example.circom       # Starter Circom circuit
    │   ├── ultrahonk/                   # Nargo.toml + main.nr
    │   └── risc0/                       # guest/ + host/ (Cargo.toml + main.rs)
    ├── contracts/
    │   ├── groth16_verifier/            # Soroban contract (Groth16 pairing check)
    │   ├── ultrahonk_verifier/          # Soroban contract (KZG pairing check)
    │   └── risc0_verifier/              # Soroban contract (Groth16 seal verification)
    └── config/
        └── input.json.tmpl              # Starter input file
```

### Scaffolded project layout

When you run `stellar-zk init myapp --backend groth16`, the generated project looks like:

```
myapp/
├── stellar-zk.config.json     # Project configuration
├── backend.config.json        # Backend-specific settings
├── circuits/
│   └── main.circom            # Your circuit (Groth16)
├── contracts/
│   └── verifier/
│       ├── Cargo.toml         # Soroban contract manifest
│       └── src/lib.rs         # Verifier contract
├── inputs/
│   └── input.json             # Proof inputs
├── proofs/                    # Generated proofs (after prove)
└── target/                    # Build artifacts (after build)
```

For RISC Zero, `circuits/` is replaced by `programs/guest/` and `programs/host/`.

---

## Workflow

The standard workflow follows a linear pipeline where each command produces artifacts consumed by the next:

```
init --> build --> prove --> deploy --> call
                    |                    |
                    +--- estimate -------+
```

### 1. Initialize (`init`)

Creates the project directory, writes configuration files, scaffolds circuit/program templates and verifier contract.

### 2. Build (`build`)

Compiles the circuit or program and generates the verifier contract WASM. Saves `build_artifacts.json` to `target/` so subsequent commands can locate compiled artifacts without manual path arguments.

Per backend:
- **Groth16**: `circom` compile -> Powers of Tau -> `snarkjs groth16 setup` -> VK serialization -> WASM pipeline
- **UltraHonk**: `nargo compile` -> `bb write_vk` -> WASM pipeline
- **RISC Zero**: `cargo build` guest -> `cargo build` host -> cache config -> WASM pipeline

### 3. Prove (`prove`)

Generates a zero-knowledge proof from the provided inputs. Writes the proof binary and `public_inputs.json` (hex-encoded field elements).

Per backend:
- **Groth16**: `snarkjs wtns calculate` -> `snarkjs groth16 prove` -> serialize to 256 bytes
- **UltraHonk**: `nargo execute` -> `bb prove_ultra_honk` -> `bb verify_ultra_honk` (off-chain check)
- **RISC Zero**: run host binary -> read seal/journal/image_id -> validate seal -> compute journal digest

### 4. Deploy (`deploy`)

Uploads the compiled WASM to a Stellar network and deploys the contract. The verification key is passed to the `__constructor` so it is available for all subsequent `verify()` calls.

### 5. Call (`call`)

Invokes the on-chain `verify()` function with the proof, public inputs, and a computed nullifier. The nullifier (`SHA256(proof || public_inputs)`) prevents the same proof from being verified twice.

### 6. Estimate (`estimate`)

Reports estimated on-chain costs. Can be run at any point:
- After `build`: includes actual WASM file size
- After `deploy`: can run on-chain simulation for real resource usage

---

## Optimization Profiles

Three preset profiles control compilation and optimization behavior:

| Setting | development | testnet | stellar-production |
|---------|------------|---------|-------------------|
| Cargo opt-level | 0 | "s" | "z" |
| LTO | off | thin | full |
| wasm-opt | skip | -Os | -Oz |
| Symbol stripping | no | no | yes |
| WASM size limit | none | 64 KB | 64 KB |
| CPU limit check | no | no | yes (100M) |

Use `--profile` to override the project default:

```bash
stellar-zk build --profile stellar-production
```

### Soroban resource limits

- **Max WASM size**: 64 KB (65,536 bytes)
- **Max CPU instructions per tx**: 100,000,000 (100M)
- **Max memory**: varies by network configuration

The `stellar-production` profile enforces all limits at build time and will fail if the contract exceeds them.

---

## On-Chain Verification

All three backends generate Soroban smart contracts that verify proofs using the BN254 elliptic curve host functions introduced in Protocol 25.

### Verifier contract interface

Every generated verifier contract exposes the same interface:

```rust
// Initialize with the verification key (called at deploy time)
fn __constructor(env: Env, vk_bytes: Bytes);

// Verify a proof (the main entry point)
fn verify(
    env: Env,
    proof: Bytes,           // Backend-specific proof bytes
    public_inputs: Bytes,   // Concatenated 32-byte field elements
    nullifier: BytesN<32>,  // Anti-replay token
) -> Result<bool, VerifierError>;

// Query nullifier status
fn is_nullifier_used(env: Env, nullifier: BytesN<32>) -> bool;

// Get total successful verifications
fn verify_count(env: Env) -> u64;
```

### Verification logic

**Groth16 and RISC Zero** use the standard Groth16 pairing check:

```
e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1
```

Where `vk_x = IC[0] + sum(public_input[i] * IC[i+1])` is computed from the public inputs and the verification key's IC (Input Commitment) points.

**UltraHonk** uses a KZG-based pairing check appropriate for the UltraHonk verification scheme.

### VK format (Groth16 / RISC Zero)

```
alpha(64 bytes, G1) | beta(128 bytes, G2) | gamma(128 bytes, G2) | delta(128 bytes, G2) | ic_count(4 bytes, big-endian u32) | IC[0..n](64 bytes each, G1)
```

All points are serialized as big-endian 32-byte coordinates (x, y for G1; x_re, x_im, y_re, y_im for G2).

### Security features

- **Nullifier tracking**: each proof can only be verified once (stored in persistent storage)
- **Event emission**: every successful verification emits a `verified` event
- **Verification counter**: contracts track the total number of successful verifications
- **Input validation**: proof length, public input alignment, and selector checks (RISC Zero) are enforced before any cryptographic operations

---

## Architecture

### Crate dependency graph

```
                    ┌──────────────────┐
                    │  stellar-zk-cli  │
                    │    (binary)      │
                    └────────┬─────────┘
                             │ depends on
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
   │  stellar-zk  │ │  stellar-zk  │ │  stellar-zk  │
   │   -groth16   │ │  -ultrahonk  │ │    -risc0    │
   └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
          │                │                │
          │  impl ZkBackend trait           │
          │                │                │
          └────────────────┼────────────────┘
                           │
                           ▼
                  ┌──────────────────┐
                  │  stellar-zk-core │
                  │  (shared types,  │
                  │   traits, config)│
                  └──────────────────┘
```

### Data flow pipeline

```
  init              build              prove             deploy            call
┌──────┐         ┌──────────┐       ┌──────────┐      ┌──────────┐     ┌──────────┐
│Create│         │ Compile  │       │ Generate │      │ Upload   │     │ Invoke   │
│ dirs,│────────▶│ circuit, │──────▶│  proof   │─────▶│ WASM +   │────▶│ verify() │
│config│         │build WASM│       │  bytes   │      │ deploy   │     │ on-chain │
└──────┘         └──────────┘       └──────────┘      └──────────┘     └──────────┘
                      │                  │                                  │
                      ▼                  ▼                                  ▼
               build_artifacts     proof.bin +                        TX result +
                   .json          public_inputs                        nullifier
                                     .json
                                                    estimate
                                              ┌─────────────────┐
                                              │ Static/Artifact/│
                                              │   Simulation    │
                                              └─────────────────┘
```

### Backend trait

stellar-zk uses a trait-based backend system. Each proving system implements [`ZkBackend`](crates/stellar-zk-core/src/backend.rs):

```rust
#[async_trait]
pub trait ZkBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn check_prerequisites(&self) -> Result<(), Vec<PrerequisiteError>>;
    async fn init_project(&self, dir: &Path, config: &ProjectConfig) -> Result<()>;
    async fn build(&self, dir: &Path, config: &BackendConfig, profile: &OptimizationProfile) -> Result<BuildArtifacts>;
    async fn prove(&self, dir: &Path, artifacts: &BuildArtifacts, input: &Path) -> Result<ProofArtifacts>;
    async fn estimate_cost(&self, dir: &Path, proof: &ProofArtifacts, build: &BuildArtifacts) -> Result<CostEstimate>;
}
```

### Shell-out strategy

All backends shell out to external toolchains rather than linking them as Rust dependencies:

- **Groth16**: `circom` (circuit compilation), `snarkjs` via Node.js (keygen, witness, proving)
- **UltraHonk**: `nargo` (compile, execute), `bb` (VK generation, prove, verify)
- **RISC Zero**: `cargo build` (guest/host compilation), host binary (proof generation)

This avoids heavy dependency trees (e.g., `risc0-zkvm` is ~300 MB) and compilation issues (e.g., `wasmer-wasix` doesn't compile on Rust 1.84+).

### Key types

```rust
// Output of build step
pub struct BuildArtifacts {
    pub circuit_artifact: PathBuf,      // R1CS, ACIR, or ELF
    pub verifier_wasm: PathBuf,         // Compiled Soroban contract
    pub proving_key: Option<PathBuf>,   // zkey (Groth16 only)
    pub verification_key: PathBuf,      // Binary format for on-chain
}

// Output of prove step
pub struct ProofArtifacts {
    pub proof: Vec<u8>,                 // Soroban-compatible proof bytes
    pub public_inputs: Vec<[u8; 32]>,   // 32-byte big-endian field elements
    pub proof_path: PathBuf,            // Disk location
}

// Cost estimation result
pub struct CostEstimate {
    pub cpu_instructions: u64,
    pub memory_bytes: u64,
    pub wasm_size: u64,
    pub ledger_reads: u32,
    pub ledger_writes: u32,
    pub estimated_fee_stroops: u64,
    pub warnings: Vec<String>,
}
```

### WASM pipeline

The build pipeline compiles and optimizes the Soroban verifier contract:

1. `cargo build` with the appropriate `--release` or `--dev` profile
2. `wasm-opt` optimization (if configured by the profile)
3. `wasm-strip` to remove debug symbols (production only)
4. Size validation against the 64 KB limit

### Adding a new backend

1. Create a new crate: `crates/stellar-zk-<name>/`
2. Implement the `ZkBackend` trait
3. Add circuit and contract templates to `templates/`
4. Register in the CLI factory (`init.rs::create_backend`)
5. Add to the `BackendChoice` enum in `main.rs`
6. Add cost model to `estimator.rs`

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## Configuration

### `stellar-zk.config.json`

Project-level configuration generated by `init`:

```json
{
  "version": "0.1.0",
  "project_name": "myapp",
  "backend": "groth16",
  "profile": "development",
  "circuit": {
    "entry_point": "circuits/main.circom",
    "input_file": "inputs/input.json"
  },
  "contract": {
    "name": "groth16_verifier",
    "source_dir": "contracts/verifier",
    "wasm_output": "target/wasm32v1-none/release/groth16_verifier.wasm"
  },
  "deploy": {
    "network": "testnet",
    "source_identity": "default"
  }
}
```

### `backend.config.json`

Backend-specific settings:

**Groth16**:
```json
{
  "backend": "groth16",
  "groth16": {
    "curve": "bn254",
    "trusted_setup": null,
    "circuit_power": 14
  }
}
```

**UltraHonk**:
```json
{
  "backend": "ultrahonk",
  "ultrahonk": {
    "oracle_hash": "keccak",
    "recursive": false
  }
}
```

**RISC Zero**:
```json
{
  "backend": "risc0",
  "risc0": {
    "guest_target": "riscv32im-risc0-zkvm-elf",
    "segment_limit_po2": 20,
    "groth16_wrap": true
  }
}
```

---

## Cost Estimation

The `estimate` command provides three tiers of increasingly accurate cost data:

### Tier 1: Static models (offline)

Baseline estimates per backend derived from the BN254 operation costs:

| Backend | Base CPU | Per-input CPU | WASM size | Memory |
|---------|----------|---------------|-----------|--------|
| Groth16 | 10M | +500K | ~45 KB | 500 KB |
| UltraHonk | 35M | +200K | ~55 KB | 2 MB |
| RISC Zero | 15M | fixed | ~48 KB | 600 KB |

Fee estimation: `stroops = 100 + (cpu_instructions / 10,000)`

### Tier 2: Artifact-based (after build)

Replaces the estimated WASM size with the actual compiled contract size from build output.

### Tier 3: Simulation (after deploy)

Runs `stellar contract invoke --sim-only` against the deployed contract to get real resource usage from the network.

---

## Security Model

### On-chain guarantees

- **Cryptographic verification**: all proofs are verified via BN254 pairing checks using Soroban's native host functions — no custom elliptic curve arithmetic in WASM
- **Nullifier tracking**: prevents double-spending by storing `SHA256(proof || public_inputs)` in persistent contract storage
- **Selector validation** (RISC Zero): verifies the 4-byte seal prefix matches the expected Groth16 circuit version

### Trust assumptions

- **Groth16**: requires a trusted setup (Powers of Tau ceremony). In development mode, a local ceremony is generated automatically. For production, use a community-generated ceremony file
- **UltraHonk**: no trusted setup — uses a universal SRS
- **RISC Zero**: no trusted setup — uses a universal verification key

### External tool security

stellar-zk shells out to external tools (`circom`, `snarkjs`, `nargo`, `bb`, `cargo-risczero`). The security of generated proofs depends on the correctness of these tools. Always use official releases from their respective repositories.

---

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
