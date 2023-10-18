use crate::utils::universal::{assert_is_true, div_rem};
use crate::utils::variable::{to_bits, to_byte_variable};
use ethers::abi::Bytes;
use itertools::Itertools;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::iop::target::BoolTarget;
use plonky2x::frontend::vars::EvmVariable;
use plonky2x::prelude::{
    BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitBuilder, CircuitVariable,
    PlonkParameters, U64Variable, Variable,
};

// TODO: use U64Variable instead of Variable
fn compute_shuffled_index<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    mut index: Variable,
    index_count: Variable,
    seed: Bytes32Variable,
) -> Variable {
    let index_lte_index_count = builder.lte(index, index_count);
    assert_is_true(builder, index_lte_index_count); // Check if that's true

    let const_1: Variable = builder.constant(L::Field::from_canonical_u8(1));
    let const_2: Variable = builder.constant(L::Field::from_canonical_u8(2));
    let const_8: Variable = builder.constant(L::Field::from_canonical_u8(8));
    let const_256: Variable = builder.constant(L::Field::from_canonical_usize(256));
    let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
    const SHUFFLE_ROUND_COUNT: usize = 90;
    const TEST: usize = 3;
    for current_round in 0..TEST {
        let current_round_bytes: ByteVariable =
            ByteVariable::constant(builder, current_round as u8);

        let mut seed_round_to_be_hashed: BytesVariable<33> = BytesVariable([const_0_byte; 33]);
        for i in 0..32 {
            seed_round_to_be_hashed.0[i] = seed.0 .0[i];
        }
        seed_round_to_be_hashed.0[32] = current_round_bytes;

        let seed_current_round_hashed = builder.sha256(&seed_round_to_be_hashed.0);

        let mut seed_current_round_hashed_first_64_bits: Vec<BoolVariable> = Vec::new();
        for i in 0..8 {
            for j in 0..8 {
                seed_current_round_hashed_first_64_bits
                    .push(seed_current_round_hashed.0 .0[i].0[j]);
            }
        }

        let mut power_of_2 = const_1;
        let mut seed_current_round_hash_to_variable = builder.constant(L::Field::ZERO);
        for i in 0..64 {
            let addend = builder.mul(seed_current_round_hashed_first_64_bits[i].0, power_of_2);
            seed_current_round_hash_to_variable =
                builder.add(addend, seed_current_round_hash_to_variable);
            power_of_2 = builder.mul(const_2, power_of_2);
        }
        let pivot = div_rem(builder, seed_current_round_hash_to_variable, index_count);

        let sum_pivot_index_count = builder.add(pivot, index_count);
        let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
        let flip = div_rem(builder, sum_pivot_index_count_sub_index, index_count);

        let index_lte_flip = builder.lte(index, flip);

        let position = builder.select(index_lte_flip, flip, index);

        let position_div_256 = builder.div(position, const_256);

        let mut position_div_256_temp = position_div_256;

        let mut position_div_256_temp_bits =
            builder.to_le_bits(position_div_256_temp);

        let mut position_div_256_temp_bytes = Vec::new();

        for i in 0..4 {
            position_div_256_temp_bytes.push(ByteVariable(
                position_div_256_temp_bits[i * 8..(i + 1) * 8]
                    .try_into()
                    .unwrap(),
            ));
        }

        let position_div_256_bytes =
            BytesVariable::<4>(position_div_256_temp_bytes.try_into().unwrap());

        builder.watch(&position_div_256_temp_bits, "bits");

        builder.watch(&position_div_256_bytes, "bytes");

        // let mut result_vec = Vec::new();
        // for _ in 0..4 {
        //     let low_bits = builder.api.low_bits(position_div_256.0, 8, 8);
        //     let bits: [BoolVariable; 8] = low_bits
        //         .iter()
        //         .map(|x| BoolVariable::from(Variable(x.target)))
        //         .collect_vec()
        //         .try_into()
        //         .unwrap();
        //     let byte_var = ByteVariable(bits);
        //     result_vec.push(byte_var);

        //     position_div_256_temp = builder.div(position_div_256_temp, const_256);
        // }

        // for i in 0..4 {
        //     for j in 0..8 {
        //         builder.watch(&result_vec[i].0[j].0, "result_vec");
        //     }
        // }

        // let position_div_256_bytes = ByteVariable::from_target(builder, position_div_256.0);
        // debug::debug(
        //     builder,
        //     "position_div_256_bytes - in variable".to_string(),
        //     position_div_256_bytes.0[0].0,
        // );

        // let position_div_256_bytes = position_div_256_bytes.to_variable(builder);

        // debug::debug(
        //     builder,
        //     "position_div_256_bytes - in variable second".to_string(),
        //     position_div_256_bytes,
        // );

        // let position_div_256_bits: [BoolVariable; 64] = position_div_256.to_bits(builder);

        // for i in 0..64 {
        //     debug::debug(
        //         builder,
        //         "position_div_256_bits".to_string(),
        //         position_div_256_bits[i].0,
        //     );
        // }

        // let position_div_256_bytes = to_byte_variable(position_div_256, builder);
        // let mut source_to_be_hashed: BytesVariable<34> = BytesVariable([const_0_byte; 34]);
        // for i in 0..32 {
        //     source_to_be_hashed.0[i] = seed.0 .0[i];
        // }
        // source_to_be_hashed.0[32] = current_round_bytes;
        // source_to_be_hashed.0[33] = position_div_256_bytes;

        // let source = builder.sha256(&source_to_be_hashed.0);

        // let position_mod_256 = div_rem(builder, position, const_8);
        // let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

        // let byte = builder.select_array(&source.0 .0, position_mod_256_div_8);
        // let byte_to_variable = byte.to_variable(builder);
        // let position_mod_8 = div_rem(builder, position, const_8);
        // let position_mod_8_to_bits: [BoolVariable; 8] = to_bits(position_mod_8, builder);
        // let position_mod_8_to_iter = position_mod_8_to_bits
        //     .into_iter()
        //     .map(|x| BoolTarget::new_unsafe(x.0 .0));
        // let const_2_pow_position_mod_8 =
        //     builder.api.exp_from_bits(const_2.0, position_mod_8_to_iter);

        // let byte_shl_position_mod_8 =
        //     builder.div(byte_to_variable, Variable(const_2_pow_position_mod_8));
        // let bit = div_rem(builder, byte_shl_position_mod_8, const_2);
        // let bit_eq_1 = builder.is_equal(bit, const_1);
        // index = builder.select(bit_eq_1, flip, index);
    }

    index
}

#[cfg(test)]
mod tests {
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2x::utils;

    use super::*;
    use plonky2x::backend::circuit::DefaultParameters;
    use plonky2x::frontend::vars::{ArrayVariable, ByteVariable};
    use plonky2x::prelude::{bytes, DefaultBuilder};

    #[test]
    fn test_compute_shuffled_index() {
        utils::setup_logger();
        let mut builder: CircuitBuilder<DefaultParameters, 2> = DefaultBuilder::new();
        type F = GoldilocksField;

        let index = builder.read::<Variable>();
        let index_count = builder.read::<Variable>();
        let seed = builder.read::<Bytes32Variable>();

        let shuffled_index = compute_shuffled_index(&mut builder, index, index_count, seed);

        builder.watch(&shuffled_index, "shuffled_index");
        let circuit = builder.mock_build();

        let mut input = circuit.input();
        let seed_bytes: Vec<u8> =
            bytes!("0x23bcd11624a07465b1c2fc1a0fe52996daae4bf87b0fb6bed45926096c644843");

        let mut seed_bytes_fixed_size = [0u8; 32];
        seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);
        input.write::<Variable>(F::from_canonical_usize(5));
        input.write::<Variable>(F::from_canonical_usize(10));
        input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

        let (_witness, mut _output) = circuit.mock_prove(&input);
    }
}
