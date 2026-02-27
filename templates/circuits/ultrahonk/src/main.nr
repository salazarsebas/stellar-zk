// Example Noir circuit: prove knowledge of a preimage.
// Replace this with your own circuit logic.

fn main(secret: Field, salt: Field, commitment: pub Field) {
    // Simplified example — replace with pedersen/poseidon hash in production
    let computed = secret * secret + salt;
    assert(computed == commitment);
}
