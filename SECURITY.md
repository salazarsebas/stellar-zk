# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, send an email to **security@stellar-zk.dev** with:

1. A description of the vulnerability
2. Steps to reproduce
3. Potential impact
4. Suggested fix (if any)

You should receive an acknowledgment within 48 hours. We will work with you to understand the issue and coordinate disclosure.

## Scope

The following areas are security-critical:

### BN254 Serialization (Critical)

The serializers in each backend crate convert proof system outputs to Soroban's expected binary format. Incorrect serialization can produce proofs that verify on-chain when they should not (or vice versa). Key invariants:

- **G2 component ordering**: Soroban expects `c1 | c0` (higher-degree coefficient first). snarkjs outputs `[c0, c1]`. The swap in `groth16/src/serializer.rs` is security-critical.
- **Endianness**: all field elements must be 32-byte big-endian. Incorrect padding or byte order breaks verification silently.
- **Proof layout**: Groth16 = 256 bytes (`A|B|C`), RISC Zero = 260 bytes (`selector|proof`), UltraHonk = variable.

### Contract Templates (Critical)

The Soroban verifier contracts in `crates/stellar-zk-core/templates/contracts/` perform on-chain cryptographic verification. Bugs here can allow invalid proofs to pass or valid proofs to fail.

### Artifact Chain (High)

`build_artifacts.json` links the build, prove, deploy, and call steps. Tampering with artifacts could cause the wrong verification key or proof to be used.

### Nullifier System (High)

The SHA256-based nullifier (`SHA256(proof || public_inputs)`) prevents double-verification. Bugs in nullifier computation could allow replay attacks.

## Groth16 Trusted Setup

Groth16 requires a per-circuit trusted setup (Powers of Tau ceremony). In development mode, stellar-zk generates a local ceremony automatically. **This is NOT secure for production.**

For production deployments:

1. Use a community-generated Powers of Tau file (e.g., from [Hermez](https://github.com/iden3/snarkjs#7-prepare-phase-2))
2. Conduct a multi-party computation ceremony for the circuit-specific phase 2
3. Verify the ceremony transcript before deploying

The trusted setup file path is configured in `backend.config.json` under `groth16.trusted_setup`.

## External Tool Dependencies

stellar-zk shells out to external tools. The security of generated proofs depends on these tools:

| Tool | Used by | Source |
|------|---------|--------|
| `circom` | Groth16 | [github.com/iden3/circom](https://github.com/iden3/circom) |
| `snarkjs` | Groth16 | [github.com/iden3/snarkjs](https://github.com/iden3/snarkjs) |
| `nargo` | UltraHonk | [noir-lang.org](https://noir-lang.org/) |
| `bb` | UltraHonk | [github.com/AztecProtocol/aztec-packages](https://github.com/AztecProtocol/aztec-packages) |
| `cargo-risczero` | RISC Zero | [risczero.com](https://risczero.com/) |

Always use official releases. Verify checksums when possible.
