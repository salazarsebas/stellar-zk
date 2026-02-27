## Summary

<!-- What does this PR do? Why is it needed? -->

## Related Issue

<!-- Link to the issue this PR addresses, e.g. Closes #123 -->

## Changes

<!-- Bullet list of key changes -->

-

## Build Checklist

- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes

## Safety Checklist

- [ ] I did **not** add heavy dependencies (`risc0-zkvm`, `ark-circom`, `wasmer`, etc.)
- [ ] If I modified templates in `templates/`, I updated the corresponding constants in `crates/stellar-zk-core/src/templates/embedded.rs`
- [ ] If I modified a serializer, I updated the corresponding contract template to match
- [ ] If I changed byte ordering, I verified **both** the Rust serializer **and** the contract template
- [ ] I did not remove `overflow_checks` from any profile
