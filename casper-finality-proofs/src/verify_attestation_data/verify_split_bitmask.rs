use plonky2x::{
    backend::circuit::Circuit,
    prelude::{ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable},
};

use crate::combine_finality_votes::verify_subcommittee_vote::{
    maybe_set_nth_bit_in_packed_bitmask, PACK_SIZE, VALIDATORS_PER_COMMITTEE,
    VARIABLES_COUNT_LITTLE_BITMASK,
};

#[derive(Debug, Clone)]
pub struct ValidatorBitmask;

impl Circuit for ValidatorBitmask {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let pack_size = builder.constant::<Variable>(
            <L as PlonkParameters<D>>::Field::from_canonical_usize(PACK_SIZE),
        );

        let validators = builder.read::<ArrayVariable<Variable, VALIDATORS_PER_COMMITTEE>>();
        let range_begin = builder.read::<Variable>();
        let range_size: Variable = builder.constant(
            <L as PlonkParameters<D>>::Field::from_canonical_usize(VARIABLES_COUNT_LITTLE_BITMASK),
        );
        let range_size = builder.mul(range_size, pack_size);

        let mut packed_bitmask_vec: Vec<Variable> = Vec::new();
        packed_bitmask_vec.resize(VARIABLES_COUNT_LITTLE_BITMASK, builder.zero());

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
    let mut pow2_consts: Vec<Variable> = Vec::with_capacity(PACK_SIZE);
    pow2_consts.resize(PACK_SIZE, builder.zero());
    for i in 0..PACK_SIZE {
        let cur_pow = (2 as u64).pow(i as u32);
        let cur_pow_field = <L as PlonkParameters<D>>::Field::from_canonical_u64(cur_pow);

        pow2_consts[i] = builder.constant(cur_pow_field);
    }
    pow2_consts
}
