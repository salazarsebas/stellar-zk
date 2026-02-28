/// Extract public inputs from a raw bb proof output.
///
/// BB UltraHonk proof format:
///   [4 bytes BE public_input_count | N x 32 bytes inputs | proof_fields x 32 bytes]
pub fn extract_public_inputs(proof_bytes: &[u8]) -> Vec<[u8; 32]> {
    if proof_bytes.len() < 4 {
        return vec![];
    }

    // First 4 bytes: big-endian public input count
    let count = u32::from_be_bytes([
        proof_bytes[0],
        proof_bytes[1],
        proof_bytes[2],
        proof_bytes[3],
    ]) as usize;

    let mut inputs = Vec::with_capacity(count);
    for i in 0..count {
        let offset = 4 + i * 32;
        if offset + 32 > proof_bytes.len() {
            break;
        }
        let mut input = [0u8; 32];
        input.copy_from_slice(&proof_bytes[offset..offset + 32]);
        inputs.push(input);
    }

    inputs
}

/// Convert a raw bb proof to Soroban-compatible format.
///
/// The proof blob is passed as-is to the on-chain verifier.
/// The VK is passed as separate bytes.
#[allow(dead_code)]
pub fn to_soroban_format(proof_bytes: &[u8], _vk_bytes: &[u8]) -> Vec<u8> {
    // BB proof format is already field elements in 32-byte chunks.
    // Pass through directly for the UltraHonk verifier.
    proof_bytes.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_public_inputs_empty() {
        assert_eq!(extract_public_inputs(&[]), Vec::<[u8; 32]>::new());
    }

    #[test]
    fn test_extract_public_inputs_short_header() {
        assert_eq!(extract_public_inputs(&[0, 0, 1]), Vec::<[u8; 32]>::new());
    }

    #[test]
    fn test_extract_public_inputs_zero_count() {
        // 4-byte BE header with count = 0
        let data = [0u8, 0, 0, 0];
        let result = extract_public_inputs(&data);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_public_inputs_one() {
        // count = 1, followed by one 32-byte input
        let mut data = vec![0u8, 0, 0, 1];
        let mut input = [0u8; 32];
        input[31] = 42;
        data.extend_from_slice(&input);
        // Trailing proof bytes
        data.extend_from_slice(&[0xff; 64]);

        let result = extract_public_inputs(&data);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0][31], 42);
    }

    #[test]
    fn test_extract_public_inputs_two() {
        // count = 2, followed by two 32-byte inputs
        let mut data = vec![0u8, 0, 0, 2];
        let mut input1 = [0u8; 32];
        input1[0] = 0xAA;
        data.extend_from_slice(&input1);
        let mut input2 = [0u8; 32];
        input2[0] = 0xBB;
        data.extend_from_slice(&input2);

        let result = extract_public_inputs(&data);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], 0xAA);
        assert_eq!(result[1][0], 0xBB);
    }

    #[test]
    fn test_extract_public_inputs_truncated() {
        // count says 3 but only space for 1 complete input
        let mut data = vec![0u8, 0, 0, 3];
        data.extend_from_slice(&[0x11; 32]); // input 1 (complete)
        data.extend_from_slice(&[0x22; 16]); // input 2 (truncated, only 16 bytes)

        let result = extract_public_inputs(&data);
        // Should only extract the 1 complete input
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], [0x11; 32]);
    }

    #[test]
    fn test_extract_public_inputs_max_count_small_buffer() {
        // count = u32::MAX but buffer is tiny — must not panic or overflow
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00];
        let result = extract_public_inputs(&data);
        // Buffer too small for any complete 32-byte input
        assert!(result.is_empty());
    }
}
