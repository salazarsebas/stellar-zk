/// The 4-byte selector prefix identifying the RISC Zero Groth16 circuit version.
pub const RISC0_SELECTOR: [u8; 4] = [0x31, 0x0f, 0xe5, 0x98];

/// Serialize a RISC Zero Groth16 seal for on-chain submission.
///
/// Format: [4-byte selector | 256-byte Groth16 proof]
#[allow(dead_code)]
pub fn serialize_seal(selector: &[u8; 4], groth16_proof: &[u8]) -> Vec<u8> {
    let mut seal = selector.to_vec();
    seal.extend_from_slice(groth16_proof);
    seal
}

/// Validate that a seal has the correct selector and length.
pub fn validate_seal(seal: &[u8]) -> bool {
    if seal.len() < 4 {
        return false;
    }
    let selector = &seal[..4];
    selector == RISC0_SELECTOR && seal.len() == 260 // 4 + 256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_seal() {
        let proof = [0xABu8; 256];
        let seal = serialize_seal(&RISC0_SELECTOR, &proof);
        assert_eq!(seal.len(), 260);
        assert_eq!(&seal[..4], &RISC0_SELECTOR);
        assert_eq!(&seal[4..], &proof);
    }

    #[test]
    fn test_validate_seal_valid() {
        let mut seal = RISC0_SELECTOR.to_vec();
        seal.extend_from_slice(&[0u8; 256]);
        assert!(validate_seal(&seal));
    }

    #[test]
    fn test_validate_seal_wrong_selector() {
        let mut seal = vec![0x00, 0x00, 0x00, 0x00];
        seal.extend_from_slice(&[0u8; 256]);
        assert!(!validate_seal(&seal));
    }

    #[test]
    fn test_validate_seal_wrong_length() {
        let mut seal = RISC0_SELECTOR.to_vec();
        seal.extend_from_slice(&[0u8; 128]); // too short
        assert!(!validate_seal(&seal));
    }

    #[test]
    fn test_validate_seal_empty() {
        assert!(!validate_seal(&[]));
    }
}
