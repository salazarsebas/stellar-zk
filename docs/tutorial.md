# Tutorial: Your First ZK Proof on Stellar

This tutorial walks you through creating, proving, and verifying a zero-knowledge proof on the Stellar network using stellar-zk. By the end, you'll have a working ZK circuit, a generated proof, and a deployed Soroban verifier contract.

**What you'll build**: A Groth16 circuit that proves knowledge of a secret and salt that produce a known commitment — without revealing the secret itself. You'll then verify this proof on-chain using Stellar's native BN254 host functions.

**Time**: ~30 minutes (excluding tool installation)

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Part 1: Create the Project](#part-1-create-the-project)
- [Part 2: Build](#part-2-build)
- [Part 3: Generate a Proof](#part-3-generate-a-proof)
- [Part 4: Estimate Costs](#part-4-estimate-costs)
- [Part 5: Deploy to Testnet](#part-5-deploy-to-testnet)
- [Part 6: Verify On-Chain](#part-6-verify-on-chain)
- [Part 7: Modify the Circuit](#part-7-modify-the-circuit)
- [Part 8: Try Another Backend](#part-8-try-another-backend)
- [Next Steps](#next-steps)

---

## Prerequisites

### macOS

```bash
# Install Rust (1.85.0+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install Circom (circuit compiler)
git clone https://github.com/iden3/circom.git
cd circom
cargo build --release
cargo install --path circom
cd ..

# Install snarkjs (proof system)
npm install -g snarkjs

# Install Stellar CLI (for deploy/call)
cargo install --locked stellar-cli --features opt

# Install stellar-zk
curl -fsSL https://raw.githubusercontent.com/salazarsebas/stellar-zk/main/scripts/install.sh | bash
```

### Linux (Ubuntu/Debian)

```bash
# Install Rust (1.85.0+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install Node.js (required by snarkjs)
sudo apt update && sudo apt install -y nodejs npm

# Install Circom
git clone https://github.com/iden3/circom.git
cd circom
cargo build --release
cargo install --path circom
cd ..

# Install snarkjs
npm install -g snarkjs

# Install Stellar CLI
cargo install --locked stellar-cli --features opt

# Install stellar-zk
curl -fsSL https://raw.githubusercontent.com/salazarsebas/stellar-zk/main/scripts/install.sh | bash
```

### Verify installation

Run these commands to confirm everything is ready:

```bash
rustc --version        # Should be 1.85.0 or higher
circom --version       # Should print circom compiler version
snarkjs --version      # Should print snarkjs version
stellar --version      # Should print stellar CLI version
stellar-zk --help      # Should print stellar-zk usage
```

If any command is not found, check the [Troubleshooting guide](troubleshooting.md#installation).

---

## Part 1: Create the Project

### Initialize

Create a new project with the Groth16 backend:

```bash
stellar-zk init myapp --backend groth16
```

You should see output like:

```
✓ Created project directory: myapp
✓ Written config: stellar-zk.config.json
✓ Written backend config: backend.config.json
✓ Scaffolded circuit: circuits/main.circom
✓ Scaffolded contract: contracts/verifier/src/lib.rs
✓ Scaffolded inputs: inputs/input.json
✓ Project 'myapp' initialized with groth16 backend
```

Move into the project:

```bash
cd myapp
```

### Explore the project structure

```
myapp/
├── stellar-zk.config.json     # Project settings (backend, profile, paths)
├── backend.config.json        # Groth16-specific settings (curve, circuit power)
├── circuits/
│   └── main.circom            # Your ZK circuit
├── contracts/
│   └── verifier/
│       ├── Cargo.toml         # Soroban contract manifest
│       └── src/lib.rs         # Verifier contract (auto-generated)
├── inputs/
│   └── input.json             # Proof inputs (private + public)
└── proofs/                    # Will contain generated proofs
```

### Understand the circuit

Open `circuits/main.circom`:

```circom
pragma circom 2.1.0;

template MembershipProof() {
    // Private inputs — the prover knows these but they stay secret
    signal input secret;
    signal input salt;

    // Public inputs — visible to everyone (including the verifier contract)
    signal input commitment;

    // Constraint: commitment == secret * secret + salt
    signal secretSquared;
    secretSquared <== secret * secret;
    commitment === secretSquared + salt;
}

component main {public [commitment]} = MembershipProof();
```

This circuit proves: "I know a `secret` and `salt` such that `secret² + salt == commitment`" — without revealing `secret` or `salt`.

The `{public [commitment]}` declaration marks `commitment` as a public input. The verifier contract will check the proof against this public value. The `secret` and `salt` remain private — they're used during proof generation but never revealed.

> **Production note**: This uses a simplified algebraic relation. For real applications, replace `secret * secret + salt` with a cryptographic hash like Poseidon or MiMC.

### Understand the verifier contract

Open `contracts/verifier/src/lib.rs`. This is a Soroban smart contract that verifies Groth16 proofs using the BN254 elliptic curve host functions from Protocol 25. The key entry points are:

- **`__constructor(vk_bytes)`** — Called once at deployment. Stores the verification key.
- **`verify(proof, public_inputs, nullifier)`** — The main function. Deserializes the proof, reconstructs the pairing equation, and checks `e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1`.
- **`is_nullifier_used(nullifier)`** — Anti-replay check. Each proof can only be verified once.
- **`verify_count()`** — Returns how many proofs have been successfully verified.

You don't need to modify this contract — it's generated to match the Groth16 proof format exactly.

### Understand the inputs

Open `inputs/input.json`:

```json
{
  "secret": "42",
  "salt": "7",
  "commitment": "1771"
}
```

The values satisfy the circuit constraint: `42² + 7 = 1764 + 7 = 1771`. Note that inputs are strings — Circom expects decimal string representations of field elements.

---

## Part 2: Build

### Execute the build

```bash
stellar-zk build
```

This runs the full Groth16 build pipeline. You should see output similar to:

```
✓ Compiled circuit: circuits/main.circom
✓ Generated R1CS (1 constraints)
✓ Running trusted setup (Powers of Tau, power=14)...
✓ Generated proving key: target/circuit.zkey
✓ Serialized verification key: target/vk.bin
✓ Building Soroban contract...
✓ Contract WASM: target/wasm32v1-none/release/groth16_verifier.wasm
✓ Build artifacts saved to target/build_artifacts.json
```

### What happened?

The build pipeline performed these steps:

1. **Circuit compilation** — `circom` compiled `main.circom` into an R1CS (Rank-1 Constraint System) file and generated a WASM witness calculator.
2. **Trusted setup** — `snarkjs` ran a Powers of Tau ceremony and a circuit-specific phase-2 setup, producing a proving key (`circuit.zkey`) and verification key.
3. **VK serialization** — The verification key was serialized into the binary format expected by the Soroban contract: `alpha(64) | beta(128) | gamma(128) | delta(128) | ic_count(4) | IC[](64 each)`.
4. **Contract build** — `cargo build` compiled the Soroban verifier contract into WASM, then optimized it according to the current profile.

### Examine the artifacts

The build created `target/build_artifacts.json`, which links all subsequent commands:

```bash
cat target/build_artifacts.json
```

```json
{
  "circuit_artifact": "target/main.r1cs",
  "verifier_wasm": "target/wasm32v1-none/release/groth16_verifier.wasm",
  "proving_key": "target/circuit.zkey",
  "verification_key": "target/vk.bin"
}
```

Every stellar-zk command after `build` reads this file to locate the artifacts it needs — no manual path arguments required.

---

## Part 3: Generate a Proof

### Define the inputs

The default `inputs/input.json` already has valid inputs (`secret=42, salt=7, commitment=1771`). If you want to change them, make sure the values satisfy the circuit constraint. For example:

```json
{
  "secret": "10",
  "salt": "3",
  "commitment": "103"
}
```

This works because `10² + 3 = 103`.

### Execute the prover

```bash
stellar-zk prove --input inputs/input.json
```

Expected output:

```
✓ Computed witness from inputs
✓ Generated Groth16 proof
✓ Proof: proofs/proof.bin (256 bytes)
✓ Public inputs: proofs/public_inputs.json
```

### Inspect the output

The prover generated two files:

**`proofs/proof.bin`** — The 256-byte Groth16 proof in binary format:
- Bytes 0–63: Point `A` on G1 (two 32-byte coordinates)
- Bytes 64–191: Point `B` on G2 (four 32-byte coordinates)
- Bytes 192–255: Point `C` on G1 (two 32-byte coordinates)

**`proofs/public_inputs.json`** — The public inputs as hex-encoded 32-byte field elements:

```json
{
  "public_inputs": ["0x00000000000000000000000000000000000000000000000000000000000006eb"]
}
```

The hex value `0x6eb` = 1771 decimal — matching our `commitment` input.

---

## Part 4: Estimate Costs

Before deploying, check how much the verification will cost on-chain:

```bash
stellar-zk estimate
```

Example output:

```
Cost Estimation (Groth16)
─────────────────────────
Tier: artifact-based

CPU instructions:  ~12,000,000 (12% of 100M budget)
Memory:            ~500 KB
WASM size:         9,842 bytes (15% of 64 KB limit)
Ledger reads:      3
Ledger writes:     2
Estimated fee:     ~1,300 stroops

✓ Within all Soroban resource limits
```

**What the numbers mean:**

- **CPU instructions**: Groth16 verification costs ~12M instructions, well within Soroban's 100M limit. This is dominated by the BN254 pairing check.
- **WASM size**: The compiled contract is ~10 KB, far under the 64 KB limit.
- **Estimated fee**: ~1,300 stroops (0.00013 XLM) — very affordable.

> **Tip**: After deploying, you can run `estimate` again with `--contract-id` to get Tier 3 (simulation) estimates using actual on-chain resource metering.

---

## Part 5: Deploy to Testnet

### Create a Stellar identity

If you don't already have a Stellar identity, create one:

```bash
stellar keys generate alice --network testnet
```

### Fund the account

Fund the account with testnet XLM (free):

```bash
stellar keys fund alice --network testnet
```

### Deploy the contract

```bash
stellar-zk deploy --network testnet --source alice
```

Expected output:

```
✓ Uploading WASM to testnet...
✓ Deploying contract...
✓ Initializing with verification key (VK: 580 bytes)
✓ Contract deployed!
  Contract ID: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABCDE
  Network: testnet
```

**Save the Contract ID** — you'll need it for the next step.

The deploy command uploaded the WASM binary, deployed a new contract instance, and called `__constructor(vk_bytes)` to initialize it with the serialized verification key.

---

## Part 6: Verify On-Chain

### Call the contract

```bash
stellar-zk call \
  --contract-id CAAAA...BCDE \
  --proof proofs/proof.bin \
  --network testnet \
  --source alice
```

Expected output:

```
✓ Loaded proof: 256 bytes
✓ Loaded public inputs: 1 field element
✓ Computed nullifier: SHA256(proof || public_inputs)
✓ Calling verify() on CAAAA...BCDE...
✓ Verification successful!
  Result: true
  Nullifier: 0xa1b2c3d4...
  TX hash: abc123...
```

### What happened on-chain?

The verifier contract executed these steps inside a single Soroban transaction:

1. **Input validation** — Checked that the proof is exactly 256 bytes and public inputs are aligned to 32-byte elements.
2. **Deserialization** — Parsed the G1/G2 points from the proof bytes and the verification key from storage.
3. **Pairing equation** — Computed `vk_x = IC[0] + commitment * IC[1]`, then checked the equation `e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1` using the native BN254 `pairing_check` host function.
4. **Anti-replay** — Stored the nullifier (`SHA256(proof || public_inputs)`) in persistent storage, preventing this exact proof from being verified again.
5. **Event emission** — Emitted a `verified` event and incremented the verification counter.

> **Try it again**: Running the same `call` command a second time will fail with `NullifierAlreadyUsed` — this is the anti-replay protection working correctly.

---

## Part 7: Modify the Circuit

Now that you've seen the full workflow, let's modify the circuit to prove something different.

### Change to a range check circuit

Replace the content of `circuits/main.circom` with a range check — proving a value lies within a range without revealing it:

```circom
pragma circom 2.1.0;

// Prove that a secret value is between 0 and 2^n - 1
// without revealing the value itself.

template RangeCheck(n) {
    signal input value;       // Private: the secret value
    signal input commitment;  // Public: a commitment to the value

    // Verify commitment = value * value (simplified binding)
    signal valueSquared;
    valueSquared <== value * value;
    commitment === valueSquared;

    // Decompose value into n bits to prove 0 <= value < 2^n
    signal bits[n];
    var sum = 0;
    for (var i = 0; i < n; i++) {
        bits[i] <-- (value >> i) & 1;
        bits[i] * (1 - bits[i]) === 0;  // Each bit is 0 or 1
        sum += bits[i] * (1 << i);
    }
    value === sum;  // Bits reconstruct the original value
}

component main {public [commitment]} = RangeCheck(8);  // 8-bit range: 0-255
```

Update `inputs/input.json` to match:

```json
{
  "value": "42",
  "commitment": "1764"
}
```

(42² = 1764, and 42 fits in 8 bits.)

### Rebuild and re-prove

```bash
stellar-zk build
stellar-zk prove --input inputs/input.json
```

The new proof verifies that you know a value whose square is 1764, and that value fits within 8 bits (0–255) — without revealing the value itself.

To verify this new proof on-chain, you'll need to re-deploy (since the verification key changed with the new circuit):

```bash
stellar-zk deploy --network testnet --source alice
stellar-zk call --contract-id <NEW_CONTRACT_ID> --proof proofs/proof.bin --source alice
```

---

## Part 8: Try Another Backend

stellar-zk supports three backends. Here's how to get started with the other two.

### UltraHonk (Noir)

UltraHonk uses [Noir](https://noir-lang.org/), a Rust-inspired ZK DSL. No trusted setup required.

**Prerequisites**: Install nargo and bb:

```bash
# Install nargo (Noir compiler)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup

# Install bb (Barretenberg prover)
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash
bbup
```

**Create a project**:

```bash
stellar-zk init myapp-noir --backend ultrahonk
cd myapp-noir
```

The circuit lives in `circuits/src/main.nr`:

```noir
fn main(secret: Field, salt: Field, commitment: pub Field) {
    let computed = secret * secret + salt;
    assert(computed == commitment);
}
```

Same logic as the Circom circuit, but in Noir syntax. The `pub` keyword marks public inputs.

The workflow is identical:

```bash
stellar-zk build
stellar-zk prove --input inputs/input.json
stellar-zk deploy --network testnet --source alice
stellar-zk call --contract-id <ID> --proof proofs/proof.bin --source alice
```

**Trade-offs**: UltraHonk proofs are larger (~14 KB vs 256 bytes) and cost more CPU (~35M instructions), but Noir is often more ergonomic for complex circuits and doesn't need a trusted setup.

### RISC Zero (Rust)

RISC Zero proves execution of arbitrary Rust programs inside a zkVM.

**Prerequisites**: Install cargo-risczero and Docker:

```bash
# Install rzup (RISC Zero toolchain manager)
curl -L https://risczero.com/install | bash
rzup install

# Docker is required for Groth16 proof wrapping
# Install from https://docs.docker.com/get-docker/
```

**Create a project**:

```bash
stellar-zk init myapp-risc0 --backend risc0
cd myapp-risc0
```

The project structure is different — instead of `circuits/`, you have:

```
programs/
├── guest/
│   ├── Cargo.toml
│   └── src/main.rs    # Runs inside the zkVM
└── host/
    ├── Cargo.toml
    └── src/main.rs     # Drives the zkVM and writes output
```

The guest program (`programs/guest/src/main.rs`) reads private inputs and commits public outputs:

```rust
#![no_main]
#![no_std]
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let secret: u64 = env::read();
    let salt: u64 = env::read();
    let commitment = secret.wrapping_mul(secret).wrapping_add(salt);
    env::commit(&commitment);
}
```

The workflow is the same:

```bash
stellar-zk build
stellar-zk prove --input inputs/input.json
stellar-zk deploy --network testnet --source alice
stellar-zk call --contract-id <ID> --proof proofs/proof.bin --source alice
```

**Trade-offs**: RISC Zero lets you write circuits in standard Rust — no new language to learn. Proofs are small (260 bytes) and affordable (~15M CPU). However, builds are slower (compiling for RISC-V target) and Docker is required for Groth16 wrapping.

---

## Next Steps

You now have a working ZK verification pipeline on Stellar. Here's where to go from here:

- **[USAGE.md](../USAGE.md)** — Complete reference for all CLI commands, configuration, and workflows
- **[README.md](../README.md)** — Architecture overview, security model, backend comparison
- **[Troubleshooting](troubleshooting.md)** — Solutions for common errors and FAQ
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** — How to contribute to stellar-zk

### Ideas to explore

- Replace the example circuit with a real use case: anonymous voting, private attestations, or Merkle proof verification
- Use the `stellar-production` profile to enforce Soroban's resource limits during build
- Write unit tests for your circuit using snarkjs or nargo's built-in test framework
- Run `stellar-zk estimate --contract-id <ID> --source alice` after deploy for Tier 3 simulation-based cost estimates
