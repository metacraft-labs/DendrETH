use plonky2x::prelude::{CircuitBuilder, PlonkParameters};

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
