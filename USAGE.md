# Usage Guide — stellar-zk

## What is stellar-zk?

stellar-zk is a CLI toolkit that lets you build, prove, and verify zero-knowledge proofs on [Stellar/Soroban](https://soroban.stellar.org/). It supports three proving systems (Groth16, UltraHonk, RISC Zero) and handles the full lifecycle: circuit compilation, trusted setup, proof generation, contract deployment, and on-chain verification. Built for Stellar Protocol 25's native BN254 host functions.

---

## Prerequisites

### All backends

- **Rust 1.85.0+** — install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Stellar CLI** — for deploy and call commands:
  ```bash
  cargo install --locked stellar-cli
  ```

### Groth16 (Circom + snarkjs)

```bash
# Install Circom
git clone https://github.com/iden3/circom.git
cd circom && cargo build --release
sudo cp target/release/circom /usr/local/bin/

# Install snarkjs (requires Node.js)
npm install -g snarkjs
```

### UltraHonk (Noir + Barretenberg)

```bash
# Install Noir toolchain
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup

# Install Barretenberg backend
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash
bbup
```

### RISC Zero

```bash
# Install cargo-risczero
curl -L https://risczero.com/install | bash
rzup install

# Docker is required for Groth16 proof wrapping
# Install from https://docs.docker.com/get-docker/
```

---

## Installation

```bash
git clone https://github.com/salazarsebas/stellar-zk.git
cd stellar-zk
cargo install --path crates/stellar-zk-cli
```

Verify:

```bash
stellar-zk --help
```

Expected output:

```
ZK DevKit for Stellar/Soroban — Groth16 + UltraHonk + RISC Zero

Usage: stellar-zk [OPTIONS] <COMMAND>

Commands:
  init      Initialize a new ZK project
  build     Build the ZK circuit/program and Soroban verifier contract
  prove     Generate a ZK proof
  deploy    Deploy the verifier contract to Stellar
  call      Call the deployed contract with a proof
  estimate  Estimate execution costs for on-chain verification
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>  Path to stellar-zk.config.json [default: stellar-zk.config.json]
  -v, --verbose...       Verbosity level (-v, -vv, -vvv)
  -h, --help             Print help
  -V, --version          Print version
```

---

## Tutorial: Your First ZK Proof on Stellar

This walkthrough uses the **Groth16** backend to create a simple circuit, generate a proof, and verify it on Stellar testnet.

### Step 1: Initialize a project

```bash
stellar-zk init myapp --backend groth16
cd myapp
```

This creates the following structure:

```
myapp/
├── stellar-zk.config.json     # Project configuration
├── backend.config.json        # Groth16-specific settings
├── circuits/
│   └── main.circom            # Your circuit (simple example)
├── contracts/
│   └── verifier/
│       ├── Cargo.toml         # Soroban contract manifest
│       └── src/lib.rs         # Groth16 verifier contract
├── inputs/
│   └── input.json             # Proof inputs
├── proofs/                    # Generated proofs (after prove)
└── target/                    # Build artifacts (after build)
```

### Step 2: Understand the circuit

Open `circuits/main.circom`. The starter circuit is a simple multiplier:

```circom
pragma circom 2.0.0;

template Multiplier() {
    signal input a;
    signal input b;
    signal output c;

    c <== a * b;
}

component main {public [a]} = Multiplier();
```

This proves: "I know a secret `b` such that `a * b = c`", where `a` is public (visible to the verifier) and `b` is private (known only to the prover).

### Step 3: Provide inputs

Edit `inputs/input.json`:

```json
{
  "a": "3",
  "b": "7"
}
```

The prover will prove knowledge of `b = 7` such that `3 * 7 = 21`.

### Step 4: Build

```bash
stellar-zk build
```

This runs the full build pipeline:
1. Compiles the Circom circuit to R1CS
2. Runs the Powers of Tau ceremony (development mode, auto-generated)
3. Generates the proving key (`zkey`) and verification key
4. Serializes the VK to Soroban-compatible binary format
5. Compiles the Soroban verifier contract to WASM

Artifacts are saved to `target/build_artifacts.json`.

### Step 5: Generate a proof

```bash
stellar-zk prove --input inputs/input.json
```

This:
1. Computes the witness from your inputs
2. Generates a Groth16 proof (256 bytes: `A | B | C` points on BN254)
3. Extracts public inputs as 32-byte big-endian field elements
4. Writes `proofs/proof.bin` and `proofs/public_inputs.json`

### Step 6: Estimate costs

```bash
stellar-zk estimate
```

Shows estimated on-chain resources:
- CPU instructions (~12M for Groth16)
- WASM contract size
- Estimated fee in stroops

### Step 7: Deploy to testnet

First, configure a Stellar testnet identity:

```bash
stellar keys generate alice --network testnet --fund
```

Then deploy:

```bash
stellar-zk deploy --network testnet --source alice
```

The contract is deployed with the verification key initialized via the constructor. Note the contract ID in the output (e.g., `CXYZ...`).

### Step 8: Verify on-chain

```bash
stellar-zk call \
  --contract-id CXYZ... \
  --proof proofs/proof.bin \
  --network testnet \
  --source alice
```

The CLI:
1. Reads the proof and public inputs
2. Computes a nullifier: `SHA256(proof || public_inputs)`
3. Calls `verify(proof, public_inputs, nullifier)` on the contract
4. The contract runs the BN254 pairing check on-chain
5. Returns `true` if verification succeeds

---

## Choosing a Backend

| | Groth16 | UltraHonk | RISC Zero |
|---|---------|-----------|-----------|
| **Language** | Circom | Noir | Rust |
| **Proof size** | 256 bytes | ~14 KB | ~260 bytes |
| **On-chain CPU** | ~12M instructions | ~35M instructions | ~15M instructions |
| **Trusted setup** | Yes (per-circuit) | No | No |
| **WASM size** | ~10 KB | ~50 KB | ~10 KB |
| **Best for** | Simple proofs, lowest cost | Complex logic, modern DSL | Arbitrary Rust programs |

**Choose Groth16** if you want the smallest proof and lowest verification cost, and your circuit is relatively simple. Requires a trusted setup (auto-generated in dev mode; use a ceremony for production).

**Choose UltraHonk** if you want a modern circuit language (Noir) with no trusted setup. Good for more complex applications, but proof size and verification cost are higher.

**Choose RISC Zero** if you want to prove arbitrary Rust computation. The guest program runs in a RISC-V zkVM, producing a STARK that's wrapped into a Groth16 seal. Requires Docker for the wrapping step.

### Try another backend

```bash
stellar-zk init myapp-noir --backend ultrahonk
stellar-zk init myapp-risc0 --backend risc0
```

---

## Optimization Profiles

| Setting | `development` | `testnet` | `stellar-production` |
|---------|--------------|---------|-------------------|
| Cargo opt-level | 0 | "s" | "z" |
| LTO | off | thin | full |
| wasm-opt | skip | -Os | -Oz |
| Symbol stripping | no | no | yes |
| WASM size limit | none | 64 KB | 64 KB |
| CPU limit check | no | no | yes (100M) |

**`development`** — Fast compile, no optimization. Use during circuit development and testing.

**`testnet`** — Balanced optimization. Use for testnet deployment and integration testing.

**`stellar-production`** — Maximum optimization, all Soroban limits enforced. Use for mainnet deployment. Will fail at build time if WASM exceeds 64 KB or estimated CPU exceeds 100M instructions.

Override the project default:

```bash
stellar-zk build --profile stellar-production
```

---

## Configuration Reference

### `stellar-zk.config.json`

Generated by `init`. Read by all other commands.

```jsonc
{
  "version": "0.1.0",              // Config schema version
  "project_name": "myapp",         // Project name
  "backend": "groth16",            // "groth16" | "ultrahonk" | "risc0"
  "profile": "development",        // "development" | "testnet" | "stellar-production"
  "circuit": {
    "entry_point": "circuits/main.circom",  // Circuit source file
    "input_file": "inputs/input.json"       // Default input file
  },
  "contract": {
    "name": "groth16_verifier",                            // Contract name
    "source_dir": "contracts/verifier",                    // Contract source directory
    "wasm_output": "target/wasm32v1-none/release/groth16_verifier.wasm"  // Compiled WASM path
  },
  "deploy": {
    "network": "testnet",           // "local" | "testnet" | "mainnet"
    "source_identity": "default"    // Stellar identity for signing
  }
}
```

### `backend.config.json`

Backend-specific settings. Only the section matching the chosen backend is populated.

**Groth16**:
```jsonc
{
  "backend": "groth16",
  "groth16": {
    "curve": "bn254",            // Elliptic curve (only bn254 supported)
    "trusted_setup": null,       // Path to ceremony file (null = auto-generate)
    "circuit_power": 14          // Powers of Tau size (2^14 = 16K constraints)
  }
}
```

**UltraHonk**:
```jsonc
{
  "backend": "ultrahonk",
  "ultrahonk": {
    "oracle_hash": "keccak",     // Hash function for Fiat-Shamir
    "recursive": false           // Enable recursive proof composition
  }
}
```

**RISC Zero**:
```jsonc
{
  "backend": "risc0",
  "risc0": {
    "guest_target": "riscv32im-risc0-zkvm-elf",  // Compilation target
    "segment_limit_po2": 20,                       // Segment size (2^20)
    "groth16_wrap": true                           // Wrap STARK proof in Groth16 seal
  }
}
```

---

## CLI Command Reference

### `stellar-zk init <name>`

Create a new ZK project.

```bash
stellar-zk init myapp --backend groth16 --profile development
```

| Flag | Default | Values |
|------|---------|--------|
| `--backend` | *(interactive)* | `groth16`, `ultrahonk`, `risc0` |
| `--profile` | `development` | `development`, `testnet`, `stellar-production` |

### `stellar-zk build`

Compile circuit and build verifier contract WASM.

```bash
stellar-zk build --profile testnet
```

| Flag | Default | Description |
|------|---------|-------------|
| `--profile` | from config | Override optimization profile |
| `--circuit-only` | `false` | Only compile the circuit |
| `--contract-only` | `false` | Only build the WASM contract |

### `stellar-zk prove`

Generate a proof from inputs.

```bash
stellar-zk prove --input inputs/input.json --output proofs/my_proof.bin
```

| Flag | Default | Description |
|------|---------|-------------|
| `--input`, `-i` | *(required)* | Path to input JSON |
| `--output`, `-o` | auto | Output path for proof |

### `stellar-zk deploy`

Deploy the verifier contract.

```bash
stellar-zk deploy --network testnet --source alice
```

| Flag | Default | Description |
|------|---------|-------------|
| `--network` | `testnet` | `local`, `testnet`, `mainnet` |
| `--source` | *(required)* | Stellar identity name |

### `stellar-zk call`

Invoke the on-chain verifier with a proof.

```bash
stellar-zk call --contract-id CXYZ... --proof proofs/proof.bin --source alice
```

| Flag | Default | Description |
|------|---------|-------------|
| `--contract-id` | *(required)* | Deployed contract address |
| `--proof` | *(required)* | Path to proof binary |
| `--public-inputs` | auto | Path to public inputs JSON |
| `--network` | `testnet` | Target network |
| `--source` | *(required)* | Stellar identity |

### `stellar-zk estimate`

Estimate on-chain verification costs.

```bash
stellar-zk estimate --public-inputs 2
```

| Flag | Default | Description |
|------|---------|-------------|
| `--proof` | *(optional)* | Path to proof file (enables artifact-based estimate) |
| `--public-inputs` | `2` | Number of public inputs (for static estimation) |
| `--network` | `testnet` | Network for simulation |

Three estimation tiers:
1. **Static** (always available) — baseline cost model per backend
2. **Artifact** (after `build`) — uses actual WASM size
3. **Simulation** (after `deploy`) — runs on-chain simulation for real resource usage

### Global flags

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to config file (default: `stellar-zk.config.json`) |
| `-v`, `-vv`, `-vvv` | Increase log verbosity (info, debug, trace) |

---

## Troubleshooting

### "required tool 'circom' not found"

Install the prerequisite tools for your backend. See [Prerequisites](#prerequisites).

```bash
# Groth16
circom --version && snarkjs --version

# UltraHonk
nargo --version && bb --version

# RISC Zero
cargo risczero --version
```

### "config file not found at stellar-zk.config.json"

You're not in a stellar-zk project directory. Run commands from inside the project, or use `--config` to point to your config file:

```bash
cd myapp
stellar-zk build

# or
stellar-zk build --config /path/to/myapp/stellar-zk.config.json
```

### "not a stellar-zk project"

Run `stellar-zk init` first to create a project:

```bash
stellar-zk init myapp --backend groth16
```

### "build artifacts not found"

Run `build` before `prove`, `deploy`, or `estimate`:

```bash
stellar-zk build
stellar-zk prove --input inputs/input.json
```

### "WASM too large: X bytes (max 65536)"

Your compiled contract exceeds Soroban's 64 KB limit. Options:

1. Use a more aggressive profile:
   ```bash
   stellar-zk build --profile stellar-production
   ```

2. Simplify the circuit to reduce constraint count

3. Check that `wasm-opt` is installed (used by `testnet` and `stellar-production` profiles):
   ```bash
   cargo install wasm-opt
   ```

### "circuit compilation failed"

Check your circuit source for syntax errors:
- **Circom**: verify `pragma circom 2.0.0;` and signal declarations
- **Noir**: verify `fn main()` signature and type annotations
- **RISC Zero**: check `guest/src/main.rs` compiles standalone with `cargo build`

### "proof generation failed"

Common causes:
- Input JSON doesn't match circuit's expected signals
- Input values are out of range for the BN254 scalar field
- Build artifacts are stale — re-run `stellar-zk build`

### "deployment failed"

- Verify your Stellar identity has funds: `stellar keys fund alice --network testnet`
- Check network connectivity: `stellar network ls`
- Ensure the WASM file exists in `target/`

---

## Next Steps

- [Tutorial](docs/tutorial.md) — step-by-step guide to your first ZK proof on Stellar (start here if you're new)
- [Troubleshooting & FAQ](docs/troubleshooting.md) — solutions for common errors and frequently asked questions
- [README.md](README.md) — full architecture overview, security model, and backend details
- [CONTRIBUTING.md](CONTRIBUTING.md) — development setup and contribution guidelines
- [ROADMAP.md](ROADMAP.md) — planned features and milestones
- [SECURITY.md](SECURITY.md) — vulnerability reporting
