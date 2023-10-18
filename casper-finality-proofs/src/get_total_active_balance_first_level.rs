use plonky2x::{
    backend::circuit::Circuit,
    frontend::{eth::beacon::vars::BeaconValidatorVariable, vars::EvmVariable},
    prelude::{
        ArrayVariable, BoolVariable, Bytes32Variable, CircuitBuilder, PlonkParameters,
        U256Variable, U64Variable,
    },
};

use crate::commitment_mapper_variable::{poseidon_hash_tree_root_leafs, CommitmentMapperVariable};

pub const VALIDATORS_PER_CHUNK: usize = 8;
pub const BALANCES_PER_CHUNK: usize = 2;

#[derive(Debug, Clone)]
pub struct TotalActiveBalanceFirstLevel;

impl Circuit for TotalActiveBalanceFirstLevel {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let validators =
            builder.read::<ArrayVariable<BeaconValidatorVariable, VALIDATORS_PER_CHUNK>>();
        let balances_leaves = builder.read::<ArrayVariable<Bytes32Variable, BALANCES_PER_CHUNK>>();

        let balances_root = builder.ssz_hash_leafs(balances_leaves.as_slice());

        let mut validators_leaves = Vec::new();

        for i in 0..VALIDATORS_PER_CHUNK {
            validators_leaves.push(CommitmentMapperVariable::hash_tree_root(
                &validators.data[i],
                builder,
            ));
        }

        let validators_hash_tree_root = poseidon_hash_tree_root_leafs(builder, &validators_leaves);

        let current_epoch = builder.read::<U256Variable>();

        let mut sum = builder.zero::<U64Variable>();

        for i in 0..VALIDATORS_PER_CHUNK {
            let balance = U64Variable::decode(
                builder,
                &balances_leaves.data[i / 4].0 .0[i % 4 * 8..i % 4 * 8 + 8],
            );

            builder.watch(&balance, "log_balance");

            let is_active = is_active_validator(builder, validators.data[i], current_epoch);

            let zero = builder.zero::<U64Variable>();

            let current = builder.select(is_active, balance, zero);

            sum = builder.add(sum, current);
        }

        builder.write(sum);
        builder.write(current_epoch);
        builder.write(validators_hash_tree_root);
        builder.write(balances_root);
    }
}

fn is_active_validator<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_validator: BeaconValidatorVariable,
    current_epoch: U256Variable,
) -> BoolVariable {
    let activation_epoch_lte_current_epoch =
        builder.lte(beacon_validator.activation_epoch, current_epoch);

    let current_epoch_lt_exit_epoch = builder.lt(current_epoch, beacon_validator.exit_epoch);

    builder.and(
        activation_epoch_lte_current_epoch,
        current_epoch_lt_exit_epoch,
    )
}
