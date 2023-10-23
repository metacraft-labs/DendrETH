use crate::utils::universal::{assert_is_true, div_rem_u64, exp_from_bits};
use plonky2::field::types::Field;
use plonky2x::prelude::{
    BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitBuilder, CircuitVariable,
    PlonkParameters, U64Variable, Variable,
};

fn compute_shuffled_index<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    mut index: U64Variable,
    index_count: U64Variable,
    seed: Bytes32Variable,
) -> U64Variable {
    let index_lte_index_count = builder.lte(index, index_count);
    assert_is_true(builder, index_lte_index_count); // Check if that's true

    let const_0: Variable = builder.constant(L::Field::from_canonical_usize(0));
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let const_1 = builder.constant::<U64Variable>(2);
    let const_2u64 = builder.constant::<U64Variable>(2);
    let const_8 = builder.constant::<U64Variable>(8);
    let const_256 = builder.constant::<U64Variable>(256);
    let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
    const SHUFFLE_ROUND_COUNT: usize = 90;
    for current_round in 0..SHUFFLE_ROUND_COUNT {
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

        let test = current_round_bytes.to_variable(builder);
        builder.watch(&test, "current_round_variable");

        let mut power_of_2u64 = builder.constant::<U64Variable>(1);
        let mut seed_current_round_hash_to_u64variable = builder.constant::<U64Variable>(0);
        for i in 0..64 {
            let current_hashed_bit = U64Variable::from_variables(builder, &[seed_current_round_hashed_first_64_bits[i].0, const_0]);
            let addend = builder.mul(current_hashed_bit, power_of_2u64);
            seed_current_round_hash_to_u64variable =
                builder.add(addend, seed_current_round_hash_to_u64variable);
            power_of_2u64 = builder.mul(const_2u64, power_of_2u64);
        }
        let pivot = div_rem_u64(builder, seed_current_round_hash_to_u64variable, index_count);

        builder.watch(&pivot, "pivot");

        let sum_pivot_index_count = builder.add(pivot, index_count);
        let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
        let flip = div_rem_u64(builder, sum_pivot_index_count_sub_index, index_count);

        builder.watch(&flip, "flip");

        builder.watch(&pivot, "after pivot");
        let index_lte_flip = builder.lte(index, flip);
        let position = builder.select(index_lte_flip, flip, index);

        builder.watch(&position, "position");

        let position_div_256 = builder.div(position, const_256);
        let position_div_256_bits = builder.to_le_bits(position_div_256);
        builder.watch(&position, "after position");
        let mut position_div_256_temp_bytes = Vec::new();

        for i in 0..4 {
            position_div_256_temp_bytes.push(ByteVariable(
                position_div_256_bits[i * 8..(i + 1) * 8]
                    .try_into()
                    .unwrap(),
            ));
        }

        let mut source_to_be_hashed: BytesVariable<37> = BytesVariable([const_0_byte; 37]);
        for i in 0..32 {
            source_to_be_hashed.0[i] = seed.0 .0[i];
        }
        source_to_be_hashed.0[32] = current_round_bytes;
        for i in 0..4 {
            source_to_be_hashed.0[33 + i] = position_div_256_temp_bytes[i];
        }

        let source = builder.sha256(&source_to_be_hashed.0);

        let position_mod_256 = div_rem_u64(builder, position, const_8);
        let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

        let position_mod_256_div_8_bits = builder.to_le_bits(position_mod_256_div_8);
        // for i in 0..position_mod_256_div_8_bits.len() {
        //     builder.watch(&position_mod_256_div_8_bits[i].0, "position_mod_256_div_8_bits index: ");
        // }

        let mut power_of_2 = builder.constant(L::Field::ONE);
        let mut position_mod_256_div_8_variable = builder.constant(L::Field::ZERO);
        for i in 0..64 {
            let addend = builder.mul(position_mod_256_div_8_bits[i].0, power_of_2);
            position_mod_256_div_8_variable =
                builder.add(addend, position_mod_256_div_8_variable);
            power_of_2 = builder.mul(const_2, power_of_2);
        }

        let byte = builder.select_array(&source.0.0, position_mod_256_div_8_variable);
        let byte_to_variable = byte.to_variable(builder);
        builder.watch(&byte_to_variable, "byte_to_variable");

        let position_mod_8 = div_rem_u64(builder, position, const_8);
        let position_mod_8_to_bits = builder.to_le_bits(position_mod_8);
        let const_2_pow_position_mod_8 = exp_from_bits(builder, const_2, &position_mod_8_to_bits);
        let byte_shl_position_mod_8 = builder.div(byte_to_variable, const_2_pow_position_mod_8);
        let byte_shl_position_mod_8u64 = U64Variable::from_variables(builder, &[byte_shl_position_mod_8, const_0]);

        let bit = div_rem_u64(builder, byte_shl_position_mod_8u64, const_2u64);
        let bit_eq_1 = builder.is_equal(bit, const_1);
        index = builder.select(bit_eq_1, flip, index);

        builder.watch(&index, "index");
    }

    index
}

#[cfg(test)]
mod tests {
    use plonky2x::utils;

    use super::*;
    use plonky2x::backend::circuit::DefaultParameters;
    use plonky2x::frontend::vars::{ArrayVariable, ByteVariable};
    use plonky2x::prelude::{bytes, DefaultBuilder};

    #[test]
    fn test_compute_shuffled_index_0() {
        utils::setup_logger();
        let mut builder: CircuitBuilder<DefaultParameters, 2> = DefaultBuilder::new();

        let index = builder.read::<U64Variable>();
        let index_count = builder.read::<U64Variable>();
        let seed = builder.read::<Bytes32Variable>();

        let shuffled_index = compute_shuffled_index(&mut builder, index, index_count, seed);

        builder.watch(&shuffled_index, "shuffled_index");
        let circuit = builder.mock_build();

        let mut input = circuit.input();
        let seed_bytes: Vec<u8> =
            bytes!("0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88");

        let mut seed_bytes_fixed_size = [0u8; 32];
        seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);


        let mapping = [0];
        let mut res: [u64; 1] = [0];
        for i in 0..1 {
            input.write::<U64Variable>(i);
            input.write::<U64Variable>(1);
            input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

            let (_witness, mut _output) = circuit.mock_prove(&input);
            let shuffled_index_res = shuffled_index.get(&_witness);
            res[i as usize] = shuffled_index_res;
        }

        for i in 0..1 {
            assert!(mapping[i] == res[i]);
            println!("{} {}", mapping[i as usize], res[i as usize]);
        }
    }

    fn test_compute_shuffled_index_10() {
        utils::setup_logger();
        let mut builder: CircuitBuilder<DefaultParameters, 2> = DefaultBuilder::new();

        let index = builder.read::<U64Variable>();
        let index_count = builder.read::<U64Variable>();
        let seed = builder.read::<Bytes32Variable>();

        let shuffled_index = compute_shuffled_index(&mut builder, index, index_count, seed);

        builder.watch(&shuffled_index, "shuffled_index");
        let circuit = builder.mock_build();

        let mut input = circuit.input();
        let seed_bytes: Vec<u8> =
            bytes!("0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88");

        let mut seed_bytes_fixed_size = [0u8; 32];
        seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);


        let mapping = [2, 5, 6, 0, 1, 7, 4, 3, 8, 9];
        let mut res: [u64; 10] = [0; 10];
        for i in 0..10 {
            input.write::<U64Variable>(i);
            input.write::<U64Variable>(10);
            input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

            let (_witness, mut _output) = circuit.mock_prove(&input);
            let shuffled_index_res = shuffled_index.get(&_witness);
            res[i as usize] = shuffled_index_res;
        }

        for i in 0..10 {
            assert!(mapping[i] == res[i]);
            println!("{} {}", mapping[i as usize], res[i as usize]);
        }
    }
}
