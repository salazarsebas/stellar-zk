# AGENTS.md — stellar-zk

Guide for AI coding agents (Claude Code, Cursor, Copilot, Devin, etc.) working on the stellar-zk codebase in single or multi-agent mode.

---

## Purpose

This file defines task decomposition, file ownership, coordination rules, and testing protocols so that AI agents can work on stellar-zk safely and in parallel. It complements `CLAUDE.md` (build/test/patterns) and `.cursorrules` (critical rules summary).

---

## Crate Ownership Map

Each crate should be owned by **one agent at a time**. Never have two agents editing files in the same crate concurrently.

| Crate | Key Files | Scope |
|-------|-----------|-------|
| `stellar-zk-cli` | `main.rs`, `output.rs`, `commands/*.rs` | CLI parsing, subcommand dispatch, terminal output |
| `stellar-zk-core` | `backend.rs`, `config.rs`, `error.rs`, `profile.rs`, `pipeline.rs`, `estimator.rs`, `templates/embedded.rs`, `templates/renderer.rs` | Shared traits, types, config schemas, template embedding |
| `stellar-zk-groth16` | `lib.rs`, `circuit.rs`, `prover.rs`, `serializer.rs` | Circom + snarkjs orchestration, BN254 serialization |
| `stellar-zk-ultrahonk` | `lib.rs`, `nargo.rs`, `proof_convert.rs`, `serializer.rs` | Noir + Barretenberg orchestration |
| `stellar-zk-risc0` | `lib.rs`, `guest.rs`, `prover.rs`, `serializer.rs` | RISC Zero zkVM orchestration, seal validation |
| `templates/` | `circuits/**`, `contracts/**`, `config/**` | Handlebars templates (`.tmpl` files) |

---

## File Coupling Rules

These file groups are tightly coupled. When you modify one file in a group, you **must** check and update the others in the same agent session.

### 1. Templates <-> Embedded Constants
- **Files**: `templates/**/*.tmpl` <-> `crates/stellar-zk-core/src/templates/embedded.rs`
- **Rule**: Adding, removing, or renaming a `.tmpl` file requires updating the `include_str!` constant in `embedded.rs`
- **Verify**: `cargo build` (compile-time error if mismatched)

### 2. Groth16 Serializer <-> Groth16 Contract Template
- **Files**: `crates/stellar-zk-groth16/src/serializer.rs` <-> `templates/contracts/groth16_verifier/src/lib.rs.tmpl`
- **Rule**: Byte layout must match exactly. G2 component order is `c1 | c0` (higher-degree first)
- **Verify**: `cargo test -p stellar-zk-groth16`

### 3. UltraHonk Serializer <-> UltraHonk Contract Template
- **Files**: `crates/stellar-zk-ultrahonk/src/serializer.rs` <-> `templates/contracts/ultrahonk_verifier/src/lib.rs.tmpl`
- **Rule**: Proof format parsing in contract must match serializer output
- **Verify**: `cargo test -p stellar-zk-ultrahonk`

### 4. RISC Zero Serializer <-> RISC Zero Contract Template
- **Files**: `crates/stellar-zk-risc0/src/serializer.rs` <-> `templates/contracts/risc0_verifier/src/lib.rs.tmpl`
- **Rule**: Selector (4 bytes) + seal layout must match
- **Verify**: `cargo test -p stellar-zk-risc0`

### 5. Backend Registration (4-file group)
- **Files**: `cli/src/main.rs` (`BackendChoice`) <-> `cli/src/commands/init.rs` (`create_backend`) <-> `core/src/config.rs` (`BackendConfig`) <-> `core/src/estimator.rs` (cost model)
- **Rule**: Adding a backend requires updating all four files
- **Verify**: `cargo build --workspace`

### 6. Profiles <-> Pipeline
- **Files**: `core/src/profile.rs` <-> `core/src/pipeline.rs`
- **Rule**: Pipeline reads profile settings for opt-level, LTO, wasm-opt flags
- **Verify**: `cargo test -p stellar-zk-core`

---

## Task Decomposition Guide

### Add a new backend

8 subtasks. Steps 1-4 can run in parallel, then 5-8 sequentially.

| # | Subtask | Crate | Parallel? |
|---|---------|-------|-----------|
| 1 | Create crate with `ZkBackend` impl | `stellar-zk-<name>/` | Yes |
| 2 | Add circuit template | `templates/circuits/<name>/` | Yes |
| 3 | Add contract template | `templates/contracts/<name>_verifier/` | Yes |
| 4 | Write serializer + tests | `stellar-zk-<name>/serializer.rs` | Yes |
| 5 | Register `include_str!` constants | `core/src/templates/embedded.rs` | No (after 2, 3) |
| 6 | Add `BackendChoice` variant | `cli/src/main.rs` | No (after 1) |
| 7 | Add factory case + `BackendConfig` variant | `cli/src/commands/init.rs`, `core/src/config.rs` | No (after 1, 6) |
| 8 | Add cost model | `core/src/estimator.rs` | No (after 1) |

**Single-agent rule**: Steps 5-8 touch the backend registration group — assign to one agent.

### Fix a serialization bug

3 subtasks, strictly sequential.

1. **Identify**: Read the serializer and contract template side-by-side, find the mismatch
2. **Fix both sides**: Update `<backend>/src/serializer.rs` AND `templates/contracts/<backend>_verifier/src/lib.rs.tmpl`
3. **Test**: Run `cargo test -p stellar-zk-<backend>` and verify proof layout

**Single-agent rule**: Serializer + template must be fixed by the same agent.

### Add a CLI command

5 subtasks, mostly sequential.

1. Add variant to `Commands` enum in `cli/src/main.rs`
2. Create handler in `cli/src/commands/<name>.rs`
3. Add `mod <name>;` to `cli/src/commands/mod.rs`
4. Wire dispatch in `main.rs` match block
5. Add any new types/errors to `core/src/error.rs` if needed

### Modify a template

4 subtasks, sequential.

1. Edit the `.tmpl` file in `templates/`
2. Verify `include_str!` path in `core/src/templates/embedded.rs` still matches
3. Run `cargo build` to confirm compile-time embedding works
4. Test with `cargo run -- init testproj --backend <backend>`

### Add a config option

4 subtasks, sequential.

1. Add field to the relevant struct in `core/src/config.rs`
2. Update `default_for_backend()` to set a default value
3. Update the template or backend code that reads the option
4. Update any CLI flags in `cli/src/main.rs` if user-facing

---

## Testing Protocol

### Before every commit

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

### After touching a serializer

```bash
cargo test -p stellar-zk-<backend>
```

Confirm that the serializer test vectors match the contract template's expected byte layout.

### After touching templates

```bash
cargo build   # Verifies include_str! paths are valid
```

If the template affects scaffolding output:

```bash
cargo run -- init /tmp/testproj --backend <backend>
# Inspect the generated files
```

### After touching CLI

```bash
cargo run -- --help                              # Verify flags parse
cargo run -- init /tmp/testproj --backend groth16  # Verify init works
```

### After touching profiles or pipeline

```bash
cargo test -p stellar-zk-core
```

---

## Coordination Patterns

Rules for multi-agent work sessions:

1. **One agent per crate** — Two agents must not edit files in the same crate simultaneously
2. **Template + embedded.rs = one agent** — The agent modifying `.tmpl` files must also update `embedded.rs`
3. **Serializer + contract template = one agent** — These must always be modified together
4. **Backend registration = one agent** — `main.rs` + `init.rs` + `config.rs` + `estimator.rs` changes must happen atomically in a single agent
5. **Core changes block backend work** — If an agent is modifying `ZkBackend` trait or `BuildArtifacts`, backend crate agents should wait
6. **Test before declaring done** — Run `cargo test --workspace` before marking any task complete

### Safe parallelism

These combinations are safe to run in parallel:

- Agent A on `stellar-zk-groth16` + Agent B on `stellar-zk-ultrahonk` (independent backends)
- Agent A on `stellar-zk-cli/commands/deploy.rs` + Agent B on `stellar-zk-risc0` (different crates)
- Agent A on `templates/circuits/` + Agent B on `core/src/estimator.rs` (no coupling)

### Unsafe parallelism (avoid)

- Two agents both editing `core/src/config.rs`
- Agent A on `groth16/serializer.rs` + Agent B on `templates/contracts/groth16_verifier/`
- Agent A on `templates/` + Agent B on `core/src/templates/embedded.rs`

---

## Safety Rules

1. **No heavy dependencies** — Never add `risc0-zkvm`, `ark-circom`, `wasmer`, or similar. Backends shell out to external tools via `Command::new()`
2. **No byte-order changes without dual update** — If you change serialization in `<backend>/serializer.rs`, you must update the contract template in `templates/contracts/<backend>_verifier/` to match. G2 point ordering (`c1 | c0`) is critical
3. **No breaking the artifact chain** — `target/build_artifacts.json` is produced by `build` and consumed by `prove`, `deploy`, `call`, and `estimate`. Do not change its schema without updating all consumers
4. **No removing overflow_checks** — ZK verification is security-critical. Cargo profiles must keep `overflow-checks = true`
5. **No `ark-circom`** — Depends on `wasmer-wasix` which doesn't compile on Rust 1.84+. Use `snarkjs` via shell
6. **Always test before done** — Run `cargo test --workspace` and confirm zero failures before marking a task as completed
7. **Preserve the shell-out pattern** — All external tool calls use `Command::new()` with error mapping to `StellarZkError::ExternalTool` / `ExternalToolFailed`

---

## Quick Reference

| What | Path |
|------|------|
| Workspace root | `Cargo.toml` |
| CLI entry point | `crates/stellar-zk-cli/src/main.rs` |
| Backend factory | `crates/stellar-zk-cli/src/commands/init.rs` |
| ZkBackend trait | `crates/stellar-zk-core/src/backend.rs` |
| Config types | `crates/stellar-zk-core/src/config.rs` |
| Error enum | `crates/stellar-zk-core/src/error.rs` |
| Optimization profiles | `crates/stellar-zk-core/src/profile.rs` |
| WASM pipeline | `crates/stellar-zk-core/src/pipeline.rs` |
| Cost estimator | `crates/stellar-zk-core/src/estimator.rs` |
| Template constants | `crates/stellar-zk-core/src/templates/embedded.rs` |
| Template renderer | `crates/stellar-zk-core/src/templates/renderer.rs` |
| Groth16 serializer | `crates/stellar-zk-groth16/src/serializer.rs` |
| UltraHonk serializer | `crates/stellar-zk-ultrahonk/src/serializer.rs` |
| RISC Zero serializer | `crates/stellar-zk-risc0/src/serializer.rs` |
| Circuit templates | `templates/circuits/` |
| Contract templates | `templates/contracts/` |
| Config templates | `templates/config/` |
