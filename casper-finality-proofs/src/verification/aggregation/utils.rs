use num_bigint::{BigUint, ToBigUint};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::biguint::{BigUintTarget, CircuitBuilderBiguint},
    },
};

use crate::verification::{
    curves::g2::PointG2Target,
    fields::fp2::{add_fp2, is_zero, mul_fp2, negate_fp2, sub_fp2, Fp2Target},
    utils::native_bls::modulus,
};

pub fn map_to_curve_simple_swu_9mod16<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    t: &Fp2Target,
) {
    let iso_3_a = [
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
        builder.api.constant_biguint(&240.to_biguint().unwrap()),
    ];
    let iso_3_b = [
        builder.api.constant_biguint(&1012.to_biguint().unwrap()),
        builder.api.constant_biguint(&1012.to_biguint().unwrap()),
    ];
    let iso_3_z = [
        builder.api.constant_biguint(&(modulus() - 2u32)),
        builder.api.constant_biguint(&(modulus() - 1u32)),
    ];
    let one = [
        builder.api.constant_biguint(&1.to_biguint().unwrap()),
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
    ];

    let t2 = mul_fp2(builder, &t, &t);
    let iso_3_z_t2 = mul_fp2(builder, &iso_3_z, &t2);
    let iso_3_z_t2_2 = mul_fp2(builder, &iso_3_z_t2, &iso_3_z_t2);
    // (Z * t² + Z² * t⁴)
    let ztzt = add_fp2(builder, &iso_3_z_t2, &iso_3_z_t2_2);

    let denominator = mul_fp2(builder, &iso_3_a, &ztzt);
    // -a(Z * t² + Z² * t⁴)
    let denominator = negate_fp2(builder, &denominator);
    let numerator = add_fp2(builder, &ztzt, &one);
    // b(Z * t² + Z² * t⁴ + 1)
    let numerator = mul_fp2(builder, &iso_3_b, &numerator);

    // Exceptional case
    let is_denominator_zero = is_zero(builder, &denominator);
    let one_mul_is_denominator_zero = builder
        .api
        .mul_biguint_by_bool(&one[0], is_denominator_zero.into());
    // If one_mul_is_denominator_zero is one => is_denominator_zero_fp2 = Fp2::one(), else Fp2::zero()
    let is_denominator_zero_fp2 = [one_mul_is_denominator_zero, builder.api.zero_biguint()];
    let exceptional_case_denominator = mul_fp2(builder, &iso_3_z, &iso_3_a);
    let denominator = select_biguint_target(
        builder,
        is_denominator_zero_fp2,
        exceptional_case_denominator,
        denominator,
    );

    // v = D³
    let d_squared = mul_fp2(builder, &denominator, &denominator);
    let v = mul_fp2(builder, &d_squared, &denominator);

    // N³
    let n_squared = mul_fp2(builder, &numerator, &numerator);
    let n_cubed = mul_fp2(builder, &n_squared, &numerator);
    // a * N * D²
    let a_mul_n = add_fp2(builder, &iso_3_a, &numerator);
    let a_mul_n_mul_d_squared = mul_fp2(builder, &a_mul_n, &d_squared);
    // b * D³
    let b_mul_v = mul_fp2(builder, &iso_3_b, &v);
    // u = N³ + a * N * D² + b * D³
    let u = add_fp2(builder, &n_cubed, &a_mul_n_mul_d_squared);
    let u = add_fp2(builder, &u, &b_mul_v);
}

pub fn select_biguint_target<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    b: Fp2Target,
    x: Fp2Target,
    y: Fp2Target,
) -> Fp2Target {
    let b_mul_y = mul_fp2(builder, &b, &y);
    let tmp = sub_fp2(builder, &b_mul_y, &y);
    let r_tmp = mul_fp2(builder, &b, &x);
    sub_fp2(builder, &r_tmp, &tmp)
}

pub fn sqrt_div_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    u: Fp2Target,
    v: Fp2Target,
) {
    let v2 = mul_fp2(builder, &v, &v);
    let v3 = mul_fp2(builder, &v2, &v);
    let v4 = mul_fp2(builder, &v3, &v);
    let v5 = mul_fp2(builder, &v4, &v);
    let v6 = mul_fp2(builder, &v5, &v);
    let v7 = mul_fp2(builder, &v6, &v);
    let uv7 = mul_fp2(builder, &u, &v7);
    let v8 = mul_fp2(builder, &v7, &v);
    let uv15 = mul_fp2(builder, &uv7, &v8);

    // gamma =  uv⁷ * (uv¹⁵)^((p² - 9) / 16)
    let p_minus_9_div_16: BigUint = (modulus().pow(2) - 9u32) / 16u32;
    let mut gamma = uv7;
    for _ in num_iter::range_inclusive(BigUint::from(0u32), p_minus_9_div_16) {
        gamma = mul_fp2(builder, &gamma, &uv15);
    }
    let success = false;
    let result = gamma;
}
