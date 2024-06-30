use crate::{
    self as circuit,
    circuit_builder_extensions::CircuitBuilderExtensions,
    define_uint_target_type,
    targets::uint::ops::{
        arithmetic::{Add, Div, Mul, One, Rem, Sub, Zero},
        comparison::{Comparison, EqualTo, LessThanOrEqual},
    },
    AddVirtualTarget, PublicInputsReadable, PublicInputsTargetReadable, SetWitness,
    TargetPrimitive, ToTargets,
};
use circuit_derive::SerdeCircuitTarget;
use itertools::Itertools;
use num_bigint::BigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::PartialWitness,
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_crypto::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    u32::{
        arithmetic_u32::{CircuitBuilderU32, U32Target},
        witness::WitnessU32,
    },
};
use primitive_types::{U256, U512};

mod r#macro;
pub mod ops;

define_uint_target_type!(Uint64Target, u64);
define_uint_target_type!(Uint128Target, u128);
define_uint_target_type!(Uint256Target, U256);
define_uint_target_type!(Uint512Target, U512);

fn assert_limbs_are_valid<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    limbs: &[U32Target],
) {
    for &limb in limbs {
        let upper_bound = builder.constant_biguint(&BigUint::from(2u64.pow(32) - 1));
        let limb_biguint = BigUintTarget { limbs: vec![limb] };
        let limb_is_valid = builder.cmp_biguint(&limb_biguint, &upper_bound);
        builder.assert_true(limb_is_valid);
    }
}

const fn num_limbs<T>() -> usize {
    debug_assert!(std::mem::size_of::<T>() % 4 == 0);
    std::mem::size_of::<T>() / 4
}
