//! Core library for the stellar-zk toolkit.
//!
//! Provides the [`backend::ZkBackend`] trait that all proving system backends implement,
//! along with shared infrastructure: configuration loading, optimization profiles,
//! WASM build pipeline, Stellar CLI wrapper, cost estimation, and template rendering.
//!
//! This crate is backend-agnostic. Specific ZK systems are implemented in their own crates:
//! - [`stellar_zk_groth16`](https://docs.rs/stellar-zk-groth16) — Groth16 via Circom + snarkjs
//! - [`stellar_zk_ultrahonk`](https://docs.rs/stellar-zk-ultrahonk) — Noir + UltraHonk via nargo + bb
//! - [`stellar_zk_risc0`](https://docs.rs/stellar-zk-risc0) — RISC Zero zkVM

pub mod artifacts;
pub mod backend;
pub mod config;
pub mod error;
pub mod estimator;
pub mod pipeline;
pub mod profile;
pub mod project;
pub mod stellar;
pub mod templates;
pub mod version;
