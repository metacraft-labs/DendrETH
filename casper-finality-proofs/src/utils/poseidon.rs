use plonky2x::{backend::circuit::PlonkParameters, frontend::{builder::CircuitBuilder, eth::vars::BLSPubkeyVariable, hash::poseidon::poseidon256::PoseidonHashOutVariable, uint::uint64::U64Variable, vars::{CircuitVariable, Variable}}};

use crate::combine_finality_votes::count_unique_pubkeys::CommitmentMapperVariable;

pub fn ssz_verify_proof_poseidon<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    root: PoseidonHashOutVariable,
    leaf: PoseidonHashOutVariable,
    branch: &[PoseidonHashOutVariable],
    gindex: U64Variable,
) where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let expected_root = ssz_restore_merkle_root_poseidon(builder, leaf, branch, gindex);
    builder.assert_is_equal(root, expected_root);
}

fn ssz_restore_merkle_root_poseidon<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    leaf: PoseidonHashOutVariable,
    branch: &[PoseidonHashOutVariable],
    gindex: U64Variable,
) -> PoseidonHashOutVariable
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let bits = builder.to_le_bits(gindex);
    let mut hash = leaf;
    for i in 0..branch.len() {

        let case1 = builder.poseidon_hash_pair(branch[i].clone(), hash.clone());
        let case2 = builder.poseidon_hash_pair(hash.clone(), branch[i].clone());
        hash = builder.select(bits[i], case1, case2);
    }

    hash
}

pub fn verify_pubkey_leaf_for_validator_index<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    validators_root: PoseidonHashOutVariable,
    branch: &[PoseidonHashOutVariable],
    pubkey: BLSPubkeyVariable,
    index: Variable,
) where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let start_validator_pubkey_gindex = 2u64.pow(43) - 1;
    let start_validator_pubkey_gindex =
        builder.constant::<U64Variable>(start_validator_pubkey_gindex);

    let index_64_variables = vec![builder.zero(), index];
    let index_u64 = U64Variable::from_variables(builder, &index_64_variables);

    let const_8 = builder.constant::<U64Variable>(8);
    let pubkey_offset = builder.mul(index_u64, const_8);

    let gindex = builder.add(start_validator_pubkey_gindex, pubkey_offset);
    let leaf = pubkey.hash_tree_root(builder);

    ssz_verify_proof_poseidon(builder, validators_root, leaf, branch, gindex);
}
