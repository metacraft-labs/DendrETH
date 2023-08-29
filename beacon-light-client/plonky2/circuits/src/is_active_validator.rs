use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    utils::biguint_is_equal,
};

pub struct IsActiveValidatorTargets {
    pub activation_epoch: [Target; 2],
    pub current_epoch: [Target; 2],
    pub exit_epoch: [Target; 2],
}

pub fn is_active_validator<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    activation_epoch: &BigUintTarget,
    current_epoch: &BigUintTarget,
    exit_epoch: &BigUintTarget,
) -> BoolTarget {
    let activation_epoch_le_current_epoch = builder.cmp_biguint(&activation_epoch, &current_epoch);

    let current_epoch_le_exit_epoch = builder.cmp_biguint(&current_epoch, &exit_epoch);

    let is_equal = biguint_is_equal(builder, current_epoch, exit_epoch);
    let not_equal = builder.not(is_equal);

    let current_epoch_lt_exit_epoch = builder.and(current_epoch_le_exit_epoch, not_equal);

    builder.and(
        activation_epoch_le_current_epoch,
        current_epoch_lt_exit_epoch,
    )
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use num::{BigUint, FromPrimitive};
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use crate::{
        biguint::{CircuitBuilderBiguint, WitnessBigUint},
        is_active_validator::is_active_validator,
    };

    fn test_helper(
        activation_epoch_value: u64,
        current_epoch_value: u64,
        exit_epoch_value: u64,
        is_positive: bool,
    ) -> std::result::Result<(), anyhow::Error> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let activation_epoch = builder.add_virtual_biguint_target(2);
        let current_epoch = builder.add_virtual_biguint_target(2);
        let exit_epoch = builder.add_virtual_biguint_target(2);

        let result =
            is_active_validator(&mut builder, &activation_epoch, &current_epoch, &exit_epoch);

        pw.set_biguint_target(
            &activation_epoch,
            &BigUint::from_u64(activation_epoch_value).unwrap(),
        );

        pw.set_biguint_target(
            &current_epoch,
            &BigUint::from_u64(current_epoch_value).unwrap(),
        );

        pw.set_biguint_target(&exit_epoch, &BigUint::from_u64(exit_epoch_value).unwrap());

        if is_positive {
            builder.assert_one(result.target);
        } else {
            builder.assert_zero(result.target);
        }

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    fn test_is_active_validator_valid() -> Result<()> {
        test_helper(6953401, 6953401, 6953402, true)
    }

    #[test]
    fn test_is_active_validator_exit_epoch_is_less_than_current_epoch() -> Result<()> {
        test_helper(6953401, 6953401, 6953400, false)
    }

    #[test]
    fn test_is_active_validator_exit_epoch_is_equal_to_current_epoch() -> Result<()> {
        test_helper(6953400, 6953401, 6953401, false)
    }

    #[test]
    fn test_is_active_validator_activation_epoch_is_bigger_than_current_epoch() -> Result<()> {
        test_helper(6953402, 6953401, 6953403, false)
    }
}
