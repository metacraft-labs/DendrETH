use itertools::Itertools;
use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};

use crate::{
    biguint::BigUintTarget,
    utils::ETH_SHA256_BIT_SIZE,
    validator_balance_circuit::{
        validator_balance_verification, ValidatorBalanceVerificationTargets,
    },
};

pub const RANGE_TOTAL_VALUE_PUB_INDEX: usize = 0;
pub const RANGE_BALANCES_ROOT_PUB_INDEX: usize = 2;
pub const WITHDRAWAL_CREDENTIALS_PUB_INDEX: usize = 258;
pub const WITHDRAWAL_CREDENTIALS_SIZE: usize = 8;
pub const RANGE_VALIDATOR_COMMITMENT_PUB_INDEX: usize = 266;
pub const CURRENT_EPOCH_PUB_INDEX: usize = 270;

pub fn build_validator_balance_circuit(
    validators_len: usize,
) -> (
    ValidatorBalanceVerificationTargets,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    let validator_balance_verification_targets =
        validator_balance_verification(&mut builder, validators_len);

    set_public_variables(
        &mut builder,
        &validator_balance_verification_targets.range_total_value,
        validator_balance_verification_targets.range_balances_root,
        &validator_balance_verification_targets.withdrawal_credentials,
        validator_balance_verification_targets.range_validator_commitment,
        &validator_balance_verification_targets.current_epoch,
    );

    let data = builder.build::<C>();

    (validator_balance_verification_targets, data)
}

pub fn set_public_variables(
    builder: &mut CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2>,
    range_total_value: &BigUintTarget,
    range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    withdrawal_credentials: &BigUintTarget,
    range_validator_commitment: HashOutTarget,
    current_epoch: &BigUintTarget,
) {
    builder.register_public_inputs(&range_total_value.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_inputs(&range_balances_root.map(|x| x.target));

    builder.register_public_inputs(
        &withdrawal_credentials
            .limbs
            .iter()
            .map(|x| x.0)
            .collect_vec(),
    );

    builder.register_public_inputs(&range_validator_commitment.elements);

    builder.register_public_inputs(&current_epoch.limbs.iter().map(|x| x.0).collect_vec());
}
