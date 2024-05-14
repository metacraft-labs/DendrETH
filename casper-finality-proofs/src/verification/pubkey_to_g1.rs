use num_bigint::BigUint;
use plonky2::field::types::Field;
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{BigUintTarget, CircuitBuilderBiguint},
            u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
        },
        vars::Variable,
    },
};

use crate::utils::plonky2x_extensions::{assert_zero, variable_to_le_bits};

use super::{curves::g1::PointG1Target, fields::fp::LIMBS, utils::native_bls::modulus};

pub const PUB_KEY_LEN: usize = 48;

pub fn pubkey_to_g1_verification<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    point: &PointG1Target,
    pk: &[Variable; PUB_KEY_LEN],
) {
    let msbs = variable_to_le_bits(builder, pk[0], 8);
    let bflag = msbs[6];
    assert_zero(builder, bflag.variable);

    let aflag = msbs[5];

    let (x, y) = (&point[0], &point[1]);
    let two = builder.api.constant_biguint(&2u32.into());
    let y_2 = builder.api.mul_biguint(y, &two);
    let p = builder.api.constant_biguint(&modulus());
    let y_2_p = builder.api.div_biguint(&y_2, &p);
    let zero = builder.api.zero_u32();
    for i in 0..y_2_p.limbs.len() {
        if i == 0 {
            builder.connect(aflag.variable, Variable(y_2_p.limbs[i].target));
        } else {
            builder.api.connect_u32(y_2_p.limbs[i], zero);
        }
    }

    let z_limbs: Vec<U32Target> = pk
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            let zero = builder.api.zero();
            let factor = builder.api.constant(L::Field::from_canonical_u32(256));
            U32Target::from_target_unsafe(
                chunk
                    .iter()
                    .fold(zero, |acc, c| builder.api.mul_add(acc, factor, (*c).0)),
            )
        })
        .rev()
        .collect();
    let z = BigUintTarget { limbs: z_limbs };

    let pow_2_383 = builder
        .api
        .constant_biguint(&(BigUint::from(1u32) << 383u32));
    let pow_2_381 = builder
        .api
        .constant_biguint(&(BigUint::from(1u32) << 381u32));
    let pow_2_381_or_zero = BigUintTarget {
        limbs: (0..LIMBS)
            .into_iter()
            .map(|i| {
                U32Target::from_target_unsafe(builder.api.select(
                    aflag.into(),
                    pow_2_381.limbs[i].target,
                    zero.target,
                ))
            })
            .collect(),
    };
    let flags = builder.api.add_biguint(&pow_2_383, &pow_2_381_or_zero);
    let z_reconstructed = builder.api.add_biguint(x, &flags);

    builder.api.connect_biguint(&z, &z_reconstructed);
}
