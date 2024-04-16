//! This module contains functions for filling the stark trace and adding constraints for the corresponding trace for some Fp2 operations (multiplication, addition, subtraction, etc). One Fp2 element is represented as \[u32; 24\] inside the trace. But most of the time, Fp2 elements are broken up into two Fp elements, hence represented as two \[u32; 12\].
use crate::verification::{
    fields::starky::fp::*,
    native::{get_u32_vec_from_literal, get_u32_vec_from_literal_24, modulus, Fp, Fp2},
    utils::*,
};
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

// Fp2 Multiplication layout offsets
/*
    These trace offsets are for Fp2 multiplication. It needs 12 rows.
    [x0, x1] * [y0, y1] = [x0*y0 - x1*y1, x0*y1 + x1*y0]
    FP2_FP2_SELECTOR_OFFSET -> Selector to ensure that the input is same across all rows. Set 1 in all rows except last one.
    FP2_FP2_X_INPUT_OFFSET -> offset where input x is set.
    FP2_FP2_Y_INPUT_OFFSET -> offset where input y is set.
    X_0_Y_0_MULTIPLICATION_OFFSET -> offset where x0*y0 multiplication is set.
    X_1_Y_1_MULTIPLICATION_OFFSET -> offset where x1*y1 multiplication is set.
    Z1_ADD_MODULUS_OFFSET -> Addition operation to add x0*y0 + p*p (required because we don't know if x0*y0 - x1*y1 will overflow).
    Z1_SUBTRACTION_OFFSET -> Subtraction operation for x0*y0 + p*p - x1*y1.
    Z1_REDUCE_OFFSET -> Reduction operation for Z1 (z1 is the real part of the result).
    Z1_RANGECHECK_OFFSET -> Range check the result of Z1 reduction.
    X_0_Y_1_MULTIPLICATION_OFFSET -> offset where x0*y1 multiplication is set.
    X_1_Y_0_MULTIPLICATION_OFFSET -> offset where x1*y0 multiplication is set.
    Z2_ADDITION_OFFSET -> Addition operation for x0*y1 + x1*y0.
    Z2_REDUCE_OFFSET -> Reduction operation for Z2 (z2 is the imaginary part of the result).
    Z2_RANGECHECK_OFFSET -> Range check the result of Z2 reduction.
*/
pub const FP2_FP2_SELECTOR_OFFSET: usize = 0;
pub const FP2_FP2_X_INPUT_OFFSET: usize = FP2_FP2_SELECTOR_OFFSET + 1;
pub const FP2_FP2_Y_INPUT_OFFSET: usize = FP2_FP2_X_INPUT_OFFSET + 24;
pub const X_0_Y_0_MULTIPLICATION_OFFSET: usize = FP2_FP2_Y_INPUT_OFFSET + 24;
pub const X_1_Y_1_MULTIPLICATION_OFFSET: usize =
    X_0_Y_0_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;

pub const Z1_ADD_MODULUS_OFFSET: usize =
    X_1_Y_1_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const Z1_SUBTRACTION_OFFSET: usize = Z1_ADD_MODULUS_OFFSET + ADDITION_TOTAL;
pub const Z1_REDUCE_OFFSET: usize = Z1_SUBTRACTION_OFFSET + SUBTRACTION_TOTAL;
pub const Z1_RANGECHECK_OFFSET: usize = Z1_REDUCE_OFFSET + REDUCTION_TOTAL;

pub const X_0_Y_1_MULTIPLICATION_OFFSET: usize = Z1_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;
pub const X_1_Y_0_MULTIPLICATION_OFFSET: usize =
    X_0_Y_1_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;

pub const Z2_ADDITION_OFFSET: usize =
    X_1_Y_0_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const Z2_REDUCE_OFFSET: usize = Z2_ADDITION_OFFSET + ADDITION_TOTAL;
pub const Z2_RANGECHECK_OFFSET: usize = Z2_REDUCE_OFFSET + REDUCTION_TOTAL;

pub const TOTAL_COLUMNS_FP2_MULTIPLICATION: usize = Z2_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;

// Fp2 * Fp multiplication layout offsets
/*
    These trace offsets are for multiplication of Fp2 with Fp. It needs 12 rows.
    [x0, x1] * y = [x0y, x1y]
    FP2_FP_MUL_SELECTOR_OFFSET -> Selector to ensure that the input is same across all rows. Set 1 in all rows except last one.
    X0_Y_REDUCE_OFFSET -> Reduction operation for x0y.
    X0_Y_RANGECHECK_OFFSET -> Range check for result of x0y reduction.
    X1_Y_REDUCE_OFFSET -> Reduction operation for x1y.
    X1_Y_RANGECHECK_OFFSET -> Range check for result of x1y reduction.
*/
pub const FP2_FP_MUL_SELECTOR_OFFSET: usize = 0;
pub const FP2_FP_X_INPUT_OFFSET: usize = FP2_FP_MUL_SELECTOR_OFFSET + 1;
pub const FP2_FP_Y_INPUT_OFFSET: usize = FP2_FP_X_INPUT_OFFSET + 24;
pub const X0_Y_MULTIPLICATION_OFFSET: usize = FP2_FP_Y_INPUT_OFFSET + 12;
pub const X0_Y_REDUCE_OFFSET: usize = X0_Y_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const X0_Y_RANGECHECK_OFFSET: usize = X0_Y_REDUCE_OFFSET + REDUCTION_TOTAL;
pub const X1_Y_MULTIPLICATION_OFFSET: usize = X0_Y_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;
pub const X1_Y_REDUCE_OFFSET: usize = X1_Y_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const X1_Y_RANGECHECK_OFFSET: usize = X1_Y_REDUCE_OFFSET + REDUCTION_TOTAL;
pub const FP2_FP_TOTAL_COLUMNS: usize = X1_Y_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;

// Multiply by B layout offsets
/*
    These trace offsets are for `multiply_by_b` function (super::native::Fp2::multiply_by_B). It needs 12 rows.
    MULTIPLY_BY_B_SELECTOR_OFFSET -> Selector to ensure that the input is same across all rows. Set 1 in all rows except last one.
    MULTIPLY_BY_B_ADD_MODSQ_OFFSET -> Addition operation to add x0*4 + p*p (required because we don't know if x0*4 - x1*4 will overflow).
    MULTIPLY_BY_B_Z0_REDUCE_OFFSET -> Reduction operation for Z0 (z0 is the real part of the result).
    MULTIPLY_BY_B_Z0_RANGECHECK_OFFSET -> Range check for result of Z0 reduction.
    MULTIPLY_BY_B_Z1_REDUCE_OFFSET -> Reduction operation for Z1 (z1 is the imaginary part of the result).
    MULTIPLY_BY_B_Z1_RANGECHECK_OFFSET -> Range check for result of Z1 reduction.
*/
pub const MULTIPLY_B_SELECTOR_OFFSET: usize = 0;
pub const MULTIPLY_B_X_OFFSET: usize = MULTIPLY_B_SELECTOR_OFFSET + 1;
pub const MULTIPLY_B_X0_B_MUL_OFFSET: usize = MULTIPLY_B_X_OFFSET + 24;
pub const MULTIPLY_B_X1_B_MUL_OFFSET: usize =
    MULTIPLY_B_X0_B_MUL_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const MULTIPLY_B_ADD_MODSQ_OFFSET: usize =
    MULTIPLY_B_X1_B_MUL_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const MULTIPLY_B_SUB_OFFSET: usize = MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_TOTAL;
pub const MULTIPLY_B_Z0_REDUCE_OFFSET: usize = MULTIPLY_B_SUB_OFFSET + SUBTRACTION_TOTAL;
pub const MULTIPLY_B_Z0_RANGECHECK_OFFSET: usize = MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCTION_TOTAL;
pub const MULTIPLY_B_ADD_OFFSET: usize = MULTIPLY_B_Z0_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;
pub const MULTIPLY_B_Z1_REDUCE_OFFSET: usize = MULTIPLY_B_ADD_OFFSET + ADDITION_TOTAL;
pub const MULTIPLY_B_Z1_RANGECHECK_OFFSET: usize = MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCTION_TOTAL;
pub const MULTIPLY_B_TOTAL_COLUMS: usize = MULTIPLY_B_Z1_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;

// Fp2 addition layout offsets
/*
    These trace offsets are for addition for two Fp2 elements. In essence it's two concatenated Fp additions. It needs 1 row.
*/
pub const FP2_ADDITION_0_OFFSET: usize = 0;
pub const FP2_ADDITION_1_OFFSET: usize = FP2_ADDITION_0_OFFSET + FP_ADDITION_TOTAL;
pub const FP2_ADDITION_TOTAL: usize = FP2_ADDITION_1_OFFSET + FP_ADDITION_TOTAL;

// Fp2 subtraction layout offsets
/*
    These trace offsets are for subtraction for two Fp2 elements. In essence it's two concatenated Fp subtractions. It needs 1 row.
*/
pub const FP2_SUBTRACTION_0_OFFSET: usize = 0;
pub const FP2_SUBTRACTION_1_OFFSET: usize = FP2_SUBTRACTION_0_OFFSET + FP_SUBTRACTION_TOTAL;
pub const FP2_SUBTRACTION_TOTAL: usize = FP2_SUBTRACTION_1_OFFSET + FP_SUBTRACTION_TOTAL;

// Fp2 multiply single
/*
    These trace offsets are for multiply by single for two Fp2 elements. In essence it's two concatenated Fp multiply by single. It needs 1 row.
*/
pub const FP2_MULTIPLY_SINGLE_0_OFFSET: usize = 0;
pub const FP2_MULTIPLY_SINGLE_1_OFFSET: usize =
    FP2_MULTIPLY_SINGLE_0_OFFSET + FP_MULTIPLY_SINGLE_TOTAL;
pub const FP2_MULTIPLY_SINGLE_TOTAL: usize =
    FP2_MULTIPLY_SINGLE_1_OFFSET + FP_MULTIPLY_SINGLE_TOTAL;

// FP2 non residue multiplication
/*
    These trace offsets are for Fp2 non residue multiplication (super::native::Fp2::mul_by_nonresidue).  It needs 1 row.
    FP2_NON_RESIDUE_MUL_CHECK_OFFSET -> Selector to indicate the operation is on.
    FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET -> This offset is for two operations in one. First is addition with bls12-381 field prime, followed by subtraction.
    FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET -> Reduction operation for Z0 (z0 is the real part of the result).
    FP2_NON_RESIDUE_MUL_Z0_RANGECHECK_OFFSET -> Range check for result of Z0 reduction.
    FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET -> Reduction operation for Z1 (z1 is the imaginary part of the result).
    FP2_NON_RESIDUE_MUL_Z1_RANGECHECK_OFFSET -> Range check for result of Z1 reduction.
*/
pub const FP2_NON_RESIDUE_MUL_CHECK_OFFSET: usize = 0;
pub const FP2_NON_RESIDUE_MUL_INPUT_OFFSET: usize = FP2_NON_RESIDUE_MUL_CHECK_OFFSET + 1;
pub const FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET: usize = FP2_NON_RESIDUE_MUL_INPUT_OFFSET + 24;
pub const FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET: usize =
    FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_TOTAL + FP_SUBTRACTION_TOTAL;
pub const FP2_NON_RESIDUE_MUL_Z0_RANGECHECK_OFFSET: usize =
    FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET + FP_SINGLE_REDUCE_TOTAL;
pub const FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET: usize =
    FP2_NON_RESIDUE_MUL_Z0_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;
pub const FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET: usize =
    FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_TOTAL;
pub const FP2_NON_RESIDUE_MUL_Z1_RANGECHECK_OFFSET: usize =
    FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET + FP_SINGLE_REDUCE_TOTAL;
pub const FP2_NON_RESIDUE_MUL_TOTAL: usize =
    FP2_NON_RESIDUE_MUL_Z1_RANGECHECK_OFFSET + RANGE_CHECK_TOTAL;

// FP4 Sq
/*
    These trace offsets are for Fp4 square function (super::native::fp4_square). It needs 12 rows.
    FP4_SQ_SELECTOR_OFFSET -> Selector to ensure that the input is same across all rows. Set 1 in all rows except last one.
    T0 -> a*a
    T1 -> b*b
    T2 -> mul_by_nonresidue(T1)
    X -> T2 + T0
    T3 -> a+b
    T4 -> T3*T3
    T5 -> T4 - T0
    Y -> T5 - T1
    FP4_SQ_X_CALC_OFFSET, FP4_SQ_T3_CALC_OFFSET -> offset including 3 operations (fp2 addition, reduction of both real and imaginary parts of the result, range check of both real and imaginary parts of the result).
    FP4_SQ_T5_CALC_OFFSET, FP4_SQ_Y_CALC_OFFSET -> offset including 4 operations (fp2 addition (adding bls12-381 field prime to mitigate overflow), fp2 subtraction, reduction of both real and imaginary parts of the result, range check of both real and imaginary parts of the result).
*/
pub const FP4_SQ_SELECTOR_OFFSET: usize = 0;
pub const FP4_SQ_INPUT_X_OFFSET: usize = FP4_SQ_SELECTOR_OFFSET + 1;
pub const FP4_SQ_INPUT_Y_OFFSET: usize = FP4_SQ_INPUT_X_OFFSET + 24;
pub const FP4_SQ_T0_CALC_OFFSET: usize = FP4_SQ_INPUT_Y_OFFSET + 24;
pub const FP4_SQ_T1_CALC_OFFSET: usize = FP4_SQ_T0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP4_SQ_T2_CALC_OFFSET: usize = FP4_SQ_T1_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP4_SQ_X_CALC_OFFSET: usize = FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_TOTAL;
pub const FP4_SQ_T3_CALC_OFFSET: usize =
    FP4_SQ_X_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP4_SQ_T4_CALC_OFFSET: usize =
    FP4_SQ_T3_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP4_SQ_T5_CALC_OFFSET: usize = FP4_SQ_T4_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP4_SQ_Y_CALC_OFFSET: usize = FP4_SQ_T5_CALC_OFFSET
    + FP2_SUBTRACTION_TOTAL
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP4_SQ_TOTAL_COLUMNS: usize = FP4_SQ_Y_CALC_OFFSET
    + FP2_SUBTRACTION_TOTAL
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;

// Forbenius map Fp2
/*
    These trace offsets are for fp2 forbenius map (super::native::Fp2::forbenius_map). It needs 12 rows.
    FP2_FORBENIUS_MAP_SELECTOR_OFFSET -> Selector to ensure that the input is same across all rows. Set 1 in all rows except last one.
    FP2_FORBENIUS_MAP_POW_OFFSET -> offset where power is set.
    FP2_FORBENIUS_MAP_DIV_OFFSET -> offset of integer division power/2.
    FP2_FORBENIUS_MAP_REM_OFFSET -> offset of power%2.
    T0 -> x1 * forbenius_constant
    FP2_FORBENIUS_MAP_T0_CALC_OFFSET -> offset including 3 operations (multiplication, reduction of the result, range check of the result).
    FP2_FORBENIUS_MAP_MUL_RES_ROW -> Selector indicating which row contains result of the multiplication. Set 1 on the 11th row.
*/
pub const FP2_FORBENIUS_MAP_SELECTOR_OFFSET: usize = 0;
pub const FP2_FORBENIUS_MAP_INPUT_OFFSET: usize = FP2_FORBENIUS_MAP_SELECTOR_OFFSET + 1;
pub const FP2_FORBENIUS_MAP_POW_OFFSET: usize = FP2_FORBENIUS_MAP_INPUT_OFFSET + 24;
pub const FP2_FORBENIUS_MAP_DIV_OFFSET: usize = FP2_FORBENIUS_MAP_POW_OFFSET + 1;
pub const FP2_FORBENIUS_MAP_REM_OFFSET: usize = FP2_FORBENIUS_MAP_DIV_OFFSET + 1;
pub const FP2_FORBENIUS_MAP_T0_CALC_OFFSET: usize = FP2_FORBENIUS_MAP_REM_OFFSET + 1;
pub const FP2_FORBENIUS_MAP_MUL_RES_ROW: usize = FP2_FORBENIUS_MAP_T0_CALC_OFFSET
    + FP_MULTIPLICATION_TOTAL_COLUMNS
    + REDUCTION_TOTAL
    + RANGE_CHECK_TOTAL;
pub const FP2_FORBENIUS_MAP_TOTAL_COLUMNS: usize = FP2_FORBENIUS_MAP_MUL_RES_ROW + 1;

/// Fills the stark trace of fp2 addition. Inputs are 12*2 limbs each. Needs 1 row.
pub fn fill_trace_addition_fp2<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    y: &[[u32; 12]; 2],
    row: usize,
    start_col: usize,
) {
    fill_trace_addition_fp(trace, &x[0], &y[0], row, start_col + FP2_ADDITION_0_OFFSET);
    fill_trace_addition_fp(trace, &x[1], &y[1], row, start_col + FP2_ADDITION_1_OFFSET);
}

/// Fills the stark trace of fp2 subtraction. Inputs are 12*2 limbs each. Needs 1 row. Assume x > y.
pub fn fill_trace_subtraction_fp2<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    y: &[[u32; 12]; 2],
    row: usize,
    start_col: usize,
) {
    fill_trace_subtraction_fp(
        trace,
        &x[0],
        &y[0],
        row,
        start_col + FP2_SUBTRACTION_0_OFFSET,
    );
    fill_trace_subtraction_fp(
        trace,
        &x[1],
        &y[1],
        row,
        start_col + FP2_SUBTRACTION_1_OFFSET,
    );
}

/// Fills the stark trace of multiplication following long multiplication. Inputs are 12\*2 limbs and 1\*2 limbs respectively. Needs 1 row.
pub fn fill_trace_multiply_single_fp2<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    y: &[u32; 2],
    row: usize,
    start_col: usize,
) {
    fill_trace_multiply_single_fp(
        trace,
        &x[0],
        y[0],
        row,
        start_col + FP2_SUBTRACTION_0_OFFSET,
    );
    fill_trace_multiply_single_fp(
        trace,
        &x[1],
        y[1],
        row,
        start_col + FP2_SUBTRACTION_1_OFFSET,
    );
}

/// Fills the stark trace of negation. Input is 12*2 limbs. Needs 1 row. In essence, it fills an addition trace with inputs as `x` and `-x`.
pub fn fill_trace_negate_fp2<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    row: usize,
    start_col: usize,
) {
    let minus_x: [[u32; 12]; 2] = (-Fp2([Fp(x[0].to_owned()), Fp(x[1].to_owned())]))
        .0
        .iter()
        .map(|x| x.0)
        .collect::<Vec<[u32; 12]>>()
        .try_into()
        .unwrap();
    fill_trace_addition_fp2(trace, x, &minus_x, row, start_col);
}

/// Fills stark trace for fp2 multiplication. Inputs are 12*2 limbs each. Needs 12 rows. Sets addition and subtraction selectors to 1 only in 11th row, becuase that's where multiplication result is set.
pub fn generate_trace_fp2_mul<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: [[u32; 12]; 2],
    y: [[u32; 12]; 2],
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    let modulus = modulus();

    for i in start_row..end_row + 1 {
        trace[i][start_col + FP2_FP2_SELECTOR_OFFSET] = F::ONE;
        assign_u32_in_series(trace, i, start_col + FP2_FP2_X_INPUT_OFFSET, &x[0]);
        assign_u32_in_series(trace, i, start_col + FP2_FP2_X_INPUT_OFFSET + 12, &x[1]);
        assign_u32_in_series(trace, i, start_col + FP2_FP2_Y_INPUT_OFFSET, &y[0]);
        assign_u32_in_series(trace, i, start_col + FP2_FP2_Y_INPUT_OFFSET + 12, &y[1]);
    }
    trace[end_row][start_col + FP2_FP2_SELECTOR_OFFSET] = F::ZERO;
    // filling trace for X0*Y0 - X1*Y1
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[0],
        &y[0],
        start_row,
        end_row,
        start_col + X_0_Y_0_MULTIPLICATION_OFFSET,
    );
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[1],
        &y[1],
        start_row,
        end_row,
        start_col + X_1_Y_1_MULTIPLICATION_OFFSET,
    );

    let x0y0 =
        get_u32_vec_from_literal_24(BigUint::new(x[0].to_vec()) * BigUint::new(y[0].to_vec()));
    let modulus_sq = get_u32_vec_from_literal_24(modulus.clone() * modulus.clone());
    fill_addition_trace(
        trace,
        &x0y0,
        &modulus_sq,
        start_row + 11,
        start_col + Z1_ADD_MODULUS_OFFSET,
    );

    let x0y0_add_modsq =
        get_u32_vec_from_literal_24(BigUint::new(x0y0.to_vec()) + modulus.clone() * modulus);
    let x1y1 =
        get_u32_vec_from_literal_24(BigUint::new(x[1].to_vec()) * BigUint::new(y[1].to_vec()));
    fill_subtraction_trace(
        trace,
        &x0y0_add_modsq,
        &x1y1,
        start_row + 11,
        start_col + Z1_SUBTRACTION_OFFSET,
    );

    let x0y0_x1y1 = get_u32_vec_from_literal_24(
        BigUint::new(x0y0_add_modsq.to_vec()) - BigUint::new(x1y1.to_vec()),
    );
    let rem = fill_reduction_trace(
        trace,
        &x0y0_x1y1,
        start_row,
        end_row,
        start_col + Z1_REDUCE_OFFSET,
    );
    fill_range_check_trace(trace, &rem, start_row, start_col + Z1_RANGECHECK_OFFSET);

    // filling trace for X0*Y1 + X1*Y0
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[0],
        &y[1],
        start_row,
        end_row,
        start_col + X_0_Y_1_MULTIPLICATION_OFFSET,
    );
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[1],
        &y[0],
        start_row,
        end_row,
        start_col + X_1_Y_0_MULTIPLICATION_OFFSET,
    );

    let x0y1 =
        get_u32_vec_from_literal_24(BigUint::new(x[0].to_vec()) * BigUint::new(y[1].to_vec()));
    let x1y0 =
        get_u32_vec_from_literal_24(BigUint::new(x[1].to_vec()) * BigUint::new(y[0].to_vec()));
    fill_addition_trace(
        trace,
        &x0y1,
        &x1y0,
        start_row + 11,
        start_col + Z2_ADDITION_OFFSET,
    );

    let x0y1_x1y0 =
        get_u32_vec_from_literal_24(BigUint::new(x0y1.to_vec()) + BigUint::new(x1y0.to_vec()));
    let rem = fill_reduction_trace(
        trace,
        &x0y1_x1y0,
        start_row,
        end_row,
        start_col + Z2_REDUCE_OFFSET,
    );
    fill_range_check_trace(trace, &rem, start_row, start_col + Z2_RANGECHECK_OFFSET);
}

/// Fill trace of fp2 fp multiplication. Inputs are 12*2 limbs and 12 limbs respectively. Needs 12 rows.
pub fn fill_trace_fp2_fp_mul<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    y: &[u32; 12],
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for i in start_row..end_row + 1 {
        trace[i][start_col + FP2_FP_MUL_SELECTOR_OFFSET] = F::ONE;
        assign_u32_in_series(trace, i, start_col + FP2_FP_X_INPUT_OFFSET, &x[0]);
        assign_u32_in_series(trace, i, start_col + FP2_FP_X_INPUT_OFFSET + 12, &x[1]);
        assign_u32_in_series(trace, i, start_col + FP2_FP_Y_INPUT_OFFSET, y);
    }
    trace[end_row][start_col + FP2_FP_MUL_SELECTOR_OFFSET] = F::ZERO;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[0],
        y,
        start_row,
        end_row,
        start_col + X0_Y_MULTIPLICATION_OFFSET,
    );
    let x0y = get_u32_vec_from_literal_24(BigUint::new(x[0].to_vec()) * BigUint::new(y.to_vec()));
    let rem = fill_reduction_trace(
        trace,
        &x0y,
        start_row,
        end_row,
        start_col + X0_Y_REDUCE_OFFSET,
    );
    fill_range_check_trace(trace, &rem, start_row, start_col + X0_Y_RANGECHECK_OFFSET);
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[1],
        y,
        start_row,
        end_row,
        start_col + X1_Y_MULTIPLICATION_OFFSET,
    );
    let x1y = get_u32_vec_from_literal_24(BigUint::new(x[1].to_vec()) * BigUint::new(y.to_vec()));
    let rem = fill_reduction_trace(
        trace,
        &x1y,
        start_row,
        end_row,
        start_col + X1_Y_REDUCE_OFFSET,
    );
    fill_range_check_trace(trace, &rem, start_row, start_col + X1_Y_RANGECHECK_OFFSET);
}

/// Fills trace of fp2 subtraction combined with reduction and range check. Inputs are 12*2 limbs each. Needs 1 row. Fills trace of adding field prime p to x first, and then the trace for subtraction with y.
pub fn fill_trace_subtraction_with_reduction<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    y: &[[u32; 12]; 2],
    row: usize,
    start_col: usize,
) {
    let modulus = get_u32_vec_from_literal(modulus());
    fill_trace_addition_fp2(trace, x, &[modulus, modulus], row, start_col);
    let x0_modulus =
        get_u32_vec_from_literal(BigUint::new(x[0].to_vec()) + BigUint::new(modulus.to_vec()));
    let x1_modulus =
        get_u32_vec_from_literal(BigUint::new(x[1].to_vec()) + BigUint::new(modulus.to_vec()));
    fill_trace_subtraction_fp2(
        trace,
        &[x0_modulus, x1_modulus],
        y,
        row,
        start_col + FP2_ADDITION_TOTAL,
    );
    let x0_y0 =
        get_u32_vec_from_literal(BigUint::new(x0_modulus.to_vec()) - BigUint::new(y[0].to_vec()));
    let x1_y1 =
        get_u32_vec_from_literal(BigUint::new(x1_modulus.to_vec()) - BigUint::new(y[1].to_vec()));
    let rem = fill_trace_reduce_single(
        trace,
        &x0_y0,
        row,
        start_col + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
    );
    fill_range_check_trace(
        trace,
        &rem,
        row,
        start_col + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL + FP_SINGLE_REDUCE_TOTAL,
    );
    let rem = fill_trace_reduce_single(
        trace,
        &x1_y1,
        row,
        start_col
            + FP2_ADDITION_TOTAL
            + FP2_SUBTRACTION_TOTAL
            + FP_SINGLE_REDUCE_TOTAL
            + RANGE_CHECK_TOTAL,
    );
    fill_range_check_trace(
        trace,
        &rem,
        row,
        start_col
            + FP2_ADDITION_TOTAL
            + FP2_SUBTRACTION_TOTAL
            + FP_SINGLE_REDUCE_TOTAL * 2
            + RANGE_CHECK_TOTAL,
    );
}

/// Fills trace of [multiply_by_b](super::native::Fp2::multiply_by_B) function. Input is 12*2 limbs. Needs 12 rows. Sets addition and subtraction selectors to 1 only in 11th row, becuase that's where multiplication result is set.
pub fn fill_multiply_by_b_trace<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for i in start_row..end_row + 1 {
        trace[i][start_col + MULTIPLY_B_SELECTOR_OFFSET] = F::ONE;
        assign_u32_in_series(trace, i, start_col + MULTIPLY_B_X_OFFSET, &x[0]);
        assign_u32_in_series(trace, i, start_col + MULTIPLY_B_X_OFFSET + 12, &x[1]);
    }
    trace[end_row][start_col + MULTIPLY_B_SELECTOR_OFFSET] = F::ZERO;
    let y = Fp::get_fp_from_biguint(BigUint::from(4 as u32)).0;
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[0],
        &y,
        start_row,
        end_row,
        start_col + MULTIPLY_B_X0_B_MUL_OFFSET,
    );
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x[1],
        &y,
        start_row,
        end_row,
        start_col + MULTIPLY_B_X1_B_MUL_OFFSET,
    );
    let x0y = get_u32_vec_from_literal_24(BigUint::new(x[0].to_vec()) * BigUint::new(y.to_vec()));
    let x1y = get_u32_vec_from_literal_24(BigUint::new(x[1].to_vec()) * BigUint::new(y.to_vec()));
    let modulus = modulus();
    let modulus_sq = get_u32_vec_from_literal_24(modulus.clone() * modulus.clone());
    fill_addition_trace(
        trace,
        &x0y,
        &modulus_sq,
        start_row + 11,
        start_col + MULTIPLY_B_ADD_MODSQ_OFFSET,
    );
    let x0y_add_modsq =
        get_u32_vec_from_literal_24(BigUint::new(x0y.to_vec()) + BigUint::new(modulus_sq.to_vec()));
    fill_subtraction_trace(
        trace,
        &x0y_add_modsq,
        &x1y,
        start_row + 11,
        start_col + MULTIPLY_B_SUB_OFFSET,
    );
    let x0y_x1y = get_u32_vec_from_literal_24(
        BigUint::new(x0y_add_modsq.to_vec()) - BigUint::new(x1y.to_vec()),
    );
    let rem = fill_reduction_trace(
        trace,
        &x0y_x1y,
        start_row,
        end_row,
        start_col + MULTIPLY_B_Z0_REDUCE_OFFSET,
    );
    fill_range_check_trace(
        trace,
        &rem,
        start_row,
        start_col + MULTIPLY_B_Z0_RANGECHECK_OFFSET,
    );

    fill_addition_trace(
        trace,
        &x0y,
        &x1y,
        start_row + 11,
        start_col + MULTIPLY_B_ADD_OFFSET,
    );
    let x0y_x1y =
        get_u32_vec_from_literal_24(BigUint::new(x0y.to_vec()) + BigUint::new(x1y.to_vec()));
    let rem = fill_reduction_trace(
        trace,
        &x0y_x1y,
        start_row,
        end_row,
        start_col + MULTIPLY_B_Z1_REDUCE_OFFSET,
    );
    fill_range_check_trace(
        trace,
        &rem,
        start_row,
        start_col + MULTIPLY_B_Z1_RANGECHECK_OFFSET,
    );
}

/// Fills trace of fp2 addition combined with reduction and range check. Inputs are 12*2 limbs each. Needs 1 row.
pub fn fill_trace_addition_with_reduction<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    y: &[[u32; 12]; 2],
    row: usize,
    start_col: usize,
) {
    fill_trace_addition_fp2(trace, x, y, row, start_col);
    let x0_y0 = get_u32_vec_from_literal(BigUint::new(x[0].to_vec()) + BigUint::new(y[0].to_vec()));
    let x1_y1 = get_u32_vec_from_literal(BigUint::new(x[1].to_vec()) + BigUint::new(y[1].to_vec()));
    let rem = fill_trace_reduce_single(trace, &x0_y0, row, start_col + FP2_ADDITION_TOTAL);
    fill_range_check_trace(
        trace,
        &rem,
        row,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL,
    );
    let rem = fill_trace_reduce_single(
        trace,
        &x1_y1,
        row,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL,
    );
    fill_range_check_trace(
        trace,
        &rem,
        row,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL * 2 + RANGE_CHECK_TOTAL,
    );
}

/// Fills trace of [mul_by_nonresidue](super::native::Fp2::mul_by_nonresidue) function. Input is 12*2 limbs. Needs 1 row.
pub fn fill_trace_non_residue_multiplication<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 2],
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + FP2_NON_RESIDUE_MUL_CHECK_OFFSET] = F::ONE;
    assign_u32_in_series(
        trace,
        row,
        start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET,
        &x.concat(),
    );
    fill_trace_addition_fp(
        trace,
        &x[0],
        &get_u32_vec_from_literal(modulus()),
        row,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET,
    );
    let add_modulus = get_u32_vec_from_literal(BigUint::new(x[0].to_vec()) + modulus());
    fill_trace_subtraction_fp(
        trace,
        &add_modulus,
        &x[1],
        row,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_TOTAL,
    );
    let c0_c1_sub =
        get_u32_vec_from_literal(BigUint::new(add_modulus.to_vec()) - BigUint::new(x[1].to_vec()));
    let rem = fill_trace_reduce_single(
        trace,
        &c0_c1_sub,
        row,
        start_col + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET,
    );
    fill_range_check_trace(
        trace,
        &rem,
        row,
        start_col + FP2_NON_RESIDUE_MUL_Z0_RANGECHECK_OFFSET,
    );
    fill_trace_addition_fp(
        trace,
        &x[0],
        &x[1],
        row,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET,
    );
    let c0_c1_add =
        get_u32_vec_from_literal(BigUint::new(x[0].to_vec()) + BigUint::new(x[1].to_vec()));
    let rem = fill_trace_reduce_single(
        trace,
        &c0_c1_add,
        row,
        start_col + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET,
    );
    fill_range_check_trace(
        trace,
        &rem,
        row,
        start_col + FP2_NON_RESIDUE_MUL_Z1_RANGECHECK_OFFSET,
    );
}

/// Fills trace of [fp4_sqaure](super::native::fp4_square) function. Inputs are 12*2 limbs each. Needs 12 rows.
pub fn fill_trace_fp4_sq<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp2,
    y: &Fp2,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + FP4_SQ_INPUT_X_OFFSET,
            &x.get_u32_slice().concat(),
        );
        assign_u32_in_series(
            trace,
            row,
            start_col + FP4_SQ_INPUT_Y_OFFSET,
            &y.get_u32_slice().concat(),
        );
        trace[row][start_col + FP4_SQ_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + FP4_SQ_SELECTOR_OFFSET] = F::ZERO;

    let t0 = (*x) * (*x);
    generate_trace_fp2_mul(
        trace,
        x.get_u32_slice(),
        x.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP4_SQ_T0_CALC_OFFSET,
    );

    let t1 = (*y) * (*y);
    generate_trace_fp2_mul(
        trace,
        y.get_u32_slice(),
        y.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP4_SQ_T1_CALC_OFFSET,
    );

    let t2 = t1.mul_by_nonresidue();
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication(
            trace,
            &t1.get_u32_slice(),
            row,
            start_col + FP4_SQ_T2_CALC_OFFSET,
        );
    }

    let _x = t2 + t0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t2.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + FP4_SQ_X_CALC_OFFSET,
        );
    }

    let t3 = (*x) + (*y);
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &x.get_u32_slice(),
            &y.get_u32_slice(),
            row,
            start_col + FP4_SQ_T3_CALC_OFFSET,
        );
    }

    let t4 = t3 * t3;
    generate_trace_fp2_mul(
        trace,
        t3.get_u32_slice(),
        t3.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP4_SQ_T4_CALC_OFFSET,
    );

    let t5 = t4 - t0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t4.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + FP4_SQ_T5_CALC_OFFSET,
        );
    }

    let _y = t5 - t1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t5.get_u32_slice(),
            &t1.get_u32_slice(),
            row,
            start_col + FP4_SQ_Y_CALC_OFFSET,
        );
    }
}

/// Fills trace of [forbenius_map](super::native::Fp2::forbenius_map) function. Input is 12*2 limbs and usize. Needs 12 rows.
pub fn fill_trace_fp2_forbenius_map<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp2,
    pow: usize,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    let div = pow / 2;
    let rem = pow % 2;
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET,
            &x.get_u32_slice().concat(),
        );
        trace[row][start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET] = F::ONE;
        trace[row][start_col + FP2_FORBENIUS_MAP_POW_OFFSET] = F::from_canonical_usize(pow);
        trace[row][start_col + FP2_FORBENIUS_MAP_DIV_OFFSET] = F::from_canonical_usize(div);
        trace[row][start_col + FP2_FORBENIUS_MAP_REM_OFFSET] = F::from_canonical_usize(rem);
    }
    trace[end_row][start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET] = F::ZERO;
    let forbenius_coefficients = Fp2::forbenius_coefficients();
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &x.0[1].0,
        &forbenius_coefficients[rem].0,
        start_row,
        end_row,
        start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET,
    );
    trace[start_row + 11][start_col + FP2_FORBENIUS_MAP_MUL_RES_ROW] = F::ONE;
    let x_y =
        get_u32_vec_from_literal_24(x.0[1].to_biguint() * forbenius_coefficients[rem].to_biguint());
    let res = fill_reduction_trace(
        trace,
        &x_y,
        start_row,
        end_row,
        start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS,
    );
    for row in start_row..end_row + 1 {
        fill_range_check_trace(
            trace,
            &res,
            row,
            start_col
                + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCTION_TOTAL,
        );
    }
    let res = Fp2([x.0[0], Fp(res)]);
    assert_eq!(res, x.forbenius_map(pow));
}

/// Constraints fp2 addition. In essence, constraints two Fp addititons.
pub fn add_addition_fp2_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_0_OFFSET,
        bit_selector,
    );
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_1_OFFSET,
        bit_selector,
    );
}

pub fn add_addition_fp2_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_0_OFFSET,
        bit_selector,
    );
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_1_OFFSET,
        bit_selector,
    );
}

/// Constraints fp2 subtraction. In essence, constraints two Fp subtractions.
pub fn add_subtraction_fp2_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    add_subtraction_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_SUBTRACTION_0_OFFSET,
        bit_selector,
    );
    add_subtraction_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_SUBTRACTION_1_OFFSET,
        bit_selector,
    );
}

pub fn add_subtraction_fp2_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    add_subtraction_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_SUBTRACTION_0_OFFSET,
        bit_selector,
    );
    add_subtraction_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_SUBTRACTION_1_OFFSET,
        bit_selector,
    );
}

/// Constraints fp2 multiply by single. In essence, constraints two Fp multiply by single.
pub fn add_fp2_single_multiply_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    add_fp_single_multiply_constraints(
        local_values,
        yield_constr,
        start_col + FP2_MULTIPLY_SINGLE_0_OFFSET,
        bit_selector,
    );
    add_fp_single_multiply_constraints(
        local_values,
        yield_constr,
        start_col + FP2_MULTIPLY_SINGLE_1_OFFSET,
        bit_selector,
    );
}

pub fn add_fp2_single_multiply_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    add_fp_single_multiply_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_MULTIPLY_SINGLE_0_OFFSET,
        bit_selector,
    );
    add_fp_single_multiply_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_MULTIPLY_SINGLE_1_OFFSET,
        bit_selector,
    );
}

/// Constraints fp2 negation. First add constraints for fp2 addition. Followed by constraining the result of the addition with bls12-381 field prime p.
pub fn add_negate_fp2_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    add_addition_fp2_constraints(local_values, yield_constr, start_col, bit_selector);
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    let mod_u32 = get_u32_vec_from_literal(modulus());
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_SUM_OFFSET + i]
                    - FE::from_canonical_u32(mod_u32[i])),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_SUM_OFFSET + i]
                    - FE::from_canonical_u32(mod_u32[i])),
        );
    }
}

pub fn add_negate_fp2_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    add_addition_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );
    let mod_u32 = get_u32_vec_from_literal(modulus());
    for i in 0..12 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(mod_u32[i]));

        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_SUM_OFFSET + i],
            lc,
        );

        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let mul_tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_SUM_OFFSET + i],
            lc,
        );

        let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
        yield_constr.constraint(builder, c2);
    }
}

/// Constraints fp2 multiplication.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints x0\*y0, x1\*y1, x0\*y1, x1\*y0 multiplication operations. Then constraints the x0\*y0 + p^2 operation, followed by x0\*y0 + p^2 - x1\*y1 operation. Constraints the reduction of result of the previous subtraction, followed by a range check operation. Constraints x0\*y1 + x1\*y0. Constraints the reduction of result of the previous addition, followed by a range check operation.
pub fn add_fp2_mul_constraints<
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
    // for i in 0..12 {
    //     yield_constr.constraint_transition(local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i])
    // }
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i]
                    - next_values[start_col + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i]
                    - next_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
    }

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_0_Y_1_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_1_Y_0_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_1_Y_1_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i + 12]),
        );
    }

    // constrain X_0*Y_0
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X_0_Y_0_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    // constrain X_1*Y_1
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X_1_Y_1_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    // constrain X0*Y0 with X0*Y0 + modulus^2
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_X_OFFSET + i]
                    - local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + SUM_OFFSET + i]),
        );
    }

    // constrain modulus^2 with X0*Y0 + modulus^2
    let modulus = modulus();
    let modulus_sq_u32 = get_u32_vec_from_literal_24(modulus.clone() * modulus);
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(modulus_sq_u32[i])),
        );
    }

    // constrain X0*Y0 + modulus^2
    add_addition_constraints(
        local_values,
        yield_constr,
        start_col + Z1_ADD_MODULUS_OFFSET,
        bit_selector,
    );

    // constrain X0*Y0 + modulus^2 with X0*Y0 + modulus^2 - X1Y1
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_X_OFFSET + i]
                    - local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_SUM_OFFSET + i]),
        );
    }

    // constrain X1*Y1 + modulus^2 with X0*Y0 + modulus^2 - X1Y1
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_Y_OFFSET + i]
                    - local_values[start_col + X_1_Y_1_MULTIPLICATION_OFFSET + SUM_OFFSET + i]),
        );
    }

    // constrain X0*Y0 + modulus^2 - X1Y1
    add_subtraction_constraints(
        local_values,
        yield_constr,
        start_col + Z1_SUBTRACTION_OFFSET,
        bit_selector,
    );

    // constrain X0*Y0 + modulus^2 - X1Y1 with reduction
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_DIFF_OFFSET + i]
                    - local_values[start_col + Z1_REDUCE_OFFSET + REDUCE_X_OFFSET + i]),
        );
    }

    // constrain reduction
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Z1_REDUCE_OFFSET,
        start_col + FP2_FP2_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + Z1_RANGECHECK_OFFSET,
        bit_selector,
    );

    // constrain X_1*Y_0
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X_0_Y_1_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    // constrain X_1*Y_0
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X_1_Y_0_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    // constrain X0*Y1 with X0*Y1 + X1*Y0
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_X_OFFSET + i]
                    - local_values[start_col + X_0_Y_1_MULTIPLICATION_OFFSET + SUM_OFFSET + i]),
        );
    }

    // constrain X1*Y0 with X0*Y1 + X1*Y0
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_Y_OFFSET + i]
                    - local_values[start_col + X_1_Y_0_MULTIPLICATION_OFFSET + SUM_OFFSET + i]),
        );
    }

    // constrain X0*Y1 + X1*Y0
    add_addition_constraints(
        local_values,
        yield_constr,
        start_col + Z2_ADDITION_OFFSET,
        bit_selector,
    );

    // constrain X0*Y1 + X1*Y0 with reduction
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_SUM_OFFSET + i]
                    - local_values[start_col + Z2_REDUCE_OFFSET + REDUCE_X_OFFSET + i]),
        );
    }

    // constrain reduction
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Z2_REDUCE_OFFSET,
        start_col + FP2_FP2_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + Z2_RANGECHECK_OFFSET,
        bit_selector,
    );
}

pub fn add_fp2_mul_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    // let constant = builder.constant_extension(F::Extension::from_canonical_u64(1<<32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_FP2_SELECTOR_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i],
            next_values[start_col + FP2_FP2_X_INPUT_OFFSET + i],
        );

        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c1);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i],
            next_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i],
        );

        let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
        yield_constr.constraint_transition(builder, c2);
    }

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_FP2_SELECTOR_OFFSET],
        );

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
        yield_constr.constraint(builder, c2);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col + X_0_Y_1_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c3 = builder.mul_extension(mul_tmp1, sub_tmp3);
        yield_constr.constraint(builder, c3);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i + 12],
        );
        let c4 = builder.mul_extension(mul_tmp1, sub_tmp4);
        yield_constr.constraint(builder, c4);

        let sub_tmp5 = builder.sub_extension(
            local_values[start_col + X_1_Y_0_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i + 12],
        );
        let c5 = builder.mul_extension(mul_tmp1, sub_tmp5);
        yield_constr.constraint(builder, c5);

        let sub_tmp6 = builder.sub_extension(
            local_values[start_col + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c6 = builder.mul_extension(mul_tmp1, sub_tmp6);
        yield_constr.constraint(builder, c6);

        let sub_tmp7 = builder.sub_extension(
            local_values[start_col + X_1_Y_1_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_X_INPUT_OFFSET + i + 12],
        );
        let c7 = builder.mul_extension(mul_tmp1, sub_tmp7);
        yield_constr.constraint(builder, c7);

        let sub_tmp8 = builder.sub_extension(
            local_values[start_col + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            local_values[start_col + FP2_FP2_Y_INPUT_OFFSET + i + 12],
        );
        let c8 = builder.mul_extension(mul_tmp1, sub_tmp8);
        yield_constr.constraint(builder, c8);
    }

    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X_0_Y_0_MULTIPLICATION_OFFSET,
        bit_selector,
    );
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X_1_Y_1_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_X_OFFSET + i],
            local_values[start_col + X_0_Y_0_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    let modulus = modulus();
    let modulus_sq_u32 = get_u32_vec_from_literal_24(modulus.clone() * modulus);
    for i in 0..24 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus_sq_u32[i]));

        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_Y_OFFSET + i],
            lc,
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_addition_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Z1_ADD_MODULUS_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_X_OFFSET + i],
            local_values[start_col + Z1_ADD_MODULUS_OFFSET + ADDITION_SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_Y_OFFSET + i],
            local_values[start_col + X_1_Y_1_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_subtraction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Z1_SUBTRACTION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z1_SUBTRACTION_OFFSET + SUBTRACTION_DIFF_OFFSET + i],
            local_values[start_col + Z1_REDUCE_OFFSET + REDUCE_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Z1_REDUCE_OFFSET,
        start_col + FP2_FP2_SELECTOR_OFFSET,
        bit_selector,
    );

    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Z1_RANGECHECK_OFFSET,
        bit_selector,
    );

    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X_0_Y_1_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X_1_Y_0_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_X_OFFSET + i],
            local_values[start_col + X_0_Y_1_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_Y_OFFSET + i],
            local_values[start_col + X_1_Y_0_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_addition_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Z2_ADDITION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + Z2_ADDITION_OFFSET + ADDITION_SUM_OFFSET + i],
            local_values[start_col + Z2_REDUCE_OFFSET + REDUCE_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Z2_REDUCE_OFFSET,
        start_col + FP2_FP2_SELECTOR_OFFSET,
        bit_selector,
    );

    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + Z2_RANGECHECK_OFFSET,
        bit_selector,
    );
}

/// Constraints fp2 fp multiplication.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints x0\*y, x1\*y multiplication operations. Constraints the reduction of result of the previous multiplications, followed by a range check operations.
pub fn add_fp2_fp_mul_constraints<
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

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col + FP2_FP_X_INPUT_OFFSET + j * 12 + i]
                        - next_values[start_col + FP2_FP_X_INPUT_OFFSET + j * 12 + i]),
            );
        }
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP_Y_INPUT_OFFSET + i]
                    - next_values[start_col + FP2_FP_Y_INPUT_OFFSET + i]),
        );
    }
    // constrain inputs to multiplication
    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP_X_INPUT_OFFSET + i]
                    - local_values[start_col + X0_Y_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP_X_INPUT_OFFSET + 12 + i]
                    - local_values[start_col + X1_Y_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP_Y_INPUT_OFFSET + i]
                    - local_values[start_col + X0_Y_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FP_Y_INPUT_OFFSET + i]
                    - local_values[start_col + X1_Y_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X0_Y_MULTIPLICATION_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + X0_Y_REDUCE_OFFSET
                    + REDUCTION_ADDITION_OFFSET
                    + ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + X0_Y_REDUCE_OFFSET
                    + REDUCTION_ADDITION_OFFSET
                    + ADDITION_SUM_OFFSET
                    + i]
                    - local_values[start_col + X0_Y_MULTIPLICATION_OFFSET + SUM_OFFSET + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X0_Y_REDUCE_OFFSET,
        start_col + FP2_FP_MUL_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + X0_Y_RANGECHECK_OFFSET,
        bit_selector,
    );
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X1_Y_MULTIPLICATION_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + X1_Y_REDUCE_OFFSET
                    + REDUCTION_ADDITION_OFFSET
                    + ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + X1_Y_REDUCE_OFFSET
                    + REDUCTION_ADDITION_OFFSET
                    + ADDITION_SUM_OFFSET
                    + i]
                    - local_values[start_col + X1_Y_MULTIPLICATION_OFFSET + SUM_OFFSET + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + X1_Y_REDUCE_OFFSET,
        start_col + FP2_FP_MUL_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + X1_Y_RANGECHECK_OFFSET,
        bit_selector,
    );
}

pub fn add_fp2_fp_mul_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        for j in 0..2 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col + FP2_FP_X_INPUT_OFFSET + j * 12 + i],
                next_values[start_col + FP2_FP_X_INPUT_OFFSET + j * 12 + i],
            );

            let c = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint_transition(builder, c);
        }
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_FP_Y_INPUT_OFFSET + i],
            next_values[start_col + FP2_FP_Y_INPUT_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_FP_MUL_SELECTOR_OFFSET],
        );

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_FP_X_INPUT_OFFSET + i],
            local_values[start_col + X0_Y_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c1);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP2_FP_X_INPUT_OFFSET + 12 + i],
            local_values[start_col + X1_Y_MULTIPLICATION_OFFSET + X_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
        yield_constr.constraint_transition(builder, c2);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col + FP2_FP_Y_INPUT_OFFSET + i],
            local_values[start_col + X0_Y_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
        );
        let c3 = builder.mul_extension(mul_tmp1, sub_tmp3);
        yield_constr.constraint_transition(builder, c3);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col + FP2_FP_Y_INPUT_OFFSET + i],
            local_values[start_col + X1_Y_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
        );
        let c4 = builder.mul_extension(mul_tmp1, sub_tmp4);
        yield_constr.constraint_transition(builder, c4);
    }

    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X0_Y_MULTIPLICATION_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + X0_Y_REDUCE_OFFSET
                + REDUCTION_ADDITION_OFFSET
                + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + X0_Y_REDUCE_OFFSET
                + REDUCTION_ADDITION_OFFSET
                + ADDITION_SUM_OFFSET
                + i],
            local_values[start_col + X0_Y_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c);
    }

    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X0_Y_REDUCE_OFFSET,
        start_col + FP2_FP_MUL_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X0_Y_RANGECHECK_OFFSET,
        bit_selector,
    );
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X1_Y_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + X1_Y_REDUCE_OFFSET
                + REDUCTION_ADDITION_OFFSET
                + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + X1_Y_REDUCE_OFFSET
                + REDUCTION_ADDITION_OFFSET
                + ADDITION_SUM_OFFSET
                + i],
            local_values[start_col + X1_Y_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c);
    }

    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + X1_Y_REDUCE_OFFSET,
        start_col + FP2_FP_MUL_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + X1_Y_RANGECHECK_OFFSET,
        bit_selector,
    );
}

/// Constraints for [multiply_by_b](super::native::Fp2::multiply_by_B) function.
///
///  Constraints inputs across this and next row, wherever selector is set to on. Constraints x0\*4, x1\*4 multiplications. Constraints y input of the multiplications to 4. Constraints respective addition and subtraction operations followed by reduction and range check constraints.
pub fn add_multiply_by_b_constraints<
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

    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                * (local_values[start_col + MULTIPLY_B_X_OFFSET + i]
                    - next_values[start_col + MULTIPLY_B_X_OFFSET + i]),
        );
    }
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                * (local_values[start_col + MULTIPLY_B_X_OFFSET + i]
                    - local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                * (local_values[start_col + MULTIPLY_B_X_OFFSET + 12 + i]
                    - local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + X_INPUT_OFFSET + i]),
        );
        if i == 0 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + Y_INPUT_OFFSET + i]
                        - FE::from_canonical_u32(4)),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + Y_INPUT_OFFSET + i]
                        - FE::from_canonical_u32(4)),
            );
        } else {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                    * local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET]
                    * local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + Y_INPUT_OFFSET + i],
            );
        }
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_B_X0_B_MUL_OFFSET,
        bit_selector,
    );
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_B_X1_B_MUL_OFFSET,
        bit_selector,
    );
    let modulus = modulus();
    let modulus_sq_u32 = get_u32_vec_from_literal_24(modulus.clone() * modulus);
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_X_OFFSET + i]
                    - local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(modulus_sq_u32[i])),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_X_OFFSET + i]
                    - local_values
                        [start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_Y_OFFSET + i]
                    - local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_X_OFFSET + i]
                    - local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_Y_OFFSET + i]
                    - local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + SUM_OFFSET + i]),
        );
    }
    add_addition_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_B_ADD_MODSQ_OFFSET,
        bit_selector,
    );
    add_subtraction_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_B_SUB_OFFSET,
        bit_selector,
    );
    add_addition_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_B_ADD_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCE_X_OFFSET + i]
                    - local_values
                        [start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_DIFF_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCE_X_OFFSET + i]
                    - local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_SUM_OFFSET + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_B_Z0_REDUCE_OFFSET,
        start_col + MULTIPLY_B_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_B_Z0_RANGECHECK_OFFSET,
        bit_selector,
    );
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_B_Z1_REDUCE_OFFSET,
        start_col + MULTIPLY_B_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_B_Z1_RANGECHECK_OFFSET,
        bit_selector,
    );
}

pub fn add_multiply_by_b_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u32(4));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_X_OFFSET + i],
            next_values[start_col + MULTIPLY_B_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET],
        );

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_X_OFFSET + i],
            local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_X_OFFSET + 12 + i],
            local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + X_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
        yield_constr.constraint(builder, c2);

        if i == 0 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + Y_INPUT_OFFSET + i],
                constant,
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + Y_INPUT_OFFSET + i],
                constant,
            );
            let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint(builder, c2);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + MULTIPLY_B_SELECTOR_OFFSET],
            );

            let c1 = builder.mul_extension(
                mul_tmp1,
                local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint(builder, c1);

            let c2 = builder.mul_extension(
                mul_tmp1,
                local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint(builder, c2);
        }
    }

    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_B_X0_B_MUL_OFFSET,
        bit_selector,
    );
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_B_X1_B_MUL_OFFSET,
        bit_selector,
    );
    let modulus = modulus();
    let modulus_sq_u32 = get_u32_vec_from_literal_24(modulus.clone() * modulus);
    for i in 0..24 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus_sq_u32[i]));

        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_CHECK_OFFSET],
        );

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_X_OFFSET + i],
            local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + SUM_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_Y_OFFSET + i],
            lc,
        );
        let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
        yield_constr.constraint(builder, c2);

        let mul_tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_CHECK_OFFSET],
        );

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_X_OFFSET + i],
            local_values[start_col + MULTIPLY_B_ADD_MODSQ_OFFSET + ADDITION_SUM_OFFSET + i],
        );
        let c3 = builder.mul_extension(mul_tmp2, sub_tmp3);
        yield_constr.constraint(builder, c3);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_Y_OFFSET + i],
            local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + SUM_OFFSET + i],
        );
        let c4 = builder.mul_extension(mul_tmp2, sub_tmp4);
        yield_constr.constraint(builder, c4);

        let mul_tmp3 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_CHECK_OFFSET],
        );

        let sub_tmp5 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_X_OFFSET + i],
            local_values[start_col + MULTIPLY_B_X0_B_MUL_OFFSET + SUM_OFFSET + i],
        );
        let c5 = builder.mul_extension(mul_tmp3, sub_tmp5);
        yield_constr.constraint(builder, c5);

        let sub_tmp6 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_Y_OFFSET + i],
            local_values[start_col + MULTIPLY_B_X1_B_MUL_OFFSET + SUM_OFFSET + i],
        );
        let c6 = builder.mul_extension(mul_tmp3, sub_tmp6);
        yield_constr.constraint(builder, c6);
    }

    add_addition_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_B_ADD_MODSQ_OFFSET,
        bit_selector,
    );
    add_subtraction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_B_SUB_OFFSET,
        bit_selector,
    );
    add_addition_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_B_ADD_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCE_X_OFFSET + i],
            local_values[start_col + MULTIPLY_B_SUB_OFFSET + SUBTRACTION_DIFF_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let mul_tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCE_X_OFFSET + i],
            local_values[start_col + MULTIPLY_B_ADD_OFFSET + ADDITION_SUM_OFFSET + i],
        );
        let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
        yield_constr.constraint(builder, c2);
    }

    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_B_Z0_REDUCE_OFFSET,
        start_col + MULTIPLY_B_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_B_Z0_RANGECHECK_OFFSET,
        bit_selector,
    );
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_B_Z1_REDUCE_OFFSET,
        start_col + MULTIPLY_B_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_B_Z1_RANGECHECK_OFFSET,
        bit_selector,
    )
}

/// Constraints fp2 subtraction followed by reduction and range check constraints. First, constraints of adding field prime p to x to prevent overflow, because x > y assumption is not valid here. Then constraints the subtraction operation. Then reduce and range check constraints.
pub fn add_subtraction_with_reduction_constranints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    let modulus = get_u32_vec_from_literal(modulus());
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(modulus[i])),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(modulus[i])),
        );
    }
    add_addition_fp2_constraints(local_values, yield_constr, start_col, bit_selector);
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_X_OFFSET
                    + i]
                    - local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_SUM_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_X_OFFSET
                    + i]
                    - local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_SUM_OFFSET + i]),
        );
    }
    add_subtraction_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_DIFF_OFFSET
                    + i]
                    - local_values[start_col
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_fp_reduce_single_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL + FP_SINGLE_REDUCE_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_DIFF_OFFSET
                    + i]
                    - local_values[start_col
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_fp_reduce_single_constraints(
        local_values,
        yield_constr,
        start_col
            + FP2_ADDITION_TOTAL
            + FP2_SUBTRACTION_TOTAL
            + FP_SINGLE_REDUCE_TOTAL
            + RANGE_CHECK_TOTAL,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col
            + FP2_ADDITION_TOTAL
            + FP2_SUBTRACTION_TOTAL
            + FP_SINGLE_REDUCE_TOTAL * 2
            + RANGE_CHECK_TOTAL,
        bit_selector,
    );
}

pub fn add_subtraction_with_reduction_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let modulus = get_u32_vec_from_literal(modulus());
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus[i]));

        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i],
            lc,
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let mul_tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i],
            lc,
        );
        let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
        yield_constr.constraint(builder, c2);
    }
    add_addition_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_X_OFFSET
                + i],
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_SUM_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let mul_tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_X_OFFSET
                + i],
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_SUM_OFFSET + i],
        );
        let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
        yield_constr.constraint(builder, c2);
    }
    add_subtraction_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_DIFF_OFFSET
                + i],
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_X_OFFSET
                + i],
        );
        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c);
    }
    add_fp_reduce_single_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL + FP_SINGLE_REDUCE_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_DIFF_OFFSET
                + i],
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCE_X_OFFSET
                + i],
        );
        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c);
    }

    add_fp_reduce_single_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col
            + FP2_ADDITION_TOTAL
            + FP2_SUBTRACTION_TOTAL
            + FP_SINGLE_REDUCE_TOTAL
            + RANGE_CHECK_TOTAL,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col
            + FP2_ADDITION_TOTAL
            + FP2_SUBTRACTION_TOTAL
            + FP_SINGLE_REDUCE_TOTAL * 2
            + RANGE_CHECK_TOTAL,
        bit_selector,
    );
}

/// Constraints fp2 addition followed by reduction and range check constraints.
pub fn add_addition_with_reduction_constranints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize, // Starting column of your multiplication trace
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    add_addition_fp2_constraints(local_values, yield_constr, start_col, bit_selector);
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_SUM_OFFSET + i]
                    - local_values[start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_X_OFFSET + i]),
        );
    }
    add_fp_reduce_single_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_SUM_OFFSET + i]
                    - local_values[start_col
                        + FP2_ADDITION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_fp_reduce_single_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL * 2 + RANGE_CHECK_TOTAL,
        bit_selector,
    );
}

pub fn add_addition_with_reduction_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    add_addition_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );
    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_ADDITION_0_OFFSET + FP_ADDITION_SUM_OFFSET + i],
            local_values[start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c);
    }
    add_fp_reduce_single_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP2_ADDITION_1_OFFSET + FP_ADDITION_SUM_OFFSET + i],
            local_values[start_col
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCE_X_OFFSET
                + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c);
    }
    add_fp_reduce_single_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCE_TOTAL * 2 + RANGE_CHECK_TOTAL,
        bit_selector,
    );
}

/// Constraints [mul_by_nonresidue](super::native::Fp2::mul_by_nonresidue) function.
///
/// For the real part, constraints addition with field prime first, and then constraints subtraction, followed by reduction and range check constraints. For imaginary part, constraints addition, followed by reduction and range check constraints.
pub fn add_non_residue_multiplication_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize,
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    let modulus = get_u32_vec_from_literal(modulus());
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_Y_OFFSET + i]
                    - FE::from_canonical_u32(modulus[i])),
        );
    }
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_SUBTRACTION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                        + FP_ADDITION_SUM_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i + 12]),
        );
    }
    add_subtraction_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_SUBTRACTION_DIFF_OFFSET
                    + i]
                    - local_values[start_col
                        + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                        + FP_SINGLE_REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_fp_reduce_single_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_Z0_RANGECHECK_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_X_OFFSET + i]
                    - local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values
                    [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_Y_OFFSET + i]
                    - local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i + 12]),
        );
    }
    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET,
        bit_selector,
    );
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET
                    + FP_ADDITION_TOTAL
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET
                    + FP_ADDITION_SUM_OFFSET
                    + i]
                    - local_values[start_col
                        + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                        + FP_SINGLE_REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_fp_reduce_single_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col + FP2_NON_RESIDUE_MUL_Z1_RANGECHECK_OFFSET,
        bit_selector,
    );
}

pub fn add_non_residue_multiplication_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let modulus = get_u32_vec_from_literal(modulus());
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus[i]));

        let mul_tmp = local_values
            [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_Y_OFFSET + i],
            lc,
        );
        let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET,
        bit_selector,
    );
    for i in 0..12 {
        let mul_tmp = local_values[start_col
            + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
            + FP_ADDITION_TOTAL
            + FP_SUBTRACTION_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                + FP_ADDITION_TOTAL
                + FP_SUBTRACTION_X_OFFSET
                + i],
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_SUM_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                + FP_ADDITION_TOTAL
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET + FP_ADDITION_TOTAL,
        bit_selector,
    );
    for i in 0..12 {
        let sub_tmp = builder.sub_extension(
            local_values[start_col
                + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                + FP_ADDITION_TOTAL
                + FP_SUBTRACTION_DIFF_OFFSET
                + i],
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET + FP_SINGLE_REDUCE_X_OFFSET + i],
        );
        let c = builder.mul_extension(
            local_values[start_col
                + FP2_NON_RESIDUE_MUL_C0_C1_SUB_OFFSET
                + FP_ADDITION_TOTAL
                + FP_SUBTRACTION_CHECK_OFFSET],
            sub_tmp,
        );
        let c = builder.mul_extension(bit_selector_val, c);
        yield_constr.constraint(builder, c);
    }
    add_fp_reduce_single_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_Z0_RANGECHECK_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let mul_tmp = local_values
            [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_X_OFFSET + i],
            local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_Y_OFFSET + i],
            local_values[start_col + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET,
        bit_selector,
    );
    for i in 0..12 {
        let sub_tmp = builder.sub_extension(
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET + FP_ADDITION_SUM_OFFSET + i],
            local_values
                [start_col + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET + FP_SINGLE_REDUCE_X_OFFSET + i],
        );
        let c = builder.mul_extension(
            local_values[start_col
                + FP2_NON_RESIDUE_MUL_C0_C1_ADD_OFFSET
                + FP_ADDITION_TOTAL
                + FP_ADDITION_CHECK_OFFSET],
            sub_tmp,
        );
        let c = builder.mul_extension(bit_selector_val, c);
        yield_constr.constraint(builder, c);
    }
    add_fp_reduce_single_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP2_NON_RESIDUE_MUL_Z1_RANGECHECK_OFFSET,
        bit_selector,
    );
}

/// Constraints for [fp4_square](super::native::fp4_square) function.
///
///  Constraints inputs across this and next row, wherever selector is set to on. Constraints the respective multiplication, addition and subtraction operations.
pub fn add_fp4_sq_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize,
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i]
                    - next_values[start_col + FP4_SQ_INPUT_X_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i]
                    - next_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i]),
        );
    }

    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP4_SQ_T0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP4_SQ_T1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values
                    [start_col + FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP4_SQ_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T2_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + 12
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + FP4_SQ_T2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T2_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T2_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP4_SQ_X_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP4_SQ_INPUT_X_OFFSET + 12 + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + 12 + i]),
        );
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP4_SQ_T3_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP4_SQ_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                    - local_values[start_col
                        + FP4_SQ_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP4_SQ_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                    - local_values[start_col
                        + FP4_SQ_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP4_SQ_T4_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T4_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T4_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_T5_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP4_SQ_T5_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T5_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T5_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP4_SQ_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP4_SQ_Y_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_fp4_sq_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP4_SQ_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i],
            next_values[start_col + FP4_SQ_INPUT_X_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i],
            next_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP4_SQ_T0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP4_SQ_T1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values
                [start_col + FP4_SQ_T2_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + 12 + i],
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP4_SQ_T2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP4_SQ_T2_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp1, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp1, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP4_SQ_T2_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp2, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp2, c);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP4_SQ_X_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP4_SQ_INPUT_X_OFFSET + i],
        );
        let c = builder.mul_extension(tmp1, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + i],
        );
        let c = builder.mul_extension(tmp1, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP4_SQ_INPUT_X_OFFSET + 12 + i],
        );
        let c = builder.mul_extension(tmp2, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_INPUT_Y_OFFSET + 12 + i],
        );
        let c = builder.mul_extension(tmp2, c);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP4_SQ_T3_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
            local_values[start_col
                + FP4_SQ_T3_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP4_SQ_T4_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let tmp3 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let tmp4 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp1, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp2, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T4_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp3, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp4, c);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP4_SQ_T5_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let tmp2 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let tmp3 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let tmp4 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp1, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp2, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP4_SQ_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp3, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values[start_col + FP4_SQ_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c = builder.mul_extension(tmp4, c);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP4_SQ_Y_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints for [forbenius_map](super::native::Fp2::forbenius_map) function.
///
///  Constraints both input and power across this and next row, wherever selector is set to on. Constraint the divisor and remainder with power for `power == divisor*2 + remainder`. Selects the forbenius constant using mupliplexer logic -> `y = (1-bit)*constant[0] + bit*constant[1]`. Then constraints multiplication, reduction and range check operations.
pub fn add_fp2_forbenius_map_constraints<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
>(
    local_values: &[P],
    next_values: &[P],
    yield_constr: &mut ConstraintConsumer<P>,
    start_col: usize,
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET + i]
                    - next_values[start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET + i]),
        );
    }
    yield_constr.constraint_transition(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values[start_col + FP2_FORBENIUS_MAP_POW_OFFSET]
                - next_values[start_col + FP2_FORBENIUS_MAP_POW_OFFSET]),
    );
    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values[start_col + FP2_FORBENIUS_MAP_DIV_OFFSET] * FE::TWO
                + local_values[start_col + FP2_FORBENIUS_MAP_REM_OFFSET]
                - local_values[start_col + FP2_FORBENIUS_MAP_POW_OFFSET]),
    );
    let bit = local_values[start_col + FP2_FORBENIUS_MAP_REM_OFFSET];
    let forbenius_coefficients = Fp2::forbenius_coefficients()
        .iter()
        .map(|fp| fp.0)
        .collect::<Vec<[u32; 12]>>();
    let y = (0..12)
        .map(|i| {
            (P::ONES - bit) * FE::from_canonical_u32(forbenius_coefficients[0][i])
                + bit * FE::from_canonical_u32(forbenius_coefficients[1][i])
        })
        .collect::<Vec<P>>();
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + X_INPUT_OFFSET + i]
                    - local_values[start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET + 12 + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + Y_INPUT_OFFSET + i]
                    - y[i]),
        );
    }
    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP2_FORBENIUS_MAP_MUL_RES_ROW]
                * (local_values[start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + SUM_OFFSET + i]
                    - local_values[start_col
                        + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCE_X_OFFSET
                        + i]),
        );
    }
    add_reduce_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints(
        local_values,
        yield_constr,
        start_col
            + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
            + FP_MULTIPLICATION_TOTAL_COLUMNS
            + REDUCTION_TOTAL,
        bit_selector,
    );
}

pub fn add_fp2_forbenius_map_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));
    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values[start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET],
    );

    for i in 0..24 {
        let c = builder.sub_extension(
            local_values[start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
            next_values[start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);
    }

    let c = builder.sub_extension(
        local_values[start_col + FP2_FORBENIUS_MAP_POW_OFFSET],
        next_values[start_col + FP2_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint_transition(builder, c);

    let two = builder.constant_extension(F::Extension::TWO);
    let c = builder.mul_extension(local_values[start_col + FP2_FORBENIUS_MAP_DIV_OFFSET], two);
    let c = builder.add_extension(c, local_values[start_col + FP2_FORBENIUS_MAP_REM_OFFSET]);
    let c = builder.sub_extension(c, local_values[start_col + FP2_FORBENIUS_MAP_POW_OFFSET]);
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    let bit = local_values[start_col + FP2_FORBENIUS_MAP_REM_OFFSET];
    let one = builder.constant_extension(F::Extension::ONE);
    let forbenius_coefficients = Fp2::forbenius_coefficients()
        .iter()
        .map(|fp| fp.0)
        .collect::<Vec<[u32; 12]>>();
    let y = (0..12)
        .map(|i| {
            let sub = builder.sub_extension(one, bit);
            let const1 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[0][i],
            ));
            let mul1 = builder.mul_extension(sub, const1);

            let const2 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[1][i],
            ));
            let mul2 = builder.mul_extension(bit, const2);

            let c = builder.add_extension(mul1, mul2);
            c
        })
        .collect::<Vec<ExtensionTarget<D>>>();
    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values
                [start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + MULTIPLICATION_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + X_INPUT_OFFSET + i],
            local_values[start_col + FP2_FORBENIUS_MAP_INPUT_OFFSET + 12 + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + Y_INPUT_OFFSET + i],
            y[i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP2_FORBENIUS_MAP_MUL_RES_ROW],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + SUM_OFFSET + i],
            local_values[start_col
                + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCE_X_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_reduce_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP2_FORBENIUS_MAP_T0_CALC_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS,
        start_col + FP2_FORBENIUS_MAP_SELECTOR_OFFSET,
        bit_selector,
    );
    add_range_check_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col
            + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
            + FP_MULTIPLICATION_TOTAL_COLUMNS
            + REDUCTION_TOTAL,
        bit_selector,
    );
}
