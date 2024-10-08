use circuit::targets::uint::{
    ops::comparison::{Comparison, LessThanOrEqual},
    Uint64Target,
};
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

pub fn get_validator_status<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    activation_epoch: Uint64Target,
    current_epoch: Uint64Target,
    exit_epoch: Uint64Target,
) -> (BoolTarget, BoolTarget, BoolTarget) {
    let is_not_activated = current_epoch.lt(activation_epoch, builder);

    let activation_epoch_le_current_epoch = activation_epoch.lte(current_epoch, builder);
    let current_epoch_lt_exit_epoch = current_epoch.lt(exit_epoch, builder);

    let is_valid_validator = builder.and(
        activation_epoch_le_current_epoch,
        current_epoch_lt_exit_epoch,
    );

    let is_validator_exited = builder.not(current_epoch_lt_exit_epoch);

    (is_not_activated, is_valid_validator, is_validator_exited)
}

pub fn get_validator_relevance<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    activation_epoch: Uint64Target,
    current_epoch: Uint64Target,
    withdrawable_epoch: Uint64Target,
) -> BoolTarget {
    let current_le_withdrawable_epoch = current_epoch.lte(withdrawable_epoch, builder);
    let activation_epoch_le_current_epoch = activation_epoch.lte(current_epoch, builder);

    builder.and(
        current_le_withdrawable_epoch,
        activation_epoch_le_current_epoch,
    )
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use circuit::{
        circuit_builder_extensions::CircuitBuilderExtensions, targets::uint::Uint64Target,
        AddVirtualTarget, SetWitness,
    };
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use crate::utils::circuit::validator_status::get_validator_status;

    use super::get_validator_relevance;

    #[test]
    fn test_get_validator_relevance() -> Result<()> {
        let mut builder =
            CircuitBuilder::<GoldilocksField, 2>::new(CircuitConfig::standard_recursion_config());

        let activation_epoch = Uint64Target::constant(28551, &mut builder);
        let current_epoch = Uint64Target::constant(285512, &mut builder);
        let withdrawable_epoch = Uint64Target::constant(2855125512, &mut builder);

        let is_validator_relevant = get_validator_relevance(
            &mut builder,
            activation_epoch,
            current_epoch,
            withdrawable_epoch,
        );

        builder.assert_true(is_validator_relevant);

        let pw = PartialWitness::new();
        let data = builder.build::<PoseidonGoldilocksConfig>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    fn test_helper(
        activation_epoch_value: u64,
        current_epoch_value: u64,
        exit_epoch_value: u64,
        assert_result: (bool, bool, bool),
    ) -> std::result::Result<(), anyhow::Error> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let activation_epoch = <Uint64Target as AddVirtualTarget>::add_virtual_target(&mut builder);
        let current_epoch = <Uint64Target as AddVirtualTarget>::add_virtual_target(&mut builder);
        let exit_epoch = <Uint64Target as AddVirtualTarget>::add_virtual_target(&mut builder);

        let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
            get_validator_status(&mut builder, activation_epoch, current_epoch, exit_epoch);

        if assert_result.0 {
            builder.assert_one(is_non_activated_validator.target);
        } else {
            builder.assert_zero(is_non_activated_validator.target);
        }

        if assert_result.1 {
            builder.assert_one(is_valid_validator.target);
        } else {
            builder.assert_zero(is_valid_validator.target);
        }

        if assert_result.2 {
            builder.assert_one(is_exited_validator.target);
        } else {
            builder.assert_zero(is_exited_validator.target);
        }

        let data = builder.build::<C>();

        let mut pw = PartialWitness::new();
        activation_epoch.set_witness(&mut pw, &activation_epoch_value);
        current_epoch.set_witness(&mut pw, &current_epoch_value);
        exit_epoch.set_witness(&mut pw, &exit_epoch_value);

        let proof = data.prove(pw).unwrap();
        data.verify(proof)
    }

    #[test]
    fn test_is_active_validator_valid() -> Result<()> {
        test_helper(6953401, 6953401, 6953402, (false, true, false))
    }

    #[test]
    fn test_is_active_validator_exit_epoch_is_less_than_current_epoch() -> Result<()> {
        test_helper(6953401, 6953401, 6953400, (false, false, true))
    }

    #[test]
    fn test_is_active_validator_exit_epoch_is_equal_to_current_epoch() -> Result<()> {
        test_helper(6953400, 6953401, 6953401, (false, false, true))
    }

    #[test]
    fn test_is_active_validator_activation_epoch_is_bigger_than_current_epoch() -> Result<()> {
        test_helper(6953402, 6953401, 6953403, (true, false, false))
    }
}
