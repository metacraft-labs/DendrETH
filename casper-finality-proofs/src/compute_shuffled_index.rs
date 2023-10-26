use crate::utils::{
    universal::{assert_is_true, div_rem, div_rem_u64, exp_from_bits, le_sum},
    variable::to_bits,
};
use plonky2::field::types::Field;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitBuilder,
        CircuitVariable, PlonkParameters, U64Variable, Variable,
    },
};

fn to_bin<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    hashed: Bytes32Variable,
    start_idx: usize,
    end_idx: usize,
) -> Variable {
    let mut hashed_half: Vec<BoolVariable> = Vec::new();
    for i in start_idx..end_idx {
        for j in 0..8 {
            hashed_half.push(hashed.0 .0[7 - i].0[j]);
        }
    }

    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));

    let mut pow_2 = builder.constant(L::Field::from_canonical_usize(1));
    let mut bin = builder.constant(L::Field::from_canonical_usize(0));
    for i in 0..32 {
        let addend = builder.mul(hashed_half[31 - i].0, pow_2);
        bin = builder.add(addend, bin);
        pow_2 = builder.mul(const_2, pow_2);
    }

    bin
}

#[derive(Debug, Clone)]
struct ComputeShuffledIndex;

impl Circuit for ComputeShuffledIndex {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let mut index = builder.read::<U64Variable>();
        let index_count = builder.read::<U64Variable>();
        let seed = builder.read::<Bytes32Variable>();

        let index_lte_index_count = builder.lte(index, index_count);
        assert_is_true(builder, index_lte_index_count); // Check if that's true

        let const_0: Variable = builder.constant(L::Field::from_canonical_usize(0));
        let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
        let const_1 = builder.constant::<U64Variable>(2);
        let const_2u64 = builder.constant::<U64Variable>(2);
        let const_8 = builder.constant::<U64Variable>(8);
        let const_256 = builder.constant::<U64Variable>(256);
        let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
        const SHUFFLE_ROUND_COUNT: usize = 10;
        for current_round in 0..SHUFFLE_ROUND_COUNT {
            let current_round_bytes: ByteVariable =
                ByteVariable::constant(builder, current_round as u8);

            let mut seed_round_to_be_hashed: BytesVariable<33> = BytesVariable([const_0_byte; 33]);
            for i in 0..32 {
                seed_round_to_be_hashed.0[i] = seed.0 .0[i];
            }
            seed_round_to_be_hashed.0[32] = current_round_bytes;

            let seed_current_round_hashed: Bytes32Variable =
                builder.sha256(&seed_round_to_be_hashed.0);
            let seed_current_round_hashed_1: Vec<[BoolVariable; 8]> = seed_current_round_hashed
                .as_bytes()
                .iter()
                .map(|x| x.as_be_bits())
                .collect();
            let seed_current_round_hashed_2: Bytes32Variable = Bytes32Variable(BytesVariable(
                seed_current_round_hashed_1
                    .iter()
                    .map(|x| ByteVariable(*x))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            ));

            let first_seed_current_round_hash_to_variable =
                to_bin(builder, seed_current_round_hashed_2, 0, 4);
            let second_seed_current_round_hash_to_variable =
                to_bin(builder, seed_current_round_hashed_2, 4, 8);

            let seed_current_round_hash_to_u64variable_combined = U64Variable::from_variables(
                builder,
                &[
                    second_seed_current_round_hash_to_variable,
                    first_seed_current_round_hash_to_variable,
                ],
            );

            let pivot = div_rem_u64(
                builder,
                seed_current_round_hash_to_u64variable_combined,
                index_count,
            );

            // builder.watch(&pivot, format!("pivot [{}]:", current_round).as_str());

            let sum_pivot_index_count = builder.add(pivot, index_count);
            let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
            let flip = div_rem_u64(builder, sum_pivot_index_count_sub_index, index_count);
            // builder.watch(&flip, format!("flip [{}]:", current_round).as_str());

            let index_lte_flip = builder.lte(index, flip);
            let position = builder.select(index_lte_flip, flip, index);

            // builder.watch(&position, "position");
            // builder.watch(&position, format!("position [{}]:", current_round).as_str());

            let position_div_256 = builder.div(position, const_256);
            // builder.watch(
            //     &position_div_256,
            //     format!("position_div_256 {}", current_round).as_str(),
            // );
            let position_div_256_bits = builder.to_le_bits(position_div_256);
            // let y = le_sum(builder, &position_div_256_bits);
            let mut position_div_256_bytes = Vec::new();

            for i in 0..4 {
                position_div_256_bytes.push(ByteVariable(
                    position_div_256_bits[i * 8..(i + 1) * 8]
                        .try_into()
                        .unwrap(),
                ));
            }

            // for i in 0..position_div_256_bytes.len() {
            //     let var = position_div_256_bytes[i].to_variable(builder);
            //     builder.watch(
            //         &var,
            //         format!("pos 256 byte [{}] {}:", i, current_round).as_str(),
            //     );
            // }

            let mut source_to_be_hashed: BytesVariable<37> = BytesVariable([const_0_byte; 37]);
            for i in 0..32 {
                source_to_be_hashed.0[i] = seed.0 .0[i];
            }
            source_to_be_hashed.0[32] = current_round_bytes;
            for i in 0..4 {
                source_to_be_hashed.0[33 + i] = position_div_256_bytes[i];
            }

            let source = builder.sha256(&source_to_be_hashed.0);

            let position_mod_256 = div_rem_u64(builder, position, const_256);
            let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

            // builder.watch(
            //     &position_mod_256,
            //     format!("position_mod_256 {}", current_round).as_str(),
            // );
            // builder.watch(
            //     &position_mod_256_div_8,
            //     format!("position_mod_256_div_8 {}", current_round).as_str(),
            // );

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

            let byte = builder.select_array(&source.0 .0, position_mod_256_div_8_variable);
            let byte_to_variable = byte.to_variable(builder);
            // builder.watch(
            //     &byte_to_variable,
            //     format!("---> byte_to_variable {}", current_round).as_str(),
            // );

            let position_mod_8 = div_rem_u64(builder, position, const_8);
            // builder.watch(
            //     &position_mod_8,
            //     format!("position_mod_8 {}", current_round).as_str(),
            // );

            let position_mod_8_to_bits = builder.to_le_bits(position_mod_8);
            // for i in 0..position_mod_8_to_bits.len() {
            //     builder.watch(&position_mod_8_to_bits[i].0, "position_mod_8_to_bits");
            // }
            let const_2_pow_position_mod_8 =
                exp_from_bits(builder, const_2, &position_mod_8_to_bits);
            // builder.watch(
            //     &const_2_pow_position_mod_8,
            //     format!("const_2_pow_position_mod_8 {}", current_round).as_str(),
            // );

            let byte_u_var = U64Variable::from_variables(builder, &[byte_to_variable, const_0]);
            // builder.watch(
            //     &byte_u_var,
            //     format!("byte_u_var {}", current_round).as_str(),
            // );

            let const_2_pow_position_mod_8_u =
                U64Variable::from_variables(builder, &[const_2_pow_position_mod_8, const_0]);

            let byte_shl_position_mod_8 = builder.div(byte_u_var, const_2_pow_position_mod_8_u);
            // builder.watch(
            //     &byte_shl_position_mod_8,
            //     format!("byte_shl_position_mod_8 {}", current_round).as_str(),
            // );
            // builder.watch(&byte_shl_position_mod_8, "byte_shl_position_mod_8");

            let bit = div_rem_u64(builder, byte_shl_position_mod_8, const_2u64);
            // builder.watch(&bit, format!("bit {}", current_round).as_str());
            // builder.watch(&bit, format!("bit [{}]:", current_round).as_str());
            // builder.watch(
            //     &bit.variables()[0],
            //     format!("bit vars[0] [{}]:", current_round).as_str(),
            // );
            // builder.watch(
            //     &bit.variables()[1],
            //     format!("bit vars[1] [{}]:", current_round).as_str(),
            // );

            // index = if builder.is_zero(bit) { flip } else { index };
            index = builder.select(BoolVariable(bit.variables()[0]), flip, index);
        }

        builder.watch(&index, "index");
        builder.write::<U64Variable>(index);
    }
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

        let seed_bytes: Vec<u8> =
            bytes!("0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88");

        let mut seed_bytes_fixed_size = [0u8; 32];
        seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);

        let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
        ComputeShuffledIndex::define(&mut builder);

        let circuit = builder.build();

        const start_idx: u64 = 0;
        const count: u64 = 100;
            let mapping = [6, 9, 13, 34, 11, 55, 71, 27, 14, 12, 22, 69, 18, 76, 63, 51, 70, 92, 73,
            79, 29, 1, 80, 91, 88, 35, 43, 30, 25, 49, 81, 65, 32, 47, 57, 93, 45, 23, 15, 59,
            89, 24, 44, 61, 95, 53, 77, 56, 60, 5, 68, 97, 48, 28, 84, 82, 16, 96, 33, 94, 67,
            41, 31, 21, 85, 38, 58, 54, 87, 8, 3, 86, 2, 50, 37, 72, 52, 42, 62, 40, 39, 46,
            0, 19, 10, 4, 26, 99, 98, 75, 90, 17, 74, 7, 64, 83, 66, 20, 78, 36];
        for i in start_idx..count {
            let mut input = circuit.input();

            input.write::<U64Variable>(i);
            input.write::<U64Variable>(count);
            input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

            let (_witness, mut _output) = circuit.prove(&input);
            circuit.verify(&_witness, &input, &_output);
            let shuffled_index_res = _output.read::<U64Variable>();

            println!("shuffled_index_res: {:?}", shuffled_index_res);
            assert!(mapping[i as usize] == shuffled_index_res);
        }
    }

    // #[test]
    // fn test_compute_shuffled_index_10() {
    //     utils::setup_logger();
    //     let mut builder: CircuitBuilder<DefaultParameters, 2> = DefaultBuilder::new();

    //     let index = builder.read::<U64Variable>();
    //     let index_count = builder.read::<U64Variable>();
    //     let seed = builder.read::<Bytes32Variable>();

    //     let shuffled_index = compute_shuffled_index(&mut builder, index, index_count, seed);

    //     builder.watch(&shuffled_index, "shuffled_index");
    //     let circuit = builder.mock_build();

    //     let mut input = circuit.input();
    //     let seed_bytes: Vec<u8> =
    //         bytes!("0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88");

    //     let mut seed_bytes_fixed_size = [0u8; 32];
    //     seed_bytes_fixed_size[..32].copy_from_slice(&seed_bytes);

    //     let mapping = [2, 5, 6, 0, 1, 7, 4, 3, 8, 9];
    //     let mut res: [u64; 10] = [0; 10];
    //     for i in 0..10 {
    //         input.write::<U64Variable>(i);
    //         input.write::<U64Variable>(10);
    //         input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());

    //         let (_witness, mut _output) = circuit.mock_prove(&input);
    //         let shuffled_index_res = shuffled_index.get(&_witness);
    //         res[i as usize] = shuffled_index_res;
    //     }

    //     for i in 0..10 {
    //         assert!(mapping[i] == res[i], "Fails at {} index", i);
    //         println!("{} {}", mapping[i as usize], res[i as usize]);
    //     }
    // }
}
