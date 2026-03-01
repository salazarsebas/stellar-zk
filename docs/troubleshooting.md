# Troubleshooting & FAQ

Common issues and their solutions when using stellar-zk.

---

## Table of Contents

- [Installation](#installation)
- [Groth16 (Circom + snarkjs)](#groth16-circom--snarkjs)
- [UltraHonk (Noir + Barretenberg)](#ultrahonk-noir--barretenberg)
- [RISC Zero](#risc-zero)
- [Build](#build)
- [Deploy & Call](#deploy--call)
- [General](#general)
- [FAQ](#faq)

---

## Installation

### Rust version incompatible (< 1.85.0)

**Error**: `error: package 'stellar-zk v0.1.x' requires rustc 1.85.0` or `getrandom` / `edition2024` errors.

**Fix**: Update Rust:

```bash
rustup update stable
rustc --version  # Verify 1.85.0+
```

stellar-zk requires Rust 1.85.0+ due to `getrandom` 0.4.1 which uses edition 2024 features.

### Install script fails

**Error**: `No release found` or `unsupported platform`.

**Cause**: The install script downloads pre-built binaries from GitHub Releases. If no release exists yet or your platform isn't supported, it will fail.

**Fix**: Install from source instead:

```bash
cargo install stellar-zk
# or
git clone https://github.com/salazarsebas/stellar-zk.git
cd stellar-zk
cargo install --path crates/stellar-zk-cli
```

**Supported platforms** for pre-built binaries:
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- macOS x86_64 (`x86_64-apple-darwin`)
- macOS ARM64 (`aarch64-apple-darwin`)
- Windows x86_64 (`x86_64-pc-windows-msvc`)

### `cargo install` fails on Windows

**Error**: Linker errors or compilation failures on Windows.

**Fix**: Ensure you have the Visual Studio Build Tools installed:

1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install the "Desktop development with C++" workload
3. Retry `cargo install stellar-zk`

### PATH not configured after installation

**Error**: `stellar-zk: command not found` after installing.

**Fix**: The binary is installed to `~/.cargo/bin/`. Add it to your PATH:

```bash
# bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# zsh
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

For the install script, the binary is placed in `~/.stellar-zk/bin/`. Add that to PATH:

```bash
echo 'export PATH="$HOME/.stellar-zk/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## Groth16 (Circom + snarkjs)

### "required tool 'circom' not found"

**Cause**: Circom is not installed or not in PATH.

**Fix**: Install Circom from source:

```bash
git clone https://github.com/iden3/circom.git
cd circom
cargo build --release
cargo install --path circom
```

Verify: `circom --version`

### "required tool 'snarkjs' not found"

**Cause**: snarkjs is not installed globally.

**Fix**:

```bash
npm install -g snarkjs
```

Verify: `snarkjs --version`

> **Note**: snarkjs requires Node.js. If `node` is not found, install it from [nodejs.org](https://nodejs.org/) or via your package manager.

### Circuit compilation errors

**Common Circom syntax issues:**

| Error | Cause | Fix |
|-------|-------|-----|
| `Expected 'signal'` | Missing semicolon | Add `;` at end of statement |
| `Undeclared signal` | Typo in signal name | Check signal declaration matches usage |
| `Non-quadratic constraint` | Using non-linear operation in `===` | Break into intermediate signals using `<==` |
| `Template not found` | Wrong include path | Check `include` statements and file paths |

**Example of non-quadratic constraint error:**

```circom
// BAD — a * b * c is degree 3 (non-quadratic)
signal output result;
result === a * b * c;

// GOOD — break into degree-2 steps
signal ab;
ab <== a * b;
result <== ab * c;
```

### "constraint system too large for ptau"

**Error**: `Error: The circuit has more constraints than the Powers of Tau supports`

**Cause**: Your circuit has more constraints than `2^circuit_power`. The default is `circuit_power: 14`, supporting up to 16,384 constraints.

**Fix**: Increase `circuit_power` in `backend.config.json`:

```json
{
  "groth16": {
    "circuit_power": 18
  }
}
```

This supports up to 262,144 constraints. Higher values increase setup time and proving key size.

### "witness generation failed: input signal not found"

**Cause**: The signals in `inputs/input.json` don't match what the circuit declares.

**Fix**: Ensure every `signal input` in your circuit has a corresponding key in the JSON:

```circom
// Circuit declares:
signal input secret;
signal input salt;
signal input commitment;
```

```json
// input.json must have all three:
{
  "secret": "42",
  "salt": "7",
  "commitment": "1771"
}
```

**Common mistakes:**
- Extra fields in JSON that don't exist in the circuit
- Missing fields that the circuit expects
- Using integers instead of strings (Circom expects string values for field elements)

### Input format: strings, not numbers

**Wrong:**
```json
{"secret": 42, "salt": 7, "commitment": 1771}
```

**Correct:**
```json
{"secret": "42", "salt": "7", "commitment": "1771"}
```

Circom/snarkjs expects decimal string representations of BN254 field elements. Hex is not supported in the input JSON.

---

## UltraHonk (Noir + Barretenberg)

### "required tool 'nargo' not found"

**Fix**: Install nargo via noirup:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup
```

Verify: `nargo --version`

### "required tool 'bb' not found"

**Fix**: Install bb (Barretenberg) via bbup:

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash
bbup
```

Verify: `bb --version`

### Version incompatibility between nargo and bb

**Error**: `Error: proof verification failed` or unexpected proof format errors.

**Cause**: nargo and bb versions must be compatible. They are developed in the same monorepo and are released in sync.

**Fix**: Update both to the latest versions:

```bash
noirup
bbup
```

Check that their versions are from the same release cycle:

```bash
nargo --version
bb --version
```

### Out of memory with large circuits

**Error**: Process killed or `out of memory` during `bb prove_ultra_honk`.

**Fix**: UltraHonk proving is memory-intensive. Options:
1. Reduce circuit size by optimizing your Noir code
2. Increase available memory (close other applications)
3. On Linux, increase swap space

---

## RISC Zero

### Docker not available

**Error**: `expected groth16 receipt — was docker running?` or `docker: command not found`

**Cause**: RISC Zero requires Docker to wrap STARK proofs into Groth16 proofs for on-chain verification.

**Fix**:
1. Install Docker from [docs.docker.com/get-docker](https://docs.docker.com/get-docker/)
2. Start the Docker daemon
3. Verify: `docker info`

On Linux, ensure your user is in the `docker` group:

```bash
sudo usermod -aG docker $USER
# Log out and back in
```

### Guest compilation fails (RISC-V target)

**Error**: `error[E0463]: can't find crate for 'std'` or target-related errors.

**Cause**: The RISC Zero guest compiles for `riscv32im-risc0-zkvm-elf`, which requires the RISC Zero toolchain.

**Fix**:

```bash
rzup install
```

If `rzup` is not found:

```bash
curl -L https://risczero.com/install | bash
source ~/.bashrc  # or ~/.zshrc
rzup install
```

### "seal validation failed"

**Cause**: The Groth16 seal has an invalid selector prefix or incorrect length.

**Common reasons:**
- The proof was generated without Docker (STARK only, no Groth16 wrapping)
- Corrupted `seal.bin` file
- Build artifacts are stale

**Fix**:
1. Ensure Docker is running: `docker info`
2. Clean and rebuild: `rm -rf target/ proofs/ && stellar-zk build`
3. Re-generate the proof: `stellar-zk prove --input inputs/input.json`

### rzup install fails (proxy/firewall)

**Error**: Download timeouts or SSL errors during `rzup install`.

**Fix**: If behind a corporate proxy:

```bash
export HTTPS_PROXY=http://your-proxy:port
rzup install
```

If the download server is blocked, you can build from source — see the [RISC Zero documentation](https://dev.risczero.com/api/zkvm/).

---

## Build

### "WASM too large" (exceeds 64 KB limit)

**Error**: `WASM size 72,431 bytes exceeds the 65,536 byte limit`

**Cause**: The compiled Soroban contract exceeds Soroban's 64 KB WASM limit. This check is enforced in the `testnet` and `stellar-production` profiles.

**Fix** (in order of effectiveness):

1. **Use a more aggressive profile**:
   ```bash
   stellar-zk build --profile stellar-production
   ```
   This enables `opt-level = "z"`, full LTO, and `wasm-opt -Oz`.

2. **Install wasm-opt** (if not already installed):
   ```bash
   cargo install wasm-opt
   ```
   The pipeline uses `wasm-opt` for additional size optimization when available.

3. **Simplify your circuit/contract** — fewer constraints generally mean smaller WASM.

4. **Use the `development` profile** for local testing (no size limit enforced):
   ```bash
   stellar-zk build --profile development
   ```

### Stale artifacts after modifying the circuit

**Symptom**: Proof generation fails or produces invalid proofs after changing the circuit.

**Cause**: Build artifacts (R1CS, proving key, VK) are from the previous circuit version.

**Fix**: Always rebuild after modifying your circuit:

```bash
stellar-zk build
stellar-zk prove --input inputs/input.json
```

The build command regenerates all artifacts. If you encounter persistent issues:

```bash
rm -rf target/
stellar-zk build
```

### Contract build fails

**Error**: `cargo build` errors during the WASM compilation step.

**Common causes:**
- Missing `wasm32` target: `rustup target add wasm32-unknown-unknown`
- Soroban SDK version mismatch
- Modified the contract template with invalid Rust code

**Fix**: If you modified the contract, revert to the generated version:

```bash
# Re-initialize to get a fresh contract (back up your circuit first!)
stellar-zk init myapp-fresh --backend groth16
cp myapp-fresh/contracts/verifier/src/lib.rs contracts/verifier/src/lib.rs
rm -rf myapp-fresh
```

---

## Deploy & Call

### "account not found"

**Error**: `error: account not found` or `AccountDoesNotExist`.

**Fix**: Fund your Stellar identity:

```bash
# Create identity if needed
stellar keys generate alice --network testnet

# Fund with testnet XLM
stellar keys fund alice --network testnet
```

### "transaction simulation failed" (CPU budget exceeded)

**Error**: Simulation reports CPU instructions exceed the 100M budget.

**Possible causes:**
- UltraHonk proofs with large circuits may exceed the budget
- Multiple pairing operations (many public inputs) increase CPU cost

**Fix**:
1. Run `stellar-zk estimate` to check expected CPU usage
2. If using UltraHonk with >35M CPU, consider switching to Groth16 (~12M)
3. Reduce the number of public inputs if possible

### Wrong network

**Error**: Contract ID not found, or `network not configured`.

**Fix**: Ensure you're using the same network for deploy and call:

```bash
# Deploy to testnet
stellar-zk deploy --network testnet --source alice

# Call on the SAME network
stellar-zk call --contract-id CXYZ... --network testnet --source alice
```

Available networks: `local`, `testnet`, `mainnet` (not recommended for testing).

### Contract ID format

Stellar contract IDs start with `C` and are 56 characters long (Strkey encoding), for example:

```
CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
```

If you receive a hex-encoded contract ID, convert it using the Stellar CLI or use it directly — stellar-zk accepts both formats.

---

## General

### Verbose output

Use the `-v` flag for more detailed output:

```bash
stellar-zk build -v        # Info-level logging
stellar-zk build -vv       # Debug-level logging
stellar-zk build -vvv      # Trace-level logging (very verbose)
```

Trace-level logging shows the exact commands being shelled out to external tools, which is useful for debugging tool invocation issues.

### Reset a project

To start fresh, remove the build artifacts:

```bash
rm -rf target/ proofs/
```

Then rebuild:

```bash
stellar-zk build
```

### "config file not found"

**Error**: `Failed to read stellar-zk.config.json`

**Cause**: You're running a stellar-zk command outside of a project directory.

**Fix**: Either:
1. `cd` into your project directory first
2. Use `--config` to specify the config path:
   ```bash
   stellar-zk build --config /path/to/myapp/stellar-zk.config.json
   ```

### "build artifacts not found"

**Error**: `target/build_artifacts.json not found` when running `prove`, `deploy`, or `estimate`.

**Cause**: You haven't run `stellar-zk build` yet, or the artifacts were cleaned.

**Fix**:

```bash
stellar-zk build
```

---

## FAQ

### Which backend should I choose?

| Use case | Recommended backend |
|----------|-------------------|
| Lowest cost, simple circuits | **Groth16** (~12M CPU, 256-byte proofs) |
| Complex logic, no trusted setup | **UltraHonk** (Noir language, universal SRS) |
| Arbitrary Rust computation | **RISC Zero** (standard Rust, zkVM) |
| Prototyping / learning | **Groth16** (simplest setup, most documentation) |

If you're unsure, start with **Groth16**. It has the lowest on-chain cost and the smallest proof size.

### Can I use stellar-zk on mainnet?

Yes, but with caution:

1. Use the `stellar-production` profile to enforce all resource limits
2. For Groth16, use a production-grade trusted setup (community Powers of Tau ceremony), not the auto-generated development setup
3. Audit your circuit and contract thoroughly
4. Test extensively on testnet first

### How does the trusted setup work?

**Groth16 only.** The trusted setup generates a proving key and verification key from a random secret (toxic waste). Anyone who knows the secret could forge proofs.

In development mode (`stellar-zk build`), a local ceremony is generated automatically — fine for testing but **not secure for production**.

For production, use a community-generated Powers of Tau file:
1. Download a ptau file from [Hermez trusted setup](https://github.com/iden3/snarkjs#7-prepare-phase-2) or [PSE ceremonies](https://ceremony.ethereum.org/)
2. Set the `trusted_setup` path in `backend.config.json`:
   ```json
   {
     "groth16": {
       "trusted_setup": "/path/to/powersOfTau28_hez_final_14.ptau"
     }
   }
   ```

UltraHonk and RISC Zero don't require trusted setups — they use universal reference strings.

### What are the Soroban resource limits?

| Resource | Limit |
|----------|-------|
| WASM binary size | 64 KB (65,536 bytes) |
| CPU instructions per transaction | 100,000,000 (100M) |
| Memory | Varies by network config |

These limits apply to the contract execution on-chain. All three backends fit within these limits for typical circuits:
- Groth16: ~10 KB WASM, ~12M CPU
- UltraHonk: ~50 KB WASM, ~35M CPU
- RISC Zero: ~10 KB WASM, ~15M CPU

### How do nullifiers prevent double verification?

Each time `verify()` is called, a nullifier is computed as `SHA256(proof || public_inputs)`. The contract:

1. Checks if this nullifier has been used before (`is_nullifier_used`)
2. If used, rejects the transaction with `NullifierAlreadyUsed`
3. If new, stores the nullifier in persistent storage and proceeds with verification

This means each unique (proof, public_inputs) pair can only be verified once on-chain. Different proofs for the same public inputs produce different nullifiers and can each be verified once.

### Do I need Docker for all backends?

No. Docker is only required for **RISC Zero**, which uses it to wrap STARK proofs into Groth16 proofs.

| Backend | Docker required? |
|---------|-----------------|
| Groth16 | No |
| UltraHonk | No |
| RISC Zero | Yes (for Groth16 wrapping) |

### Can I customize the verifier contract?

The generated verifier contract in `contracts/verifier/src/lib.rs` can be modified, but be careful:

- **Don't change** the proof deserialization logic or byte layout — it must match the serializer exactly
- **Don't change** the pairing check equation
- **Safe to modify**: event data, additional storage, access control, custom error messages
- After modifying, rebuild with `stellar-zk build` (only the contract step)

### How do I add access control to the verifier?

The generated contract doesn't include access control by default. To restrict who can call `verify()`, add an admin check:

```rust
// In the contract, add a storage key for the admin
fn verify(env: Env, proof: Bytes, public_inputs: Bytes, nullifier: BytesN<32>) -> Result<bool, VerifierError> {
    // Add at the top of verify():
    let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
    admin.require_auth();

    // ... rest of verification logic
}
```

Initialize the admin in `__constructor` alongside the VK.
