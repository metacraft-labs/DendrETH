use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::utils::{epoch_to_mixed_endian};

pub struct IsActiveValidatorTargets {
    pub activation_epoch: [Target; 2],
    pub current_epoch: [Target; 2],
    pub exit_epoch: [Target; 2],
    pub result: BoolTarget,
}

pub fn is_active_validator<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> IsActiveValidatorTargets {
    let activation_epoch = [builder.add_virtual_target(), builder.add_virtual_target()];
    let current_epoch = [builder.add_virtual_target(), builder.add_virtual_target()];
    let exit_epoch = [builder.add_virtual_target(), builder.add_virtual_target()];

    let activation_epoch_bits = epoch_to_mixed_endian(builder, activation_epoch);

    let current_epoch_bits = epoch_to_mixed_endian(builder, current_epoch);

    let exit_epoch_bits = epoch_to_mixed_endian(builder, exit_epoch);

    let le1 = is_less_than_or_equal_epoch(builder, &activation_epoch_bits, &current_epoch_bits);

    let le2 = is_less_than_or_equal_epoch(builder, &current_epoch_bits, &exit_epoch_bits);

    let is_equal1 = builder.is_equal(current_epoch[0], exit_epoch[0]);
    let is_equal2 = builder.is_equal(current_epoch[1], exit_epoch[1]);

    let is_equal = builder.and(is_equal1, is_equal2);

    let _false = builder._false();

    let lt = builder._if(is_equal, _false.target, le2.target);

    IsActiveValidatorTargets {
        activation_epoch,
        current_epoch,
        exit_epoch,
        result: builder.and(le1, BoolTarget::new_unsafe(lt)),
    }
}

fn is_less_than_or_equal_epoch<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    activation_epoch_bits: &Vec<BoolTarget>,
    current_epoch_bits: &Vec<BoolTarget>,
) -> BoolTarget {
    let mut all_valid = Vec::<BoolTarget>::new();
    let mut prev_is_less_than = Vec::<BoolTarget>::new();

    for i in (0..64).rev() {
        let is_equal = builder.is_equal(
            activation_epoch_bits[i].target,
            current_epoch_bits[i].target,
        );
        let is_less_than = is_less_than(builder, activation_epoch_bits[i], current_epoch_bits[i]);
        let _true = builder._true();

        let current_bit_is_valid =
            BoolTarget::new_unsafe(builder._if(is_equal, _true.target, is_less_than.target));

        if i == 63 {
            prev_is_less_than.push(is_less_than);
            all_valid.push(current_bit_is_valid);
        } else {
            prev_is_less_than.push(builder.or(prev_is_less_than[62 - i], is_less_than));

            let _false = builder._false();

            let is_valid = builder.or(prev_is_less_than[62 - i], current_bit_is_valid);

            all_valid.push(BoolTarget::new_unsafe(builder._if(
                all_valid[62 - i],
                is_valid.target,
                _false.target,
            )));
        }
    }

    all_valid[63]
}

pub fn is_less_than<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: BoolTarget,
    right: BoolTarget,
) -> BoolTarget {
    let _false = builder._false();
    let _true = builder._true();

    let _result = BoolTarget::new_unsafe(builder._if(left, _false.target, right.target));
    builder.assert_bool(_result);

    _result
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use crate::is_active_validator::{is_active_validator, is_less_than};

    #[test]
    fn test_is_active_validator_valid() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = is_active_validator(&mut builder);

        pw.set_target_arr(
            &targets.activation_epoch,
            &[F::from_canonical_u64(12585651), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.current_epoch,
            &[F::from_canonical_u64(12585651), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.exit_epoch,
            &[F::from_canonical_u64(12585587), F::from_canonical_u64(0)],
        );

        builder.assert_one(targets.result.target);

        builder.register_public_input(targets.result.target);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    fn test_is_active_validator_exit_epoch_is_less_than_current_epoch() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = is_active_validator(&mut builder);

        pw.set_target_arr(
            &targets.activation_epoch,
            &[F::from_canonical_u64(0), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.current_epoch,
            &[F::from_canonical_u64(12585587), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.exit_epoch,
            &[F::from_canonical_u64(12585651), F::from_canonical_u64(0)],
        );

        builder.assert_zero(targets.result.target);

        builder.register_public_input(targets.result.target);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    fn test_is_active_validator_exit_epoch_is_equal_to_current_epoch() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = is_active_validator(&mut builder);

        pw.set_target_arr(
            &targets.activation_epoch,
            &[F::from_canonical_u64(0), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.current_epoch,
            &[F::from_canonical_u64(12585651), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.exit_epoch,
            &[F::from_canonical_u64(12585651), F::from_canonical_u64(0)],
        );

        builder.assert_zero(targets.result.target);

        builder.register_public_input(targets.result.target);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    fn test_is_active_validator_activation_epoch_is_bigger_than_current_epoch() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = is_active_validator(&mut builder);

        pw.set_target_arr(
            &targets.activation_epoch,
            &[F::from_canonical_u64(12585587), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.current_epoch,
            &[F::from_canonical_u64(12585651), F::from_canonical_u64(0)],
        );

        pw.set_target_arr(
            &targets.exit_epoch,
            &[F::from_canonical_u64(12585587), F::from_canonical_u64(0)],
        );

        builder.assert_zero(targets.result.target);

        builder.register_public_input(targets.result.target);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }


    #[test]
    fn is_less_than_test() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let left = builder.add_virtual_bool_target_safe();
        let right = builder.add_virtual_bool_target_safe();

        let targets = is_less_than(&mut builder, left, right);

        let mut pw = PartialWitness::new();

        pw.set_bool_target(left, false);
        pw.set_bool_target(right, true);

        builder.assert_one(targets.target);

        builder.register_public_input(targets.target);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }
}
