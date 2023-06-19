use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use sha2::{Digest, Sha256};

pub fn hash_bit_array(validator_pubkey: Vec<&str>) -> Vec<String> {
    // Concatenate the array into a single binary string
    let binary_string: String = validator_pubkey.join("");

    // Convert binary string to bytes
    let mut byte_string: Vec<u8> = binary_string
        .as_str()
        .chars()
        .collect::<Vec<char>>()
        .chunks(8)
        .map(|chunk| {
            let byte_str: String = chunk.into_iter().collect();
            u8::from_str_radix(&byte_str, 2).unwrap()
        })
        .collect();

    byte_string.resize(64, 0);

    let mut hasher = Sha256::new();
    hasher.update(byte_string);
    let result = hasher.finalize();

    let pubkey_binary_result: Vec<String> = result
        .iter()
        .map(|byte| {
            format!("{:08b}", byte)
                .chars()
                .map(|ch| ch.to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect();
    pubkey_binary_result
}

pub fn create_bool_target_array<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; 256] {
    (0..256)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
