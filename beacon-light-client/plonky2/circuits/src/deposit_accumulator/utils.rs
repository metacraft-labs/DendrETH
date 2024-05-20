use itertools::Itertools;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget, plonk::circuit_builder::CircuitBuilder
};

use crate::utils::ETH_SHA256_BIT_SIZE;

use super::objects::{AccumulatedDataTargets, NodeTargets, RangeObject, ValidatorStatsTargets};

pub fn set_validator_stats_public_variables<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator_stats: &ValidatorStatsTargets,
) {
    builder.register_public_input(validator_stats.non_activated_validators_count);
    builder.register_public_input(validator_stats.active_validators_count);
    builder.register_public_input(validator_stats.exited_validators_count);
    builder.register_public_input(validator_stats.slashed_validators_count);
}

pub fn set_accumulated_data_public_variables<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    accumulated: &AccumulatedDataTargets,
) {
    builder.register_public_inputs(
        &accumulated
            .balance_sum
            .limbs
            .iter()
            .map(|x| x.0)
            .collect_vec(),
    );
    builder.register_public_input(accumulated.deposits_count);

    set_validator_stats_public_variables(builder, &accumulated.validator_stats);
}

pub fn set_range_object_public_variables<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    range: &RangeObject,
) {
    builder.register_public_inputs(
        range
            .pubkey
            .iter()
            .map(|x| x.target)
            .collect_vec()
            .as_slice(),
    );
    builder.register_public_inputs(&range.deposit_index.limbs.iter().map(|x| x.0).collect_vec());
    builder.register_public_input(range.is_counted.target);
    builder.register_public_input(range.is_dummy.target);
}

pub fn set_node_public_variables<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    node: &NodeTargets,
) {
    set_range_object_public_variables(builder, &node.leftmost);
    set_range_object_public_variables(builder, &node.rightmost);

    set_accumulated_data_public_variables(builder, &node.accumulated);
    builder.register_public_inputs(&node.current_epoch.limbs.iter().map(|x| x.0).collect_vec());
    builder.register_public_inputs(
        &node
            .eth1_deposit_index
            .limbs
            .iter()
            .map(|x| x.0)
            .collect_vec(),
    );
    // builder.register_public_inputs(
    //     &node
    //         .is_valid_commitment_mapper_proof_root
    //         .elements
    //         .iter()
    //         .map(|x| x.clone())
    //         .collect_vec(),
    // );
    // builder.register_public_inputs(
    //     &node
    //         .is_valid_merkle_tree_deposit_branch_root
    //         .elements
    //         .iter()
    //         .map(|x| x.clone())
    //         .collect_vec()
    // );
}

pub fn set_final_layer_public_variables<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) {
    todo!()
}


pub fn add_sha_target<F: RichField + Extendable<D>, const D: usize>(builder: &mut CircuitBuilder<F, D>,) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
    (0..ETH_SHA256_BIT_SIZE)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
