use num_bigint::BigUint;
use plonky2::{
    field::{
        extension::{Extendable, FieldExtension},
        packed::PackedField,
        types::Field,
    },
    hash::hash_types::RichField,
    iop::ext_target::ExtensionTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::verification::{
    fields::starky::fp::*,
    utils::{
        native_bls::{get_u32_vec_from_literal_ref, get_u32_vec_from_literal_ref_24, modulus, Fp},
        starky_utils::assign_u32_in_series,
    },
};

pub const G1_POINT_ADDITION_X1: usize = 0;
pub const G1_POINT_ADDITION_Y1: usize = G1_POINT_ADDITION_X1 + 12;
pub const G1_POINT_ADDITION_X2: usize = G1_POINT_ADDITION_Y1 + 12;
pub const G1_POINT_ADDITION_Y2: usize = G1_POINT_ADDITION_X2 + 12;
pub const G1_POINT_ADDITION_X3: usize = G1_POINT_ADDITION_Y2 + 12;
pub const G1_POINT_ADDITION_Y3: usize = G1_POINT_ADDITION_X3 + 12;
pub const X2_X1_DIFF: usize = G1_POINT_ADDITION_Y3 + 12;
pub const Y2_Y1_DIFF: usize = X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_TOTAL;
pub const X2_X1_SQ: usize = Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_TOTAL;
pub const Y2_Y1_SQ: usize =
    X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL + RANGE_CHECK_TOTAL;
pub const X1_X2_X3_SUM: usize =
    Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL + RANGE_CHECK_TOTAL;
pub const X1_X2_X3_X2_X1_SQ: usize = X1_X2_X3_SUM + FP_ADDITION_TOTAL * 2;
pub const Y1_Y3: usize =
    X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL + RANGE_CHECK_TOTAL;
pub const X1_X3: usize = Y1_Y3 + FP_ADDITION_TOTAL;
pub const Y1_Y3_X2_X1: usize = X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_TOTAL;
pub const Y2_Y1_X1_X3: usize =
    Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL + RANGE_CHECK_TOTAL;
pub const TOT_COL: usize =
    Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL + RANGE_CHECK_TOTAL;

/// Fills the stark trace of g1 ec addition
pub fn fill_trace_g1_addition<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    pt1: &[Fp; 2],
    pt2: &[Fp; 2],
    start_row: usize,
    start_col: usize,
) -> [Fp; 2] {
    let dy = pt2[1] - pt1[1];
    let dx = pt2[0] - pt1[0];
    let lambda = dy / dx;
    let lambda_sq = lambda * lambda;
    let x3_fp = lambda_sq - pt2[0] - pt1[0];
    let y3_fp = lambda * (pt1[0] - x3_fp) - pt1[1];

    let end_row = start_row + 11;
    for row in start_row..end_row + 1 {
        assign_u32_in_series(trace, row, start_col + G1_POINT_ADDITION_X1, &pt1[0].0);
        assign_u32_in_series(trace, row, start_col + G1_POINT_ADDITION_Y1, &pt1[1].0);
        assign_u32_in_series(trace, row, start_col + G1_POINT_ADDITION_X2, &pt2[0].0);
        assign_u32_in_series(trace, row, start_col + G1_POINT_ADDITION_Y2, &pt2[1].0);
        assign_u32_in_series(trace, row, start_col + G1_POINT_ADDITION_X3, &x3_fp.0);
        assign_u32_in_series(trace, row, start_col + G1_POINT_ADDITION_Y3, &y3_fp.0);
    }

    let x1 = pt1[0].to_biguint();
    let y1 = pt1[1].to_biguint();
    let x2 = pt2[0].to_biguint();
    let y2 = pt2[1].to_biguint();
    let x3 = x3_fp.to_biguint();
    let y3 = y3_fp.to_biguint();
    let p = modulus();

    let x2_mod = &x2 + &p;
    for row in start_row..end_row + 1 {
        fill_trace_addition_fp(
            trace,
            &get_u32_vec_from_literal_ref(&x2),
            &get_u32_vec_from_literal_ref(&p),
            row,
            start_col + X2_X1_DIFF,
        );
    }
    let x2_x1 = &x2_mod - &x1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_fp(
            trace,
            &get_u32_vec_from_literal_ref(&x2_mod),
            &get_u32_vec_from_literal_ref(&x1),
            row,
            start_col + X2_X1_DIFF + FP_ADDITION_TOTAL,
        );
    }
    let y2_mod = &y2 + &p;
    for row in start_row..end_row + 1 {
        fill_trace_addition_fp(
            trace,
            &get_u32_vec_from_literal_ref(&y2),
            &get_u32_vec_from_literal_ref(&p),
            row,
            start_col + Y2_Y1_DIFF,
        );
    }
    let y2_y1 = &y2_mod - &y1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_fp(
            trace,
            &get_u32_vec_from_literal_ref(&y2_mod),
            &get_u32_vec_from_literal_ref(&y1),
            row,
            start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL,
        );
    }
    let x2_x1_sq = &x2_x1 * &x2_x1;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &get_u32_vec_from_literal_ref(&x2_x1),
        &get_u32_vec_from_literal_ref(&x2_x1),
        start_row,
        end_row,
        start_col + X2_X1_SQ,
    );
    let res = fill_reduction_trace(
        trace,
        &get_u32_vec_from_literal_ref_24(&x2_x1_sq),
        start_row,
        end_row,
        start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
    );
    let x2_x1_sq = &x2_x1_sq % &p;
    // for row in start_row..end_row+1 {
    fill_range_check_trace(
        trace,
        &res,
        end_row,
        start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
    );
    // }
    let y2_y1_sq = &y2_y1 * &y2_y1;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &get_u32_vec_from_literal_ref(&y2_y1),
        &get_u32_vec_from_literal_ref(&y2_y1),
        start_row,
        end_row,
        start_col + Y2_Y1_SQ,
    );
    let res = fill_reduction_trace(
        trace,
        &get_u32_vec_from_literal_ref_24(&y2_y1_sq),
        start_row,
        end_row,
        start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
    );
    let y2_y1_sq = &y2_y1_sq % &p;
    // for row in start_row..end_row+1 {
    fill_range_check_trace(
        trace,
        &res,
        end_row,
        start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
    );
    // }
    let x1_x2 = &x1 + &x2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_fp(
            trace,
            &get_u32_vec_from_literal_ref(&x1),
            &get_u32_vec_from_literal_ref(&x2),
            row,
            start_col + X1_X2_X3_SUM,
        );
    }
    let x1_x2_x3 = &x1_x2 + &x3;
    for row in start_row..end_row + 1 {
        fill_trace_addition_fp(
            trace,
            &get_u32_vec_from_literal_ref(&x1_x2),
            &get_u32_vec_from_literal_ref(&x3),
            row,
            start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL,
        );
    }
    let x1_x2_x3_x2_x1_sq = &x1_x2_x3 * &x2_x1_sq;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &get_u32_vec_from_literal_ref(&x1_x2_x3),
        &get_u32_vec_from_literal_ref(&x2_x1_sq),
        start_row,
        end_row,
        start_col + X1_X2_X3_X2_X1_SQ,
    );
    let res = fill_reduction_trace(
        trace,
        &get_u32_vec_from_literal_ref_24(&x1_x2_x3_x2_x1_sq),
        start_row,
        end_row,
        start_col + X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
    );
    let x1_x2_x3_x2_x1_sq = &x1_x2_x3_x2_x1_sq % &p;
    // for row in start_row..end_row+1 {
    fill_range_check_trace(
        trace,
        &res,
        end_row,
        start_col + X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
    );
    // }
    assert_eq!(&x1_x2_x3_x2_x1_sq, &y2_y1_sq);
    let y1_y3 = &y1 + &y3;
    for row in start_row..end_row + 1 {
        fill_trace_addition_fp(
            trace,
            &get_u32_vec_from_literal_ref(&y1),
            &get_u32_vec_from_literal_ref(&y3),
            row,
            start_col + Y1_Y3,
        );
    }
    let x1_mod = &x1 + &p;
    for row in start_row..end_row + 1 {
        fill_trace_addition_fp(
            trace,
            &get_u32_vec_from_literal_ref(&x1),
            &get_u32_vec_from_literal_ref(&p),
            row,
            start_col + X1_X3,
        );
    }
    let x1_x3 = &x1_mod - &x3;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_fp(
            trace,
            &get_u32_vec_from_literal_ref(&x1_mod),
            &get_u32_vec_from_literal_ref(&x3),
            row,
            start_col + X1_X3 + FP_ADDITION_TOTAL,
        );
    }
    let y1_y3_x2_x1 = &y1_y3 * &x2_x1;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &get_u32_vec_from_literal_ref(&y1_y3),
        &get_u32_vec_from_literal_ref(&x2_x1),
        start_row,
        end_row,
        start_col + Y1_Y3_X2_X1,
    );
    let res = fill_reduction_trace(
        trace,
        &get_u32_vec_from_literal_ref_24(&y1_y3_x2_x1),
        start_row,
        end_row,
        start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS,
    );
    let y1_y3_x2_x1 = BigUint::new(res.to_vec());
    // for row in start_row..end_row+1 {
    fill_range_check_trace(
        trace,
        &res,
        end_row,
        start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
    );
    // }
    let y2_y1_x1_x3 = &y2_y1 * &x1_x3;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &get_u32_vec_from_literal_ref(&y2_y1),
        &get_u32_vec_from_literal_ref(&x1_x3),
        start_row,
        end_row,
        start_col + Y2_Y1_X1_X3,
    );
    let res = fill_reduction_trace(
        trace,
        &get_u32_vec_from_literal_ref_24(&y2_y1_x1_x3),
        start_row,
        end_row,
        start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS,
    );
    let y2_y1_x1_x3 = BigUint::new(res.to_vec());
    // for row in start_row..end_row+1 {
    fill_range_check_trace(
        trace,
        &res,
        end_row,
        start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
    );
    // }
    assert_eq!(&y1_y3_x2_x1, &y2_y1_x1_x3);
    [x3_fp, y3_fp]
}

/// Constraints the g1 ec addition.
pub fn add_g1_addition_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);
    let p = modulus().to_u32_digits();

    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + G1_POINT_ADDITION_X1 + i]
                    - next_values[start_col + G1_POINT_ADDITION_X1 + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + G1_POINT_ADDITION_Y1 + i]
                    - next_values[start_col + G1_POINT_ADDITION_Y1 + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + G1_POINT_ADDITION_X2 + i]
                    - next_values[start_col + G1_POINT_ADDITION_X2 + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + G1_POINT_ADDITION_Y2 + i]
                    - next_values[start_col + G1_POINT_ADDITION_Y2 + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + G1_POINT_ADDITION_X3 + i]
                    - next_values[start_col + G1_POINT_ADDITION_X3 + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + G1_POINT_ADDITION_Y3 + i]
                    - next_values[start_col + G1_POINT_ADDITION_Y3 + i]),
        );
    }
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X2_X1_DIFF + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + X2_X1_DIFF + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X2 + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X2_X1_DIFF + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + X2_X1_DIFF + FP_ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(p[i])),
        );
    }
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + X2_X1_DIFF,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values
                    [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_X_OFFSET + i]
                    - local_values[start_col + X2_X1_DIFF + FP_ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values
                    [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_Y_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X1 + i]),
        );
    }
    add_subtraction_fp_constraints(
        local_values,
        yield_constr,
        start_col + X2_X1_DIFF + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_Y2 + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(p[i])),
        );
    }
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + Y2_Y1_DIFF,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values
                    [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_X_OFFSET + i]
                    - local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values
                    [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_Y_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_Y1 + i]),
        );
    }
    add_subtraction_fp_constraints(
        local_values,
        yield_constr,
        start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + X2_X1_SQ + X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + X2_X1_DIFF
                        + FP_ADDITION_TOTAL
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + X2_X1_SQ + Y_INPUT_OFFSET + i]
                    - local_values[start_col
                        + X2_X1_DIFF
                        + FP_ADDITION_TOTAL
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X2_X1_SQ,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + X2_X1_SQ
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCTION_TOTAL
                    + RANGE_CHECK_SELECTOR_OFFSET]
                * (local_values[start_col + X2_X1_SQ + SUM_OFFSET + i]
                    - local_values[start_col
                        + X2_X1_SQ
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y2_Y1_SQ + X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + Y2_Y1_DIFF
                        + FP_ADDITION_TOTAL
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y2_Y1_SQ + Y_INPUT_OFFSET + i]
                    - local_values[start_col
                        + Y2_Y1_DIFF
                        + FP_ADDITION_TOTAL
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Y2_Y1_SQ,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + Y2_Y1_SQ
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCTION_TOTAL
                    + RANGE_CHECK_SELECTOR_OFFSET]
                * (local_values[start_col + Y2_Y1_SQ + SUM_OFFSET + i]
                    - local_values[start_col
                        + Y2_Y1_SQ
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + Y2_Y1_SQ + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X1 + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_Y_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X2 + i]),
        );
    }
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + X1_X2_X3_SUM,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_CHECK_OFFSET]
                * (local_values
                    [start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_CHECK_OFFSET]
                * (local_values
                    [start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_Y_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X3 + i]),
        );
    }
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + X1_X2_X3_X2_X1_SQ + X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + X1_X2_X3_SUM
                        + FP_ADDITION_TOTAL
                        + FP_ADDITION_SUM_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + X1_X2_X3_X2_X1_SQ + Y_INPUT_OFFSET + i]
                    - local_values[start_col
                        + X2_X1_SQ
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X1_X2_X3_X2_X1_SQ,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + X1_X2_X3_X2_X1_SQ
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCTION_TOTAL
                    + RANGE_CHECK_SELECTOR_OFFSET]
                * (local_values[start_col + X1_X2_X3_X2_X1_SQ + SUM_OFFSET + i]
                    - local_values[start_col
                        + X1_X2_X3_X2_X1_SQ
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col
                    + X1_X2_X3_X2_X1_SQ
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCED_OFFSET
                    + i]
                    - local_values[start_col
                        + Y2_Y1_SQ
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y1_Y3 + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Y1_Y3 + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_Y1 + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y1_Y3 + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Y1_Y3 + FP_ADDITION_Y_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_Y3 + i]),
        );
    }
    add_addition_fp_constraints(local_values, yield_constr, start_col + Y1_Y3, bit_selector);

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X3 + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + X1_X3 + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X1 + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X3 + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + X1_X3 + FP_ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(p[i])),
        );
    }
    add_addition_fp_constraints(local_values, yield_constr, start_col + X1_X3, bit_selector);

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values
                    [start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_X_OFFSET + i]
                    - local_values[start_col + X1_X3 + FP_ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values
                    [start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_Y_OFFSET + i]
                    - local_values[start_col + G1_POINT_ADDITION_X3 + i]),
        );
    }
    add_subtraction_fp_constraints(
        local_values,
        yield_constr,
        start_col + X1_X3 + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y1_Y3_X2_X1 + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y1_Y3_X2_X1 + X_INPUT_OFFSET + i]
                    - local_values[start_col + Y1_Y3 + FP_ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y1_Y3_X2_X1 + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y1_Y3_X2_X1 + Y_INPUT_OFFSET + i]
                    - local_values[start_col
                        + X2_X1_DIFF
                        + FP_ADDITION_TOTAL
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Y1_Y3_X2_X1,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + Y1_Y3_X2_X1
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCTION_TOTAL
                    + RANGE_CHECK_SELECTOR_OFFSET]
                * (local_values[start_col + Y1_Y3_X2_X1 + SUM_OFFSET + i]
                    - local_values[start_col
                        + Y1_Y3_X2_X1
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + Y1_Y3_X2_X1 + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y2_Y1_X1_X3 + X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + Y2_Y1_DIFF
                        + FP_ADDITION_TOTAL
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y2_Y1_X1_X3 + Y_INPUT_OFFSET + i]
                    - local_values
                        [start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Y2_Y1_X1_X3,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + Y2_Y1_X1_X3
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCTION_TOTAL
                    + RANGE_CHECK_SELECTOR_OFFSET]
                * (local_values[start_col + Y2_Y1_X1_X3 + SUM_OFFSET + i]
                    - local_values[start_col
                        + Y2_Y1_X1_X3
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col
                    + Y2_Y1_X1_X3
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCED_OFFSET
                    + i]
                    - local_values[start_col
                        + Y1_Y3_X2_X1
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCED_OFFSET
                        + i]),
        );
    }
}

pub fn add_g1_addition_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));
    let p = modulus().to_u32_digits();

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET],
        );

        let sub = builder.sub_extension(
            local_values[start_col + G1_POINT_ADDITION_X1 + i],
            next_values[start_col + G1_POINT_ADDITION_X1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint_transition(builder, c);
        let sub = builder.sub_extension(
            local_values[start_col + G1_POINT_ADDITION_Y1 + i],
            next_values[start_col + G1_POINT_ADDITION_Y1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint_transition(builder, c);
        let sub = builder.sub_extension(
            local_values[start_col + G1_POINT_ADDITION_X2 + i],
            next_values[start_col + G1_POINT_ADDITION_X2 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint_transition(builder, c);
        let sub = builder.sub_extension(
            local_values[start_col + G1_POINT_ADDITION_Y2 + i],
            next_values[start_col + G1_POINT_ADDITION_Y2 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint_transition(builder, c);
        let sub = builder.sub_extension(
            local_values[start_col + G1_POINT_ADDITION_X3 + i],
            next_values[start_col + G1_POINT_ADDITION_X3 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint_transition(builder, c);
        let sub = builder.sub_extension(
            local_values[start_col + G1_POINT_ADDITION_Y3 + i],
            next_values[start_col + G1_POINT_ADDITION_Y3 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint_transition(builder, c);
    }
    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X2 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let p_const = builder.constant_extension(F::Extension::from_canonical_u32(p[i]));
        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_Y_OFFSET + i],
            p_const,
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X2_X1_DIFF,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_X_OFFSET + i],
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_SUM_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_Y_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X2_X1_DIFF + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_Y2 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let p_const = builder.constant_extension(F::Extension::from_canonical_u32(p[i]));
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_Y_OFFSET + i],
            p_const,
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Y2_Y1_DIFF,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_X_OFFSET + i],
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_SUM_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_Y_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_Y1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_SQ + X_INPUT_OFFSET + i],
            local_values
                [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_SQ + Y_INPUT_OFFSET + i],
            local_values
                [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X2_X1_SQ,
        bit_selector,
    );
    for i in 0..24 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + X2_X1_SQ
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCTION_TOTAL
                + RANGE_CHECK_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X2_X1_SQ + SUM_OFFSET + i],
            local_values
                [start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCE_X_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y2_Y1_SQ + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_SQ + X_INPUT_OFFSET + i],
            local_values
                [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_SQ + Y_INPUT_OFFSET + i],
            local_values
                [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Y2_Y1_SQ,
        bit_selector,
    );
    for i in 0..24 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + Y2_Y1_SQ
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCTION_TOTAL
                + RANGE_CHECK_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_SQ + SUM_OFFSET + i],
            local_values
                [start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCE_X_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + Y2_Y1_SQ + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_Y_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X2 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X1_X2_X3_SUM,
        bit_selector,
    );
    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_SUM_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_Y_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X3 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_X2_X1_SQ + X_INPUT_OFFSET + i],
            local_values[start_col + X1_X2_X3_SUM + FP_ADDITION_TOTAL + FP_ADDITION_SUM_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_X2_X1_SQ + Y_INPUT_OFFSET + i],
            local_values
                [start_col + X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X1_X2_X3_X2_X1_SQ,
        bit_selector,
    );
    for i in 0..24 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + X1_X2_X3_X2_X1_SQ
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCTION_TOTAL
                + RANGE_CHECK_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X1_X2_X3_X2_X1_SQ + SUM_OFFSET + i],
            local_values[start_col
                + X1_X2_X3_X2_X1_SQ
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCE_X_OFFSET
                + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X1_X2_X3_X2_X1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X1_X2_X3_X2_X1_SQ + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col
                + X1_X2_X3_X2_X1_SQ
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCED_OFFSET
                + i],
            local_values
                [start_col + Y2_Y1_SQ + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y1_Y3 + FP_ADDITION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y1_Y3 + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_Y1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + Y1_Y3 + FP_ADDITION_Y_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_Y3 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Y1_Y3,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X1_X3 + FP_ADDITION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X1_X3 + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X1 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let p_const = builder.constant_extension(F::Extension::from_canonical_u32(p[i]));
        let sub = builder.sub_extension(
            local_values[start_col + X1_X3 + FP_ADDITION_Y_OFFSET + i],
            p_const,
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X1_X3,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_X_OFFSET + i],
            local_values[start_col + X1_X3 + FP_ADDITION_SUM_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_Y_OFFSET + i],
            local_values[start_col + G1_POINT_ADDITION_X3 + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X1_X3 + FP_ADDITION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y1_Y3_X2_X1 + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y1_Y3_X2_X1 + X_INPUT_OFFSET + i],
            local_values[start_col + Y1_Y3 + FP_ADDITION_SUM_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + Y1_Y3_X2_X1 + Y_INPUT_OFFSET + i],
            local_values
                [start_col + X2_X1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Y1_Y3_X2_X1,
        bit_selector,
    );
    for i in 0..24 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + Y1_Y3_X2_X1
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCTION_TOTAL
                + RANGE_CHECK_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y1_Y3_X2_X1 + SUM_OFFSET + i],
            local_values
                [start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCE_X_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + Y1_Y3_X2_X1 + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_X1_X3 + X_INPUT_OFFSET + i],
            local_values
                [start_col + Y2_Y1_DIFF + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);

        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_X1_X3 + Y_INPUT_OFFSET + i],
            local_values[start_col + X1_X3 + FP_ADDITION_TOTAL + FP_SUBTRACTION_DIFF_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Y2_Y1_X1_X3,
        bit_selector,
    );
    for i in 0..24 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + Y2_Y1_X1_X3
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCTION_TOTAL
                + RANGE_CHECK_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values[start_col + Y2_Y1_X1_X3 + SUM_OFFSET + i],
            local_values
                [start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCE_X_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCTION_TOTAL,
        bit_selector,
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Y2_Y1_X1_X3 + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let sub = builder.sub_extension(
            local_values
                [start_col + Y2_Y1_X1_X3 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCED_OFFSET + i],
            local_values
                [start_col + Y1_Y3_X2_X1 + FP_MULTIPLICATION_TOTAL_COLUMNS + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(mul, sub);
        yield_constr.constraint(builder, c);
    }
}
