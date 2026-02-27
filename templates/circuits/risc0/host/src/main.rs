// RISC Zero host program.
// Runs on the host machine, drives zkVM execution, and writes structured
// output files for the stellar-zk prove pipeline.

use risc0_zkvm::{compute_image_id, default_prover, ExecutorEnv, ProverOpts};
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Load the guest ELF binary
    let guest_elf = fs::read("target/riscv32im-risc0-zkvm-elf/release/guest")?;

    // Load inputs from env var or default path
    let input_path = std::env::var("RISC0_INPUT")
        .unwrap_or_else(|_| "inputs/input.json".to_string());
    let input_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&input_path)?)?;

    // Prepare inputs for the guest
    // Default example: read "secret" and "salt" from JSON
    let secret: u64 = input_json["secret"]
        .as_u64()
        .expect("input.json must contain 'secret' as u64");
    let salt: u64 = input_json["salt"]
        .as_u64()
        .expect("input.json must contain 'salt' as u64");

    let env = ExecutorEnv::builder()
        .write(&secret)?
        .write(&salt)?
        .build()?;

    // Generate the Groth16-wrapped proof
    println!("Generating RISC Zero Groth16 proof...");
    let prover = default_prover();
    let receipt = prover.prove_with_opts(env, &guest_elf, &ProverOpts::groth16())?;

    // Verify locally
    receipt.verify_integrity()?;
    println!("Proof verified locally!");

    // Extract Groth16 seal
    let groth16_receipt = receipt
        .inner
        .groth16()
        .expect("expected groth16 receipt — was docker running?");
    let seal = &groth16_receipt.seal;

    // Compute image ID
    let image_id = compute_image_id(&guest_elf)?;

    // Write structured output files
    let proof_dir = Path::new("proofs");
    fs::create_dir_all(proof_dir)?;

    fs::write(proof_dir.join("seal.bin"), seal)?;
    fs::write(proof_dir.join("journal.bin"), &receipt.journal.bytes)?;
    fs::write(proof_dir.join("image_id.hex"), hex::encode(image_id))?;

    println!("Output written to proofs/");
    println!("  seal.bin:     {} bytes", seal.len());
    println!("  journal.bin:  {} bytes", receipt.journal.bytes.len());
    println!("  image_id.hex: {}", hex::encode(image_id));

    Ok(())
}
