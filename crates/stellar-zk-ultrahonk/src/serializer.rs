/// Serialize an UltraHonk verification key for on-chain storage.
///
/// The VK format follows the Barretenberg output format
/// and is stored as raw bytes in contract instance storage.
pub fn serialize_vk_for_soroban(vk_bytes: &[u8]) -> Vec<u8> {
    // BB VK format is already suitable for the on-chain verifier.
    // The ultrahonk_rust_verifier crate parses this format directly.
    vk_bytes.to_vec()
}
