use sha2::{Digest, Sha256};

pub mod circuit;

pub fn u64_to_ssz_leaf(value: u64) -> [u8; 32] {
    let mut ret = vec![0u8; 32];
    ret[0..8].copy_from_slice(value.to_le_bytes().as_slice());
    ret.try_into().unwrap()
}

pub fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::new();

    for value in bytes {
        for i in (0..8).rev() {
            let mask = 1 << i;
            bits.push(value & mask != 0);
        }
    }

    bits
}

pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|bits| {
            (0..8usize).fold(0u8, |byte, pos| {
                byte | ((bits[pos] as usize) << (7 - pos)) as u8
            })
        })
        .collect::<Vec<_>>()
}
