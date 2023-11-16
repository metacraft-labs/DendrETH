use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        ArrayVariable, BoolVariable, Bytes32Variable, CircuitBuilder, Field, PlonkParameters,
        Variable,
    },
};

use crate::combine_finality_votes::verify_subcommittee_vote::{
    maybe_set_nth_bit_in_packed_bitmask, VARIABLES_COUNT_LITTLE_BITMASK,
};

const TOTAL_BITMASK_SIZE: usize = 13_600_000; // MUST BE DIVISIBLE BY 63
const BITMASK_PARTITIONS: usize = 400;
const PARTITION_SIZE: usize = TOTAL_BITMASK_SIZE / BITMASK_PARTITIONS; //This must be divisible by 63

const VALIDATORS_PER_COMMITTEE: usize = 128; // 2048 MAX
const BITS_IN_FIELD: usize = 63;
const PACKED_VALIDATOR_COUNT: usize = PARTITION_SIZE.div_ceil(BITS_IN_FIELD);

#[derive(Debug, Clone)]
pub struct ValidatorBitmask;

impl Circuit for ValidatorBitmask {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        // let zero_field = <L as PlonkParameters<D>>::Field::from_canonical_u64(0);
        /*
        let bits_in_field_field =
            <L as PlonkParameters<D>>::Field::from_canonical_u64(BITS_IN_FIELD as u64);
            */

        let validators = builder.read::<ArrayVariable<Variable, VALIDATORS_PER_COMMITTEE>>();
        let range_begin = builder.read::<Variable>();
        let range_size: Variable = builder.constant(
            <L as PlonkParameters<D>>::Field::from_canonical_usize(PARTITION_SIZE),
        );

        // let mut validators_vec: Vec<Variable> = Vec::with_capacity(VALIDATORS_PER_COMMITTEE);
        // validators_vec.resize(VALIDATORS_PER_COMMITTEE, builder.constant(zero_field));

        let mut packed_bitmask_vec: Vec<Variable> = Vec::new();
        packed_bitmask_vec.resize(VARIABLES_COUNT_LITTLE_BITMASK, builder.zero());

        // let bits_in_field_const = builder.constant(bits_in_field_field);
        // let zero_u64: Variable = builder.constant(zero_field);
        let range_end = builder.add(range_begin, range_size);

        let pow2_consts = get_pow2_arr(builder);

        // Instantiate packed bitmask
        for cur_validator in validators.data {
            let is_validator_in_partition =
                is_valid_validator_range(builder, range_begin, range_end, cur_validator);

            let partition_idx = builder.sub(cur_validator, range_begin); // Index in Partition (if valid)

            maybe_set_nth_bit_in_packed_bitmask(
                builder,
                &mut packed_bitmask_vec,
                partition_idx,
                &pow2_consts,
                is_validator_in_partition,
            );

            /*
                let (quotient, cur_bit) =
                    variable_int_div_rem(builder, partition_idx, bits_in_field_const); // quotient := Packed Partition Index; cur_bit := Bit to set in Field element

                let cur_pow2 = builder.select_array(&pow2_consts, cur_bit); // Value to Add

                // let mut packed_variable = builder.select_array(packed_bitmask_vec.as_slice(), quotient);

                let packed_idx_field =
                    <L as PlonkParameters<D>>::Field::from_canonical_u64(packed_idx as u64);
                let idx_to_var = builder.constant(packed_idx_field);
                let are_at_idx = builder.is_equal(idx_to_var, quotient);
                // let cur_to_add = builder.select(are_at_idx, cur_pow2, zero_u64);

                let correct_idx_and_valid_partition =
                    builder.and(are_at_idx, is_validator_in_partition);
                let value_to_add =
                    builder.select(correct_idx_and_valid_partition, cur_pow2, zero_u64);
                packed_bitmask_vec[packed_idx] =
                    builder.add(packed_bitmask_vec[packed_idx], value_to_add);

                // Carry-over vulnerability check
                let is_counted =
                    variable_int_rem(builder, packed_bitmask_vec[packed_idx], cur_pow2);
                builder.assert_is_equal(is_counted, zero_u64);
                //TODO: Underconstrained

                // packed_bitmask_vec[packed_idx] = builder.add(cur_to_add, cur_to_add);
            }
            */
        }

        let packed_bitmask: ArrayVariable<Variable, VARIABLES_COUNT_LITTLE_BITMASK> =
            ArrayVariable::new(packed_bitmask_vec);

        let source: Variable =
            builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(1));
        let target: Variable =
            builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(2));
        let voted_count: Variable =
            builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(3));
        let bls_signature: Variable =  // trqq si Bytes32Variable
            builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(3));

        builder.write(target);
        builder.write(source);

        builder.write(bls_signature);
        builder.write(voted_count);

        builder.write(range_begin);
        builder.write(packed_bitmask);
    }
}

fn is_valid_validator_range<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    begin: Variable,
    end: Variable,
    validator: Variable,
) -> BoolVariable {
    let is_gte = builder.gte(validator, begin);
    let is_lt = builder.lt(validator, end);
    let validator_is_in_range = builder.and(is_gte, is_lt);
    validator_is_in_range
}

fn get_pow2_arr<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
) -> Vec<Variable> {
    let mut pow2_consts: Vec<Variable> = Vec::with_capacity(BITS_IN_FIELD);
    pow2_consts.resize(BITS_IN_FIELD, builder.zero());
    for i in 0..BITS_IN_FIELD {
        let cur_pow = (2 as u64).pow(i as u32);
        let cur_pow_field = <L as PlonkParameters<D>>::Field::from_canonical_u64(cur_pow);

        pow2_consts[i] = builder.constant(cur_pow_field);
    }
    pow2_consts
}
