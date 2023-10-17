use plonky2::field::types::Field;
use plonky2::iop::target::BoolTarget;
use plonky2x::prelude::{BoolVariable, Bytes32Variable, CircuitBuilder, PlonkParameters, BytesVariable, Variable};
use crate::utils::variable::{to_bits, to_byte_variable};
use crate::utils::universal::{assert_is_true, le_sum, div_rem, exp_from_bits};

fn compute_shuffled_index<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    mut index: Variable,
    index_count: Variable,
    seed: Bytes32Variable,
) -> Variable {
    let index_lte_index_count = builder.lte(index, index_count);
    assert_is_true(builder, index_lte_index_count);

    let const_1: Variable = builder.constant(L::Field::from_canonical_u8(1));
    let const_2: Variable = builder.constant(L::Field::from_canonical_u8(2));
    let const_8: Variable = builder.constant(L::Field::from_canonical_u8(8));
    let const_256: Variable = builder.constant(L::Field::from_canonical_u16(256));
    const SHUFFLE_ROUND_COUNT: usize = 90;
    const TEST: usize = 5;
    for current_round in 0..SHUFFLE_ROUND_COUNT {
        let current_round_variable: Variable =
            builder.constant(L::Field::from_canonical_u8(current_round as u8));
        let current_round_bytes = to_byte_variable(current_round_variable, builder);

        let mut seed_round_to_be_hashed: BytesVariable<33> = builder.init::<BytesVariable<33>>();
        for i in 0..32 {
            seed_round_to_be_hashed.0[i] = seed.0 .0[i];
        }
        seed_round_to_be_hashed.0[32] = current_round_bytes;

        // debug::debug(builder, "index_count checkpoint".to_string(), index_count);
        let seed_current_round_hashed = builder.sha256(&seed_round_to_be_hashed.0);
        // debug::debug(
        //     builder,
        //     "AFTER SHA256".to_string(),
        //     seed_current_round_hashed.0 .0[0].0[0].0,
        // );

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
        // debug::debug(builder, "pivot".to_string(), pivot);

        let sum_pivot_index_count = builder.add(pivot, index_count);
        let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
        let flip = div_rem(builder, sum_pivot_index_count_sub_index, index_count);

        let index_lte_flip = builder.lte(index, flip);
        let position = builder.select(index_lte_flip, flip, index);

        let position_div_256 = builder.div(position, const_256);
        let position_div_256_bytes = to_byte_variable(position_div_256, builder);

        let mut source_to_be_hashed: BytesVariable<34> = builder.init::<BytesVariable<34>>();
        for i in 0..32 {
            source_to_be_hashed.0[i] = seed.0 .0[i];
        }
        source_to_be_hashed.0[32] = current_round_bytes;
        source_to_be_hashed.0[33] = position_div_256_bytes;

        let source = builder.sha256(&source_to_be_hashed.0);

        let position_mod_256 = div_rem(builder, position, const_8);
        let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

        let byte = builder.select_array(&source.0 .0, position_mod_256_div_8);
        let byte_to_variable = byte.to_variable(builder);

        let position_mod_8 = div_rem(builder, position, const_8);
        let position_mod_8_to_bits: [BoolVariable; 8] = to_bits(position_mod_8, builder);
        let position_mod_8_to_iter = position_mod_8_to_bits
            .into_iter()
            .map(|x| BoolTarget::new_unsafe(x.0 .0));
        let const_2_pow_position_mod_8 =
            builder.api.exp_from_bits(const_2.0, position_mod_8_to_iter);

        let byte_shl_position_mod_8 =
            builder.div(byte_to_variable, Variable(const_2_pow_position_mod_8));
        let bit = div_rem(builder, byte_shl_position_mod_8, const_2);
        let bit_eq_1 = builder.is_equal(bit, const_1);
        index = builder.select(bit_eq_1, flip, index);
    }

    index
}

#[cfg(test)]
mod tests {
    use plonky2::field::goldilocks_field::GoldilocksField;

    use super::*;
    use plonky2x::backend::circuit::DefaultParameters;
    use plonky2x::frontend::vars::{ArrayVariable, ByteVariable};
    use plonky2x::prelude::{bytes, DefaultBuilder};

    #[test]
    fn test_compute_shuffled_index() {
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
        input.write::<ArrayVariable<ByteVariable, 32>>(seed_bytes_fixed_size.to_vec());
        input.write::<Variable>(F::from_canonical_usize(10));
        input.write::<Variable>(F::from_canonical_usize(5));

        let (_witness, mut _output) = circuit.mock_prove(&input);
    }
}
