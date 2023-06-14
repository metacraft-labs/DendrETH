use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::circuit_builder::CircuitBuilder,
};

use plonky2_sha256::circuit::{array_to_bits, make_circuits, Sha256Targets};

use sha2::{Digest, Sha256};

pub fn is_valid_merkle_branch<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pw: &mut PartialWitness<F>,
    root: &Vec<bool>,
    leaf: &Vec<bool>,
    branch: &[Vec<bool>],
    index: &u64,
) {
    let mut leaf = leaf.clone();

    let mut index = index.clone();

    for (i, sibling) in branch.iter().enumerate() {
        let is_right = index % 2 == 1;
        let mut lhs = leaf.clone();
        let mut rhs = sibling.clone();

        if is_right {
            std::mem::swap(&mut lhs, &mut rhs);
        }

        let hasher = make_circuits(builder, 512);

        for i in 0..256 {
            pw.set_bool_target(hasher.message[i], lhs[i]);
        }

        for i in 0..256 {
            pw.set_bool_target(hasher.message[i + 256], rhs[i]);
        }

        leaf = hash_values(lhs, rhs);

        // constraint the root
        if i == branch.len() - 1 {
            assert_hasher(root, builder, hasher)
        }

        index /= 2;
    }
}

fn hash_values(lhs: Vec<bool>, rhs: Vec<bool>) -> Vec<bool> {
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

    return array_to_bits(finalized.as_slice());
}

fn assert_hasher<F: RichField + Extendable<D>, const D: usize>(
    result: &Vec<bool>,
    builder: &mut CircuitBuilder<F, D>,
    hasher: Sha256Targets,
) {
    for i in 0..256 {
        if result[i] {
            builder.assert_one(hasher.digest[i].target);
        } else {
            builder.assert_zero(hasher.digest[i].target);
        }
    }
}
