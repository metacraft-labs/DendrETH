use crate::utils::plonky2x_extensions::assert_is_true;
use crate::utils::poseidon::verify_pubkey_leaf_for_validator_index;
use crate::weigh_justification_and_finalization::checkpoint::CheckpointVariable;

use plonky2x::frontend::eth::vars::BLSPubkeyVariable;
use plonky2x::frontend::hash::poseidon::poseidon256::PoseidonHashOutVariable;
use plonky2x::prelude::{
    ArrayVariable, BoolVariable, CircuitBuilder, PlonkParameters, U256Variable,
    U64Variable, Variable,
};
use plonky2x::prelude::CircuitVariable;


const VALIDATORS_IN_SPLIT: usize = 20;

fn u256_variable_from_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    variable: Variable,
) -> U256Variable {
    let zero = builder.zero();
    let variables = vec![variable, zero, zero, zero, zero, zero, zero, zero];
    U256Variable::from_variables(builder, &variables)
}

/*
fn commit_pubkey<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    pubkey: PublicKey,
    random_value: Variable,
) -> Commitment {
    let random_value_256 = u256_variable_from_variable(builder, random_value);
    builder.mul(pubkey, random_value_256)
}

pub fn compute_commitment<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    pubkeys: &[PublicKey],
    trusted_validators_bitmask: &[BoolVariable],
    random_value: Variable,
) -> Commitment {
    /*
    let one = builder.one();
    let at_least_one_validator_pred = builder.gte(count, one);
    assert_is_true(builder, at_least_one_validator_pred);
    */

    // let mut commitment = commit_pubkey(builder, pubkeys.first().unwrap().clone());
    let mut commitment = builder.zero();

    for i in 0..pubkeys.len() {
        // let key_out_of_range_pred = builder.gte(idx, count);

        /*
        if enforce_ordering {
            let ordering_pred = builder.lt(pubkeys[i - 1], pubkeys[i]);
            let valid_key_pred = builder.or(ordering_pred, key_out_of_range_pred);
            assert_is_true(builder, valid_key_pred);
        }
        */

        // let key_in_range_pred = builder.lt(idx, count);
        let mut key_commitment = commit_pubkey(builder, pubkeys[i], random_value);
        let is_trusted_validator_256 =
            u256_variable_from_variable(builder, trusted_validators_bitmask[i].variable);
        key_commitment = builder.mul(key_commitment, is_trusted_validator_256);
        commitment = builder.add(commitment, key_commitment);
    }

    commitment
}
*/

type PubkeyProof = ArrayVariable<PoseidonHashOutVariable, 43>;

fn verify_ordering_is_satisfied_across_chunks<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    chunk_start: Variable,
    prev_chunk_end_validator_index: Variable,
    start_validator_index: Variable,
) -> BoolVariable {
    let chunk_is_first_chunk_pred = builder.is_zero(chunk_start);
    let prev_chunk_end_lt_start_validator_index_pred =
        builder.lt(prev_chunk_end_validator_index, start_validator_index);

    let prev_chunk_end_eq_start_validator_index_pred =
        builder.is_equal(prev_chunk_end_validator_index, start_validator_index);

    let prev_chunk_end_lte_start_validator_index_pred = builder.or(
        prev_chunk_end_lt_start_validator_index_pred,
        prev_chunk_end_eq_start_validator_index_pred,
    );

    let ordering_is_valid_pred = builder.or(
        chunk_is_first_chunk_pred,
        prev_chunk_end_lte_start_validator_index_pred,
    );
    assert_is_true(builder, ordering_is_valid_pred);

    prev_chunk_end_lt_start_validator_index_pred
}

pub trait CommitmentMapperVariable: CircuitVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> PoseidonHashOutVariable
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>;
}

impl CommitmentMapperVariable for BLSPubkeyVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> PoseidonHashOutVariable
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        builder.watch(self, "pubkey");
        builder.poseidon_hash(&self.variables()) //TODO: extend pubkey with ByteVariable<16> zeroes to the right
    }
}

#[derive(Debug, Clone)]
pub struct CountUniquePubkeys;

impl CountUniquePubkeys {
    pub fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let random_value = builder.read::<Variable>();
        let source = builder.read::<CheckpointVariable>();
        let target = builder.read::<CheckpointVariable>();

        let pubkeys = builder.read::<ArrayVariable<BLSPubkeyVariable, VALIDATORS_IN_SPLIT>>();
        let indices = builder.read::<ArrayVariable<Variable, VALIDATORS_IN_SPLIT>>();
        let branches = builder.read::<ArrayVariable<PubkeyProof, VALIDATORS_IN_SPLIT>>();

        let chunk_start = builder.read::<Variable>();
        let prev_chunk_end_validator_index = builder.read::<Variable>();

        let validators_root_poseidon = builder.read::<PoseidonHashOutVariable>();

        let count = verify_ordering_is_satisfied_across_chunks(
            builder,
            chunk_start,
            prev_chunk_end_validator_index,
            indices[0],
        )
        .variable;

        for i in 0..indices.len() {
            verify_pubkey_leaf_for_validator_index(
                builder,
                validators_root_poseidon.clone(),
                branches[i].as_slice(),
                pubkeys[i],
                indices[i],
            );
        }

        /*
        let commitment = compute_commitment(
            builder,
            pubkeys.as_slice(),
            trusted_validators_bitmask.as_slice(),
            random_value,
        );
        */

        // ssz hash_tree_root za pubkey
        // merkle proof za pubkey po validator index

        // builder.write(commitment);
        /*
        builder.write(count);
        builder.write(target);
        builder.write(source);
        builder.write(validators_root_poseidon);
        */
    }
}
