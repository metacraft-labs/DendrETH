use num_bigint::BigUint;
use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
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

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub fn pubkey_to_g1_check<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    point: &PointG1Target,
    pk: &[Target; PUB_KEY_LEN],
) {
    let msbs = builder.split_le(pk[0], 8);
    let bflag = msbs[6];
    builder.assert_zero(bflag.target);

    let aflag = msbs[5];

    let (x, y) = (&point[0], &point[1]);
    let two = builder.constant_biguint(&2u32.into());
    let y_2 = builder.mul_biguint(y, &two);
    let p = builder.constant_biguint(&modulus());
    let y_2_p = builder.div_biguint(&y_2, &p);
    let zero = builder.zero_u32();
    for i in 0..y_2_p.limbs.len() {
        if i == 0 {
            builder.connect(aflag.target, y_2_p.limbs[i].target);
        } else {
            builder.connect_u32(y_2_p.limbs[i], zero);
        }
    }

    let z_limbs: Vec<U32Target> = pk
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            let zero = builder.zero();
            let factor = builder.constant(F::from_canonical_u32(256));
            U32Target::from_target_unsafe(
                chunk
                    .iter()
                    .fold(zero, |acc, c| builder.mul_add(acc, factor, *c)),
            )
        })
        .rev()
        .collect();
    let z = BigUintTarget { limbs: z_limbs };

    let pow_2_383 = builder.constant_biguint(&(BigUint::from(1u32) << 383u32));
    let pow_2_381 = builder.constant_biguint(&(BigUint::from(1u32) << 381u32));
    let pow_2_381_or_zero = BigUintTarget {
        limbs: (0..LIMBS)
            .into_iter()
            .map(|i| {
                U32Target::from_target_unsafe(builder.select(
                    aflag,
                    pow_2_381.limbs[i].target,
                    zero.target,
                ))
            })
            .collect(),
    };
    let flags = builder.add_biguint(&pow_2_383, &pow_2_381_or_zero);
    let z_reconstructed = builder.add_biguint(x, &flags);

    builder.connect_biguint(&z, &z_reconstructed);
}
