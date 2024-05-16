use itertools::Itertools;
use plonky2::{
    field::{
        extension::{Extendable, FieldExtension},
        packed::PackedField,
    },
    hash::hash_types::RichField,
    iop::ext_target::ExtensionTarget,
};
use starky::{
    constraint_consumer::ConstraintConsumer,
    evaluation_frame::{StarkEvaluationFrame, StarkFrame},
    stark::Stark,
};

use crate::verification::{
    curves::starky::g1::{
        add_g1_addition_constraints, add_g1_addition_constraints_ext_circuit,
        fill_trace_g1_addition, G1_POINT_ADDITION_X1, G1_POINT_ADDITION_X2, G1_POINT_ADDITION_X3,
        G1_POINT_ADDITION_Y1, G1_POINT_ADDITION_Y2, G1_POINT_ADDITION_Y3, TOT_COL,
    },
    utils::native_bls::Fp,
};

pub const NUM_POINTS: usize = 1;

pub const ROW_NUM: usize = 0;
pub const PIS_IDX: usize = ROW_NUM + 12;
pub const A_IS_INF: usize = PIS_IDX + NUM_POINTS;
pub const B_IS_INF: usize = A_IS_INF + 1;
pub const OP: usize = B_IS_INF + 1;
pub const TOTAL_COLUMNS: usize = OP + TOT_COL;
pub const COLUMNS: usize = TOTAL_COLUMNS;

pub const POINTS: usize = 0;
pub const BITS: usize = POINTS + 24 * NUM_POINTS;
pub const RES: usize = BITS + NUM_POINTS;
pub const PUBLIC_INPUTS: usize = RES + 24;

#[derive(Clone, Copy)]
pub struct ECCAggStark<F: RichField + Extendable<D>, const D: usize> {
    num_rows: usize,
    _f: std::marker::PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> ECCAggStark<F, D> {
    pub fn new(num_rows: usize) -> Self {
        Self {
            num_rows,
            _f: std::marker::PhantomData,
        }
    }

    /// Ensure that both of the initial points, i.e. points\[0\] and points\[1\], at least one of them is not a point at infinty.
    pub fn generate_trace(&self, points: &[[Fp; 2]], bits: &[bool]) -> Vec<[F; TOTAL_COLUMNS]> {
        assert_eq!(NUM_POINTS, points.len());
        assert_eq!(points.len(), bits.len());
        let num_additions = points.len() - 1;
        let num_rows_req = num_additions * 12;
        assert!(
            num_rows_req < self.num_rows,
            "stark doesn't have enough rows"
        );

        let mut trace = vec![[F::ZERO; TOTAL_COLUMNS]; self.num_rows];
        (0..self.num_rows).chunks(12).into_iter().for_each(|c| {
            c.into_iter().for_each(|i| {
                trace[i][ROW_NUM + i % 12] = F::ONE;
            });
        });
        let mut row = 0;
        for i in 0..NUM_POINTS {
            if i < 2 {
                println!("enters");
                let dali_shte_se_printira = (row..row + 12).into_iter().for_each(|rw| {
                    trace[rw][PIS_IDX + i] = F::ONE;
                    println!("trace[rw][PIS_IDX + i] is: {:?}", trace[rw][PIS_IDX + i]);
                    println!("trace[rw][PIS_IDX + i] is: {:?}", rw);
                });
                println!("dali_shte_se_printira is: {:?}", dali_shte_se_printira);
            } else {
                row += 12;
                (row..row + 12)
                    .into_iter()
                    .for_each(|rw| trace[rw][PIS_IDX + i] = F::ONE);
            }
        }
        row = 0;
        let mut res = fill_trace_g1_addition(&mut trace, &points[0], &points[1], row, OP);
        for r in row..row + 12 {
            trace[r][A_IS_INF] = F::from_bool(!bits[0]);
            trace[r][B_IS_INF] = F::from_bool(!bits[1]);
        }
        if !bits[0] {
            res = points[1];
        } else if !bits[1] {
            res = points[0];
        }
        for i in 2..NUM_POINTS {
            row += 12;
            let res_tmp = fill_trace_g1_addition(&mut trace, &res, &points[i], row, OP);
            for r in row..row + 12 {
                trace[r][A_IS_INF] = F::from_bool(false);
                trace[r][B_IS_INF] = F::from_bool(!bits[i]);
            }
            if bits[i] {
                res = res_tmp;
            }
        }
        trace
    }
}

// Implement constraint generator
impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for ECCAggStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, COLUMNS, PUBLIC_INPUTS>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        vars: &Self::EvaluationFrame<FE, P, D2>,
        yield_constr: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local_values = vars.get_local_values();
        let next_values = vars.get_next_values();
        let public_inputs = vars.get_public_inputs();

        for i in 0..12 {
            if i == 0 {
                yield_constr.constraint_first_row(local_values[ROW_NUM] - FE::ONE);
            } else {
                yield_constr.constraint_first_row(local_values[ROW_NUM + i])
            }
        }
        for i in 0..12 {
            if i < 11 {
                yield_constr.constraint_transition(
                    local_values[ROW_NUM + i] - next_values[ROW_NUM + i + 1],
                );
            } else {
                yield_constr
                    .constraint_transition(local_values[ROW_NUM + i] - next_values[ROW_NUM]);
            }
        }

        for i in 0..NUM_POINTS {
            if i < 2 {
                yield_constr.constraint_first_row(local_values[PIS_IDX + i] - FE::ONE);
            } else {
                yield_constr.constraint_first_row(local_values[PIS_IDX + i]);
            }
        }

        for i in 1..NUM_POINTS - 1 {
            yield_constr.constraint_transition(
                (P::ONES - local_values[PIS_IDX + NUM_POINTS - 1])
                    * next_values[ROW_NUM]
                    * (local_values[PIS_IDX + i] - next_values[PIS_IDX + i + 1]),
            );
        }

        for i in 0..NUM_POINTS {
            yield_constr.constraint_transition(
                local_values[PIS_IDX + NUM_POINTS - 1]
                    * next_values[ROW_NUM]
                    * next_values[PIS_IDX + i],
            );
        }

        for i in 0..12 {
            yield_constr.constraint_first_row(
                local_values[OP + G1_POINT_ADDITION_X1 + i] - public_inputs[POINTS + i],
            );
            yield_constr.constraint_first_row(
                local_values[OP + G1_POINT_ADDITION_Y1 + i] - public_inputs[POINTS + i + 12],
            );
            yield_constr.constraint_first_row(
                local_values[OP + G1_POINT_ADDITION_X2 + i] - public_inputs[POINTS + 24 + i],
            );
            yield_constr.constraint_first_row(
                local_values[OP + G1_POINT_ADDITION_Y2 + i] - public_inputs[POINTS + 24 + i + 12],
            );
        }

        yield_constr.constraint_first_row(P::ONES - local_values[A_IS_INF] - public_inputs[BITS]);
        yield_constr
            .constraint_first_row(P::ONES - local_values[B_IS_INF] - public_inputs[BITS + 1]);

        for idx in 2..NUM_POINTS {
            for i in 0..12 {
                yield_constr.constraint_transition(
                    next_values[ROW_NUM]
                        * next_values[PIS_IDX + idx]
                        * (next_values[OP + G1_POINT_ADDITION_X2 + i]
                            - public_inputs[POINTS + 24 * idx + i]),
                );
                yield_constr.constraint_transition(
                    next_values[ROW_NUM]
                        * next_values[PIS_IDX + idx]
                        * (next_values[OP + G1_POINT_ADDITION_Y2 + i]
                            - public_inputs[POINTS + 24 * idx + i + 12]),
                );
            }
            yield_constr.constraint_transition(
                next_values[ROW_NUM]
                    * next_values[PIS_IDX + idx]
                    * (P::ONES - next_values[B_IS_INF] - public_inputs[BITS + idx]),
            );
        }

        for i in 0..12 {
            yield_constr.constraint_transition(
                (P::ONES - next_values[ROW_NUM])
                    * (local_values[OP + G1_POINT_ADDITION_X1 + i]
                        - next_values[OP + G1_POINT_ADDITION_X1 + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[ROW_NUM])
                    * (local_values[OP + G1_POINT_ADDITION_Y1 + i]
                        - next_values[OP + G1_POINT_ADDITION_Y1 + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[ROW_NUM])
                    * (local_values[OP + G1_POINT_ADDITION_X2 + i]
                        - next_values[OP + G1_POINT_ADDITION_X2 + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[ROW_NUM])
                    * (local_values[OP + G1_POINT_ADDITION_Y2 + i]
                        - next_values[OP + G1_POINT_ADDITION_Y2 + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[ROW_NUM])
                    * (local_values[OP + G1_POINT_ADDITION_X3 + i]
                        - next_values[OP + G1_POINT_ADDITION_X3 + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[ROW_NUM])
                    * (local_values[OP + G1_POINT_ADDITION_Y3 + i]
                        - next_values[OP + G1_POINT_ADDITION_Y3 + i]),
            );
        }

        yield_constr.constraint(local_values[A_IS_INF] * (P::ONES - local_values[A_IS_INF]));
        yield_constr.constraint(local_values[B_IS_INF] * (P::ONES - local_values[B_IS_INF]));
        yield_constr.constraint(local_values[A_IS_INF] * local_values[B_IS_INF]);

        yield_constr.constraint_transition(
            (P::ONES - next_values[ROW_NUM]) * (local_values[A_IS_INF] - next_values[A_IS_INF]),
        );
        yield_constr.constraint_transition(
            (P::ONES - next_values[ROW_NUM]) * (local_values[B_IS_INF] - next_values[B_IS_INF]),
        );

        for i in 0..12 {
            yield_constr.constraint_transition(
                next_values[ROW_NUM]
                    * (P::ONES - local_values[PIS_IDX + NUM_POINTS - 1])
                    * (local_values[A_IS_INF] * local_values[OP + G1_POINT_ADDITION_X2 + i]
                        + local_values[B_IS_INF] * local_values[OP + G1_POINT_ADDITION_X1 + i]
                        + (P::ONES - local_values[A_IS_INF] - local_values[B_IS_INF])
                            * local_values[OP + G1_POINT_ADDITION_X3 + i]
                        - next_values[OP + G1_POINT_ADDITION_X1 + i]),
            );
            yield_constr.constraint_transition(
                next_values[ROW_NUM]
                    * (P::ONES - local_values[PIS_IDX + NUM_POINTS - 1])
                    * (local_values[A_IS_INF] * local_values[OP + G1_POINT_ADDITION_Y2 + i]
                        + local_values[B_IS_INF] * local_values[OP + G1_POINT_ADDITION_Y1 + i]
                        + (P::ONES - local_values[A_IS_INF] - local_values[B_IS_INF])
                            * local_values[OP + G1_POINT_ADDITION_Y3 + i]
                        - next_values[OP + G1_POINT_ADDITION_Y1 + i]),
            );
        }

        add_g1_addition_constraints(local_values, next_values, yield_constr, OP, None);

        for i in 0..12 {
            yield_constr.constraint_transition(
                next_values[ROW_NUM]
                    * local_values[PIS_IDX + NUM_POINTS - 1]
                    * (local_values[A_IS_INF] * local_values[OP + G1_POINT_ADDITION_X2 + i]
                        + local_values[B_IS_INF] * local_values[OP + G1_POINT_ADDITION_X1 + i]
                        + (P::ONES - local_values[A_IS_INF] - local_values[B_IS_INF])
                            * local_values[OP + G1_POINT_ADDITION_X3 + i]
                        - public_inputs[RES + i]),
            );
            yield_constr.constraint_transition(
                next_values[ROW_NUM]
                    * local_values[PIS_IDX + NUM_POINTS - 1]
                    * (local_values[A_IS_INF] * local_values[OP + G1_POINT_ADDITION_Y2 + i]
                        + local_values[B_IS_INF] * local_values[OP + G1_POINT_ADDITION_Y1 + i]
                        + (P::ONES - local_values[A_IS_INF] - local_values[B_IS_INF])
                            * local_values[OP + G1_POINT_ADDITION_Y3 + i]
                        - public_inputs[RES + i + 12]),
            );
        }
    }

    type EvaluationFrameTarget =
        StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, COLUMNS, PUBLIC_INPUTS>;

    fn eval_ext_circuit(
        &self,
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
        vars: &Self::EvaluationFrameTarget,
        yield_constr: &mut starky::constraint_consumer::RecursiveConstraintConsumer<F, D>,
    ) {
        let local_values = vars.get_local_values();
        let next_values = vars.get_next_values();
        let public_inputs = vars.get_public_inputs();

        for i in 0..12 {
            if i == 0 {
                let one = builder.one_extension();
                let c = builder.sub_extension(local_values[ROW_NUM], one);
                yield_constr.constraint_first_row(builder, c);
            } else {
                yield_constr.constraint_first_row(builder, local_values[ROW_NUM + i]);
            }
        }
        for i in 0..12 {
            if i < 11 {
                let c =
                    builder.sub_extension(local_values[ROW_NUM + i], next_values[ROW_NUM + i + 1]);
                yield_constr.constraint_transition(builder, c);
            } else {
                let c = builder.sub_extension(local_values[ROW_NUM + i], next_values[ROW_NUM]);
                yield_constr.constraint_transition(builder, c);
            }
        }

        for i in 0..NUM_POINTS {
            if i < 2 {
                let one = builder.one_extension();
                let c = builder.sub_extension(local_values[PIS_IDX + i], one);
                yield_constr.constraint_first_row(builder, c);
            } else {
                yield_constr.constraint_first_row(builder, local_values[PIS_IDX + i]);
            }
        }

        for i in 1..NUM_POINTS - 1 {
            let one = builder.one_extension();
            let sub1 = builder.sub_extension(one, local_values[PIS_IDX + NUM_POINTS - 1]);
            let mul = builder.mul_extension(sub1, next_values[ROW_NUM]);
            let sub2 =
                builder.sub_extension(local_values[PIS_IDX + i], next_values[PIS_IDX + i + 1]);
            let c = builder.mul_extension(mul, sub2);
            yield_constr.constraint_transition(builder, c);
        }

        for i in 0..NUM_POINTS {
            let mul =
                builder.mul_extension(local_values[PIS_IDX + NUM_POINTS - 1], next_values[ROW_NUM]);
            let c = builder.mul_extension(mul, next_values[PIS_IDX + i]);
            yield_constr.constraint_transition(builder, c);
        }

        for i in 0..12 {
            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_X1 + i],
                public_inputs[POINTS + i],
            );
            yield_constr.constraint_first_row(builder, c);
            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_Y1 + i],
                public_inputs[POINTS + i + 12],
            );
            yield_constr.constraint_first_row(builder, c);
            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_X2 + i],
                public_inputs[POINTS + 24 + i],
            );
            yield_constr.constraint_first_row(builder, c);
            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_Y2 + i],
                public_inputs[POINTS + 24 + i + 12],
            );
            yield_constr.constraint_first_row(builder, c);
        }

        let one = builder.one_extension();
        let c = builder.sub_extension(one, local_values[A_IS_INF]);
        let c = builder.sub_extension(c, public_inputs[BITS]);
        yield_constr.constraint_first_row(builder, c);
        let c = builder.sub_extension(one, local_values[B_IS_INF]);
        let c = builder.sub_extension(c, public_inputs[BITS + 1]);
        yield_constr.constraint_first_row(builder, c);

        for idx in 2..NUM_POINTS {
            for i in 0..12 {
                let mul = builder.mul_extension(next_values[ROW_NUM], next_values[PIS_IDX + idx]);
                let c = builder.sub_extension(
                    next_values[OP + G1_POINT_ADDITION_X2 + i],
                    public_inputs[POINTS + 24 * idx + i],
                );
                let c = builder.mul_extension(mul, c);
                yield_constr.constraint_transition(builder, c);

                let c = builder.sub_extension(
                    next_values[OP + G1_POINT_ADDITION_Y2 + i],
                    public_inputs[POINTS + 24 * idx + i + 12],
                );
                let c = builder.mul_extension(mul, c);
                yield_constr.constraint_transition(builder, c);
            }
            let one = builder.one_extension();
            let mul = builder.mul_extension(next_values[ROW_NUM], next_values[PIS_IDX + idx]);
            let c = builder.sub_extension(one, next_values[B_IS_INF]);
            let c = builder.sub_extension(c, public_inputs[BITS + idx]);
            let c = builder.mul_extension(mul, c);
            yield_constr.constraint_transition(builder, c);
        }

        for i in 0..12 {
            let one = builder.one_extension();
            let sub1 = builder.sub_extension(one, next_values[ROW_NUM]);

            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_X1 + i],
                next_values[OP + G1_POINT_ADDITION_X1 + i],
            );
            let c = builder.mul_extension(sub1, c);
            yield_constr.constraint_transition(builder, c);

            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_Y1 + i],
                next_values[OP + G1_POINT_ADDITION_Y1 + i],
            );
            let c = builder.mul_extension(sub1, c);
            yield_constr.constraint_transition(builder, c);

            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_X2 + i],
                next_values[OP + G1_POINT_ADDITION_X2 + i],
            );
            let c = builder.mul_extension(sub1, c);
            yield_constr.constraint_transition(builder, c);

            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_Y2 + i],
                next_values[OP + G1_POINT_ADDITION_Y2 + i],
            );
            let c = builder.mul_extension(sub1, c);
            yield_constr.constraint_transition(builder, c);

            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_X3 + i],
                next_values[OP + G1_POINT_ADDITION_X3 + i],
            );
            let c = builder.mul_extension(sub1, c);
            yield_constr.constraint_transition(builder, c);

            let c = builder.sub_extension(
                local_values[OP + G1_POINT_ADDITION_Y3 + i],
                next_values[OP + G1_POINT_ADDITION_Y3 + i],
            );
            let c = builder.mul_extension(sub1, c);
            yield_constr.constraint_transition(builder, c);
        }

        let one = builder.one_extension();
        let c = builder.sub_extension(one, local_values[A_IS_INF]);
        let c = builder.mul_extension(local_values[A_IS_INF], c);
        yield_constr.constraint(builder, c);
        let c = builder.sub_extension(one, local_values[B_IS_INF]);
        let c = builder.mul_extension(local_values[B_IS_INF], c);
        yield_constr.constraint(builder, c);
        let c = builder.mul_extension(local_values[A_IS_INF], local_values[B_IS_INF]);
        yield_constr.constraint(builder, c);

        let sub1 = builder.sub_extension(one, next_values[ROW_NUM]);
        let c = builder.sub_extension(local_values[A_IS_INF], next_values[A_IS_INF]);
        let c = builder.mul_extension(sub1, c);
        yield_constr.constraint_transition(builder, c);

        let c = builder.sub_extension(local_values[B_IS_INF], next_values[B_IS_INF]);
        let c = builder.mul_extension(sub1, c);
        yield_constr.constraint_transition(builder, c);

        for i in 0..12 {
            let one = builder.one_extension();
            let sub1 = builder.sub_extension(one, local_values[PIS_IDX + NUM_POINTS - 1]);
            let mul = builder.mul_extension(sub1, next_values[ROW_NUM]);

            let mul1 = builder.mul_extension(
                local_values[A_IS_INF],
                local_values[OP + G1_POINT_ADDITION_X2 + i],
            );
            let mul2 = builder.mul_extension(
                local_values[B_IS_INF],
                local_values[OP + G1_POINT_ADDITION_X1 + i],
            );
            let a_b_inf_or = builder.add_extension(local_values[A_IS_INF], local_values[B_IS_INF]);
            let res_selector = builder.sub_extension(one, a_b_inf_or);
            let mul3 =
                builder.mul_extension(res_selector, local_values[OP + G1_POINT_ADDITION_X3 + i]);
            let sel = builder.add_many_extension([mul1, mul2, mul3].iter());
            let sub2 = builder.sub_extension(sel, next_values[OP + G1_POINT_ADDITION_X1 + i]);
            let c = builder.mul_extension(mul, sub2);
            yield_constr.constraint_transition(builder, c);

            let mul1 = builder.mul_extension(
                local_values[A_IS_INF],
                local_values[OP + G1_POINT_ADDITION_Y2 + i],
            );
            let mul2 = builder.mul_extension(
                local_values[B_IS_INF],
                local_values[OP + G1_POINT_ADDITION_Y1 + i],
            );
            let a_b_inf_or = builder.add_extension(local_values[A_IS_INF], local_values[B_IS_INF]);
            let res_selector = builder.sub_extension(one, a_b_inf_or);
            let mul3 =
                builder.mul_extension(res_selector, local_values[OP + G1_POINT_ADDITION_Y3 + i]);
            let sel = builder.add_many_extension([mul1, mul2, mul3].iter());
            let sub2 = builder.sub_extension(sel, next_values[OP + G1_POINT_ADDITION_Y1 + i]);
            let c = builder.mul_extension(mul, sub2);
            yield_constr.constraint_transition(builder, c);
        }

        add_g1_addition_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            OP,
            None,
        );

        for i in 0..12 {
            let mul =
                builder.mul_extension(local_values[PIS_IDX + NUM_POINTS - 1], next_values[ROW_NUM]);

            let mul1 = builder.mul_extension(
                local_values[A_IS_INF],
                local_values[OP + G1_POINT_ADDITION_X2 + i],
            );
            let mul2 = builder.mul_extension(
                local_values[B_IS_INF],
                local_values[OP + G1_POINT_ADDITION_X1 + i],
            );
            let a_b_inf_or = builder.add_extension(local_values[A_IS_INF], local_values[B_IS_INF]);
            let res_selector = builder.sub_extension(one, a_b_inf_or);
            let mul3 =
                builder.mul_extension(res_selector, local_values[OP + G1_POINT_ADDITION_X3 + i]);
            let sel = builder.add_many_extension([mul1, mul2, mul3].iter());
            let sub2 = builder.sub_extension(sel, public_inputs[RES + i]);
            let c = builder.mul_extension(mul, sub2);
            yield_constr.constraint_transition(builder, c);

            let mul1 = builder.mul_extension(
                local_values[A_IS_INF],
                local_values[OP + G1_POINT_ADDITION_Y2 + i],
            );
            let mul2 = builder.mul_extension(
                local_values[B_IS_INF],
                local_values[OP + G1_POINT_ADDITION_Y1 + i],
            );
            let a_b_inf_or = builder.add_extension(local_values[A_IS_INF], local_values[B_IS_INF]);
            let res_selector = builder.sub_extension(one, a_b_inf_or);
            let mul3 =
                builder.mul_extension(res_selector, local_values[OP + G1_POINT_ADDITION_Y3 + i]);
            let sel = builder.add_many_extension([mul1, mul2, mul3].iter());
            let sub2 = builder.sub_extension(sel, public_inputs[RES + i + 12]);
            let c = builder.mul_extension(mul, sub2);
            yield_constr.constraint_transition(builder, c);
        }
    }

    fn constraint_degree(&self) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, time::Instant};

    use num_bigint::BigUint;
    use plonky2::{
        field::types::Field,
        plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
        util::timing::TimingTree,
    };
    use starky::{
        config::StarkConfig, prover::prove, stark_testing::test_stark_circuit_constraints,
        util::trace_rows_to_poly_values, verifier::verify_stark_proof,
    };

    use crate::verification::{
        proofs::ecc_aggregate::{ECCAggStark, PUBLIC_INPUTS},
        utils::native_bls::Fp,
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_stark() {
        let points = vec![
            [
                Fp::get_fp_from_biguint(BigUint::from_str("1126623738681067087257746233621637126057761795105632825039721241530561605789561587401946101488319534304696021688867").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("1234387340521756581420450482431370540586970509879406431297819985144306829815967947105769314182001941730344316001350").unwrap()),
            ],
            [
                Fp::get_fp_from_biguint(BigUint::from_str("2227077755005763044330380583051825752563137755581948302467438657174056912044402195092391651898529973204169901068783").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("3938630597268339120028117703099158472921664122822994005207596611335679241381714852577868690525915355086230780305947").unwrap()),
            ],
            [
                Fp::get_fp_from_biguint(BigUint::from_str("2053421366648413666933823320372384475868365657546365151314366566305943228812274803081977171134139973672272644986467").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("2207186006724644389273078744925282614132324652989136375251464172117985207634291886391161234997127125369095093999580").unwrap()),
            ],
            [
                Fp::get_fp_from_biguint(BigUint::from_str("1169033241627732070028418158513086714573774583755322258758869360993605333207425506760926925188242096359787524711048").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("1939098195156802692901565413605879295070375396037882592352367327738278109451906629210913080936054556354569965872525").unwrap()),
            ],
            [
                Fp::get_fp_from_biguint(BigUint::from_str("757685478162556714953738341385841404889192281968043194927346142717614691781995093597748984673977386749617478360670").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("1622292944328971247507221252705479796178069451942507208036486950244675416773645537482644705283188691681973682881721").unwrap()),
            ],
        ];

        let res = [
            Fp::get_fp_from_biguint(BigUint::from_str("234946323920378256253926848256725419729562691368307398516954172389919862735998320714103660807049810312016222352882").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("2296889008503245272184437522688584268823483443227037325600916018718669246920962157509760406568054757825100071836929").unwrap()),
        ];

        let bits = vec![true, true, true, true, false];

        let mut config = StarkConfig::standard_fast_config();
        config.fri_config.rate_bits = 2;
        let stark = ECCAggStark::<F, D>::new(64);
        let s = Instant::now();
        let mut public_inputs = Vec::<F>::new();
        for pt in &points {
            for x in &pt[0].0 {
                public_inputs.push(F::from_canonical_u32(*x));
            }
            for y in &pt[1].0 {
                public_inputs.push(F::from_canonical_u32(*y));
            }
        }
        for b in bits.iter() {
            public_inputs.push(F::from_bool(*b));
        }
        for x in res[0].0 {
            public_inputs.push(F::from_canonical_u32(x));
        }
        for y in res[1].0 {
            public_inputs.push(F::from_canonical_u32(y));
        }
        assert_eq!(public_inputs.len(), PUBLIC_INPUTS);
        let trace = stark.generate_trace(&points, &bits);
        let trace_poly_values = trace_rows_to_poly_values(trace);
        let proof = prove::<F, C, ECCAggStark<F, D>, D>(
            stark,
            &config,
            trace_poly_values,
            &public_inputs,
            &mut TimingTree::default(),
        )
        .unwrap();
        println!("Time taken for acc_agg stark proof {:?}", s.elapsed());
        verify_stark_proof(stark, proof.clone(), &config).unwrap();
    }

    #[test]
    fn test_stark_circuit() {
        let stark = ECCAggStark::<F, D>::new(64);
        test_stark_circuit_constraints::<F, C, ECCAggStark<F, D>, D>(stark).unwrap();
    }
}
