//! This module contains functions for filling the stark trace and adding constraints for the corresponding trace for some Fp operations (multiplication, addition, subtraction, etc). One fp element is represented as \[u32; 12\] inside the trace.
use num_bigint::{BigUint, ToBigUint};
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
use starkyx::plonky2::parser::consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::verification::{
    native::{
        add_u32_slices, add_u32_slices_12, get_bits_as_array, get_div_rem_modulus_from_biguint_12,
        get_selector_bits_from_u32, get_u32_vec_from_literal, get_u32_vec_from_literal_24, modulus,
        mul_u32_slice_u32, multiply_by_slice, sub_u32_slices, sub_u32_slices_12, Fp,
    },
    utils::*,
};

// Fp Multiplication layout offsets
/*
    These trace offsets are for long multiplication. The inputs are each of 12 limbs. The trace needs 12 rows.
    to compute the result of the multiplication. The final result is stored in the slice [SUM_OFFSET..SUM_OFFSET+24].
    X_INPUT_OFFSET -> offset at which the first input is set.
    Y_INPUT_OFFSET -> offset at which the second input is set.
    XY_OFFSET -> offset of x * y[i] where 0 <= i < 12.
    XY_CARRIES_OFFSET -> offset of carries which resulted from the operation x * y[i].
    SHIFTED_XY_OFFSET -> offset at which the shifted values of x * y\[i\] are set. In long multiplication, the multiplication of the i-th digit
        is set after shifting the result by i places. This is exactly that shift. The maximum shift can be 11, hence the maximum result can be of
        length 24. Therefore, 24 placesa are reserved for this field.
    SELECTOR_OFFSET -> offset specifying which index of y are we using for multiplication in the current row. Total 12 selectors, one for each limb.
    SUM_OFFSET -> offset at which the sum of the individual multiplications done so far are stored.
    SUM_CARRIES_OFFSET -> offset of carries which resulted from the additions.
    MULTIPLICATION_SELECTOR_OFFSET -> Selector to ensure that the input is same across all rows. Set 1 in all rows except last one.
    MULTIPLICATION_FIRST_ROW_OFFSET -> Selector to indicate the first row of multiplication operation
*/
pub const X_INPUT_OFFSET: usize = 0;
pub const Y_INPUT_OFFSET: usize = X_INPUT_OFFSET + 12;
pub const XY_OFFSET: usize = Y_INPUT_OFFSET + 12;
pub const XY_CARRIES_OFFSET: usize = XY_OFFSET + 13;
pub const SHIFTED_XY_OFFSET: usize = XY_CARRIES_OFFSET + 12;
pub const SELECTOR_OFFSET: usize = SHIFTED_XY_OFFSET + 24;
pub const SUM_OFFSET: usize = SELECTOR_OFFSET + 12;
pub const SUM_CARRIES_OFFSET: usize = SUM_OFFSET + 24;
pub const MULTIPLICATION_SELECTOR_OFFSET: usize = SUM_CARRIES_OFFSET + 24;
pub const MULTIPLICATION_FIRST_ROW_OFFSET: usize = MULTIPLICATION_SELECTOR_OFFSET + 1;

pub const FP_MULTIPLICATION_TOTAL_COLUMNS: usize = MULTIPLICATION_FIRST_ROW_OFFSET + 1;

// Non reduced addition layout offsets
/*
    These trace offsets are for long addition. The inputs are 24 limbs each. The trace needs 1 row to compute the result.
    ADDITION_CHECK_OFFSET -> Selector to indicate this operation is on.
    ADDITION_X_OFFSET -> offset at which first input set.
    ADDITION_Y_OFFSET -> offset at which first second set.
    ADDITION_SUM_OFFSET -> offset at which the result of the addition is set.
    ADDITION_CARRY_OFFSET -> offset of carries which resulted from the addition operation.
*/
pub const ADDITION_CHECK_OFFSET: usize = 0;
pub const ADDITION_X_OFFSET: usize = ADDITION_CHECK_OFFSET + 1;
pub const ADDITION_Y_OFFSET: usize = ADDITION_X_OFFSET + 24;
pub const ADDITION_SUM_OFFSET: usize = ADDITION_Y_OFFSET + 24;
pub const ADDITION_CARRY_OFFSET: usize = ADDITION_SUM_OFFSET + 24;
pub const ADDITION_TOTAL: usize = ADDITION_CARRY_OFFSET + 24;

// Non reduced subtraction layout offsets
/*
    These trace offsets are for long subtraction. The inputs are 24 limbs each. The trace needs 1 row to compute the result. Assume x > y.
    SUBTRACTION_CHECK_OFFSET -> Selector to indicate this operation is on.
    SUBTRACTION_X_OFFSET -> offset at which first input set.
    SUBTRACTION_Y_OFFSET -> offset at which first second set.
    SUBTRACTION_SUM_OFFSET -> offset at which the result of the subtraction is set.
    SUBTRACTION_CARRY_OFFSET -> offset of borrows which resulted from the subtraction operation.
*/
pub const SUBTRACTION_CHECK_OFFSET: usize = 0;
pub const SUBTRACTION_X_OFFSET: usize = SUBTRACTION_CHECK_OFFSET + 1;
pub const SUBTRACTION_Y_OFFSET: usize = SUBTRACTION_X_OFFSET + 24;
pub const SUBTRACTION_DIFF_OFFSET: usize = SUBTRACTION_Y_OFFSET + 24;
pub const SUBTRACTION_BORROW_OFFSET: usize = SUBTRACTION_DIFF_OFFSET + 24;
pub const SUBTRACTION_TOTAL: usize = SUBTRACTION_BORROW_OFFSET + 24;

// Reduce and rangecheck layout offsets
/*
    These trace offsets are for reducing a [u32; 24] input with the bls12-381 field prime. Ensures, x = d*p + r. Where x is the input,
    d is the quotient, p is the prime and r is the reduced output. The trace needs 12 rows.
    REDUCE_MULTIPLICATION_OFFSET -> offset at which the multiplication operation is done.
    REDUCE_X_OFFSET -> offset at which input is set.
    REDUCTION_ADDITION_OFFSET -> offset at which addition operation is done.
    REDUCED_OFFSET -> offset at which the reduced value is set
*/
pub const REDUCE_MULTIPLICATION_OFFSET: usize = 0;
pub const REDUCE_X_OFFSET: usize = REDUCE_MULTIPLICATION_OFFSET + FP_MULTIPLICATION_TOTAL_COLUMNS;
pub const REDUCTION_ADDITION_OFFSET: usize = REDUCE_X_OFFSET + 24;
pub const REDUCED_OFFSET: usize = REDUCTION_ADDITION_OFFSET + ADDITION_TOTAL;
pub const REDUCTION_TOTAL: usize = REDUCED_OFFSET + 12;

// Rangecheck offsets
// whenever range check is used, start_col - 12 will contain the element being rangechecked
/*
    These trace offsets are for checking if a given input is less than the bls12-381 field prime. Needs 1 row for the computation. The check works as follows ->
        1. Compute y = (2**382 - p + x)
        2. If (y>>382)&1 == 0, then x in less than p.
    RANGE_CHECK_SELECTOR_OFFSET -> selector to indicate this operation is on.
    RANGE_CHECK_SUM_OFFSET -> offset which stores the sum.
    RANGE_CHECK_SUM_CARRY_OFFSET -> offset which stores the carries resulted from the addition operation.
    RANGE_CHECK_BIT_DECOMP_OFFSET -> offset at which the bit decomposition of the most significant limb of the sum is stored.
*/
pub const RANGE_CHECK_SELECTOR_OFFSET: usize = 0;
pub const RANGE_CHECK_SUM_OFFSET: usize = RANGE_CHECK_SELECTOR_OFFSET + 1;
pub const RANGE_CHECK_SUM_CARRY_OFFSET: usize = RANGE_CHECK_SUM_OFFSET + 12;
pub const RANGE_CHECK_BIT_DECOMP_OFFSET: usize = RANGE_CHECK_SUM_CARRY_OFFSET + 12;
pub const RANGE_CHECK_TOTAL: usize = RANGE_CHECK_BIT_DECOMP_OFFSET + 32;

// Fp addition layout offsets
/*
    These trace offsets are for long addition. The inputs are 12 limbs each. The trace needs 1 row to compute the result.
    FP_ADDITION_CHECK_OFFSET -> Selector to indicate this operation is on.
    FP_ADDITION_X_OFFSET -> offset at which first input set.
    FP_ADDITION_Y_OFFSET -> offset at which first second set.
    FP_ADDITION_SUM_OFFSET -> offset at which the result of the addition is set.
    FP_ADDITION_CARRY_OFFSET -> offset of carries which resulted from the addition operation.
*/
pub const FP_ADDITION_CHECK_OFFSET: usize = 0;
pub const FP_ADDITION_X_OFFSET: usize = FP_ADDITION_CHECK_OFFSET + 1;
pub const FP_ADDITION_Y_OFFSET: usize = FP_ADDITION_X_OFFSET + 12;
pub const FP_ADDITION_SUM_OFFSET: usize = FP_ADDITION_Y_OFFSET + 12;
pub const FP_ADDITION_CARRY_OFFSET: usize = FP_ADDITION_SUM_OFFSET + 12;
pub const FP_ADDITION_TOTAL: usize = FP_ADDITION_CARRY_OFFSET + 12;

// Fp subtraction layout offsets
/*
    These trace offsets are for long subtraction. The inputs are 12 limbs each. The trace needs 1 row to compute the result. Assume x > y.
    FP_SUBTRACTION_CHECK_OFFSET -> Selector to indicate this operation is on.
    FP_SUBTRACTION_X_OFFSET -> offset at which first input set.
    FP_SUBTRACTION_Y_OFFSET -> offset at which first second set.
    FP_SUBTRACTION_SUM_OFFSET -> offset at which the result of the subtraction is set.
    FP_SUBTRACTION_CARRY_OFFSET -> offset of borrows which resulted from the subtraction operation.
*/
pub const FP_SUBTRACTION_CHECK_OFFSET: usize = 0;
pub const FP_SUBTRACTION_X_OFFSET: usize = FP_SUBTRACTION_CHECK_OFFSET + 1;
pub const FP_SUBTRACTION_Y_OFFSET: usize = FP_SUBTRACTION_X_OFFSET + 12;
pub const FP_SUBTRACTION_DIFF_OFFSET: usize = FP_SUBTRACTION_Y_OFFSET + 12;
pub const FP_SUBTRACTION_BORROW_OFFSET: usize = FP_SUBTRACTION_DIFF_OFFSET + 12;
pub const FP_SUBTRACTION_TOTAL: usize = FP_SUBTRACTION_BORROW_OFFSET + 12;

// Fp multiply single
/*
    These trace offsets are for long multiplication. The first input is 12 limbs, the second input is 1 limb. The trace needs 1 row to compute the result.
    FP_MULTIPLY_SINGLE_CHECK_OFFSET -> Selector to indicate this operation is on.
    FP_MULTIPLY_SINGLE_X_OFFSET -> offset at which first input set.
    FP_MULTIPLY_SINGLE_Y_OFFSET -> offset at which first second set.
    FP_MULTIPLY_SINGLE_SUM_OFFSET -> offset at which the result of the addition is set.
    FP_MULTIPLY_SINGLE_CARRY_OFFSET -> offset of carries which resulted from the addition operation.
*/
pub const FP_MULTIPLY_SINGLE_CHECK_OFFSET: usize = 0;
pub const FP_MULTIPLY_SINGLE_X_OFFSET: usize = FP_MULTIPLY_SINGLE_CHECK_OFFSET + 1;
pub const FP_MULTIPLY_SINGLE_Y_OFFSET: usize = FP_MULTIPLY_SINGLE_X_OFFSET + 12;
pub const FP_MULTIPLY_SINGLE_SUM_OFFSET: usize = FP_MULTIPLY_SINGLE_Y_OFFSET + 1;
pub const FP_MULTIPLY_SINGLE_CARRY_OFFSET: usize = FP_MULTIPLY_SINGLE_SUM_OFFSET + 12;
pub const FP_MULTIPLY_SINGLE_TOTAL: usize = FP_MULTIPLY_SINGLE_CARRY_OFFSET + 12;

// Fp reduce rangecheck single
/*
    These trace offsets are for for reducing a [u32; 12] input with the bls12-381 field prime. Ensures, x = d*p + r. Where x is the input,
    d is the quotient, p is the prime and r is the reduced output.
    FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET -> offset at which the multiplication operation is done.
    FP_SINGLE_REDUCE_X_OFFSET -> offset at which input is set.
    FP_SINGLE_REDUCTION_ADDITION_OFFSET -> offset at which addition operation is done.
    FP_SINGLE_REDUCED_OFFSET -> offset at which the reduced value is set
*/
pub const FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET: usize = 0;
pub const FP_SINGLE_REDUCE_X_OFFSET: usize =
    FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET + FP_MULTIPLY_SINGLE_TOTAL;
pub const FP_SINGLE_REDUCTION_ADDITION_OFFSET: usize = FP_SINGLE_REDUCE_X_OFFSET + 12;
pub const FP_SINGLE_REDUCED_OFFSET: usize = FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_TOTAL;
pub const FP_SINGLE_REDUCE_TOTAL: usize = FP_SINGLE_REDUCED_OFFSET + 12;

macro_rules! bit_decomp_32 {
    ($row:expr, $col:expr, $f:ty, $p:ty) => {
        ((0..32).fold(<$p>::ZEROS, |acc, i| {
            acc + $row[$col + i] * <$f>::from_canonical_u64(1 << i)
        }))
    };
}

macro_rules! bit_decomp_32_circuit {
    ($builder:expr, $row:expr, $col:expr, $f:ty) => {{
        let zero = $builder.constant_extension(<$f>::Extension::ZERO);
        ((0..32).fold(zero, |acc, i| {
            let tmp_const =
                $builder.constant_extension(<$f>::Extension::from_canonical_u64(1 << i));
            let mul_tmp = $builder.mul_extension($row[$col + i], tmp_const);
            $builder.add_extension(acc, mul_tmp)
        }))
    }};
}

/// Fills the stark trace of addition following long addition. Inputs are 24 limbs each. Needs 1 row.
pub fn fill_addition_trace<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 24],
    y: &[u32; 24],
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + ADDITION_CHECK_OFFSET] = F::ONE;
    let (x_y_sum, x_y_sum_carry) = add_u32_slices(&x, &y);
    assign_u32_in_series(trace, row, start_col + ADDITION_X_OFFSET, x);
    assign_u32_in_series(trace, row, start_col + ADDITION_Y_OFFSET, y);
    assign_u32_in_series(trace, row, start_col + ADDITION_SUM_OFFSET, &x_y_sum);
    assign_u32_in_series(
        trace,
        row,
        start_col + ADDITION_CARRY_OFFSET,
        &x_y_sum_carry,
    );
}

/// Fills the stark trace of addition following long addition. Inputs are 12 limbs each. Needs 1 row.
pub fn fill_trace_addition_fp<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    y: &[u32; 12],
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + FP_ADDITION_CHECK_OFFSET] = F::ONE;
    let (x_y_sum, x_y_sum_carry) = add_u32_slices_12(&x, &y);
    assign_u32_in_series(trace, row, start_col + FP_ADDITION_X_OFFSET, x);
    assign_u32_in_series(trace, row, start_col + FP_ADDITION_Y_OFFSET, y);
    assign_u32_in_series(trace, row, start_col + FP_ADDITION_SUM_OFFSET, &x_y_sum);
    assign_u32_in_series(
        trace,
        row,
        start_col + FP_ADDITION_CARRY_OFFSET,
        &x_y_sum_carry,
    );
}

/// Fills the stark trace of negation. Input is 12 limbs. Needs 1 row. In essence, it fills an addition trace with inputs as `x` and `-x`.
pub fn fill_trace_negate_fp<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    row: usize,
    start_col: usize,
) {
    let minus_x = (-Fp(x.to_owned())).0;
    fill_trace_addition_fp(trace, x, &minus_x, row, start_col);
}

/// Fills the stark trace of subtraction following long subtraction. Inputs are 24 limbs each. Needs 1 row. Assume x > y.
pub fn fill_subtraction_trace<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 24],
    y: &[u32; 24],
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + SUBTRACTION_CHECK_OFFSET] = F::ONE;
    let (x_y_diff, x_y_diff_borrow) = sub_u32_slices(&x, &y);
    assign_u32_in_series(trace, row, start_col + SUBTRACTION_X_OFFSET, x);
    assign_u32_in_series(trace, row, start_col + SUBTRACTION_Y_OFFSET, y);
    assign_u32_in_series(trace, row, start_col + SUBTRACTION_DIFF_OFFSET, &x_y_diff);
    assign_u32_in_series(
        trace,
        row,
        start_col + SUBTRACTION_BORROW_OFFSET,
        &x_y_diff_borrow,
    );
}

/// Fills the stark trace of subtraction following long subtraction. Inputs are 12 limbs each. Needs 1 row. Assume x > y.
pub fn fill_trace_subtraction_fp<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    y: &[u32; 12],
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + FP_SUBTRACTION_CHECK_OFFSET] = F::ONE;
    let (x_y_diff, x_y_borrow) = sub_u32_slices_12(&x, &y);
    assign_u32_in_series(trace, row, start_col + FP_SUBTRACTION_X_OFFSET, x);
    assign_u32_in_series(trace, row, start_col + FP_SUBTRACTION_Y_OFFSET, y);
    assign_u32_in_series(
        trace,
        row,
        start_col + FP_SUBTRACTION_DIFF_OFFSET,
        &x_y_diff,
    );
    assign_u32_in_series(
        trace,
        row,
        start_col + FP_SUBTRACTION_BORROW_OFFSET,
        &x_y_borrow,
    );
}

/// Fills the stark trace of multiplication following long multiplication. Inputs are 12 limbs and 1 limb respectively. Needs 1 row.
pub fn fill_trace_multiply_single_fp<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    y: u32,
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + FP_MULTIPLY_SINGLE_CHECK_OFFSET] = F::ONE;
    let (x_y_sum, x_y_carry) = mul_u32_slice_u32(x, y);
    assign_u32_in_series(trace, row, start_col + FP_MULTIPLY_SINGLE_X_OFFSET, x);
    trace[row][start_col + FP_MULTIPLY_SINGLE_Y_OFFSET] = F::from_canonical_u32(y);
    assign_u32_in_series(
        trace,
        row,
        start_col + FP_MULTIPLY_SINGLE_SUM_OFFSET,
        &x_y_sum,
    );
    assign_u32_in_series(
        trace,
        row,
        start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET,
        &x_y_carry,
    );
}

/// Fills the stark trace of reducing wrt modulo p. Input is 12 limbs. Needs 1 row. Returns the answer as \[u32; 12\].
pub fn fill_trace_reduce_single<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    row: usize,
    start_col: usize,
) -> [u32; 12] {
    let (div, rem) = get_div_rem_modulus_from_biguint_12(BigUint::new(x.to_vec()));
    let div = div[0];
    let modulus = get_u32_vec_from_literal(modulus());
    fill_trace_multiply_single_fp(
        trace,
        &modulus,
        div,
        row,
        start_col + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET,
    );
    assign_u32_in_series(trace, row, start_col + FP_SINGLE_REDUCE_X_OFFSET, x);
    let div_x_mod =
        get_u32_vec_from_literal(div.to_biguint().unwrap() * BigUint::new(modulus.to_vec()));
    assign_u32_in_series(trace, row, start_col + FP_SINGLE_REDUCED_OFFSET, &rem);
    fill_trace_addition_fp(
        trace,
        &div_x_mod,
        &rem,
        row,
        start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET,
    );
    rem
}

/// Fills the stark trace for range check operation wrt the field prime p. Input is 12 limbs. Needs 1 row.
pub fn fill_range_check_trace<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    row: usize,
    start_col: usize,
) {
    let y = (BigUint::from(1u32) << 382) - modulus();
    let y_u32 = get_u32_vec_from_literal(y);
    let (x_y_sum, x_y_carry) = add_u32_slices_12(&x, &y_u32);
    trace[row][start_col + RANGE_CHECK_SELECTOR_OFFSET] = F::ONE;
    assign_u32_in_series(trace, row, start_col + RANGE_CHECK_SUM_OFFSET, &x_y_sum);
    assign_u32_in_series(
        trace,
        row,
        start_col + RANGE_CHECK_SUM_CARRY_OFFSET,
        &x_y_carry,
    );
    assign_u32_in_series(
        trace,
        row,
        start_col + RANGE_CHECK_BIT_DECOMP_OFFSET,
        &get_bits_as_array(x_y_sum[11]),
    );
}

/// Fills stark trace for multiplication following long multiplication. Inputs are 12 limbs each. Needs 12 rows.
pub fn fill_multiplication_trace_no_mod_reduction<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 12],
    y: &[u32; 12],
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    let mut selector = 1;
    // Inputs are filled from start_row..end_row + 1
    trace[start_row][start_col + MULTIPLICATION_FIRST_ROW_OFFSET] = F::ONE;
    for i in start_row..start_row + 11 {
        trace[i][start_col + MULTIPLICATION_SELECTOR_OFFSET] = F::ONE;
    }
    for row in start_row..end_row + 1 {
        assign_u32_in_series(trace, row, start_col + X_INPUT_OFFSET, x);
        assign_u32_in_series(trace, row, start_col + Y_INPUT_OFFSET, y);
        let selector_u32 = get_selector_bits_from_u32(selector);
        assign_u32_in_series(trace, row, start_col + SELECTOR_OFFSET, &selector_u32);
        selector *= 2;
    }

    // We have calcualted multiplying two max bls12_381 Fp numbers
    // dont exceed [u32; 24] so no need of [u32; 25]
    let mut prev_xy_sum = [0u32; 24];

    for i in 0..12 {
        let (xy, xy_carry) = multiply_by_slice(&x, y[i]);
        assign_u32_in_series(trace, start_row + i, start_col + XY_OFFSET, &xy);
        assign_u32_in_series(
            trace,
            start_row + i,
            start_col + XY_CARRIES_OFFSET,
            &xy_carry,
        );

        // fill shifted XY's
        // XY's will have 0-11 number of shifts in their respective rows
        let mut xy_shifted = [0u32; 24];
        for j in 0..13 {
            let shift = i;
            xy_shifted[j + shift] = xy[j];
        }
        assign_u32_in_series(
            trace,
            start_row + i,
            start_col + SHIFTED_XY_OFFSET,
            &xy_shifted,
        );

        // Fill XY_SUM, XY_SUM_CARRIES
        let (xy_sum, xy_sum_carry) = add_u32_slices(&xy_shifted, &prev_xy_sum);
        assign_u32_in_series(trace, start_row + i, start_col + SUM_OFFSET, &xy_sum);
        assign_u32_in_series(
            trace,
            start_row + i,
            start_col + SUM_CARRIES_OFFSET,
            &xy_sum_carry,
        );

        prev_xy_sum = xy_sum;
    }
}

/// Fills the stark trace of reducing wrt modulo p. Input is 24 limbs. Needs 12 rows. Set addition selector to 1 only in the 11th row, because that's where multiplication result is set. Returns the answer as \[u32; 12\].
pub fn fill_reduction_trace<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[u32; 24],
    start_row: usize,
    end_row: usize,
    start_col: usize,
) -> [u32; 12] {
    let (div, rem) = get_div_rem_modulus_from_biguint_12(BigUint::new(x.to_vec()));
    let modulus = get_u32_vec_from_literal(modulus());
    fill_multiplication_trace_no_mod_reduction(
        trace,
        &div,
        &modulus,
        start_row,
        end_row,
        start_col + REDUCE_MULTIPLICATION_OFFSET,
    );

    for row in start_row..end_row + 1 {
        assign_u32_in_series(trace, row, start_col + REDUCE_X_OFFSET, x);
    }

    let div_x_mod =
        get_u32_vec_from_literal_24(BigUint::new(div.to_vec()) * BigUint::new(modulus.to_vec()));

    for i in start_row..end_row + 1 {
        assign_u32_in_series(trace, i, start_col + REDUCED_OFFSET, &rem);
    }
    let mut rem_24 = [0u32; 24];
    rem_24[0..12].copy_from_slice(&rem);

    fill_addition_trace(
        trace,
        &div_x_mod,
        &rem_24,
        start_row + 11,
        start_col + REDUCTION_ADDITION_OFFSET,
    );
    rem
}

/// Constraints the operation for multiplication of two \[u32; 12\].
///
/// Constraint the input values across this row and next row wherever selector is on.
///
/// Constraints the following -> `selector[i] * (product[j] + carries[j]*(2**32) - x[j] * y[i] - carries[j-1]) == 0`. for 0 <= j < 12, for 0 <= i < 12.
/// which encapsulates the condition "either selector is off or the multiplication is correct".
///
/// Constraints the shifted value with product of the current limb as `selector[i] * (shifted[i + j] - product[j]) == 0`. for 0 <= j < 12, for 0 <= i < 12.
/// which encapsulates the condition "either selector is off or product is shifted by i places".
///
/// Constraint the first row of multiplication that `sum == shifted` for all limbs
///
/// Constraints `next_row_sum[i] + next_row_carries[i]*(2**32) == curr_row_sum[i] + shifted[i] + next_row_carries[i-1]` for 0 <= i < 24.
pub fn add_multiplication_constraints<
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
    // Constrains the X and Y is filled same across the rows
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);
    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + X_INPUT_OFFSET + i]
                    - next_values[start_col + X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET]
                * (local_values[start_col + Y_INPUT_OFFSET + i]
                    - next_values[start_col + Y_INPUT_OFFSET + i]),
        );
    }

    // Constrain that multiplication happens correctly at each level
    for i in 0..12 {
        for j in 0..12 {
            if j == 0 {
                yield_constr.constraint_transition(
                    //local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET] *
                    bit_selector_val
                        * local_values[start_col + SELECTOR_OFFSET + i]
                        * (local_values[start_col + X_INPUT_OFFSET + j]
                            * local_values[start_col + Y_INPUT_OFFSET + i]
                            - local_values[start_col + XY_OFFSET + j]
                            - (local_values[start_col + XY_CARRIES_OFFSET + j]
                                * FE::from_canonical_u64(1 << 32))),
                )
            } else {
                yield_constr.constraint_transition(
                    //local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET] *
                    bit_selector_val
                        * local_values[start_col + SELECTOR_OFFSET + i]
                        * (local_values[start_col + X_INPUT_OFFSET + j]
                            * local_values[start_col + Y_INPUT_OFFSET + i]
                            + local_values[start_col + XY_CARRIES_OFFSET + j - 1]
                            - local_values[start_col + XY_OFFSET + j]
                            - (local_values[start_col + XY_CARRIES_OFFSET + j]
                                * FE::from_canonical_u64(1 << 32))),
                )
            }
        }
    }
    yield_constr.constraint_transition(
        bit_selector_val
            * local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET]
            * (local_values[start_col + XY_OFFSET + 12]
                - local_values[start_col + XY_CARRIES_OFFSET + 11]),
    );

    // Constrain XY SHIFTING
    for i in 0..12 {
        // shift is decided by selector
        for j in 0..13 {
            yield_constr.constraint_transition(
                //local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET] *
                bit_selector_val
                    * local_values[start_col + SELECTOR_OFFSET + i]
                    * (local_values[start_col + SHIFTED_XY_OFFSET + j + i]
                        - local_values[start_col + XY_OFFSET + j]),
            )
        }
    }

    // Constrain addition at each row
    // 1. Constrain XY_SUM at row 0 is same as XY_SHIFTED
    // 2. Constrain XY_SUM_CARRIES at row 0 are all 0
    for j in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLICATION_FIRST_ROW_OFFSET]
                * (local_values[start_col + SUM_OFFSET + j]
                    - local_values[start_col + SHIFTED_XY_OFFSET + j]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLICATION_FIRST_ROW_OFFSET]
                * local_values[start_col + SUM_CARRIES_OFFSET + j],
        )
    }
    // yield_constr.constraint_first_row(//local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET] *
    //     local_values[start_col + SUM_OFFSET + 24]);

    // 3. Constrain addition
    yield_constr.constraint_transition(
        bit_selector_val
            * local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET]
            * (next_values[start_col + SUM_OFFSET]
                + (next_values[start_col + SUM_CARRIES_OFFSET] * FE::from_canonical_u64(1 << 32))
                - next_values[start_col + SHIFTED_XY_OFFSET]
                - local_values[start_col + SUM_OFFSET]),
    );

    for j in 1..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET]
                * (next_values[start_col + SUM_OFFSET + j]
                    + (next_values[start_col + SUM_CARRIES_OFFSET + j]
                        * FE::from_canonical_u64(1 << 32))
                    - next_values[start_col + SHIFTED_XY_OFFSET + j]
                    - local_values[start_col + SUM_OFFSET + j]
                    - next_values[start_col + SUM_CARRIES_OFFSET + j - 1]),
        )
    }
    // yield_constr.constraint_transition(local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET] * (next_values[start_col + SUM_OFFSET + 24] - next_values[start_col + SUM_CARRIES_OFFSET + 23]));
}

pub fn add_multiplication_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let c1 = builder.sub_extension(
            local_values[start_col + X_INPUT_OFFSET + i],
            next_values[start_col + X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(tmp, c1);
        yield_constr.constraint_transition(builder, c1);
        let c2 = builder.sub_extension(
            local_values[start_col + Y_INPUT_OFFSET + i],
            next_values[start_col + Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(tmp, c2);
        yield_constr.constraint_transition(builder, c2);
    }

    for i in 0..12 {
        for j in 0..12 {
            if j == 0 {
                let mul_tmp1 = builder.mul_extension(
                    bit_selector_val,
                    local_values[start_col + SELECTOR_OFFSET + i],
                );
                let mul_tmp2 = builder.mul_extension(
                    local_values[start_col + X_INPUT_OFFSET + j],
                    local_values[start_col + Y_INPUT_OFFSET + i],
                );
                let mul_tmp3 = builder
                    .mul_extension(local_values[start_col + XY_CARRIES_OFFSET + j], constant);

                let sub_tmp1 = builder.sub_extension(mul_tmp2, mul_tmp3);
                let sub_tmp2 =
                    builder.sub_extension(sub_tmp1, local_values[start_col + XY_OFFSET + j]);

                let c = builder.mul_extension(mul_tmp1, sub_tmp2);
                yield_constr.constraint_transition(builder, c);
            } else {
                let mul_tmp1 = builder.mul_extension(
                    bit_selector_val,
                    local_values[start_col + SELECTOR_OFFSET + i],
                );
                let mul_tmp2 = builder.mul_extension(
                    local_values[start_col + X_INPUT_OFFSET + j],
                    local_values[start_col + Y_INPUT_OFFSET + i],
                );

                let mul_tmp3 = builder
                    .mul_extension(local_values[start_col + XY_CARRIES_OFFSET + j], constant);

                let sub_tmp1 = builder.sub_extension(mul_tmp2, mul_tmp3);
                let sub_tmp2 =
                    builder.sub_extension(sub_tmp1, local_values[start_col + XY_OFFSET + j]);

                let add_tmp1 = builder.add_extension(
                    sub_tmp2,
                    local_values[start_col + XY_CARRIES_OFFSET + j - 1],
                );

                let c = builder.mul_extension(mul_tmp1, add_tmp1);
                yield_constr.constraint_transition(builder, c);
            }
        }
    }

    let mul_tmp1 = builder.mul_extension(
        bit_selector_val,
        local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET],
    );
    let sub_tmp1 = builder.sub_extension(
        local_values[start_col + XY_OFFSET + 12],
        local_values[start_col + XY_CARRIES_OFFSET + 11],
    );

    let c = builder.mul_extension(mul_tmp1, sub_tmp1);
    yield_constr.constraint_transition(builder, c);

    for i in 0..12 {
        for j in 0..13 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + SELECTOR_OFFSET + i],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col + SHIFTED_XY_OFFSET + j + i],
                local_values[start_col + XY_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint_transition(builder, c);
        }
    }

    for j in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLICATION_FIRST_ROW_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + SUM_OFFSET + j],
            local_values[start_col + SHIFTED_XY_OFFSET + j],
        );

        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let c2 = builder.mul_extension(mul_tmp1, local_values[start_col + SUM_CARRIES_OFFSET + j]);
        yield_constr.constraint(builder, c2);
    }

    let mul_tmp1 = builder.mul_extension(
        bit_selector_val,
        local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET],
    );
    let mul_tmp2 = builder.mul_extension(next_values[start_col + SUM_CARRIES_OFFSET], constant);

    let sub_tmp1 = builder.sub_extension(mul_tmp2, next_values[start_col + SHIFTED_XY_OFFSET]);
    let sub_tmp2 = builder.sub_extension(sub_tmp1, local_values[start_col + SUM_OFFSET]);

    let add_tmp1 = builder.add_extension(sub_tmp2, next_values[start_col + SUM_OFFSET]);

    let c = builder.mul_extension(mul_tmp1, add_tmp1);
    yield_constr.constraint_transition(builder, c);

    for j in 1..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + MULTIPLICATION_SELECTOR_OFFSET],
        );
        let mul_tmp2 =
            builder.mul_extension(next_values[start_col + SUM_CARRIES_OFFSET + j], constant);

        let sub_tmp1 =
            builder.sub_extension(mul_tmp2, next_values[start_col + SHIFTED_XY_OFFSET + j]);
        let sub_tmp2 = builder.sub_extension(sub_tmp1, local_values[start_col + SUM_OFFSET + j]);
        let sub_tmp3 = builder.sub_extension(
            sub_tmp2,
            next_values[start_col + SUM_CARRIES_OFFSET + j - 1],
        );

        let add_tmp1 = builder.add_extension(sub_tmp3, next_values[start_col + SUM_OFFSET + j]);

        let c = builder.mul_extension(mul_tmp1, add_tmp1);
        yield_constr.constraint_transition(builder, c);
    }
}

/// Constraints the addition for addition of two \[u32; 24\].
/// Constraints the following for every limb -> `sum[i] + carries[i]*(2**32) == x[i] + y[i] + carries[i-1]`.
pub fn add_addition_constraints<
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

    for j in 0..24 {
        if j == 0 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + ADDITION_CHECK_OFFSET]
                    * (local_values[start_col + ADDITION_SUM_OFFSET + j]
                        + (local_values[start_col + ADDITION_CARRY_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + ADDITION_X_OFFSET + j]
                        - local_values[start_col + ADDITION_Y_OFFSET + j]),
            )
        } else {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + ADDITION_CHECK_OFFSET]
                    * (local_values[start_col + ADDITION_SUM_OFFSET + j]
                        + (local_values[start_col + ADDITION_CARRY_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + ADDITION_X_OFFSET + j]
                        - local_values[start_col + ADDITION_Y_OFFSET + j]
                        - local_values[start_col + ADDITION_CARRY_OFFSET + j - 1]),
            )
        }
    }
}
pub fn add_addition_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for j in 0..24 {
        if j == 0 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + ADDITION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + ADDITION_CARRY_OFFSET + j],
                constant,
            );

            let sub_tmp1 =
                builder.sub_extension(mul_tmp2, local_values[start_col + ADDITION_X_OFFSET + j]);
            let sub_tmp2 =
                builder.sub_extension(sub_tmp1, local_values[start_col + ADDITION_Y_OFFSET + j]);

            let add_tmp1 =
                builder.add_extension(sub_tmp2, local_values[start_col + ADDITION_SUM_OFFSET + j]);

            let c = builder.mul_extension(mul_tmp1, add_tmp1);
            yield_constr.constraint_transition(builder, c);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + ADDITION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + ADDITION_CARRY_OFFSET + j],
                constant,
            );

            let sub_tmp1 =
                builder.sub_extension(mul_tmp2, local_values[start_col + ADDITION_X_OFFSET + j]);
            let sub_tmp2 =
                builder.sub_extension(sub_tmp1, local_values[start_col + ADDITION_Y_OFFSET + j]);
            let sub_tmp3 = builder.sub_extension(
                sub_tmp2,
                local_values[start_col + ADDITION_CARRY_OFFSET + j - 1],
            );

            let add_tmp1 =
                builder.add_extension(sub_tmp3, local_values[start_col + ADDITION_SUM_OFFSET + j]);

            let c = builder.mul_extension(mul_tmp1, add_tmp1);
            yield_constr.constraint_transition(builder, c);
        }
    }
}

/// Constraints the operation for addition of two \[u32; 12\].
/// Constraints the following for every limb -> `sum[i] + carries[i]*(2**32) == x[i] + y[i] + carries[i-1]`.
pub fn add_addition_fp_constraints<
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

    for j in 0..12 {
        if j == 0 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col + FP_ADDITION_SUM_OFFSET + j]
                        + (local_values[start_col + FP_ADDITION_CARRY_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + FP_ADDITION_X_OFFSET + j]
                        - local_values[start_col + FP_ADDITION_Y_OFFSET + j]),
            )
        } else {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col + FP_ADDITION_SUM_OFFSET + j]
                        + (local_values[start_col + FP_ADDITION_CARRY_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + FP_ADDITION_X_OFFSET + j]
                        - local_values[start_col + FP_ADDITION_Y_OFFSET + j]
                        - local_values[start_col + FP_ADDITION_CARRY_OFFSET + j - 1]),
            )
        }
    }
}

pub fn add_addition_fp_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for j in 0..12 {
        if j == 0 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP_ADDITION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + FP_ADDITION_CARRY_OFFSET + j],
                constant,
            );

            let sub_tmp1 =
                builder.sub_extension(mul_tmp2, local_values[start_col + FP_ADDITION_X_OFFSET + j]);
            let sub_tmp2 =
                builder.sub_extension(sub_tmp1, local_values[start_col + FP_ADDITION_Y_OFFSET + j]);

            let add_tmp1 = builder.add_extension(
                sub_tmp2,
                local_values[start_col + FP_ADDITION_SUM_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, add_tmp1);
            yield_constr.constraint(builder, c);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP_ADDITION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + FP_ADDITION_CARRY_OFFSET + j],
                constant,
            );

            let sub_tmp1 =
                builder.sub_extension(mul_tmp2, local_values[start_col + FP_ADDITION_X_OFFSET + j]);
            let sub_tmp2 =
                builder.sub_extension(sub_tmp1, local_values[start_col + FP_ADDITION_Y_OFFSET + j]);
            let sub_tmp3 = builder.sub_extension(
                sub_tmp2,
                local_values[start_col + FP_ADDITION_CARRY_OFFSET + j - 1],
            );

            let add_tmp1 = builder.add_extension(
                sub_tmp3,
                local_values[start_col + FP_ADDITION_SUM_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, add_tmp1);
            yield_constr.constraint(builder, c);
        }
    }
}

/// Constraints the operation for subtraction of two \[u32; 12\].
/// Constraints the following for every limb -> `diff[i] - borrows[i]*(2**32) == x[i] - y[i] - borrows[i-1]`.
pub fn add_subtraction_fp_constraints<
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

    for j in 0..12 {
        if j == 0 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col + FP_SUBTRACTION_DIFF_OFFSET + j]
                        + local_values[start_col + FP_SUBTRACTION_Y_OFFSET + j]
                        - (local_values[start_col + FP_SUBTRACTION_BORROW_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + FP_SUBTRACTION_X_OFFSET + j]),
            )
        } else {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col + FP_SUBTRACTION_DIFF_OFFSET + j]
                        + local_values[start_col + FP_SUBTRACTION_Y_OFFSET + j]
                        + local_values[start_col + FP_SUBTRACTION_BORROW_OFFSET + j - 1]
                        - (local_values[start_col + FP_SUBTRACTION_BORROW_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + FP_SUBTRACTION_X_OFFSET + j]),
            )
        }
    }
}

pub fn add_subtraction_fp_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for j in 0..12 {
        if j == 0 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + FP_SUBTRACTION_BORROW_OFFSET + j],
                constant,
            );

            let add_tmp1 = builder.add_extension(
                local_values[start_col + FP_SUBTRACTION_DIFF_OFFSET + j],
                local_values[start_col + FP_SUBTRACTION_Y_OFFSET + j],
            );

            let sub_tmp1 = builder.sub_extension(add_tmp1, mul_tmp2);
            let sub_tmp2 = builder.sub_extension(
                sub_tmp1,
                local_values[start_col + FP_SUBTRACTION_X_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint(builder, c);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + FP_SUBTRACTION_BORROW_OFFSET + j],
                constant,
            );

            let add_tmp1 = builder.add_extension(
                local_values[start_col + FP_SUBTRACTION_DIFF_OFFSET + j],
                local_values[start_col + FP_SUBTRACTION_Y_OFFSET + j],
            );
            let add_tmp2 = builder.add_extension(
                add_tmp1,
                local_values[start_col + FP_SUBTRACTION_BORROW_OFFSET + j - 1],
            );

            let sub_tmp1 = builder.sub_extension(add_tmp2, mul_tmp2);
            let sub_tmp2 = builder.sub_extension(
                sub_tmp1,
                local_values[start_col + FP_SUBTRACTION_X_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint(builder, c);
        }
    }
}

/// Constraints the negation operation for \[u32; 12\].
/// Constraints an addition operation, following by constraining `result == p`, where p is the field prime.
pub fn add_negate_fp_constraints<
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

    add_addition_fp_constraints(local_values, yield_constr, start_col, bit_selector);
    let mod_u32 = get_u32_vec_from_literal(modulus());
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP_ADDITION_SUM_OFFSET + i]
                    - FE::from_canonical_u32(mod_u32[i])),
        );
    }
}

pub fn add_negate_fp_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    add_addition_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    let mod_u32 = get_u32_vec_from_literal(modulus());

    for i in 0..12 {
        let constant = builder.constant_extension(F::Extension::from_canonical_u32(mod_u32[i]));
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP_ADDITION_SUM_OFFSET + i],
            constant,
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }
}
/// Constraints the operation for multiplication of \[u32; 12\] with a u32.
/// Constraints the following for every limb -> `product[i] + carries[i]*(2**32) == x[i] * y + carries[i-1]`.
pub fn add_fp_single_multiply_constraints<
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

    for j in 0..12 {
        if j == 0 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP_MULTIPLY_SINGLE_CHECK_OFFSET]
                    * (local_values[start_col + FP_MULTIPLY_SINGLE_SUM_OFFSET + j]
                        + (local_values[start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + FP_MULTIPLY_SINGLE_X_OFFSET + j]
                            * local_values[start_col + FP_MULTIPLY_SINGLE_Y_OFFSET]),
            )
        } else {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP_MULTIPLY_SINGLE_CHECK_OFFSET]
                    * (local_values[start_col + FP_MULTIPLY_SINGLE_SUM_OFFSET + j]
                        + (local_values[start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + FP_MULTIPLY_SINGLE_X_OFFSET + j]
                            * local_values[start_col + FP_MULTIPLY_SINGLE_Y_OFFSET]
                        - local_values[start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET + j - 1]),
            )
        }
    }
}

pub fn add_fp_single_multiply_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for j in 0..12 {
        if j == 0 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP_MULTIPLY_SINGLE_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET + j],
                constant,
            );
            let mul_tmp3 = builder.mul_extension(
                local_values[start_col + FP_MULTIPLY_SINGLE_X_OFFSET + j],
                local_values[start_col + FP_MULTIPLY_SINGLE_Y_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(mul_tmp2, mul_tmp3);

            let add_tmp1 = builder.add_extension(
                sub_tmp1,
                local_values[start_col + FP_MULTIPLY_SINGLE_SUM_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, add_tmp1);
            yield_constr.constraint(builder, c);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + FP_MULTIPLY_SINGLE_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET + j],
                constant,
            );
            let mul_tmp3 = builder.mul_extension(
                local_values[start_col + FP_MULTIPLY_SINGLE_X_OFFSET + j],
                local_values[start_col + FP_MULTIPLY_SINGLE_Y_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(mul_tmp2, mul_tmp3);
            let sub_tmp2 = builder.sub_extension(
                sub_tmp1,
                local_values[start_col + FP_MULTIPLY_SINGLE_CARRY_OFFSET + j - 1],
            );

            let add_tmp1 = builder.add_extension(
                sub_tmp2,
                local_values[start_col + FP_MULTIPLY_SINGLE_SUM_OFFSET + j],
            );

            let c = builder.mul_extension(mul_tmp1, add_tmp1);
            yield_constr.constraint(builder, c);
        }
    }
}

/// Constraints the reduction operation for \[u32; 12\].
/// Constraints a single multiplication operation with `p` as `x` input. Then constraints an addition operation with the result of the previous multiplication and the reduced answer as inputs. Then constraints the result of the addition with the input of reduction operation.
pub fn add_fp_reduce_single_constraints<
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

    let modulus = modulus();
    let modulus_u32 = get_u32_vec_from_literal(modulus);
    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col
                    + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET
                    + FP_MULTIPLY_SINGLE_CHECK_OFFSET]
                * (local_values[start_col
                    + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET
                    + FP_MULTIPLY_SINGLE_X_OFFSET
                    + i]
                    - FE::from_canonical_u32(modulus_u32[i])),
        );
    }

    add_fp_single_multiply_constraints(
        local_values,
        yield_constr,
        start_col + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values
                    [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET
                    + FP_MULTIPLY_SINGLE_SUM_OFFSET
                    + i]
                    - local_values[start_col
                        + FP_SINGLE_REDUCTION_ADDITION_OFFSET
                        + FP_ADDITION_X_OFFSET
                        + i]),
        );
    }

    add_addition_fp_constraints(
        local_values,
        yield_constr,
        start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values
                    [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP_SINGLE_REDUCED_OFFSET + i]
                    - local_values[start_col
                        + FP_SINGLE_REDUCTION_ADDITION_OFFSET
                        + FP_ADDITION_Y_OFFSET
                        + i]),
        );
    }

    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values
                    [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col + FP_SINGLE_REDUCE_X_OFFSET + i]
                    - local_values[start_col
                        + FP_SINGLE_REDUCTION_ADDITION_OFFSET
                        + FP_ADDITION_SUM_OFFSET
                        + i]),
        )
    }
}

pub fn add_fp_reduce_single_constraints_ext_circuit<
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let modulus = modulus();
    let modulus_u32 = get_u32_vec_from_literal(modulus);
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus_u32[i]));
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col
                + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET
                + FP_MULTIPLY_SINGLE_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET
                + FP_MULTIPLY_SINGLE_X_OFFSET
                + i],
            lc,
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_fp_single_multiply_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values
                [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP_SINGLE_REDUCE_MULTIPLICATION_OFFSET
                + FP_MULTIPLY_SINGLE_SUM_OFFSET
                + i],
            local_values
                [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_addition_fp_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values
                [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP_SINGLE_REDUCED_OFFSET + i],
            local_values
                [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_Y_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values
                [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP_SINGLE_REDUCE_X_OFFSET + i],
            local_values
                [start_col + FP_SINGLE_REDUCTION_ADDITION_OFFSET + FP_ADDITION_SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }
}

/// Constraints the operation for subtraction of two \[u32; 24\].
/// Constraints the following for every limb -> `diff[i] - borrows[i]*(2**32) == x[i] - y[i] - borrows[i-1]`.
pub fn add_subtraction_constraints<
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

    for j in 0..24 {
        if j == 0 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col + SUBTRACTION_DIFF_OFFSET + j]
                        + local_values[start_col + SUBTRACTION_Y_OFFSET + j]
                        - (local_values[start_col + SUBTRACTION_BORROW_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + SUBTRACTION_X_OFFSET + j]),
            )
        } else {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col + SUBTRACTION_DIFF_OFFSET + j]
                        + local_values[start_col + SUBTRACTION_Y_OFFSET + j]
                        + local_values[start_col + SUBTRACTION_BORROW_OFFSET + j - 1]
                        - (local_values[start_col + SUBTRACTION_BORROW_OFFSET + j]
                            * FE::from_canonical_u64(1 << 32))
                        - local_values[start_col + SUBTRACTION_X_OFFSET + j]),
            )
        }
    }
}

pub fn add_subtraction_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for j in 0..24 {
        if j == 0 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + SUBTRACTION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + SUBTRACTION_BORROW_OFFSET + j],
                constant,
            );

            let add_tmp1 = builder.add_extension(
                local_values[start_col + SUBTRACTION_DIFF_OFFSET + j],
                local_values[start_col + SUBTRACTION_Y_OFFSET + j],
            );

            let sub_tmp1 = builder.sub_extension(add_tmp1, mul_tmp2);
            let sub_tmp2 =
                builder.sub_extension(sub_tmp1, local_values[start_col + SUBTRACTION_X_OFFSET + j]);

            let c = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint_transition(builder, c);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + SUBTRACTION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + SUBTRACTION_BORROW_OFFSET + j],
                constant,
            );

            let add_tmp1 = builder.add_extension(
                local_values[start_col + SUBTRACTION_DIFF_OFFSET + j],
                local_values[start_col + SUBTRACTION_Y_OFFSET + j],
            );
            let add_tmp2 = builder.add_extension(
                add_tmp1,
                local_values[start_col + SUBTRACTION_BORROW_OFFSET + j - 1],
            );

            let sub_tmp1 = builder.sub_extension(add_tmp2, mul_tmp2);
            let sub_tmp2 =
                builder.sub_extension(sub_tmp1, local_values[start_col + SUBTRACTION_X_OFFSET + j]);

            let c = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint_transition(builder, c);
        }
    }
}
/// Constraints the range check operation of a \[u23; 12\].
/// Constraints the addition of the input and (2**382)-p. Then constraints the bit decomposition of the most significant limb of the result of the previous addition. Then constraints the 30th bit of the decomposition (which is overall 382nd bit of the result) to zero.
pub fn add_range_check_constraints<
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

    let y = (BigUint::from(1u32) << 382) - modulus();
    let y_u32 = get_u32_vec_from_literal(y);

    for i in 0..12 {
        if i == 0 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET]
                    * (local_values[start_col + RANGE_CHECK_SUM_OFFSET + i]
                        + (local_values[start_col + RANGE_CHECK_SUM_CARRY_OFFSET + i]
                            * FE::from_canonical_u64(1 << 32))
                        - FE::from_canonical_u32(y_u32[i])
                        - local_values[start_col - 12 + i]),
            );
        } else if i < 12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET]
                    * (local_values[start_col + RANGE_CHECK_SUM_OFFSET + i]
                        + (local_values[start_col + RANGE_CHECK_SUM_CARRY_OFFSET + i]
                            * FE::from_canonical_u64(1 << 32))
                        - FE::from_canonical_u32(y_u32[i])
                        - local_values[start_col - 12 + i]
                        - local_values[start_col + RANGE_CHECK_SUM_CARRY_OFFSET + i - 1]),
            );
        }
        let bit_col: usize = start_col + RANGE_CHECK_BIT_DECOMP_OFFSET;
        let val_reconstructed = bit_decomp_32!(local_values, bit_col, FE, P);
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET]
                * (val_reconstructed - local_values[start_col + RANGE_CHECK_SUM_OFFSET + 11]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET]
                * local_values[bit_col + 30],
        );
    }
}

pub fn add_range_check_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let constant = builder.constant_extension(F::Extension::from_canonical_u64(1 << 32));
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    let y = (BigUint::from(1u32) << 382) - modulus();
    let y_u32 = get_u32_vec_from_literal(y);

    for i in 0..12 {
        if i == 0 {
            let lc = builder.constant_extension(F::Extension::from_canonical_u32(y_u32[i]));

            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + RANGE_CHECK_SUM_CARRY_OFFSET + i],
                constant,
            );

            let sub_tmp1 = builder.sub_extension(mul_tmp2, lc);
            let sub_tmp2 = builder.sub_extension(sub_tmp1, local_values[start_col - 12 + i]);

            let add_tmp1 = builder.add_extension(
                sub_tmp2,
                local_values[start_col + RANGE_CHECK_SUM_OFFSET + i],
            );

            let c = builder.mul_extension(mul_tmp1, add_tmp1);

            yield_constr.constraint(builder, c);
        } else if i < 12 {
            let lc = builder.constant_extension(F::Extension::from_canonical_u32(y_u32[i]));

            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                local_values[start_col + RANGE_CHECK_SUM_CARRY_OFFSET + i],
                constant,
            );

            let sub_tmp1 = builder.sub_extension(mul_tmp2, lc);
            let sub_tmp2 = builder.sub_extension(sub_tmp1, local_values[start_col - 12 + i]);
            let sub_tmp3 = builder.sub_extension(
                sub_tmp2,
                local_values[start_col + RANGE_CHECK_SUM_CARRY_OFFSET + i - 1],
            );

            let add_tmp1 = builder.add_extension(
                sub_tmp3,
                local_values[start_col + RANGE_CHECK_SUM_OFFSET + i],
            );

            let c = builder.mul_extension(mul_tmp1, add_tmp1);

            yield_constr.constraint(builder, c);
        }

        let bit_col: usize = start_col + RANGE_CHECK_BIT_DECOMP_OFFSET;
        let val_reconstructed = bit_decomp_32_circuit!(builder, local_values, bit_col, F);
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + RANGE_CHECK_SELECTOR_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            val_reconstructed,
            local_values[start_col + RANGE_CHECK_SUM_OFFSET + 11],
        );

        let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint(builder, c1);

        let c2 = builder.mul_extension(mul_tmp1, local_values[bit_col + 30]);
        yield_constr.constraint(builder, c2);
    }
}

/// Constraints the reduction operation for \[u32; 24\].
/// Constraints that input and result is same across this row and next row wherever the selector is on.
/// Constraints a multiplication operation with `p` as `x` input. Then constraints an addition operation with the result of the previous multiplication and the reduced answer as inputs. Then constraints the result of the addition with the input of reduction operation.
pub fn add_reduce_constraints<
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
    selector_col: usize,
    bit_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    let modulus = modulus();
    let modulus_u32 = get_u32_vec_from_literal(modulus);
    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[selector_col]
                * (local_values[start_col + REDUCE_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i]
                    - FE::from_canonical_u32(modulus_u32[i])),
        );
    }

    add_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + REDUCE_MULTIPLICATION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[selector_col]
                * (local_values[start_col + REDUCE_X_OFFSET + i]
                    - next_values[start_col + REDUCE_X_OFFSET + i]),
        );
    }

    for i in 0..12 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[selector_col]
                * (local_values[start_col + REDUCED_OFFSET + i]
                    - next_values[start_col + REDUCED_OFFSET + i]),
        );
    }

    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + REDUCE_MULTIPLICATION_OFFSET + SUM_OFFSET + i]
                    - local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_X_OFFSET + i]),
        );
    }

    add_addition_constraints(
        local_values,
        yield_constr,
        start_col + REDUCTION_ADDITION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        if i < 12 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                    * (local_values[start_col + REDUCED_OFFSET + i]
                        - local_values
                            [start_col + REDUCTION_ADDITION_OFFSET + ADDITION_Y_OFFSET + i]),
            );
        } else {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                    * local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_Y_OFFSET + i],
            );
        }
    }

    for i in 0..24 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET]
                * (local_values[start_col + REDUCE_X_OFFSET + i]
                    - local_values
                        [start_col + REDUCTION_ADDITION_OFFSET + ADDITION_SUM_OFFSET + i]),
        )
    }
}

pub fn add_reduce_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    selector_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    let modulus = modulus();
    let modulus_u32 = get_u32_vec_from_literal(modulus);

    for i in 0..12 {
        let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus_u32[i]));

        let mul_tmp1 = builder.mul_extension(bit_selector_val, local_values[selector_col]);
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + REDUCE_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            lc,
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + REDUCE_MULTIPLICATION_OFFSET,
        bit_selector,
    );
    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(bit_selector_val, local_values[selector_col]);
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + REDUCE_X_OFFSET + i],
            next_values[start_col + REDUCE_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..12 {
        let mul_tmp1 = builder.mul_extension(bit_selector_val, local_values[selector_col]);
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + REDUCED_OFFSET + i],
            next_values[start_col + REDUCED_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + REDUCE_MULTIPLICATION_OFFSET + SUM_OFFSET + i],
            local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_X_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }

    add_addition_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + REDUCTION_ADDITION_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        if i < 12 {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col + REDUCED_OFFSET + i],
                local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_Y_OFFSET + i],
            );

            let c = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint_transition(builder, c);
        } else {
            let mul_tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
            );

            let c = builder.mul_extension(
                mul_tmp1,
                local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_Y_OFFSET + i],
            );
            yield_constr.constraint_transition(builder, c);
        }
    }

    for i in 0..24 {
        let mul_tmp1 = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_CHECK_OFFSET],
        );
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + REDUCE_X_OFFSET + i],
            local_values[start_col + REDUCTION_ADDITION_OFFSET + ADDITION_SUM_OFFSET + i],
        );

        let c = builder.mul_extension(mul_tmp1, sub_tmp1);
        yield_constr.constraint_transition(builder, c);
    }
}
