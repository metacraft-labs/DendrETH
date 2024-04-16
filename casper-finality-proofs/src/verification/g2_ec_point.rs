use plonky2x::{backend::circuit::PlonkParameters, frontend::builder::CircuitBuilder};

use super::{
    fields::plonky2::fp2_plonky2::{add_fp2, inv_fp2, mul_fp2, sub_fp2, Fp2Target},
    native::get_bls_12_381_parameter,
};

pub type PointG2Target = [Fp2Target; 2];

pub fn my_g2_add<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &PointG2Target,
    b: &PointG2Target,
) -> PointG2Target {
    let x1 = &a[0];
    let y1 = &a[1];
    let x2 = &b[0];
    let y2 = &b[1];

    let mut builder = builder.api;

    let u = sub_fp2(builder, &y2, &y1);
    let v = sub_fp2(builder, &x2, &x1);
    let v_inv = inv_fp2(builder, &v);
    let s = mul_fp2(builder, &u, &v_inv);
    let s_squared = mul_fp2(builder, &s, &s);
    let x_sum = add_fp2(builder, &x2, &x1);
    let x3 = sub_fp2(builder, &s_squared, &x_sum);
    let x_diff = sub_fp2(builder, &x1, &x3);
    let prod = mul_fp2(builder, &s, &x_diff);
    let y3 = sub_fp2(builder, &prod, &y1);

    [x3, y3]
}

pub fn g2_scalar_mul<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    p: &PointG2Target,
    iso_3_b: &Fp2Target,
) -> PointG2Target {
    let iso_3_a = [
        builder.constant_biguint(&0.to_biguint().unwrap()),
        builder.constant_biguint(&0.to_biguint().unwrap()),
    ];
    let mut r = [
        [
            builder.add_virtual_biguint_target(N),
            builder.add_virtual_biguint_target(N),
        ],
        [
            builder.add_virtual_biguint_target(N),
            builder.add_virtual_biguint_target(N),
        ],
    ];
    let fals = builder._false();
    for i in (0..get_bls_12_381_parameter().bits()).rev() {
        if i == get_bls_12_381_parameter().bits() - 1 {
            for idx in 0..2 {
                for jdx in 0..2 {
                    builder.connect_biguint(&r[idx][jdx], &p[idx][jdx]);
                }
            }
        } else {
            let pdouble = g2_double(builder, &r, &iso_3_a, iso_3_b);
            if !get_bls_12_381_parameter().bit(i) {
                r = pdouble;
            } else {
                r = my_g2_add(builder, &pdouble, fals, p, fals, &iso_3_a, iso_3_b);
            }
        }
    }
    r
}
