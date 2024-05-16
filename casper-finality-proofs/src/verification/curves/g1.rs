use ark_std::One;
use num_bigint::BigUint;
use plonky2::plonk::{
    circuit_builder::CircuitBuilder,
    config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2x::frontend::uint::num::biguint::CircuitBuilderBiguint;

use crate::verification::fields::fp::FpTarget;

pub type PointG1Target = [FpTarget; 2];

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub fn g1_ecc_aggregate(
    builder: &mut CircuitBuilder<F, D>,
    lhs: PointG1Target,
    rhs: PointG1Target,
) -> PointG1Target {
    let x1 = lhs[0].clone();
    let y1 = lhs[1].clone();
    let x2 = rhs[0].clone();
    let y2 = rhs[1].clone();

    let one = builder.constant_biguint(&BigUint::one());
    let u = builder.sub_biguint(&y2, &y1);
    let v = builder.sub_biguint(&x2, &x1);
    let v_inv = builder.div_biguint(&one, &v);
    let s = builder.mul_biguint(&u, &v_inv);
    let s_squared = builder.mul_biguint(&s, &s);
    let x_sum = builder.add_biguint(&x2, &x1);
    let x3 = builder.sub_biguint(&s_squared, &x_sum);
    let x_diff = builder.sub_biguint(&x1, &x3);
    let prod = builder.mul_biguint(&s, &x_diff);
    let y3 = builder.sub_biguint(&prod, &y1);

    [x3, y3]
}
