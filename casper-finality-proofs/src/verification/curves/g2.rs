use num_bigint::{BigUint, ToBigUint};
use plonky2::field::types::Field;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        generator::SimpleGenerator,
        target::{BoolTarget, Target},
    },
};
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

pub const SIG_LEN: usize = 96;

use crate::verification::{
    fields::{
        fp2::{
            add_fp2, is_equal, is_zero, mul_fp2, negate_fp2, range_check_fp2, sub_fp2, Fp2Target,
        },
        fp::{fp_is_zero, N},
    },
    utils::native_bls::{get_bls_12_381_parameter, modulus, Fp, Fp2},
};

pub type PointG2Target = [Fp2Target; 2];

pub fn g2_add_unequal<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &PointG2Target,
    b: &PointG2Target,
) -> PointG2Target {
    let dy = sub_fp2(builder, &b[1], &a[1]);
    let dx = sub_fp2(builder, &b[0], &a[0]);
    let outx_c0 = builder.api.add_virtual_biguint_target_unsafe(N);
    let outx_c1 = builder.api.add_virtual_biguint_target_unsafe(N);
    let outy_c0 = builder.api.add_virtual_biguint_target_unsafe(N);
    let outy_c1 = builder.api.add_virtual_biguint_target_unsafe(N);
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
    let outx_c0 = builder.api.add_virtual_biguint_target_unsafe(N);
    let outx_c1 = builder.api.add_virtual_biguint_target_unsafe(N);
    let outy_c0 = builder.api.add_virtual_biguint_target_unsafe(N);
    let outy_c1 = builder.api.add_virtual_biguint_target_unsafe(N);
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
    is_infinity_a: BoolTarget,
    b: &PointG2Target,
    is_infinity_b: BoolTarget,
    iso_3_a: &Fp2Target,
    iso_3_b: &Fp2Target,
) -> PointG2Target {
    let x_equal = is_equal(builder, &a[0], &b[0]);
    let y_equal = is_equal(builder, &a[1], &b[1]);
    let do_double = builder.and(x_equal, y_equal);
    let add_input_b = [
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
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
    let both_inf = builder.api.and(is_infinity_a, is_infinity_b);
    let a_not_inf = builder.api.not(is_infinity_a);
    let b_not_inf = builder.api.not(is_infinity_b);
    let both_not_inf = builder.api.and(a_not_inf, b_not_inf);
    let not_y_equal = builder.not(y_equal);
    let a_neg_b = builder.and(x_equal, not_y_equal);
    let inverse = builder.api.and(both_not_inf, a_neg_b.into());
    let out_inf = builder.api.or(both_inf, inverse);
    builder.api.assert_zero(out_inf.target);
    let add_or_double_select = [
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
    ];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..N {
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
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
    ];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..N {
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
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
    ];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..N {
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
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
        ],
        [
            builder.api.add_virtual_biguint_target_unsafe(N),
            builder.api.add_virtual_biguint_target_unsafe(N),
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
    sig: &[Target; SIG_LEN],
) {
    let msbs = builder.api.split_le(sig[0], 8);
    let bflag = msbs[6];
    builder.api.assert_zero(bflag.target);

    let aflag = msbs[5];

    let (x0, x1, y0, y1) = (&point[0][0], &point[0][1], &point[1][0], &point[1][1]);
    let y1_zero = fp_is_zero(builder, &y1);
    let zero = builder.api.zero_u32();
    let y_select_limbs: Vec<U32Target> = (0..N)
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
                    .fold(zero, |acc, c| builder.api.mul_add(acc, factor, *c)),
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
                    .fold(zero, |acc, c| builder.api.mul_add(acc, factor, *c)),
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
        limbs: (0..N)
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
