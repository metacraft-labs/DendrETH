use ark_bls12_381::G1Affine;
use ark_std::One;
use num_bigint::BigUint;
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{builder::CircuitBuilder, uint::num::biguint::CircuitBuilderBiguint},
};

use crate::verification::fields::fp::FpTarget;

pub type PointG1Target = [FpTarget; 2];

pub fn generate_new_g1_point_target<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    g1: G1Affine,
) -> PointG1Target {
    [
        builder
            .api
            .constant_biguint(&g1.x.to_string().parse::<BigUint>().unwrap()),
        builder
            .api
            .constant_biguint(&g1.y.to_string().parse::<BigUint>().unwrap()),
    ]
}

pub fn g1_ecc_aggregate<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    lhs: PointG1Target,
    rhs: PointG1Target,
) -> PointG1Target {
    let x1 = lhs[0].clone();
    let y1 = lhs[1].clone();
    let x2 = rhs[0].clone();
    let y2 = rhs[1].clone();

    let one = builder.api.constant_biguint(&BigUint::one());
    let u = builder.api.sub_biguint(&y2, &y1);
    let v = builder.api.sub_biguint(&x2, &x1);
    let v_inv = builder.api.div_biguint(&one, &v);
    let s = builder.api.mul_biguint(&u, &v_inv);
    let s_squared = builder.api.mul_biguint(&s, &s);
    let x_sum = builder.api.add_biguint(&x2, &x1);
    let x3 = builder.api.sub_biguint(&s_squared, &x_sum);
    let x_diff = builder.api.sub_biguint(&x1, &x3);
    let prod = builder.api.mul_biguint(&s, &x_diff);
    let y3 = builder.api.sub_biguint(&prod, &y1);

    [x3, y3]
}
