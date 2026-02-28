# Contributing to stellar-zk

Thank you for your interest in contributing to stellar-zk!

## Development Setup

```bash
# Clone the repository
git clone https://github.com/salazarsebas/stellar-zk.git
cd stellar-zk

# Build the workspace
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

### Rust Version

stellar-zk requires **Rust 1.85.0** or later. Install via [rustup](https://rustup.rs/).

## Running Tests

All unit tests are **pure Rust** — no external tools (circom, snarkjs, nargo, etc.) or Docker required.

```bash
# All tests
cargo test

# Tests for a specific crate
cargo test -p stellar-zk-groth16    # 9 tests (BN254 serialization)
cargo test -p stellar-zk-ultrahonk  # 6 tests
cargo test -p stellar-zk-risc0      # 5 tests (seal validation)
cargo test -p stellar-zk-core       # core types

# With output
cargo test -- --nocapture
```

## Code Style

- Format with `cargo fmt --all` before committing
- Lint with `cargo clippy --workspace -- -D warnings`
- All public items should have doc comments (`///`)
- Module-level docs (`//!`) are required in `lib.rs` files
- Error handling: `thiserror` for `StellarZkError` (in core), `anyhow` at the CLI boundary
- Serde: derive `Serialize`/`Deserialize` on all config and artifact types

## Critical Restrictions

These rules exist to maintain correctness and prevent subtle bugs:

1. **No heavy dependencies** — Do NOT add `risc0-zkvm`, `ark-circom`, `wasmer`, or similar. The shell-out strategy is intentional.
2. **Template coupling** — If you modify a template in `templates/`, update the corresponding `include_str!` constant in `crates/stellar-zk-core/src/templates/embedded.rs`.
3. **Serializer coupling** — If you modify a serializer, update the corresponding contract template to match. Byte layouts must be identical.
4. **G2 component ordering** — snarkjs outputs `[c0, c1]` but Soroban expects `c1 | c0`. Do NOT change this without updating both sides.
5. **Artifact chain** — Do NOT break the `build_artifacts.json` flow between commands.
6. **overflow_checks** — Do NOT remove from any Cargo profile. ZK verification is security-critical.

## Pull Requests

We use [issue templates](https://github.com/salazarsebas/stellar-zk/issues/new/choose) and a [PR template](https://github.com/salazarsebas/stellar-zk/blob/main/.github/PULL_REQUEST_TEMPLATE.md) to keep contributions structured.

### Process

1. Fork the repository and create a feature branch from `main`
2. Make your changes with clear commit messages
3. Ensure all checks pass:
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   ```
4. Open a pull request — the PR template includes build and safety checklists

### Branch naming

- `feat/<description>` — new features
- `fix/<description>` — bug fixes
- `docs/<description>` — documentation changes
- `refactor/<description>` — code restructuring

### Commit messages

Use imperative mood: "Add backend", not "Added backend" or "Adds backend".

## Adding a New Backend

To add support for a new ZK proving system:

1. Create a new crate: `crates/stellar-zk-<name>/`
2. Implement the `ZkBackend` trait from `stellar-zk-core`
3. Add circuit template to `crates/stellar-zk-core/templates/circuits/<name>/`
4. Add contract template to `crates/stellar-zk-core/templates/contracts/<name>_verifier/`
5. Register `include_str!` constants in `crates/stellar-zk-core/src/templates/embedded.rs`
6. Add variant to `BackendChoice` in `crates/stellar-zk-cli/src/main.rs`
7. Add factory case in `crates/stellar-zk-cli/src/commands/init.rs::create_backend()`
8. Add cost model in `crates/stellar-zk-core/src/estimator.rs`
9. Add `BackendConfig` variant in `crates/stellar-zk-core/src/config.rs`

See the [CLAUDE.md](CLAUDE.md) "Common Workflows" section for detailed step-by-step guides.

## Reporting Issues

Please use the [issue templates](https://github.com/salazarsebas/stellar-zk/issues/new/choose):

- **Bug Report**: for reproducible problems
- **Feature Request**: for suggestions and enhancements

For security vulnerabilities, see [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
