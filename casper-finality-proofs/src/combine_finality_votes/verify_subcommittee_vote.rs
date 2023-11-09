use plonky2x::{
    backend::circuit::Circuit,
    prelude::{ArrayVariable, CircuitBuilder, Field, PlonkParameters, Variable},
};

use crate::utils::{bits::variable_set_nth_bit, variable::variable_int_div_rem};

pub const BITMASK_SIZE: usize = 100_000;
pub const PACK_SIZE: usize = 63;
pub const PACKS_COUNT: usize = BITMASK_SIZE.div_ceil(PACK_SIZE);

fn compute_powers_of_two<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
) -> Vec<Variable> {
    let mut powers = vec![builder.one()];
    let two = builder.constant::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(2));

    for _ in 1..64 {
        let next_power = builder.mul(*powers.last().unwrap(), two);
        powers.push(next_power);
    }

    powers
}

fn set_nth_bit_in_packed_bitmask<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    packed_bitmask: &mut [Variable],
    n: Variable,
    powers_of_two: &[Variable],
) {
    let const_pack_size = builder.constant::<Variable>(
        <L as PlonkParameters<D>>::Field::from_canonical_usize(PACK_SIZE),
    );

    let (pack_to_modify, bit_to_set_in_pack) = variable_int_div_rem(builder, n, const_pack_size);

    for i in 0..packed_bitmask.len() {
        let current_pack_idx =
            builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(i));

        let pack = builder.select_array(&packed_bitmask, current_pack_idx);
        let should_modify_pack_pred = builder.is_equal(current_pack_idx, pack_to_modify);
        let modified_pack = variable_set_nth_bit(builder, pack, bit_to_set_in_pack, &powers_of_two);

        packed_bitmask[i] = builder.select(should_modify_pack_pred, modified_pack, pack);
    }
}

#[derive(Debug, Clone)]
pub struct VerifySubcommitteeVote;

impl Circuit for VerifySubcommitteeVote {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let set_bit = builder.read::<Variable>();

        let source = builder.zero();
        let target = builder.one();
        let voted_count =
            builder.constant::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(1));

        let powers_of_two = compute_powers_of_two(builder);

        let mut bitmask_data = vec![builder.zero::<Variable>(); PACKS_COUNT];
        set_nth_bit_in_packed_bitmask(builder, &mut bitmask_data, set_bit, &powers_of_two);
        /*
        for i in 0..PACKS_COUNT {
            let current_pack_idx =
                builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(i));

            let pack = builder.select_array(&bitmask_data, current_pack_idx);
            let should_modify_pack_pred = builder.is_equal(current_pack_idx, pack_to_modify);
            let modified_pack =
                variable_set_nth_bit(builder, pack, bit_to_set_in_pack, &powers_of_two);

            bitmask_data[i] = builder.select(should_modify_pack_pred, modified_pack, pack);
        }
        */

        let bitmask: ArrayVariable<Variable, PACKS_COUNT> = ArrayVariable::new(bitmask_data);
        builder.watch(&bitmask, "bitmask");

        builder.write::<Variable>(source);
        builder.write::<Variable>(target);
        builder.write::<Variable>(voted_count);
        builder.write::<ArrayVariable<Variable, PACKS_COUNT>>(bitmask);
    }
}
