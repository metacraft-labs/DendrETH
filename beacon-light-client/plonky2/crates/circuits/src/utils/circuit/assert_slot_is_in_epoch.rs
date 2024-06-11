use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions,
    targets::uint::{ops::arithmetic::Div, Uint64Target},
};
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField,
    plonk::circuit_builder::CircuitBuilder,
};

pub fn assert_slot_is_in_epoch<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    slot: Uint64Target,
    current_epoch: Uint64Target,
) {
    let slots_per_epoch = Uint64Target::constant(32, builder);
    let slot_epoch = slot.div(slots_per_epoch, builder);
    builder.assert_targets_are_equal(&slot_epoch, &current_epoch);
}

#[cfg(test)]
mod test_assert_slot_is_in_epoch {
    use circuit::{serde::serde_u64_str, targets::uint::Uint64Target, Circuit, SetWitness};
    use circuit_derive::CircuitTarget;
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };
    use serde_json::json;

    use crate::utils::circuit::assert_slot_is_in_epoch::assert_slot_is_in_epoch;

    #[derive(CircuitTarget)]
    struct SlotIsInEpochTestCircuitTarget {
        #[target(in)]
        #[serde(with = "serde_u64_str")]
        pub slot: Uint64Target,

        #[target(in)]
        #[serde(with = "serde_u64_str")]
        pub epoch: Uint64Target,
    }

    struct AssertSlotIsInEpochTestCircuit;

    impl Circuit for AssertSlotIsInEpochTestCircuit {
        type F = GoldilocksField;
        type C = PoseidonGoldilocksConfig;
        const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

        type Target = SlotIsInEpochTestCircuitTarget;

        fn define(builder: &mut CircuitBuilder<Self::F, 2>, _: &Self::Params) -> Self::Target {
            let input = Self::read_circuit_input_target(builder);

            assert_slot_is_in_epoch(builder, input.slot, input.epoch);

            Self::Target {
                slot: input.slot,
                epoch: input.epoch,
            }
        }
    }

    fn run_test_case(slot: u64, epoch: u64) {
        let (target, data) = AssertSlotIsInEpochTestCircuit::build(&());

        let mut pw = PartialWitness::new();
        target.set_witness(
            &mut pw,
            &serde_json::from_str(
                &json!({
                    "slot": slot.to_string(),
                    "epoch": epoch.to_string()
                })
                .to_string(),
            )
            .unwrap(),
        );

        let proof = data.prove(pw).unwrap();
        data.verify(proof).unwrap();
    }

    #[test]
    fn test_assert_slot_is_in_epoch() {
        run_test_case(6953401, 217293);
    }

    #[test]
    fn test_assert_slot_is_in_epoch_slot_in_epoch() {
        run_test_case(7314752, 228586);
    }

    #[test]
    fn test_assert_slot_is_in_epoch_last_slot_in_epoch() {
        run_test_case(7314751, 228585);
    }

    #[test]
    #[should_panic]
    fn test_assert_slot_is_not_in_epoch() {
        run_test_case(7314751, 228586);
    }
}
