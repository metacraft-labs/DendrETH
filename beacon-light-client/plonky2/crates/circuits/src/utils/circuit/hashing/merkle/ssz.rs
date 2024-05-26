use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::BoolTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2_crypto::biguint::BigUintTarget;

use crate::common_targets::SSZLeafTarget;
use crate::utils::circuit::{biguint_to_bits_target, bits_to_biguint_target, reverse_endianness};

pub fn ssz_merklelize_bool<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bool_target: BoolTarget,
) -> SSZLeafTarget {
    let mut ssz_leaf = [BoolTarget::new_unsafe(builder.zero()); 256];
    ssz_leaf[7] = bool_target;
    ssz_leaf
}
pub fn ssz_num_to_bits<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    num: &BigUintTarget,
    bit_len: usize,
) -> SSZLeafTarget {
    assert!(bit_len <= 256);

    let mut bits = reverse_endianness(&biguint_to_bits_target::<F, D, 2>(builder, num));
    bits.extend((bit_len..256).map(|_| builder._false()));

    bits.try_into().unwrap()
}
pub fn ssz_num_from_bits<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bits: &[BoolTarget],
) -> BigUintTarget {
    bits_to_biguint_target(builder, reverse_endianness(bits))
}
