use crate::{
    make_uint32_n,
    targets::uint::ops::{
        arithmetic::{Add, Div, LessThanOrEqual, Mul, One, Rem, Zero},
        comparison::EqualTo,
    },
    AddVirtualTarget, PublicInputsReadable, PublicInputsTargetReadable, SetWitness,
    TargetPrimitive, ToTargets,
};
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

make_uint32_n!(Uint64Target, u64, 2);
make_uint32_n!(Uint128Target, u128, 4);
make_uint32_n!(Uint256Target, U256, 8);
make_uint32_n!(Uint512Target, U512, 16);

fn assert_limbs_are_valid<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    limbs: &[U32Target],
) -> BoolTarget {
    let mut is_valid = builder._true();

    for &limb in limbs {
        let upper_bound = builder.constant_biguint(&BigUint::from(2u64.pow(32) - 1));
        let limb_biguint = BigUintTarget { limbs: vec![limb] };
        let limb_is_valid = builder.cmp_biguint(&limb_biguint, &upper_bound);
        is_valid = builder.and(is_valid, limb_is_valid)
    }
    is_valid
}
