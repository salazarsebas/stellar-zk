use num_bigint::BigUint;
use num_traits::Num;

/// Convert a decimal string to a 32-byte big-endian array.
///
/// snarkjs represents BN254 field elements as decimal strings (e.g., "21888242871...").
/// Soroban's BN254 host functions expect 32-byte big-endian byte arrays, left-padded
/// with zeros. This function performs that conversion.
fn decimal_to_be_bytes(s: &str) -> Result<[u8; 32], String> {
    let n = BigUint::from_str_radix(s, 10).map_err(|e| format!("invalid decimal: {s}: {e}"))?;
    let be_bytes = n.to_bytes_be();
    if be_bytes.len() > 32 {
        return Err(format!("value too large for 32 bytes: {s}"));
    }
    let mut out = [0u8; 32];
    let offset = 32 - be_bytes.len();
    out[offset..].copy_from_slice(&be_bytes);
    Ok(out)
}

/// Serialize a G1 point from snarkjs JSON to 64 bytes big-endian.
///
/// snarkjs G1 format: `["x_dec", "y_dec", "1"]` (projective, but z=1 for affine).
/// Output: `x(32 BE) | y(32 BE)` = 64 bytes.
pub fn serialize_g1_from_json(coords: &[serde_json::Value]) -> Result<[u8; 64], String> {
    if coords.len() < 2 {
        return Err("G1 point must have at least 2 coordinates".into());
    }
    let x_str = coords[0].as_str().ok_or("G1.x must be a string")?;
    let y_str = coords[1].as_str().ok_or("G1.y must be a string")?;

    let x = decimal_to_be_bytes(x_str)?;
    let y = decimal_to_be_bytes(y_str)?;

    let mut out = [0u8; 64];
    out[..32].copy_from_slice(&x);
    out[32..64].copy_from_slice(&y);
    Ok(out)
}

/// Serialize a G2 point from snarkjs JSON to 128 bytes big-endian.
///
/// snarkjs G2 format: `[["x_c0_dec", "x_c1_dec"], ["y_c0_dec", "y_c1_dec"], ["1","0"]]`.
/// Note: snarkjs outputs `[c0, c1]` but Soroban expects `c1 | c0` (higher-degree first).
/// Output: `x_c1(32 BE) | x_c0(32 BE) | y_c1(32 BE) | y_c0(32 BE)` = 128 bytes.
pub fn serialize_g2_from_json(coords: &[serde_json::Value]) -> Result<[u8; 128], String> {
    if coords.len() < 2 {
        return Err("G2 point must have at least 2 coordinate pairs".into());
    }

    let x_pair = coords[0]
        .as_array()
        .ok_or("G2.x must be an array [c0, c1]")?;
    let y_pair = coords[1]
        .as_array()
        .ok_or("G2.y must be an array [c0, c1]")?;

    if x_pair.len() < 2 || y_pair.len() < 2 {
        return Err("G2 coordinate pairs must have 2 elements".into());
    }

    let x_c0_str = x_pair[0].as_str().ok_or("G2.x.c0 must be a string")?;
    let x_c1_str = x_pair[1].as_str().ok_or("G2.x.c1 must be a string")?;
    let y_c0_str = y_pair[0].as_str().ok_or("G2.y.c0 must be a string")?;
    let y_c1_str = y_pair[1].as_str().ok_or("G2.y.c1 must be a string")?;

    let x_c0 = decimal_to_be_bytes(x_c0_str)?;
    let x_c1 = decimal_to_be_bytes(x_c1_str)?;
    let y_c0 = decimal_to_be_bytes(y_c0_str)?;
    let y_c1 = decimal_to_be_bytes(y_c1_str)?;

    // IMPORTANT: Component order swap (security-critical).
    //
    // snarkjs outputs G2 extension field components as [c0, c1] (low-degree first),
    // but Soroban's BN254 host functions expect [c1, c0] (high-degree first).
    // This matches the convention used by the EIP-197 precompile and the Soroban
    // SDK's `bls12_381` / `bn254` representation where Fp2 is stored as (c1, c0).
    //
    // If this order is wrong, pairing checks will silently fail on-chain.
    // Any change here MUST be reflected in the contract template:
    //   templates/contracts/groth16_verifier/src/lib.rs.tmpl
    let mut out = [0u8; 128];
    out[0..32].copy_from_slice(&x_c1);
    out[32..64].copy_from_slice(&x_c0);
    out[64..96].copy_from_slice(&y_c1);
    out[96..128].copy_from_slice(&y_c0);
    Ok(out)
}

/// Serialize a snarkjs Groth16 proof JSON into 256 bytes for Soroban.
///
/// Expects the proof.json structure from `snarkjs groth16 prove`:
/// ```json
/// {
///   "pi_a": ["x", "y", "1"],
///   "pi_b": [["x_c0", "x_c1"], ["y_c0", "y_c1"], ["1", "0"]],
///   "pi_c": ["x", "y", "1"]
/// }
/// ```
///
/// Output: `A(G1:64) | B(G2:128) | C(G1:64)` = 256 bytes.
pub fn serialize_proof_from_snarkjs(proof_json: &serde_json::Value) -> Result<Vec<u8>, String> {
    let pi_a = proof_json["pi_a"]
        .as_array()
        .ok_or("proof.pi_a must be an array")?;
    let pi_b = proof_json["pi_b"]
        .as_array()
        .ok_or("proof.pi_b must be an array")?;
    let pi_c = proof_json["pi_c"]
        .as_array()
        .ok_or("proof.pi_c must be an array")?;

    let a = serialize_g1_from_json(pi_a)?;
    let b = serialize_g2_from_json(pi_b)?;
    let c = serialize_g1_from_json(pi_c)?;

    let mut out = Vec::with_capacity(256);
    out.extend_from_slice(&a);
    out.extend_from_slice(&b);
    out.extend_from_slice(&c);
    debug_assert_eq!(out.len(), 256);
    Ok(out)
}

/// Serialize a snarkjs verification_key.json into Soroban binary format.
///
/// Expects:
/// ```json
/// {
///   "vk_alpha_1": ["x", "y", "1"],
///   "vk_beta_2": [["x_c0", "x_c1"], ["y_c0", "y_c1"], ["1", "0"]],
///   "vk_gamma_2": [...],
///   "vk_delta_2": [...],
///   "IC": [["x", "y", "1"], ...]
/// }
/// ```
///
/// Output: `alpha(G1:64) | beta(G2:128) | gamma(G2:128) | delta(G2:128) | ic_count(u32 BE:4) | ic[](G1:64 each)`
pub fn serialize_vk_from_snarkjs(vk_json: &serde_json::Value) -> Result<Vec<u8>, String> {
    let alpha = vk_json["vk_alpha_1"]
        .as_array()
        .ok_or("vk.vk_alpha_1 must be an array")?;
    let beta = vk_json["vk_beta_2"]
        .as_array()
        .ok_or("vk.vk_beta_2 must be an array")?;
    let gamma = vk_json["vk_gamma_2"]
        .as_array()
        .ok_or("vk.vk_gamma_2 must be an array")?;
    let delta = vk_json["vk_delta_2"]
        .as_array()
        .ok_or("vk.vk_delta_2 must be an array")?;
    let ic = vk_json["IC"].as_array().ok_or("vk.IC must be an array")?;

    let alpha_bytes = serialize_g1_from_json(alpha)?;
    let beta_bytes = serialize_g2_from_json(beta)?;
    let gamma_bytes = serialize_g2_from_json(gamma)?;
    let delta_bytes = serialize_g2_from_json(delta)?;

    let ic_count = ic.len() as u32;
    let total_size = 64 + 128 + 128 + 128 + 4 + (ic_count as usize * 64);
    let mut out = Vec::with_capacity(total_size);

    out.extend_from_slice(&alpha_bytes);
    out.extend_from_slice(&beta_bytes);
    out.extend_from_slice(&gamma_bytes);
    out.extend_from_slice(&delta_bytes);
    out.extend_from_slice(&ic_count.to_be_bytes());

    for (i, ic_point) in ic.iter().enumerate() {
        let ic_arr = ic_point
            .as_array()
            .ok_or_else(|| format!("IC[{i}] must be an array"))?;
        let ic_bytes = serialize_g1_from_json(ic_arr)?;
        out.extend_from_slice(&ic_bytes);
    }

    debug_assert_eq!(out.len(), total_size);
    Ok(out)
}

/// Serialize snarkjs public inputs (public.json) to Soroban format.
///
/// snarkjs outputs `["decimal1", "decimal2", ...]`.
/// Output: `Vec<[u8; 32]>` — each field element as 32-byte big-endian.
pub fn serialize_public_inputs_from_snarkjs(
    public_json: &serde_json::Value,
) -> Result<Vec<[u8; 32]>, String> {
    let arr = public_json
        .as_array()
        .ok_or("public inputs must be an array")?;

    arr.iter()
        .enumerate()
        .map(|(i, v)| {
            let s = v
                .as_str()
                .ok_or_else(|| format!("public_input[{i}] must be a string"))?;
            decimal_to_be_bytes(s)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_to_be_bytes_zero() {
        let bytes = decimal_to_be_bytes("0").unwrap();
        assert!(bytes.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_decimal_to_be_bytes_one() {
        let bytes = decimal_to_be_bytes("1").unwrap();
        assert_eq!(bytes[31], 1);
        assert!(bytes[..31].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_decimal_to_be_bytes_256() {
        let bytes = decimal_to_be_bytes("256").unwrap();
        assert_eq!(bytes[30], 1);
        assert_eq!(bytes[31], 0);
        assert!(bytes[..30].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_g1_serialization_size() {
        let coords = serde_json::json!(["1", "2", "1"]);
        let bytes = serialize_g1_from_json(coords.as_array().unwrap()).unwrap();
        assert_eq!(bytes.len(), 64);
        assert_eq!(bytes[31], 1); // x = 1
        assert_eq!(bytes[63], 2); // y = 2
    }

    #[test]
    fn test_g2_serialization_size_and_order() {
        // snarkjs format: [[x_c0, x_c1], [y_c0, y_c1], [1, 0]]
        let coords = serde_json::json!([["10", "20"], ["30", "40"], ["1", "0"]]);
        let bytes = serialize_g2_from_json(coords.as_array().unwrap()).unwrap();
        assert_eq!(bytes.len(), 128);
        // Soroban order: x_c1 | x_c0 | y_c1 | y_c0
        assert_eq!(bytes[31], 20); // x_c1
        assert_eq!(bytes[63], 10); // x_c0
        assert_eq!(bytes[95], 40); // y_c1
        assert_eq!(bytes[127], 30); // y_c0
    }

    #[test]
    fn test_proof_serialization_layout() {
        let proof = serde_json::json!({
            "pi_a": ["1", "2", "1"],
            "pi_b": [["3", "4"], ["5", "6"], ["1", "0"]],
            "pi_c": ["7", "8", "1"],
            "protocol": "groth16",
            "curve": "bn128"
        });
        let bytes = serialize_proof_from_snarkjs(&proof).unwrap();
        assert_eq!(bytes.len(), 256);
        // A starts at 0
        assert_eq!(bytes[31], 1); // A.x
        assert_eq!(bytes[63], 2); // A.y
                                  // B starts at 64 (G2: c1|c0|c1|c0)
        assert_eq!(bytes[95], 4); // B.x_c1
        assert_eq!(bytes[127], 3); // B.x_c0
        assert_eq!(bytes[159], 6); // B.y_c1
        assert_eq!(bytes[191], 5); // B.y_c0
                                   // C starts at 192
        assert_eq!(bytes[223], 7); // C.x
        assert_eq!(bytes[255], 8); // C.y
    }

    #[test]
    fn test_vk_serialization_layout() {
        let vk = serde_json::json!({
            "protocol": "groth16",
            "curve": "bn128",
            "nPublic": 1,
            "vk_alpha_1": ["1", "2", "1"],
            "vk_beta_2": [["3", "4"], ["5", "6"], ["1", "0"]],
            "vk_gamma_2": [["7", "8"], ["9", "10"], ["1", "0"]],
            "vk_delta_2": [["11", "12"], ["13", "14"], ["1", "0"]],
            "IC": [
                ["100", "200", "1"],
                ["300", "400", "1"]
            ]
        });
        let bytes = serialize_vk_from_snarkjs(&vk).unwrap();
        // 64 + 128 + 128 + 128 + 4 + 2*64 = 580
        assert_eq!(bytes.len(), 580);

        // Check ic_count at offset 448
        let ic_count = u32::from_be_bytes([bytes[448], bytes[449], bytes[450], bytes[451]]);
        assert_eq!(ic_count, 2);

        // Check IC[0] starts at 452
        assert_eq!(bytes[483], 100); // IC[0].x
        assert_eq!(bytes[515], 200); // IC[0].y
    }

    #[test]
    fn test_public_inputs_serialization() {
        let inputs = serde_json::json!(["42", "1771"]);
        let serialized = serialize_public_inputs_from_snarkjs(&inputs).unwrap();
        assert_eq!(serialized.len(), 2);
        assert_eq!(serialized[0][31], 42);
        // 1771 = 0x06EB
        assert_eq!(serialized[1][30], 0x06);
        assert_eq!(serialized[1][31], 0xEB);
    }

    #[test]
    fn test_large_field_element() {
        // BN254 field modulus is ~254 bits
        let large = "21888242871839275222246405745257275088696311157297823662689037894645226208583";
        let bytes = decimal_to_be_bytes(large).unwrap();
        assert_eq!(bytes.len(), 32);
        // Should not be all zeros
        assert!(!bytes.iter().all(|&b| b == 0));
    }

    // --- Edge case tests ---

    #[test]
    fn test_decimal_overflow_exceeds_32_bytes() {
        // 2^256 = 115792089237316195423570985008687907853269984665640564039457584007913129639936
        let too_large =
            "115792089237316195423570985008687907853269984665640564039457584007913129639936";
        let result = decimal_to_be_bytes(too_large);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }

    #[test]
    fn test_decimal_invalid_input() {
        assert!(decimal_to_be_bytes("not_a_number").is_err());
        assert!(decimal_to_be_bytes("").is_err());
        assert!(decimal_to_be_bytes("-1").is_err());
    }

    #[test]
    fn test_g1_empty_coords() {
        let coords: Vec<serde_json::Value> = vec![];
        let result = serialize_g1_from_json(&coords);
        assert!(result.is_err());
    }

    #[test]
    fn test_g2_non_array_components() {
        // G2 expects arrays for x and y, not plain strings
        let coords = serde_json::json!(["10", "20", "1"]);
        let result = serialize_g2_from_json(coords.as_array().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_proof_missing_field() {
        let proof = serde_json::json!({
            "pi_a": ["1", "2", "1"],
            // pi_b missing
            "pi_c": ["7", "8", "1"],
        });
        let result = serialize_proof_from_snarkjs(&proof);
        assert!(result.is_err());
    }

    #[test]
    fn test_public_inputs_empty_array() {
        let inputs = serde_json::json!([]);
        let result = serialize_public_inputs_from_snarkjs(&inputs).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_public_inputs_non_string_element() {
        let inputs = serde_json::json!([42]);
        let result = serialize_public_inputs_from_snarkjs(&inputs);
        assert!(result.is_err());
    }
}
