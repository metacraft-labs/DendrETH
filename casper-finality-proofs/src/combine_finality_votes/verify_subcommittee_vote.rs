use plonky2x::{
    backend::circuit::Circuit,
    prelude::{ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable},
};

use crate::utils::{bits::variable_set_nth_bit, variable::variable_int_div_rem};

pub const BITMASK_SIZE: usize = 100_000;
pub const PACK_SIZE: usize = 63;
pub const PACKS_COUNT: usize = BITMASK_SIZE.div_ceil(PACK_SIZE);
pub const VALIDATORS_PER_COMMITTEE: usize = 128;
pub const VALIDATOR_SIZE_UPPER_BOUND: usize = 100_000;

// deli na 63
// n - na kolko bitmaski razdelqme mama bitmaska (n trqq da e stepen na dvoikata)

pub const VARIABLES_COUNT_LITTLE_BITMASK: usize = 100;
pub const BITMASK_SPLITS_COUNT: usize = 2usize.pow(1);

// v - kolko variable-a e edna malka bitmaska
// razmera na golqmata bitmaska v * n
// v * n * 63

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

pub fn maybe_set_nth_bit_in_packed_bitmask<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    packed_bitmask: &mut [Variable],
    n: Variable,
    powers_of_two: &[Variable],
    should_set_bit: BoolVariable,
) {
    let const_pack_size = builder.constant::<Variable>(
        <L as PlonkParameters<D>>::Field::from_canonical_usize(PACK_SIZE),
    );

    let (pack_to_modify, bit_to_set_in_pack) = variable_int_div_rem(builder, n, const_pack_size);

    for i in 0..packed_bitmask.len() {
        let pack = packed_bitmask[i];
        let current_pack_idx =
            builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(i));

        let mut should_modify_pack_pred = builder.is_equal(current_pack_idx, pack_to_modify);
        should_modify_pack_pred = builder.and(should_modify_pack_pred, should_set_bit);
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
        let validator_indices = builder.read::<ArrayVariable<Variable, VALIDATORS_PER_COMMITTEE>>();

        let source = builder.zero();
        let target = builder.one();
        let voted_count =
            builder.constant::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(1));

        let powers_of_two = compute_powers_of_two(builder);

        let mut bitmask_data = vec![builder.zero::<Variable>(); PACKS_COUNT];

        let _true = builder._true();

        for index in 0..VALIDATORS_PER_COMMITTEE {
            let validator_index = validator_indices[index];
            maybe_set_nth_bit_in_packed_bitmask(
                builder,
                &mut bitmask_data,
                validator_index,
                &powers_of_two,
                _true,
            );
        }

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

        builder.write::<Variable>(source);
        builder.write::<Variable>(target);
        builder.write::<Variable>(voted_count);
        builder.write::<ArrayVariable<Variable, PACKS_COUNT>>(bitmask);
    }
}
