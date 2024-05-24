use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    plonk::circuit_builder::CircuitBuilder,
};

use crate::utils::hashing::poseidon::poseidon_pair;

pub struct HashTreeRootPoseidonTargets {
    pub leaves: Vec<HashOutTarget>,
    pub hash_tree_root: HashOutTarget,
}

pub fn hash_tree_root_poseidon_new<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves: &[HashOutTarget],
) -> HashOutTarget {
    assert!(leaves.len().is_power_of_two());

    let mut level = leaves.to_owned();

    while level.len() != 1 {
        level = level
            .iter()
            .tuples()
            .map(|(&left, &right)| poseidon_pair(builder, left, right))
            .collect_vec();
    }

    level[0]
}

pub fn hash_tree_root_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves_len: usize,
) -> HashTreeRootPoseidonTargets {
    let leaves: Vec<HashOutTarget> = (0..leaves_len)
        .map(|_| builder.add_virtual_hash())
        .collect();

    let mut hashers: Vec<HashOutTarget> = Vec::new();
    for i in 0..(leaves_len / 2) {
        let hash_target = builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            leaves[i * 2]
                .elements
                .iter()
                .copied()
                .chain(leaves[i * 2 + 1].elements.iter().copied())
                .collect(),
        );
        hashers.push(hash_target);
    }

    let mut k = 0;
    for _ in leaves_len / 2..leaves_len - 1 {
        hashers.push(
            builder.hash_n_to_hash_no_pad::<PoseidonHash>(
                hashers[k * 2]
                    .elements
                    .iter()
                    .copied()
                    .chain(hashers[k * 2 + 1].elements.iter().copied())
                    .collect(),
            ),
        );

        k += 1;
    }

    HashTreeRootPoseidonTargets {
        leaves,
        hash_tree_root: hashers[leaves_len - 2],
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        hash::{
            hashing::hash_n_to_hash_no_pad,
            poseidon::{PoseidonHash, PoseidonPermutation},
        },
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    // TODO: CHANGE THIS TEST TO ACTUALLY TEST HASH TREE ROOT
    #[test]
    fn test_poseidon_hash() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();

        let c = builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![a, b]);

        builder.register_public_inputs(&c.elements);

        let mut pw: PartialWitness<F> = PartialWitness::new();

        pw.set_target(a, F::from_canonical_u32(1));
        pw.set_target(b, F::from_canonical_u32(2));

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        let result = hash_n_to_hash_no_pad::<F, PoseidonPermutation<F>>(&vec![
            F::from_canonical_u32(1),
            F::from_canonical_u32(2),
        ]);

        assert_eq!(result.elements, proof.public_inputs[0..4]);

        data.verify(proof)
    }
}
