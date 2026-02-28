pragma circom 2.1.0;

// Example: Prove knowledge of a preimage that hashes to a public value.
// Replace this with your own circuit logic.

template MembershipProof() {
    // Private inputs
    signal input secret;
    signal input salt;

    // Public inputs
    signal input commitment;

    // Constraint: commitment == secret * secret + salt
    // (simplified example — replace with Poseidon/MiMC hash in production)
    signal secretSquared;
    secretSquared <== secret * secret;
    commitment === secretSquared + salt;
}

component main {public [commitment]} = MembershipProof();
