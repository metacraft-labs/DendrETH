use num_bigint::{BigUint, ToBigUint};
use plonky2::field::types::Field;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{generator::SimpleGenerator, target::Target},
};
use plonky2x::frontend::vars::{BoolVariable, Variable};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{
                BigUintTarget, CircuitBuilderBiguint, GeneratedValuesBigUint, WitnessBigUint,
            },
            u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
        },
    },
};
use std::iter::Iterator;
pub const SIG_LEN: usize = 96;

use crate::verification::fields::fp2::inv_fp2;
use crate::verification::{
    fields::{
        fp::{fp_is_zero, LIMBS},
        fp2::{
            add_fp2, is_equal, is_zero, mul_fp2, negate_fp2, range_check_fp2, sub_fp2, Fp2Target,
        },
    },
    utils::native_bls::{get_bls_12_381_parameter, modulus, Fp, Fp2},
};
const TWO: usize = 2;
pub type PointG2Target = [Fp2Target; TWO];

pub fn g2_add_without_generator<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &PointG2Target,
    b: &PointG2Target,
) -> PointG2Target {
    let x1 = &a[0];
    let y1 = &a[1];
    let x2 = &b[0];
    let y2 = &b[1];

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

pub fn g2_add_unequal<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &PointG2Target,
    b: &PointG2Target,
) -> PointG2Target {
    let dy = sub_fp2(builder, &b[1], &a[1]);
    let dx = sub_fp2(builder, &b[0], &a[0]);
    let outx_c0 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let outx_c1 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let outy_c0 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let outy_c1 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let out = [[outx_c0, outx_c1], [outy_c0, outy_c1]];
    builder.api.add_simple_generator(G2AdditionGenerator {
        a: a.clone(),
        b: b.clone(),
        dx: dx.clone(),
        dy: dy.clone(),
        out: out.clone(),
    });
    range_check_fp2(builder, &out[0]);
    range_check_fp2(builder, &out[1]);
    let dx_sq = mul_fp2(builder, &dx, &dx);
    let dy_sq = mul_fp2(builder, &dy, &dy);

    let x1x2 = add_fp2(builder, &a[0], &b[0]);
    let x1x2x3 = add_fp2(builder, &x1x2, &out[0]);
    let cubic = mul_fp2(builder, &x1x2x3, &dx_sq);

    let cubic_dysq = sub_fp2(builder, &cubic, &dy_sq);
    let cubic_dysq_check = is_zero(builder, &cubic_dysq);
    builder.api.assert_one(cubic_dysq_check.variable.0);

    let y1y3 = add_fp2(builder, &a[1], &out[1]);
    let y1y3dx = mul_fp2(builder, &y1y3, &dx);

    let x1x3 = sub_fp2(builder, &a[0], &out[0]);
    let x1x3dy = mul_fp2(builder, &x1x3, &dy);

    let check = is_equal(builder, &y1y3dx, &x1x3dy);
    builder.api.assert_one(check.variable.0);

    out
}

pub fn g2_double<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &PointG2Target,
    iso_3_a: &Fp2Target,
    iso_3_b: &Fp2Target,
) -> PointG2Target {
    let outx_c0 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let outx_c1 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let outy_c0 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let outy_c1 = builder.api.add_virtual_biguint_target_unsafe(LIMBS);
    let out = [[outx_c0, outx_c1], [outy_c0, outy_c1]];
    builder.api.add_simple_generator(G2DoublingGenerator {
        a: a.clone(),
        iso_3_a: iso_3_a.clone(),
        out: out.clone(),
    });
    range_check_fp2(builder, &out[0]);
    range_check_fp2(builder, &out[1]);

    // point on tangent
    let x_sq = mul_fp2(builder, &a[0], &a[0]);
    let x_sq2 = add_fp2(builder, &x_sq, &x_sq);
    let x_sq3 = add_fp2(builder, &x_sq2, &x_sq);
    let x_sq3_a = add_fp2(builder, &x_sq3, iso_3_a);
    let x1_x3 = sub_fp2(builder, &a[0], &out[0]);
    let right = mul_fp2(builder, &x_sq3_a, &x1_x3);

    let y1_2 = add_fp2(builder, &a[1], &a[1]);
    let y1_y3 = add_fp2(builder, &a[1], &out[1]);
    let left = mul_fp2(builder, &y1_2, &y1_y3);

    let check = is_equal(builder, &right, &left);
    builder.api.assert_one(check.variable.0);

    // point on curve
    let outx_sq = mul_fp2(builder, &out[0], &out[0]);
    let outx_cu = mul_fp2(builder, &outx_sq, &out[0]);
    let a_outx = mul_fp2(builder, &out[0], iso_3_a);
    let outx_cu_a_outx = add_fp2(builder, &outx_cu, &a_outx);
    let right = add_fp2(builder, &outx_cu_a_outx, iso_3_b);

    let left = mul_fp2(builder, &out[1], &out[1]);

    let check = is_equal(builder, &right, &left);
    builder.api.assert_one(check.variable.0);

    let check = is_equal(builder, &a[0], &out[0]);
    builder.api.assert_zero(check.variable.0);

    out
}

pub fn g2_add<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &PointG2Target,
    is_infinity_a: BoolVariable,
    b: &PointG2Target,
    is_infinity_b: BoolVariable,
    iso_3_a: &Fp2Target,
    iso_3_b: &Fp2Target,
) -> PointG2Target {
    let x_equal = is_equal(builder, &a[0], &b[0]);
    let y_equal = is_equal(builder, &a[1], &b[1]);
    let do_double = builder.and(x_equal, y_equal);
    let add_input_b = [
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
    ];
    for i in 0..12 {
        if i == 0 {
            let zero = builder.api.zero();
            let is_zero = builder.api.is_equal(b[0][0].limbs[i].target, zero);
            let select =
                builder
                    .api
                    .select(do_double.into(), is_zero.target, b[0][0].limbs[i].target);
            builder
                .api
                .connect(add_input_b[0][0].limbs[i].target, select);
        } else {
            builder
                .api
                .connect_u32(add_input_b[0][0].limbs[i], b[0][0].limbs[i]);
        }
    }
    builder.api.connect_biguint(&add_input_b[0][1], &b[0][1]);
    builder.api.connect_biguint(&add_input_b[1][0], &b[1][0]);
    builder.api.connect_biguint(&add_input_b[1][1], &b[1][1]);
    let addition = g2_add_unequal(builder, a, &add_input_b);
    let doubling = g2_double(builder, a, iso_3_a, iso_3_b);
    let both_inf = builder.api.and(is_infinity_a.into(), is_infinity_b.into());
    let a_not_inf = builder.api.not(is_infinity_a.into());
    let b_not_inf = builder.api.not(is_infinity_b.into());
    let both_not_inf = builder.api.and(a_not_inf, b_not_inf);
    let not_y_equal = builder.not(y_equal);
    let a_neg_b = builder.and(x_equal, not_y_equal);
    let inverse = builder.api.and(both_not_inf, a_neg_b.into());
    let out_inf = builder.api.or(both_inf, inverse);
    builder.api.assert_zero(out_inf.target);
    let add_or_double_select = [
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
    ];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..LIMBS {
                let s = builder.api.select(
                    do_double.into(),
                    doubling[i][j].limbs[k].target,
                    addition[i][j].limbs[k].target,
                );
                builder
                    .api
                    .connect(add_or_double_select[i][j].limbs[k].target, s);
            }
        }
    }
    let a_inf_select = [
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
    ];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..LIMBS {
                let s = builder.api.select(
                    is_infinity_a.into(),
                    b[i][j].limbs[k].target,
                    add_or_double_select[i][j].limbs[k].target,
                );
                builder.api.connect(a_inf_select[i][j].limbs[k].target, s);
            }
        }
    }
    let b_inf_select = [
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
    ];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..LIMBS {
                let s = builder.api.select(
                    is_infinity_b.into(),
                    a[i][j].limbs[k].target,
                    a_inf_select[i][j].limbs[k].target,
                );
                builder.api.connect(b_inf_select[i][j].limbs[k].target, s);
            }
        }
    }

    b_inf_select
}

pub fn g2_negate<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    p: &PointG2Target,
) -> PointG2Target {
    [p[0].clone(), negate_fp2(builder, &p[1])]
}

pub fn g2_scalar_mul<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    p: &PointG2Target,
    iso_3_b: &Fp2Target,
) -> PointG2Target {
    let iso_3_a = [
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
    ];
    let mut r = [
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
            builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        ],
    ];
    let fals = builder._false();
    for i in (0..get_bls_12_381_parameter().bits()).rev() {
        if i == get_bls_12_381_parameter().bits() - 1 {
            for idx in 0..2 {
                for jdx in 0..2 {
                    builder.api.connect_biguint(&r[idx][jdx], &p[idx][jdx]);
                }
            }
        } else {
            let pdouble = g2_double(builder, &r, &iso_3_a, iso_3_b);
            if !get_bls_12_381_parameter().bit(i) {
                r = pdouble;
            } else {
                r = g2_add(
                    builder,
                    &pdouble,
                    fals.into(),
                    p,
                    fals.into(),
                    &iso_3_a,
                    iso_3_b,
                );
            }
        }
    }
    r
}

pub fn signature_point_check<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    point: &PointG2Target,
    sig: &[Variable; SIG_LEN],
) {
    let msbs = builder.api.split_le(sig[0].0, 8);
    let bflag = msbs[6];
    builder.api.assert_zero(bflag.target);

    let aflag = msbs[5];

    let (x0, x1, y0, y1) = (&point[0][0], &point[0][1], &point[1][0], &point[1][1]);
    let y1_zero = fp_is_zero(builder, &y1);
    let zero = builder.api.zero_u32();
    let y_select_limbs: Vec<U32Target> = (0..LIMBS)
        .into_iter()
        .map(|i| {
            U32Target::from_target_unsafe(builder.api.select(
                y1_zero.into(),
                y0.limbs.get(i).unwrap_or(&zero).target,
                y1.limbs.get(i).unwrap_or(&zero).target,
            ))
        })
        .collect();
    let y_select = BigUintTarget {
        limbs: y_select_limbs,
    };
    let two = builder.api.constant_biguint(&2u32.into());
    let y_select_2 = builder.api.mul_biguint(&y_select, &two);
    let p = builder.api.constant_biguint(&modulus());
    let y_select_2_p = builder.api.div_biguint(&y_select_2, &p);
    for i in 0..y_select_2_p.limbs.len() {
        if i == 0 {
            builder
                .api
                .connect(aflag.target, y_select_2_p.limbs[i].target);
        } else {
            builder.api.connect_u32(y_select_2_p.limbs[i], zero);
        }
    }

    let z1_limbs: Vec<U32Target> = sig[0..SIG_LEN / 2]
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            let zero = builder.api.zero();
            let factor = builder.api.constant(L::Field::from_canonical_u32(256));
            U32Target::from_target_unsafe(
                chunk
                    .iter()
                    .fold(zero, |acc, c| builder.api.mul_add(acc, factor, c.0)),
            )
        })
        .rev()
        .collect();
    let z1 = BigUintTarget { limbs: z1_limbs };

    let z2_limbs: Vec<U32Target> = sig[SIG_LEN / 2..SIG_LEN]
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            let zero = builder.api.zero();
            let factor = builder.api.constant(L::Field::from_canonical_u32(256));
            U32Target::from_target_unsafe(
                chunk
                    .iter()
                    .fold(zero, |acc, c| builder.api.mul_add(acc, factor, c.0)),
            )
        })
        .rev()
        .collect();
    let z2 = BigUintTarget { limbs: z2_limbs };

    builder.api.connect_biguint(&x0, &z2);

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
    let z1_reconstructed = builder.api.add_biguint(x1, &flags);

    builder.api.connect_biguint(&z1, &z1_reconstructed);
}

#[derive(Debug, Default)]
pub struct G2AdditionGenerator {
    a: PointG2Target,
    b: PointG2Target,
    dx: Fp2Target,
    dy: Fp2Target,
    out: PointG2Target,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for G2AdditionGenerator {
    fn id(&self) -> String {
        "G2AdditionGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<plonky2::iop::target::Target> {
        let a_targets = self
            .a
            .iter()
            .flat_map(|f2| {
                f2.iter()
                    .flat_map(|f| f.limbs.iter().map(|l| l.target).collect::<Vec<Target>>())
                    .collect::<Vec<Target>>()
            })
            .collect::<Vec<Target>>();
        let b_targets = self
            .b
            .iter()
            .flat_map(|f2| {
                f2.iter()
                    .flat_map(|f| f.limbs.iter().map(|l| l.target).collect::<Vec<Target>>())
                    .collect::<Vec<Target>>()
            })
            .collect::<Vec<Target>>();
        let dx_targets = self
            .dx
            .iter()
            .flat_map(|f| f.limbs.iter().map(|l| l.target).collect::<Vec<Target>>())
            .collect::<Vec<Target>>();
        let dy_targets = self
            .dy
            .iter()
            .flat_map(|f| f.limbs.iter().map(|l| l.target).collect::<Vec<Target>>())
            .collect::<Vec<Target>>();
        [a_targets, b_targets, dx_targets, dy_targets].concat()
    }

    fn run_once(
        &self,
        witness: &plonky2::iop::witness::PartitionWitness<F>,
        out_buffer: &mut plonky2::iop::generator::GeneratedValues<F>,
    ) {
        let ax = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[0][0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[0][1].clone())),
        ]);
        let ay = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[1][0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[1][1].clone())),
        ]);
        let bx = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.b[0][0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.b[0][1].clone())),
        ]);
        let dx = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.dx[0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.dx[1].clone())),
        ]);
        let dy = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.dy[0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.dy[1].clone())),
        ]);
        let dx_inv = dx.invert();
        let lambda = dy * dx_inv;
        let lambda_sq = lambda * lambda;
        let outx = lambda_sq - ax - bx;
        let outy = lambda * (ax - outx) - ay;
        out_buffer.set_biguint_target(&self.out[0][0], &outx.0[0].to_biguint());
        out_buffer.set_biguint_target(&self.out[0][1], &outx.0[1].to_biguint());
        out_buffer.set_biguint_target(&self.out[1][0], &outy.0[0].to_biguint());
        out_buffer.set_biguint_target(&self.out[1][1], &outy.0[1].to_biguint());
    }

    fn serialize(
        &self,
        dst: &mut Vec<u8>,
        _common_data: &plonky2::plonk::circuit_data::CommonCircuitData<F, D>,
    ) -> plonky2::util::serialization::IoResult<()> {
        self.a[0][0].serialize(dst)?;
        self.a[0][1].serialize(dst)?;
        self.a[1][0].serialize(dst)?;
        self.a[1][1].serialize(dst)?;
        self.b[0][0].serialize(dst)?;
        self.b[0][1].serialize(dst)?;
        self.b[1][0].serialize(dst)?;
        self.b[1][1].serialize(dst)?;
        self.dx[0].serialize(dst)?;
        self.dx[1].serialize(dst)?;
        self.dy[0].serialize(dst)?;
        self.dy[1].serialize(dst)?;
        self.out[0][0].serialize(dst)?;
        self.out[0][1].serialize(dst)?;
        self.out[1][0].serialize(dst)?;
        self.out[1][1].serialize(dst)
    }

    fn deserialize(
        src: &mut plonky2::util::serialization::Buffer,
        _common_data: &plonky2::plonk::circuit_data::CommonCircuitData<F, D>,
    ) -> plonky2::util::serialization::IoResult<Self>
    where
        Self: Sized,
    {
        let ax_c0 = BigUintTarget::deserialize(src)?;
        let ax_c1 = BigUintTarget::deserialize(src)?;
        let ay_c0 = BigUintTarget::deserialize(src)?;
        let ay_c1 = BigUintTarget::deserialize(src)?;
        let bx_c0 = BigUintTarget::deserialize(src)?;
        let bx_c1 = BigUintTarget::deserialize(src)?;
        let by_c0 = BigUintTarget::deserialize(src)?;
        let by_c1 = BigUintTarget::deserialize(src)?;
        let dx_c0 = BigUintTarget::deserialize(src)?;
        let dx_c1 = BigUintTarget::deserialize(src)?;
        let dy_c0 = BigUintTarget::deserialize(src)?;
        let dy_c1 = BigUintTarget::deserialize(src)?;
        let outx_c0 = BigUintTarget::deserialize(src)?;
        let outx_c1 = BigUintTarget::deserialize(src)?;
        let outy_c0 = BigUintTarget::deserialize(src)?;
        let outy_c1 = BigUintTarget::deserialize(src)?;
        Ok(Self {
            a: [[ax_c0, ax_c1], [ay_c0, ay_c1]],
            b: [[bx_c0, bx_c1], [by_c0, by_c1]],
            dx: [dx_c0, dx_c1],
            dy: [dy_c0, dy_c1],
            out: [[outx_c0, outx_c1], [outy_c0, outy_c1]],
        })
    }
}

#[derive(Debug, Default)]
pub struct G2DoublingGenerator {
    a: PointG2Target,
    iso_3_a: Fp2Target,
    out: PointG2Target,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for G2DoublingGenerator {
    fn id(&self) -> String {
        "G2DoublingGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<plonky2::iop::target::Target> {
        let a_targets = self
            .a
            .iter()
            .flat_map(|f2| {
                f2.iter()
                    .flat_map(|f| f.limbs.iter().map(|l| l.target).collect::<Vec<Target>>())
                    .collect::<Vec<Target>>()
            })
            .collect::<Vec<Target>>();
        let iso_3_a_targets = self
            .iso_3_a
            .iter()
            .flat_map(|f| f.limbs.iter().map(|l| l.target).collect::<Vec<Target>>())
            .collect::<Vec<Target>>();
        [a_targets, iso_3_a_targets].concat()
    }

    fn run_once(
        &self,
        witness: &plonky2::iop::witness::PartitionWitness<F>,
        out_buffer: &mut plonky2::iop::generator::GeneratedValues<F>,
    ) {
        let iso_3_a = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.iso_3_a[0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.iso_3_a[1].clone())),
        ]);
        let ax = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[0][0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[0][1].clone())),
        ]);
        let ay = Fp2([
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[1][0].clone())),
            Fp::get_fp_from_biguint(witness.get_biguint_target(self.a[1][1].clone())),
        ]);
        let lambda_num = iso_3_a + ax * ax * Fp::get_fp_from_biguint(3u32.into());
        let lambda_denom = ay + ay;
        let lambda = lambda_num / lambda_denom;
        let lambda_sq = lambda * lambda;
        let outx = lambda_sq - ax - ax;
        let outy = lambda * (ax - outx) - ay;
        out_buffer.set_biguint_target(&self.out[0][0], &outx.0[0].to_biguint());
        out_buffer.set_biguint_target(&self.out[0][1], &outx.0[1].to_biguint());
        out_buffer.set_biguint_target(&self.out[1][0], &outy.0[0].to_biguint());
        out_buffer.set_biguint_target(&self.out[1][1], &outy.0[1].to_biguint());
    }

    fn serialize(
        &self,
        dst: &mut Vec<u8>,
        _common_data: &plonky2::plonk::circuit_data::CommonCircuitData<F, D>,
    ) -> plonky2::util::serialization::IoResult<()> {
        self.a[0][0].serialize(dst)?;
        self.a[0][1].serialize(dst)?;
        self.a[1][0].serialize(dst)?;
        self.a[1][1].serialize(dst)?;
        self.iso_3_a[0].serialize(dst)?;
        self.iso_3_a[1].serialize(dst)?;
        self.out[0][0].serialize(dst)?;
        self.out[0][1].serialize(dst)?;
        self.out[1][0].serialize(dst)?;
        self.out[1][1].serialize(dst)
    }

    fn deserialize(
        src: &mut plonky2::util::serialization::Buffer,
        _common_data: &plonky2::plonk::circuit_data::CommonCircuitData<F, D>,
    ) -> plonky2::util::serialization::IoResult<Self>
    where
        Self: Sized,
    {
        let ax_c0 = BigUintTarget::deserialize(src)?;
        let ax_c1 = BigUintTarget::deserialize(src)?;
        let ay_c0 = BigUintTarget::deserialize(src)?;
        let ay_c1 = BigUintTarget::deserialize(src)?;
        let iso_3_a_c0 = BigUintTarget::deserialize(src)?;
        let iso_3_a_c1 = BigUintTarget::deserialize(src)?;
        let outx_c0 = BigUintTarget::deserialize(src)?;
        let outx_c1 = BigUintTarget::deserialize(src)?;
        let outy_c0 = BigUintTarget::deserialize(src)?;
        let outy_c1 = BigUintTarget::deserialize(src)?;
        Ok(Self {
            a: [[ax_c0, ax_c1], [ay_c0, ay_c1]],
            iso_3_a: [iso_3_a_c0, iso_3_a_c1],
            out: [[outx_c0, outx_c1], [outy_c0, outy_c1]],
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;
    use num_bigint::{BigUint, ToBigUint};
    use plonky2::field::{
        goldilocks_field::GoldilocksField,
        types::{Field, Field64},
    };
    use plonky2x::frontend::{
        builder::DefaultBuilder, uint::num::biguint::CircuitBuilderBiguint, vars::Variable,
    };

    use crate::verification::{
        fields::fp::LIMBS,
        utils::native_bls::{Fp, Fp2},
    };

    use super::{g2_add, g2_add_unequal, g2_double, g2_scalar_mul, signature_point_check, TWO};

    #[test]
    fn test_g2_add_unequal() {
        let mut builder = DefaultBuilder::new();
        let ax = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "337725438187709982817188701931175748950561864071211469604211805451542415352866003578718608366094520056481699232210"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "325784474482020989596135374893471919876505088991873421195308352667079503424389512976821068246925718319548045276021"
            ).unwrap()),
        ]);
        let ay = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2965841325781469856973174148258785715970498867849450741444982165189412687797594966692602501064144340797710797471604"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1396501224612541682947972324170488919567615665343008985787728980681572855276817422483173426760119128141672533354119"
            ).unwrap()),
        ]);
        let bx = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3310291183651938419676930134503606039576251708119934019650494815974674760881379622302324811830103490883079904029190"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "845507222118475144290150023685070019360459684233155402409229752404383900284940551672371362493058110240418657298132"
            ).unwrap()),
        ]);
        let by = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "569469686320544423596306308487126199229297307366529623191489815159190893993668979352767262071942316086625514601662"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2551756239942517806379811015764241238167383065214268002625836091916337464087928632477808357405422759164808763594986"
            ).unwrap()),
        ]);
        let outx = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3768960129599410557225162537737286003238400530051754572454824471200864202913026112975152396185116175737023068710834"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2843653242501816279232983717246998149289638605923450990196321568072224346134709601553669097144892265594669670100681"
            ).unwrap()),
        ]);
        let outy = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2136473314670056131183153764113091685196675640973971063848296586048702180604877062503412214120535118046733529576506"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3717743359948639609414970569174500186381762539811697438986507840606082550875593852503699874848297189142874182531754"
            ).unwrap()),
        ]);

        let a = [
            [
                builder.api.constant_biguint(&ax.to_biguint()[0]),
                builder.api.constant_biguint(&ax.to_biguint()[1]),
            ],
            [
                builder.api.constant_biguint(&ay.to_biguint()[0]),
                builder.api.constant_biguint(&ay.to_biguint()[1]),
            ],
        ];

        let b = [
            [
                builder.api.constant_biguint(&bx.to_biguint()[0]),
                builder.api.constant_biguint(&bx.to_biguint()[1]),
            ],
            [
                builder.api.constant_biguint(&by.to_biguint()[0]),
                builder.api.constant_biguint(&by.to_biguint()[1]),
            ],
        ];

        let out = g2_add_unequal(&mut builder, &a, &b);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..TWO {
            for j in 0..out[i].len() {
                for k in 0..LIMBS {
                    builder.write(Variable(out[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);

        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..TWO {
            for _ in 0..out[i].len() {
                for _ in 0..LIMBS {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..2 * TWO {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| (f.0 % GoldilocksField::ORDER) as u32)
                    .collect_vec(),
            ));
        }

        let outx = outx.to_biguint();
        let outy = outy.to_biguint();
        for i in 0..TWO {
            assert_eq!(biguint_res[i], outx[i]);
        }
        for i in 0..TWO {
            assert_eq!(biguint_res[i + TWO], outy[i]);
        }
    }

    #[test]
    fn test_g2_double() {
        let mut builder = DefaultBuilder::new();
        let ax = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "337725438187709982817188701931175748950561864071211469604211805451542415352866003578718608366094520056481699232210"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "325784474482020989596135374893471919876505088991873421195308352667079503424389512976821068246925718319548045276021"
            ).unwrap()),
        ]);
        let ay = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2965841325781469856973174148258785715970498867849450741444982165189412687797594966692602501064144340797710797471604"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1396501224612541682947972324170488919567615665343008985787728980681572855276817422483173426760119128141672533354119"
            ).unwrap()),
        ]);
        let outx = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1706600946883407123219281831938721281378271382276249190372502550662898700659312875480967274178992951148217552181426"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3667242666443602243234297601464303917352028754060836539777371958000208843208072408275476423902876206704592938302165"
            ).unwrap()),
        ]);
        let outy = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1455123735227984325271077817690334450857761312547658768990224051882209081684526238004573782051265522918945273385158"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3320466234608127782197732106422214686550406898681784249598895322673540642018347203281877363138090179901504571209003"
            ).unwrap()),
        ]);

        let a = [
            [
                builder.api.constant_biguint(&ax.to_biguint()[0]),
                builder.api.constant_biguint(&ax.to_biguint()[1]),
            ],
            [
                builder.api.constant_biguint(&ay.to_biguint()[0]),
                builder.api.constant_biguint(&ay.to_biguint()[1]),
            ],
        ];

        let iso_3_a = [
            builder.api.constant_biguint(&0.to_biguint().unwrap()),
            builder.api.constant_biguint(&240.to_biguint().unwrap()),
        ];
        let iso_3_b = [
            builder.api.constant_biguint(&1012.to_biguint().unwrap()),
            builder.api.constant_biguint(&1012.to_biguint().unwrap()),
        ];

        let out = g2_double(&mut builder, &a, &iso_3_a, &iso_3_b);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..TWO {
            for j in 0..out[i].len() {
                for k in 0..LIMBS {
                    builder.write(Variable(out[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);

        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..TWO {
            for _ in 0..out[i].len() {
                for _ in 0..LIMBS {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..2 * TWO {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| (f.0 % GoldilocksField::ORDER) as u32)
                    .collect_vec(),
            ));
        }

        let outx = outx.to_biguint();
        let outy = outy.to_biguint();
        for i in 0..TWO {
            assert_eq!(biguint_res[i], outx[i]);
        }
        for i in 0..TWO {
            assert_eq!(biguint_res[i + TWO], outy[i]);
        }
    }

    #[test]
    fn test_g2_add() {
        let mut builder = DefaultBuilder::new();
        let ax = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "337725438187709982817188701931175748950561864071211469604211805451542415352866003578718608366094520056481699232210"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "325784474482020989596135374893471919876505088991873421195308352667079503424389512976821068246925718319548045276021"
            ).unwrap()),
        ]);
        let ay = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2965841325781469856973174148258785715970498867849450741444982165189412687797594966692602501064144340797710797471604"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1396501224612541682947972324170488919567615665343008985787728980681572855276817422483173426760119128141672533354119"
            ).unwrap()),
        ]);
        let bx = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3310291183651938419676930134503606039576251708119934019650494815974674760881379622302324811830103490883079904029190"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "845507222118475144290150023685070019360459684233155402409229752404383900284940551672371362493058110240418657298132"
            ).unwrap()),
        ]);
        let by = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "569469686320544423596306308487126199229297307366529623191489815159190893993668979352767262071942316086625514601662"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2551756239942517806379811015764241238167383065214268002625836091916337464087928632477808357405422759164808763594986"
            ).unwrap()),
        ]);
        let outx = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3768960129599410557225162537737286003238400530051754572454824471200864202913026112975152396185116175737023068710834"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2843653242501816279232983717246998149289638605923450990196321568072224346134709601553669097144892265594669670100681"
            ).unwrap()),
        ]);
        let outy = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2136473314670056131183153764113091685196675640973971063848296586048702180604877062503412214120535118046733529576506"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3717743359948639609414970569174500186381762539811697438986507840606082550875593852503699874848297189142874182531754"
            ).unwrap()),
        ]);

        let iso_3_a = [
            builder.api.constant_biguint(&0.to_biguint().unwrap()),
            builder.api.constant_biguint(&240.to_biguint().unwrap()),
        ];
        let iso_3_b = [
            builder.api.constant_biguint(&1012.to_biguint().unwrap()),
            builder.api.constant_biguint(&1012.to_biguint().unwrap()),
        ];

        let a = [
            [
                builder.api.constant_biguint(&ax.to_biguint()[0]),
                builder.api.constant_biguint(&ax.to_biguint()[1]),
            ],
            [
                builder.api.constant_biguint(&ay.to_biguint()[0]),
                builder.api.constant_biguint(&ay.to_biguint()[1]),
            ],
        ];

        let b = [
            [
                builder.api.constant_biguint(&bx.to_biguint()[0]),
                builder.api.constant_biguint(&bx.to_biguint()[1]),
            ],
            [
                builder.api.constant_biguint(&by.to_biguint()[0]),
                builder.api.constant_biguint(&by.to_biguint()[1]),
            ],
        ];

        let fals = builder._false();

        let out = g2_add(&mut builder, &a, fals, &b, fals, &iso_3_a, &iso_3_b);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..TWO {
            for j in 0..out[i].len() {
                for k in 0..LIMBS {
                    builder.write(Variable(out[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);

        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..TWO {
            for _ in 0..out[i].len() {
                for _ in 0..LIMBS {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..2 * TWO {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| (f.0 % GoldilocksField::ORDER) as u32)
                    .collect_vec(),
            ));
        }

        let outx = outx.to_biguint();
        let outy = outy.to_biguint();
        for i in 0..TWO {
            assert_eq!(biguint_res[i], outx[i]);
        }
        for i in 0..TWO {
            assert_eq!(biguint_res[i + TWO], outy[i]);
        }
    }

    #[test]
    fn test_g2_scalar_mul() {
        let mut builder = DefaultBuilder::new();
        let ax = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3219922746671482828210036408711997441423671614254909325234707044434520756052360285257107968950769890523504628275940"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1689252599334450651431125834598273362703914442067213087777626885820814565104897473205802289043260096634945919754747"
            ).unwrap()),
        ]);
        let ay = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3277365552217223927730141275188890184833071787772555827000840921808443941258778716588573376888715070179970391655322"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "583921403203359937897773959554466412643567032578544897698779952656397892876222999644067619700087458377600564507453"
            ).unwrap()),
        ]);
        let outx = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2523579754967640238723918616351685721284996518144674649571478689837790667637298382703328020485789979179436650708908"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "926383654583210622704996942518380628779065643276946198453367351460754762515870939199945068184689019420502882527581"
            ).unwrap()),
        ]);
        let outy = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3787164088273368384415735450659985644624425652571718026503769291441565414050570276349393167238939810080925158072505"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "3689766260810296892747853615583585529622598940500344733471060314731353498148974741263844587195375375425544954703339"
            ).unwrap()),
        ]);

        let a = [
            [
                builder.api.constant_biguint(&ax.to_biguint()[0]),
                builder.api.constant_biguint(&ax.to_biguint()[1]),
            ],
            [
                builder.api.constant_biguint(&ay.to_biguint()[0]),
                builder.api.constant_biguint(&ay.to_biguint()[1]),
            ],
        ];

        let iso_3_b = [
            builder.api.constant_biguint(&4.to_biguint().unwrap()),
            builder.api.constant_biguint(&4.to_biguint().unwrap()),
        ];

        let out = g2_scalar_mul(&mut builder, &a, &iso_3_b);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..TWO {
            for j in 0..out[i].len() {
                for k in 0..LIMBS {
                    builder.write(Variable(out[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);

        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..TWO {
            for _ in 0..out[i].len() {
                for _ in 0..LIMBS {
                    res_output.push(output.read::<Variable>())
                }
            }
        }
        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..2 * TWO {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| (f.0 % GoldilocksField::ORDER) as u32)
                    .collect_vec(),
            ));
        }

        let outx = outx.to_biguint();
        let outy = outy.to_biguint();
        for i in 0..TWO {
            assert_eq!(biguint_res[i], outx[i]);
        }
        for i in 0..TWO {
            assert_eq!(biguint_res[i + TWO], outy[i]);
        }
    }

    #[test]
    fn test_signature_point_check() {
        let mut builder = DefaultBuilder::new();
        let x = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "2132190448044539512343458281906429348357553485972550361022637600291474790426714276782518732598485406127127542511958"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1768967113711705180967647921989767607043027235135825860038026636952386389242730816293578938377273126163720266364901"
            ).unwrap()),
        ]);
        let y = Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str(
                "1601269830186296343258204708609068858787525822280553591425324687245481424080606221266002538737401918289754033770338"
            ).unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str(
                "508402758079580872259652181430201489694066144504950057753724961054091567713555160539784585997814439522141428760875"
            ).unwrap()),
        ]);
        let x_t = [
            builder.api.constant_biguint(&x.to_biguint()[0]),
            builder.api.constant_biguint(&x.to_biguint()[1]),
        ];
        let y_t = [
            builder.api.constant_biguint(&y.to_biguint()[0]),
            builder.api.constant_biguint(&y.to_biguint()[1]),
        ];

        let sig: Vec<Variable> = [
            139, 126, 67, 23, 196, 226, 59, 211, 144, 232, 136, 101, 183, 50, 126, 215, 210, 110,
            97, 248, 215, 138, 135, 11, 184, 144, 5, 162, 250, 243, 244, 51, 140, 27, 110, 7, 158,
            63, 35, 135, 61, 90, 233, 5, 135, 72, 183, 229, 13, 218, 102, 33, 65, 70, 85, 67, 129,
            210, 109, 61, 39, 103, 248, 6, 238, 111, 155, 116, 213, 81, 130, 121, 92, 156, 15, 149,
            69, 65, 43, 98, 117, 125, 244, 59, 143, 22, 72, 75, 38, 67, 175, 183, 249, 6, 57, 86,
        ]
        .iter()
        .map(|f| builder.constant(GoldilocksField::from_canonical_u8(*f)))
        .collect();

        let sig: [Variable; 96] = sig
            .into_iter()
            .collect::<Vec<Variable>>()
            .try_into()
            .unwrap();

        let point = [x_t, y_t];

        signature_point_check(&mut builder, &point, &sig);

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, output) = circuit.prove(&input);

        // Verify proof.
        circuit.verify(&proof, &input, &output);
    }
}
