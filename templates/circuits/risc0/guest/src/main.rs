// RISC Zero guest program.
// This runs inside the zkVM and produces a provable execution receipt.

#![no_main]
#![no_std]

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    // Read private input from the host
    let secret: u64 = env::read();
    let salt: u64 = env::read();

    // Compute the result (replace with your logic)
    let commitment = secret.wrapping_mul(secret).wrapping_add(salt);

    // Write public output to the journal
    // The journal is the publicly visible output of the zkVM execution
    env::commit(&commitment);
}
