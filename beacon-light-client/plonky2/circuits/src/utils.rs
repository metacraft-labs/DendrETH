use plonky2_sha256::circuit::array_to_bits;
use sha2::{Sha256, Digest};

pub fn hash_values(lhs: [bool; 256], rhs: [bool; 256]) -> [bool; 256] {
    let bytes: Vec<u8> = [lhs, rhs]
        .concat()
        .chunks(8)
        .map(|chunk| {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1u8 << (7 - i);
                }
            }
            byte
        })
        .collect();

    let mut hasher = Sha256::default();
    hasher.update(&bytes);

    let finalized = hasher.finalize();

    array_to_bits(finalized.as_slice()).try_into()?;
}
