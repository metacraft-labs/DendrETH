use plonky2::plonk::{
    circuit_builder::CircuitBuilder,
    circuit_data::CircuitConfig,
    config::{GenericConfig, PoseidonGoldilocksConfig},
};

use crate::validator_balance_circuit::{
    validator_balance_verification, ValidatorBalanceVerificationTargets,
};

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

    builder.register_public_input(validator_balance_verification_targets.range_total_value);

    builder.register_public_inputs(
        &validator_balance_verification_targets
            .range_balances_root
            .map(|x| x.target),
    );

    builder.register_public_inputs(&validator_balance_verification_targets.withdrawal_credentials);

    builder.register_public_inputs(
        &validator_balance_verification_targets
            .range_validator_commitment
            .elements,
    );

    builder.register_public_inputs(&validator_balance_verification_targets.current_epoch);

    let data = builder.build::<C>();

    (validator_balance_verification_targets, data)
}
