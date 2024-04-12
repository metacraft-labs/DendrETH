//! This module contains functions for filling the stark trace and adding constraints for the corresponding trace for some Fp6 operations (multiplication, addition, subtraction, etc). One Fp6 element is represented as \[u32; 72\] inside the trace. But most of the time, Fp6 elements are broken up into six Fp elements, hence represented as six \[u32; 12\].
use crate::verification::{
    fp::*,
    fp2::*,
    native::{get_u32_vec_from_literal, modulus, Fp2, Fp6},
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
use starkyx::plonky2::parser::consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

// FP6 multiplication offsets
/*
    These trace offsets are for fp6 multiplication. It needs 12 rows. The Ti's are defined in (super::native::mul_Fp6).
*/
pub const FP6_MUL_SELECTOR_OFFSET: usize = 0;
pub const FP6_MUL_X_INPUT_OFFSET: usize = FP6_MUL_SELECTOR_OFFSET + 1;
pub const FP6_MUL_Y_INPUT_OFFSET: usize = FP6_MUL_X_INPUT_OFFSET + 24 * 3;
pub const FP6_MUL_T0_CALC_OFFSET: usize = FP6_MUL_Y_INPUT_OFFSET + 24 * 3;
pub const FP6_MUL_T1_CALC_OFFSET: usize = FP6_MUL_T0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_MUL_T2_CALC_OFFSET: usize = FP6_MUL_T1_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_MUL_T3_CALC_OFFSET: usize = FP6_MUL_T2_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_MUL_T4_CALC_OFFSET: usize =
    FP6_MUL_T3_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T5_CALC_OFFSET: usize =
    FP6_MUL_T4_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T6_CALC_OFFSET: usize = FP6_MUL_T5_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_MUL_T7_CALC_OFFSET: usize = FP6_MUL_T6_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T8_CALC_OFFSET: usize = FP6_MUL_T7_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_X_CALC_OFFSET: usize = FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_TOTAL;
pub const FP6_MUL_T9_CALC_OFFSET: usize =
    FP6_MUL_X_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T10_CALC_OFFSET: usize =
    FP6_MUL_T9_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T11_CALC_OFFSET: usize =
    FP6_MUL_T10_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T12_CALC_OFFSET: usize =
    FP6_MUL_T11_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_MUL_T13_CALC_OFFSET: usize = FP6_MUL_T12_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T14_CALC_OFFSET: usize = FP6_MUL_T13_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_Y_CALC_OFFSET: usize = FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_TOTAL;
pub const FP6_MUL_T15_CALC_OFFSET: usize =
    FP6_MUL_Y_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T16_CALC_OFFSET: usize =
    FP6_MUL_T15_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T17_CALC_OFFSET: usize =
    FP6_MUL_T16_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_T18_CALC_OFFSET: usize =
    FP6_MUL_T17_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_MUL_T19_CALC_OFFSET: usize = FP6_MUL_T18_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_Z_CALC_OFFSET: usize = FP6_MUL_T19_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const FP6_MUL_TOTAL_COLUMNS: usize =
    FP6_MUL_Z_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;

// FP6 non residue multiplication
/*
    These trace offsets are for fp6 non residue multiplication (super::native::mul_by_nonresidue). It needs 1 row.
*/
pub const FP6_NON_RESIDUE_MUL_CHECK_OFFSET: usize = 0;
pub const FP6_NON_RESIDUE_MUL_INPUT_OFFSET: usize = FP6_NON_RESIDUE_MUL_CHECK_OFFSET + 1;
pub const FP6_NON_RESIDUE_MUL_C2: usize = FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 24 * 3;
pub const FP6_NON_RESIDUE_MUL_TOTAL: usize = FP6_NON_RESIDUE_MUL_C2 + FP2_NON_RESIDUE_MUL_TOTAL;

// FP6 add
/*
    These trace offsets are for addition for two Fp6 elements. In essence it's three concatenated Fp2 additions. It needs 1 row.
*/
pub const FP6_ADDITION_0_OFFSET: usize = 0;
pub const FP6_ADDITION_1_OFFSET: usize = FP6_ADDITION_0_OFFSET + FP2_ADDITION_TOTAL;
pub const FP6_ADDITION_2_OFFSET: usize = FP6_ADDITION_1_OFFSET + FP2_ADDITION_TOTAL;
pub const FP6_ADDITION_TOTAL: usize = FP6_ADDITION_2_OFFSET + FP2_ADDITION_TOTAL;

// FP6 sub
/*
    These trace offsets are for subtraction for two Fp6 elements. In essence it's three concatenated Fp2 subtractions. It needs 1 row.
*/
pub const FP6_SUBTRACTION_0_OFFSET: usize = 0;
pub const FP6_SUBTRACTION_1_OFFSET: usize = FP6_SUBTRACTION_0_OFFSET + FP2_SUBTRACTION_TOTAL;
pub const FP6_SUBTRACTION_2_OFFSET: usize = FP6_SUBTRACTION_1_OFFSET + FP2_SUBTRACTION_TOTAL;
pub const FP6_SUBTRACTION_TOTAL: usize = FP6_SUBTRACTION_2_OFFSET + FP2_SUBTRACTION_TOTAL;

// MultiplyBy01
/*
    These trace offsets are for multiplyBy01 (super::native::Fp6::multiplyBy01) function. The Ti's are defined in the native function definition. It needs 12 rows.
*/
pub const MULTIPLY_BY_01_SELECTOR_OFFSET: usize = 0;
pub const MULTIPLY_BY_01_INPUT_OFFSET: usize = MULTIPLY_BY_01_SELECTOR_OFFSET + 1;
pub const MULTIPLY_BY_01_B0_OFFSET: usize = MULTIPLY_BY_01_INPUT_OFFSET + 24 * 3;
pub const MULTIPLY_BY_01_B1_OFFSET: usize = MULTIPLY_BY_01_B0_OFFSET + 24;
pub const MULTIPLY_BY_01_T0_CALC_OFFSET: usize = MULTIPLY_BY_01_B1_OFFSET + 24;
pub const MULTIPLY_BY_01_T1_CALC_OFFSET: usize =
    MULTIPLY_BY_01_T0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_01_T2_CALC_OFFSET: usize =
    MULTIPLY_BY_01_T1_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_01_T3_CALC_OFFSET: usize =
    MULTIPLY_BY_01_T2_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_01_X_CALC_OFFSET: usize =
    MULTIPLY_BY_01_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_TOTAL;
pub const MULTIPLY_BY_01_T4_CALC_OFFSET: usize = MULTIPLY_BY_01_X_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const MULTIPLY_BY_01_T5_CALC_OFFSET: usize = MULTIPLY_BY_01_T4_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const MULTIPLY_BY_01_T6_CALC_OFFSET: usize = MULTIPLY_BY_01_T5_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const MULTIPLY_BY_01_T7_CALC_OFFSET: usize =
    MULTIPLY_BY_01_T6_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_01_Y_CALC_OFFSET: usize = MULTIPLY_BY_01_T7_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const MULTIPLY_BY_01_T8_CALC_OFFSET: usize = MULTIPLY_BY_01_Y_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const MULTIPLY_BY_01_Z_CALC_OFFSET: usize =
    MULTIPLY_BY_01_T8_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_01_TOTAL: usize = MULTIPLY_BY_01_Z_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;

// MultiplyBy1
/*
    These trace offsets are for multiplyBy1 (super::native::Fp6::multiplyBy1) function. The Ti's are defined in the native function definition. It needs 12 rows.
*/
pub const MULTIPLY_BY_1_SELECTOR_OFFSET: usize = 0;
pub const MULTIPLY_BY_1_INPUT_OFFSET: usize = MULTIPLY_BY_1_SELECTOR_OFFSET + 1;
pub const MULTIPLY_BY_1_B1_OFFSET: usize = MULTIPLY_BY_1_INPUT_OFFSET + 24 * 3;
pub const MULTIPLY_BY_1_T0_CALC_OFFSET: usize = MULTIPLY_BY_1_B1_OFFSET + 24;
pub const MULTIPLY_BY_1_X_CALC_OFFSET: usize =
    MULTIPLY_BY_1_T0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_1_Y_CALC_OFFSET: usize =
    MULTIPLY_BY_1_X_CALC_OFFSET + FP2_NON_RESIDUE_MUL_TOTAL;
pub const MULTIPLY_BY_1_Z_CALC_OFFSET: usize =
    MULTIPLY_BY_1_Y_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const MULTIPLY_BY_1_TOTAL: usize =
    MULTIPLY_BY_1_Z_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;

// Forbenius map Fp6
/*
    These trace offsets are for forbenius_map (super::native::Fp6::forbenius_map) function. It needs 12 rows.
    FP6_FORBENIUS_MAP_DIV_OFFSET -> offset which stores integer division power/6.
    FP6_FORBENIUS_MAP_REM_OFFSET -> offset which stores power%6.
    FP6_FORBENIUS_MAP_BIT0_OFFSET, FP6_FORBENIUS_MAP_BIT1_OFFSET, FP6_FORBENIUS_MAP_BIT2_OFFSET -> offsets which store the bit decomposition of remainder (power%6).
*/
pub const FP6_FORBENIUS_MAP_SELECTOR_OFFSET: usize = 0;
pub const FP6_FORBENIUS_MAP_INPUT_OFFSET: usize = FP6_FORBENIUS_MAP_SELECTOR_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_POW_OFFSET: usize = FP6_FORBENIUS_MAP_INPUT_OFFSET + 24 * 3;
pub const FP6_FORBENIUS_MAP_DIV_OFFSET: usize = FP6_FORBENIUS_MAP_POW_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_REM_OFFSET: usize = FP6_FORBENIUS_MAP_DIV_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_BIT0_OFFSET: usize = FP6_FORBENIUS_MAP_REM_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_BIT1_OFFSET: usize = FP6_FORBENIUS_MAP_BIT0_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_BIT2_OFFSET: usize = FP6_FORBENIUS_MAP_BIT1_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_X_CALC_OFFSET: usize = FP6_FORBENIUS_MAP_BIT2_OFFSET + 1;
pub const FP6_FORBENIUS_MAP_T0_CALC_OFFSET: usize =
    FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_TOTAL_COLUMNS;
pub const FP6_FORBENIUS_MAP_Y_CALC_OFFSET: usize =
    FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_TOTAL_COLUMNS;
pub const FP6_FORBENIUS_MAP_T1_CALC_OFFSET: usize =
    FP6_FORBENIUS_MAP_Y_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP6_FORBENIUS_MAP_Z_CALC_OFFSET: usize =
    FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_TOTAL_COLUMNS;
pub const FP6_FORBENIUS_MAP_TOTAL_COLUMNS: usize =
    FP6_FORBENIUS_MAP_Z_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;

/// Fills the stark trace of fp6 addition. Inputs are 12*6 limbs each. Needs 1 row.
pub fn fill_trace_addition_fp6<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 6],
    y: &[[u32; 12]; 6],
    row: usize,
    start_col: usize,
) {
    fill_trace_addition_fp2(
        trace,
        &[x[0], x[1]],
        &[y[0], y[1]],
        row,
        start_col + FP6_ADDITION_0_OFFSET,
    );
    fill_trace_addition_fp2(
        trace,
        &[x[2], x[3]],
        &[y[2], y[3]],
        row,
        start_col + FP6_ADDITION_1_OFFSET,
    );
    fill_trace_addition_fp2(
        trace,
        &[x[4], x[5]],
        &[y[4], y[5]],
        row,
        start_col + FP6_ADDITION_2_OFFSET,
    );
}

/// Fills trace of fp6 addition combined with reduction and range check. Inputs are 12*6 limbs each. Needs 1 row.
pub fn fill_trace_addition_with_reduction_fp6<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    y: &Fp6,
    row: usize,
    start_col: usize,
) {
    fill_trace_addition_fp6(
        trace,
        &x.get_u32_slice(),
        &y.get_u32_slice(),
        row,
        start_col,
    );
    for i in 0..6 {
        let sum = get_u32_vec_from_literal(
            BigUint::new(x.0[i].0.to_vec()) + BigUint::new(y.0[i].0.to_vec()),
        );
        let rem = fill_trace_reduce_single(
            trace,
            &sum,
            row,
            start_col + FP6_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i,
        );
        fill_range_check_trace(
            trace,
            &rem,
            row,
            start_col
                + FP6_ADDITION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                + FP_SINGLE_REDUCE_TOTAL,
        );
    }
}

/// Fills trace of fp6 subtraction combined with reduction and range check. Inputs are 12*6 limbs each. Needs 1 row. Fills trace of adding field prime p to x first, and then the trace for subtraction with y.
pub fn fill_trace_subtraction_with_reduction_fp6<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    y: &Fp6,
    row: usize,
    start_col: usize,
) {
    let modulus = vec![get_u32_vec_from_literal(modulus()); 6]
        .try_into()
        .unwrap();
    fill_trace_addition_fp6(trace, &x.get_u32_slice(), &modulus, row, start_col);
    let x_modulus = modulus
        .iter()
        .zip(x.get_u32_slice())
        .map(|(m, f)| get_u32_vec_from_literal(BigUint::new(m.to_vec()) + BigUint::new(f.to_vec())))
        .collect::<Vec<[u32; 12]>>()
        .try_into()
        .unwrap();
    fill_trace_subtraction_fp6(
        trace,
        &x_modulus,
        &y.get_u32_slice(),
        row,
        start_col + FP6_ADDITION_TOTAL,
    );
    for i in 0..6 {
        let diff = get_u32_vec_from_literal(
            BigUint::new(x_modulus[i].to_vec()) - BigUint::new(y.0[i].0.to_vec()),
        );
        let rem = fill_trace_reduce_single(
            trace,
            &diff,
            row,
            start_col
                + FP6_ADDITION_TOTAL
                + FP6_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i,
        );
        fill_range_check_trace(
            trace,
            &rem,
            row,
            start_col
                + FP6_ADDITION_TOTAL
                + FP6_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                + FP_SINGLE_REDUCE_TOTAL,
        );
    }
}

/// Fills the stark trace of fp6 subtraction. Inputs are 12*6 limbs each. Needs 1 row. Assume x > y.
pub fn fill_trace_subtraction_fp6<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &[[u32; 12]; 6],
    y: &[[u32; 12]; 6],
    row: usize,
    start_col: usize,
) {
    fill_trace_subtraction_fp2(
        trace,
        &[x[0], x[1]],
        &[y[0], y[1]],
        row,
        start_col + FP6_SUBTRACTION_0_OFFSET,
    );
    fill_trace_subtraction_fp2(
        trace,
        &[x[2], x[3]],
        &[y[2], y[3]],
        row,
        start_col + FP6_SUBTRACTION_1_OFFSET,
    );
    fill_trace_subtraction_fp2(
        trace,
        &[x[4], x[5]],
        &[y[4], y[5]],
        row,
        start_col + FP6_SUBTRACTION_2_OFFSET,
    );
}

/// Fills the stark trace of negation. Input is 12*6 limbs. Needs 1 row. In essence, it fills an addition trace with inputs as `x` and `-x`.
pub fn fill_trace_negate_fp6<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    row: usize,
    start_col: usize,
) {
    fill_trace_addition_fp6(
        trace,
        &x.get_u32_slice(),
        &(-(*x)).get_u32_slice(),
        row,
        start_col,
    );
}

/// Fills trace of [mul_by_nonresidue](super::native::mul_by_nonresidue) function. Input is 12*6 limbs. Needs 1 row.
pub fn fill_trace_non_residue_multiplication_fp6<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    row: usize,
    start_col: usize,
) {
    trace[row][start_col + FP6_NON_RESIDUE_MUL_CHECK_OFFSET] = F::ONE;
    for (i, e) in x.0.iter().enumerate() {
        assign_u32_in_series(
            trace,
            row,
            start_col + FP6_NON_RESIDUE_MUL_INPUT_OFFSET + i * 12,
            &e.0,
        );
    }
    let c2 = Fp2([x.0[4], x.0[5]]);
    fill_trace_non_residue_multiplication(
        trace,
        &c2.get_u32_slice(),
        row,
        start_col + FP6_NON_RESIDUE_MUL_C2,
    );
}

/// Fills stark trace for fp6 multiplication. Inputs are 12*6 limbs each. Needs 12 rows.
pub fn fill_trace_fp6_multiplication<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    y: &Fp6,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for i in 0..6 {
        for row in start_row..end_row + 1 {
            assign_u32_in_series(
                trace,
                row,
                start_col + FP6_MUL_X_INPUT_OFFSET + 12 * i,
                &x.0[i].0,
            );
            assign_u32_in_series(
                trace,
                row,
                start_col + FP6_MUL_Y_INPUT_OFFSET + 12 * i,
                &y.0[i].0,
            );
            trace[row][start_col + FP6_MUL_SELECTOR_OFFSET] = F::ONE;
        }
    }
    trace[end_row][start_col + FP6_MUL_SELECTOR_OFFSET] = F::ZERO;
    let (c0, c1, c2) = (
        Fp2([x.0[0], x.0[1]]),
        Fp2([x.0[2], x.0[3]]),
        Fp2([x.0[4], x.0[5]]),
    );
    let (r0, r1, r2) = (
        Fp2([y.0[0], y.0[1]]),
        Fp2([y.0[2], y.0[3]]),
        Fp2([y.0[4], y.0[5]]),
    );

    let t0 = c0 * r0;
    generate_trace_fp2_mul(
        trace,
        c0.get_u32_slice(),
        r0.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_MUL_T0_CALC_OFFSET,
    );
    let t1 = c1 * r1;
    generate_trace_fp2_mul(
        trace,
        c1.get_u32_slice(),
        r1.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_MUL_T1_CALC_OFFSET,
    );
    let t2 = c2 * r2;
    generate_trace_fp2_mul(
        trace,
        c2.get_u32_slice(),
        r2.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_MUL_T2_CALC_OFFSET,
    );

    let t3 = c1 + c2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &c1.get_u32_slice(),
            &c2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T3_CALC_OFFSET,
        );
    }
    let t4 = r1 + r2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &r1.get_u32_slice(),
            &r2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T4_CALC_OFFSET,
        );
    }
    let t5 = t3 * t4;
    generate_trace_fp2_mul(
        trace,
        t3.get_u32_slice(),
        t4.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_MUL_T5_CALC_OFFSET,
    );
    let t6 = t5 - t1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t5.get_u32_slice(),
            &t1.get_u32_slice(),
            row,
            start_col + FP6_MUL_T6_CALC_OFFSET,
        );
    }
    let t7 = t6 - t2;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t6.get_u32_slice(),
            &t2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T7_CALC_OFFSET,
        );
    }
    let t8 = t7.mul_by_nonresidue();
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication(
            trace,
            &t7.get_u32_slice(),
            row,
            start_col + FP6_MUL_T8_CALC_OFFSET,
        );
    }
    let _x = t8 + t0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t8.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + FP6_MUL_X_CALC_OFFSET,
        );
    }

    let t9 = c0 + c1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &c0.get_u32_slice(),
            &c1.get_u32_slice(),
            row,
            start_col + FP6_MUL_T9_CALC_OFFSET,
        );
    }
    let t10 = r0 + r1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &r0.get_u32_slice(),
            &r1.get_u32_slice(),
            row,
            start_col + FP6_MUL_T10_CALC_OFFSET,
        );
    }
    let t11 = t9 * t10;
    generate_trace_fp2_mul(
        trace,
        t9.get_u32_slice(),
        t10.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_MUL_T11_CALC_OFFSET,
    );
    let t12 = t11 - t0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t11.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + FP6_MUL_T12_CALC_OFFSET,
        );
    }
    let t13 = t12 - t1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t12.get_u32_slice(),
            &t1.get_u32_slice(),
            row,
            start_col + FP6_MUL_T13_CALC_OFFSET,
        );
    }
    let t14 = t2.mul_by_nonresidue();
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication(
            trace,
            &t2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T14_CALC_OFFSET,
        );
    }
    let _y = t13 + t14;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t13.get_u32_slice(),
            &t14.get_u32_slice(),
            row,
            start_col + FP6_MUL_Y_CALC_OFFSET,
        );
    }

    let t15 = c0 + c2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &c0.get_u32_slice(),
            &c2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T15_CALC_OFFSET,
        );
    }
    let t16 = r0 + r2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &r0.get_u32_slice(),
            &r2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T16_CALC_OFFSET,
        );
    }
    let t17 = t15 * t16;
    generate_trace_fp2_mul(
        trace,
        t15.get_u32_slice(),
        t16.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_MUL_T17_CALC_OFFSET,
    );
    let t18 = t17 - t0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t17.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + FP6_MUL_T18_CALC_OFFSET,
        );
    }
    let t19 = t18 - t2;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t18.get_u32_slice(),
            &t2.get_u32_slice(),
            row,
            start_col + FP6_MUL_T19_CALC_OFFSET,
        );
    }
    let _z = t19 + t1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t19.get_u32_slice(),
            &t1.get_u32_slice(),
            row,
            start_col + FP6_MUL_Z_CALC_OFFSET,
        );
    }
}

/// Fills trace of [multiplyBy1](super::native::Fp6::multiplyBy1) function. Input is 12\*6 limbs and 12\*2 limbs. Needs 12 rows.
pub fn fill_trace_multiply_by_1<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    b1: &Fp2,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        for i in 0..6 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_1_INPUT_OFFSET + i * 12,
                &x.0[i].0,
            );
        }
        for i in 0..2 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_1_B1_OFFSET + i * 12,
                &b1.0[i].0,
            );
        }
        trace[row][start_col + MULTIPLY_BY_1_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + MULTIPLY_BY_1_SELECTOR_OFFSET] = F::ZERO;

    let c0 = Fp2([x.0[0], x.0[1]]);
    let c1 = Fp2([x.0[2], x.0[3]]);
    let c2 = Fp2([x.0[4], x.0[5]]);
    let t0 = c2 * (*b1);
    generate_trace_fp2_mul(
        trace,
        c2.get_u32_slice(),
        b1.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_1_T0_CALC_OFFSET,
    );
    let _x = t0.mul_by_nonresidue();
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication(
            trace,
            &t0.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_1_X_CALC_OFFSET,
        );
    }
    let _y = c0 * (*b1);
    generate_trace_fp2_mul(
        trace,
        c0.get_u32_slice(),
        b1.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_1_Y_CALC_OFFSET,
    );
    let _z = c1 * (*b1);
    generate_trace_fp2_mul(
        trace,
        c1.get_u32_slice(),
        b1.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_1_Z_CALC_OFFSET,
    );
}

/// Fills trace of [multiplyBy01](super::native::Fp6::multiplyBy01) function. Input is 12\*6 limbs and two 12\*2 limbs. Needs 12 rows.
pub fn fill_trace_multiply_by_01<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    b0: &Fp2,
    b1: &Fp2,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        for i in 0..6 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_01_INPUT_OFFSET + i * 12,
                &x.0[i].0,
            );
        }
        for i in 0..2 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_01_B0_OFFSET + i * 12,
                &b0.0[i].0,
            );
        }
        for i in 0..2 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_01_B1_OFFSET + i * 12,
                &b1.0[i].0,
            );
        }
        trace[row][start_col + MULTIPLY_BY_01_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + MULTIPLY_BY_01_SELECTOR_OFFSET] = F::ZERO;

    let c0 = Fp2([x.0[0], x.0[1]]);
    let c1 = Fp2([x.0[2], x.0[3]]);
    let c2 = Fp2([x.0[4], x.0[5]]);

    let t0 = c0 * (*b0);
    generate_trace_fp2_mul(
        trace,
        c0.get_u32_slice(),
        b0.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_01_T0_CALC_OFFSET,
    );
    let t1 = c1 * (*b1);
    generate_trace_fp2_mul(
        trace,
        c1.get_u32_slice(),
        b1.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_01_T1_CALC_OFFSET,
    );

    let t2 = c2 * (*b1);
    generate_trace_fp2_mul(
        trace,
        c2.get_u32_slice(),
        b1.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_01_T2_CALC_OFFSET,
    );
    let t3 = t2.mul_by_nonresidue();
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication(
            trace,
            &t2.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_T3_CALC_OFFSET,
        );
    }
    let _x = t3 + t0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t3.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_X_CALC_OFFSET,
        );
    }

    let t4 = (*b0) + (*b1);
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &b0.get_u32_slice(),
            &b1.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_T4_CALC_OFFSET,
        );
    }
    let t5 = c0 + c1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &c0.get_u32_slice(),
            &c1.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_T5_CALC_OFFSET,
        );
    }
    let t6 = t4 * t5;
    generate_trace_fp2_mul(
        trace,
        t4.get_u32_slice(),
        t5.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_01_T6_CALC_OFFSET,
    );
    let t7 = t6 - t0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t6.get_u32_slice(),
            &t0.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_T7_CALC_OFFSET,
        );
    }
    let _y = t7 - t1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t7.get_u32_slice(),
            &t1.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_Y_CALC_OFFSET,
        );
    }

    let t8 = c2 * (*b0);
    generate_trace_fp2_mul(
        trace,
        c2.get_u32_slice(),
        b0.get_u32_slice(),
        start_row,
        end_row,
        start_col + MULTIPLY_BY_01_T8_CALC_OFFSET,
    );
    let _z = t8 + t1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t8.get_u32_slice(),
            &t1.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_01_Z_CALC_OFFSET,
        );
    }
}

/// Fills trace of [forbenius_map](super::native::Fp6::forbenius_map) function. Input is 12*6 limbs and usize. Needs 12 rows.
pub fn fill_trace_fp6_forbenius_map<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp6,
    pow: usize,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    let div = pow / 6;
    let rem = pow % 6;
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET,
            &x.get_u32_slice().concat(),
        );
        trace[row][start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET] = F::ONE;
        trace[row][start_col + FP6_FORBENIUS_MAP_POW_OFFSET] = F::from_canonical_usize(pow);
        trace[row][start_col + FP6_FORBENIUS_MAP_DIV_OFFSET] = F::from_canonical_usize(div);
        trace[row][start_col + FP6_FORBENIUS_MAP_REM_OFFSET] = F::from_canonical_usize(rem);
        trace[row][start_col + FP6_FORBENIUS_MAP_BIT0_OFFSET] = F::from_canonical_usize(rem & 1);
        trace[row][start_col + FP6_FORBENIUS_MAP_BIT1_OFFSET] =
            F::from_canonical_usize((rem >> 1) & 1);
        trace[row][start_col + FP6_FORBENIUS_MAP_BIT2_OFFSET] = F::from_canonical_usize(rem >> 2);
    }
    trace[end_row][start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET] = F::ZERO;
    let c0 = Fp2(x.0[0..2].to_vec().try_into().unwrap());
    let c1 = Fp2(x.0[2..4].to_vec().try_into().unwrap());
    let c2 = Fp2(x.0[4..6].to_vec().try_into().unwrap());
    let forbenius_coefficients_1 = Fp6::forbenius_coefficients_1();
    let forbenius_coefficients_2 = Fp6::forbenius_coefficients_2();
    let _x = c0.forbenius_map(pow);
    fill_trace_fp2_forbenius_map(
        trace,
        &c0,
        pow,
        start_row,
        end_row,
        start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET,
    );
    let t0 = c1.forbenius_map(pow);
    fill_trace_fp2_forbenius_map(
        trace,
        &c1,
        pow,
        start_row,
        end_row,
        start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET,
    );
    let _y = t0 * forbenius_coefficients_1[pow % 6];
    generate_trace_fp2_mul(
        trace,
        t0.get_u32_slice(),
        forbenius_coefficients_1[pow % 6].get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET,
    );
    let t1 = c2.forbenius_map(pow);
    fill_trace_fp2_forbenius_map(
        trace,
        &c2,
        pow,
        start_row,
        end_row,
        start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET,
    );
    let _z = t1 * forbenius_coefficients_2[pow % 6];
    generate_trace_fp2_mul(
        trace,
        t1.get_u32_slice(),
        forbenius_coefficients_2[pow % 6].get_u32_slice(),
        start_row,
        end_row,
        start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET,
    );
}

/// Constraints fp2 addition. In essence, constraints three Fp2 addititons.
pub fn add_addition_fp6_constraints<
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
    add_addition_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP6_ADDITION_0_OFFSET,
        bit_selector,
    );
    add_addition_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP6_ADDITION_1_OFFSET,
        bit_selector,
    );
    add_addition_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP6_ADDITION_2_OFFSET,
        bit_selector,
    );
}

pub fn add_addition_fp6_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    add_addition_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_ADDITION_0_OFFSET,
        bit_selector,
    );
    add_addition_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_ADDITION_1_OFFSET,
        bit_selector,
    );
    add_addition_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_ADDITION_2_OFFSET,
        bit_selector,
    );
}

/// Constraints fp6 addition followed by reduction and range check constraints.
pub fn add_addition_with_reduction_constranints_fp6<
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

    add_addition_fp6_constraints(local_values, yield_constr, start_col, bit_selector);
    for j in 0..6 {
        let fp2_offset = if j < 2 {
            FP6_ADDITION_0_OFFSET
        } else if j < 4 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        let fp_offset = if j % 2 == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + fp2_offset + fp_offset + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [start_col + fp2_offset + fp_offset + FP_ADDITION_SUM_OFFSET + i]
                        - local_values[start_col
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCE_X_OFFSET
                            + i]),
            );
        }
        add_fp_reduce_single_constraints(
            local_values,
            yield_constr,
            start_col + FP6_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j,
            bit_selector,
        );
        add_range_check_constraints(
            local_values,
            yield_constr,
            start_col
                + FP6_ADDITION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                + FP_SINGLE_REDUCE_TOTAL,
            bit_selector,
        );
    }
}

pub fn add_addition_with_reduction_constraints_fp6_ext_circuit<
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

    add_addition_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );
    for j in 0..6 {
        let fp2_offset = if j < 2 {
            FP6_ADDITION_0_OFFSET
        } else if j < 4 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        let fp_offset = if j % 2 == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for i in 0..12 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col + fp2_offset + fp_offset + FP_ADDITION_SUM_OFFSET + i],
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCE_X_OFFSET
                    + i],
            );
            let c = builder.mul_extension(
                local_values[start_col + fp2_offset + fp_offset + FP_ADDITION_CHECK_OFFSET],
                sub_tmp,
            );
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
        add_fp_reduce_single_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            start_col + FP6_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j,
            bit_selector,
        );
        add_range_check_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            start_col
                + FP6_ADDITION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                + FP_SINGLE_REDUCE_TOTAL,
            bit_selector,
        );
    }
}

/// Constraints fp6 subtraction. In essence, constraints three Fp2 subtractions.
pub fn add_subtraction_fp6_constraints<
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
    add_subtraction_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP6_SUBTRACTION_0_OFFSET,
        bit_selector,
    );
    add_subtraction_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP6_SUBTRACTION_1_OFFSET,
        bit_selector,
    );
    add_subtraction_fp2_constraints(
        local_values,
        yield_constr,
        start_col + FP6_SUBTRACTION_2_OFFSET,
        bit_selector,
    );
}

pub fn add_subtraction_fp6_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    add_subtraction_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_SUBTRACTION_0_OFFSET,
        bit_selector,
    );
    add_subtraction_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_SUBTRACTION_1_OFFSET,
        bit_selector,
    );
    add_subtraction_fp2_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_SUBTRACTION_2_OFFSET,
        bit_selector,
    );
}

/// Constraints fp6 negation. First add constraints for fp6 addition. Followed by constraining the result of the addition with bls12-381 field prime p.
pub fn add_negate_fp6_constraints<
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

    add_addition_fp6_constraints(local_values, yield_constr, start_col, bit_selector);
    let mod_u32 = get_u32_vec_from_literal(modulus());
    for i in 0..12 {
        for j in 0..3 {
            let fp2_offset = if j == 0 {
                FP6_ADDITION_0_OFFSET
            } else if j == 1 {
                FP6_ADDITION_1_OFFSET
            } else {
                FP6_ADDITION_2_OFFSET
            };
            for k in 0..2 {
                let fp_offset = if k == 0 {
                    FP2_ADDITION_0_OFFSET
                } else {
                    FP2_ADDITION_1_OFFSET
                };
                yield_constr.constraint(
                    bit_selector_val
                        * local_values
                            [start_col + fp2_offset + fp_offset + FP_ADDITION_CHECK_OFFSET]
                        * (local_values
                            [start_col + fp2_offset + fp_offset + FP_ADDITION_SUM_OFFSET + i]
                            - FE::from_canonical_u32(mod_u32[i])),
                );
            }
        }
    }
}

pub fn add_negate_fp6_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    add_addition_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );
    let mod_u32 = get_u32_vec_from_literal(modulus());
    for i in 0..12 {
        for j in 0..3 {
            let fp2_offset = if j == 0 {
                FP6_ADDITION_0_OFFSET
            } else if j == 1 {
                FP6_ADDITION_1_OFFSET
            } else {
                FP6_ADDITION_2_OFFSET
            };
            for k in 0..2 {
                let lc = builder.constant_extension(F::Extension::from_canonical_u32(mod_u32[i]));
                let fp_offset = if k == 0 {
                    FP2_ADDITION_0_OFFSET
                } else {
                    FP2_ADDITION_1_OFFSET
                };

                let sub_tmp = builder.sub_extension(
                    local_values[start_col + fp2_offset + fp_offset + FP_ADDITION_SUM_OFFSET + i],
                    lc,
                );
                let c = builder.mul_extension(
                    local_values[start_col + fp2_offset + fp_offset + FP_ADDITION_CHECK_OFFSET],
                    sub_tmp,
                );
                let c = builder.mul_extension(bit_selector_val, c);
                yield_constr.constraint(builder, c);
            }
        }
    }
}

/// Constraints fp6 subtraction followed by reduction and range check constraints. First, constraints of adding field prime p to x to prevent overflow, because x > y assumption is not valid here. Then constraints the subtraction operation. Then reduce and range check constraints.
pub fn add_subtraction_with_reduction_constranints_fp6<
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

    add_addition_fp6_constraints(local_values, yield_constr, start_col, bit_selector);
    add_subtraction_fp6_constraints(
        local_values,
        yield_constr,
        start_col + FP6_ADDITION_TOTAL,
        bit_selector,
    );

    let modulus = get_u32_vec_from_literal(modulus());
    for j in 0..6 {
        let (fp2_add_offset, fp2_sub_offset) = if j < 2 {
            (FP6_ADDITION_0_OFFSET, FP6_SUBTRACTION_0_OFFSET)
        } else if j < 4 {
            (FP6_ADDITION_1_OFFSET, FP6_SUBTRACTION_1_OFFSET)
        } else {
            (FP6_ADDITION_2_OFFSET, FP6_SUBTRACTION_2_OFFSET)
        };
        let (_fp_add_offset, fp_sub_offset) = if j % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
        } else {
            (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
        };
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + fp2_add_offset
                        + FP2_ADDITION_0_OFFSET
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + fp2_add_offset
                        + FP2_ADDITION_0_OFFSET
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - FE::from_canonical_u32(modulus[i])),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + fp2_add_offset
                        + FP2_ADDITION_1_OFFSET
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + fp2_add_offset
                        + FP2_ADDITION_1_OFFSET
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - FE::from_canonical_u32(modulus[i])),
            );
        }
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + fp2_add_offset
                            + FP2_ADDITION_0_OFFSET
                            + FP_ADDITION_SUM_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + fp2_add_offset
                            + FP2_ADDITION_1_OFFSET
                            + FP_ADDITION_SUM_OFFSET
                            + i]),
            );
        }
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + fp_sub_offset
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + fp_sub_offset
                        + FP_SUBTRACTION_DIFF_OFFSET
                        + i]
                        - local_values[start_col
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCE_X_OFFSET
                            + i]),
            );
        }
        add_fp_reduce_single_constraints(
            local_values,
            yield_constr,
            start_col
                + FP6_ADDITION_TOTAL
                + FP6_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j,
            bit_selector,
        );
        add_range_check_constraints(
            local_values,
            yield_constr,
            start_col
                + FP6_ADDITION_TOTAL
                + FP6_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                + FP_SINGLE_REDUCE_TOTAL,
            bit_selector,
        );
    }
}

pub fn add_subtraction_with_reduction_constraints_fp6_ext_circuit<
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

    add_addition_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col,
        bit_selector,
    );
    add_subtraction_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_ADDITION_TOTAL,
        bit_selector,
    );
    let modulus = get_u32_vec_from_literal(modulus());
    for j in 0..6 {
        let (fp2_add_offset, fp2_sub_offset) = if j < 2 {
            (FP6_ADDITION_0_OFFSET, FP6_SUBTRACTION_0_OFFSET)
        } else if j < 4 {
            (FP6_ADDITION_1_OFFSET, FP6_SUBTRACTION_1_OFFSET)
        } else {
            (FP6_ADDITION_2_OFFSET, FP6_SUBTRACTION_2_OFFSET)
        };
        let (_fp_add_offset, fp_sub_offset) = if j % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
        } else {
            (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
        };
        for i in 0..12 {
            let lc = builder.constant_extension(F::Extension::from_canonical_u32(modulus[i]));

            let sub_tmp1 = builder.sub_extension(
                local_values
                    [start_col + fp2_add_offset + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i],
                lc,
            );
            let c1 = builder.mul_extension(
                local_values
                    [start_col + fp2_add_offset + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
                sub_tmp1,
            );
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values
                    [start_col + fp2_add_offset + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i],
                lc,
            );
            let c2 = builder.mul_extension(
                local_values
                    [start_col + fp2_add_offset + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
                sub_tmp2,
            );
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }

        for i in 0..12 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + fp2_sub_offset
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_X_OFFSET
                    + i],
                local_values[start_col
                    + fp2_add_offset
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_SUM_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(
                sub_tmp1,
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + fp2_sub_offset
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + fp2_sub_offset
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_X_OFFSET
                    + i],
                local_values[start_col
                    + fp2_add_offset
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_SUM_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(
                sub_tmp2,
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + fp2_sub_offset
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
        for i in 0..12 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + fp2_sub_offset
                    + fp_sub_offset
                    + FP_SUBTRACTION_DIFF_OFFSET
                    + i],
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCE_X_OFFSET
                    + i],
            );
            let c = builder.mul_extension(
                sub_tmp,
                local_values[start_col
                    + FP6_ADDITION_TOTAL
                    + fp2_sub_offset
                    + fp_sub_offset
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
        add_fp_reduce_single_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            start_col
                + FP6_ADDITION_TOTAL
                + FP6_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j,
            bit_selector,
        );
        add_range_check_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            start_col
                + FP6_ADDITION_TOTAL
                + FP6_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                + FP_SINGLE_REDUCE_TOTAL,
            bit_selector,
        );
    }
}

/// Constraints [mul_by_nonresidue](super::native::mul_by_nonresidue) function.
pub fn add_non_residue_multiplication_fp6_constraints<
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
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col + FP6_NON_RESIDUE_MUL_INPUT_OFFSET + i + 48]
                    - local_values[start_col
                        + FP6_NON_RESIDUE_MUL_C2
                        + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + FP6_NON_RESIDUE_MUL_C2,
        bit_selector,
    );
}

pub fn add_non_residue_multiplication_fp6_constraints_ext_circuit<
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

    for i in 0..24 {
        let sub_tmp = builder.sub_extension(
            local_values[start_col + FP6_NON_RESIDUE_MUL_INPUT_OFFSET + i + 48],
            local_values[start_col + FP6_NON_RESIDUE_MUL_C2 + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(
            sub_tmp,
            local_values[start_col + FP6_NON_RESIDUE_MUL_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_NON_RESIDUE_MUL_C2,
        bit_selector,
    );
}

/// Constraints fp6 multiplication.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints all the Ti's (defined in the [function](super::native::mul_Fp6)) accordinng to their respective operations.
pub fn add_fp6_multiplication_constraints<
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
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    for i in 0..24 * 3 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP6_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i]
                    - next_values[start_col + FP6_MUL_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP6_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i]
                    - next_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i]),
        );
    }

    // T0
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i]
                    - local_values
                        [start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i]
                    - local_values
                        [start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_MUL_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24]
                    - local_values
                        [start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24]
                    - local_values
                        [start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_MUL_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48]
                    - local_values
                        [start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48]
                    - local_values
                        [start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_MUL_T2_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24 + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48 + 12]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T3_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24 + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48 + 12]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_MUL_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T5_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T5_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T6_CALC_OFFSET,
        bit_selector,
    );

    // T7
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T2_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T2_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T7_CALC_OFFSET,
        bit_selector,
    );

    // T8
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values
                    [start_col + FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP6_MUL_T7_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T8_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + FP6_MUL_T7_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T8_CALC_OFFSET,
        bit_selector,
    );

    // X calc offset
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T8_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T8_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_X_CALC_OFFSET,
        bit_selector,
    );

    // T9
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24 + 12]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T9_CALC_OFFSET,
        bit_selector,
    );

    // T10
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24 + 12]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T10_CALC_OFFSET,
        bit_selector,
    );

    // T11
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T9_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T10_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_MUL_T11_CALC_OFFSET,
        bit_selector,
    );

    // T12
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T11_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T11_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T12_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T12_CALC_OFFSET,
        bit_selector,
    );

    // T13
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T12_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T12_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T13_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T13_CALC_OFFSET,
        bit_selector,
    );

    // T14
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values
                    [start_col + FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP6_MUL_T2_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T14_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + FP6_MUL_T2_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T14_CALC_OFFSET,
        bit_selector,
    );

    // Y calc offset
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T13_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T13_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T14_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T14_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_Y_CALC_OFFSET,
        bit_selector,
    );

    // T15
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48 + 12]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T15_CALC_OFFSET,
        bit_selector,
    );

    // T16
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48 + 12]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T16_CALC_OFFSET,
        bit_selector,
    );

    // T17
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T15_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_MUL_T17_CALC_OFFSET,
        bit_selector,
    );

    // T18
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T17_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T17_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T18_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T18_CALC_OFFSET,
        bit_selector,
    );

    // T19
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T18_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T18_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T2_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_T19_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T2_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_T19_CALC_OFFSET,
        bit_selector,
    );

    // Z calc offset
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T19_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T19_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + FP6_MUL_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + FP6_MUL_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }

    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + FP6_MUL_Z_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_fp6_multiplication_constraints_ext_circuit<
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

    for i in 0..24 * 3 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i],
            next_values[start_col + FP6_MUL_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, local_values[start_col + FP6_MUL_SELECTOR_OFFSET]);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint_transition(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i],
            next_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, local_values[start_col + FP6_MUL_SELECTOR_OFFSET]);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint_transition(builder, c);
    }

    // T0
    for i in 0..24 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i],
            local_values[start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i],
            local_values[start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col + FP6_MUL_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_MUL_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..24 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24],
            local_values[start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24],
            local_values[start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col + FP6_MUL_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_MUL_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for i in 0..24 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48],
            local_values[start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48],
            local_values[start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col + FP6_MUL_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_MUL_T2_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24 + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48 + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T3_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24 + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48 + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..12 {
        let mul_tmp = local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T3_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c3 = builder.mul_extension(sub_tmp3, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T4_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T5_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
        );
        let c4 = builder.mul_extension(sub_tmp4, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_MUL_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T5_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T5_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T6_CALC_OFFSET,
        bit_selector,
    );

    // T7
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T6_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T7_CALC_OFFSET,
        bit_selector,
    );

    // T8
    for i in 0..12 {
        let mul_tmp =
            local_values[start_col + FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values
                [start_col + FP6_MUL_T8_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i + 12],
            local_values[start_col
                + FP6_MUL_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T8_CALC_OFFSET,
        bit_selector,
    );

    // X calc offset
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T8_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T8_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_X_CALC_OFFSET,
        bit_selector,
    );

    // T9
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 24 + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T9_CALC_OFFSET,
        bit_selector,
    );

    // T10
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 24 + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T10_CALC_OFFSET,
        bit_selector,
    );

    // T11
    for i in 0..12 {
        let mul_tmp = local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T9_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c3 = builder.mul_extension(sub_tmp3, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T10_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
        );
        let c4 = builder.mul_extension(sub_tmp4, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_MUL_T11_CALC_OFFSET,
        bit_selector,
    );

    // T12
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T11_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T11_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T12_CALC_OFFSET,
        bit_selector,
    );

    // T13
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T12_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T13_CALC_OFFSET,
        bit_selector,
    );

    // T14
    for i in 0..12 {
        let mul_tmp =
            local_values[start_col + FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values
                [start_col + FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
            local_values
                [start_col + FP6_MUL_T2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values
                [start_col + FP6_MUL_T14_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i + 12],
            local_values
                [start_col + FP6_MUL_T2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T14_CALC_OFFSET,
        bit_selector,
    );

    // Y calc offset
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T13_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T14_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T14_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_Y_CALC_OFFSET,
        bit_selector,
    );

    // T15
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_X_INPUT_OFFSET + i + 48 + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T15_CALC_OFFSET,
        bit_selector,
    );

    // T16
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + FP6_MUL_Y_INPUT_OFFSET + i + 48 + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T16_CALC_OFFSET,
        bit_selector,
    );

    // T17
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T15_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T16_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col + FP6_MUL_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_MUL_T17_CALC_OFFSET,
        bit_selector,
    );

    // T18
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T17_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T17_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T18_CALC_OFFSET,
        bit_selector,
    );

    // T19
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T18_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_T19_CALC_OFFSET,
        bit_selector,
    );

    // Z calc offset
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + FP6_MUL_T19_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + FP6_MUL_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + FP6_MUL_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP6_MUL_Z_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints [multiplyBy1](super::native::Fp6::multiplyBy1) function.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints all the Ti's (defined in the native function) accordinng to their respective operations.
pub fn add_multiply_by_1_constraints<
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
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    for i in 0..24 {
        for j in 0..3 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_BY_1_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + j * 24 + i]
                        - next_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + j * 24 + i]),
            );
        }
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_SELECTOR_OFFSET]
                * (local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i]
                    - next_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i]),
        );
    }

    // T0
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_1_T0_CALC_OFFSET,
        bit_selector,
    );

    // X
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + MULTIPLY_BY_1_X_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_1_X_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_1_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + MULTIPLY_BY_1_X_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_1_X_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + MULTIPLY_BY_1_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_1_X_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_1_Y_CALC_OFFSET,
        bit_selector,
    );

    // Z
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_1_Z_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_multiply_by_1_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..24 {
        let mul_tmp = local_values[start_col + MULTIPLY_BY_1_SELECTOR_OFFSET];
        for j in 0..3 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + j * 24 + i],
                next_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + j * 24 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint_transition(builder, c);
        }
        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i],
            next_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint_transition(builder, c);
    }

    // T0
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + i + 48],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_1_T0_CALC_OFFSET,
        bit_selector,
    );

    // X
    for i in 0..12 {
        let mul_tmp = local_values
            [start_col + MULTIPLY_BY_1_X_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values
                [start_col + MULTIPLY_BY_1_X_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
            local_values
                [start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_1_X_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                + i
                + 12],
            local_values
                [start_col + MULTIPLY_BY_1_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_1_X_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_Y_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_1_Y_CALC_OFFSET,
        bit_selector,
    );

    // Z
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_1_INPUT_OFFSET + i + 24],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_1_Z_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_1_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_1_Z_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints [multiplyBy01](super::native::Fp6::multiplyBy01) function.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints all the Ti's (defined in the native function) accordinng to their respective operations.
pub fn add_multiply_by_01_constraints<
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
    let bit_selector_val = bit_selector.unwrap_or(P::ONES);

    for i in 0..24 {
        for j in 0..3 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + j * 24 + i]
                        - next_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + j * 24 + i]),
            );
        }
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_SELECTOR_OFFSET]
                * (local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i]
                    - next_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_SELECTOR_OFFSET]
                * (local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i]
                    - next_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i]),
        );
    }

    // T0
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T2_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + MULTIPLY_BY_01_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T3_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T2_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values
                    [start_col + MULTIPLY_BY_01_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T3_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T2_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T3_CALC_OFFSET,
        bit_selector,
    );

    // X
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T3_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T3_CALC_OFFSET
                        + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_X_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i + 12]),
        );
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 24]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 36]),
        );
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T6_CALC_OFFSET
                        + FP2_FP2_X_INPUT_OFFSET
                        + i
                        + 12]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values
                        [start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T5_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T6_CALC_OFFSET
                        + FP2_FP2_Y_INPUT_OFFSET
                        + i
                        + 12]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T6_CALC_OFFSET,
        bit_selector,
    );

    // T7
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T6_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T6_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T0_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_T7_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T0_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T7_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T7_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T7_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_Y_CALC_OFFSET,
        bit_selector,
    );

    // T8
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 48]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i]),
        )
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_T8_CALC_OFFSET,
        bit_selector,
    );

    // Z
    for i in 0..12 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T8_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_X_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T8_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_0_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T1_CALC_OFFSET
                        + Z1_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_CHECK_OFFSET]
                * (local_values[start_col
                    + MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_1_OFFSET
                    + FP_ADDITION_Y_OFFSET
                    + i]
                    - local_values[start_col
                        + MULTIPLY_BY_01_T1_CALC_OFFSET
                        + Z2_REDUCE_OFFSET
                        + REDUCED_OFFSET
                        + i]),
        );
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_01_Z_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_multiply_by_01_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..24 {
        for j in 0..3 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + j * 24 + i],
                next_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + j * 24 + i],
            );
            let c = builder.mul_extension(
                sub_tmp,
                local_values[start_col + MULTIPLY_BY_01_SELECTOR_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint_transition(builder, c);
        }

        let mul_tmp = local_values[start_col + MULTIPLY_BY_01_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i],
            next_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint_transition(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i],
            next_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint_transition(builder, c);
    }

    // T0
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_01_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 24],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_01_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 48],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_01_T2_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for i in 0..12 {
        let mul_tmp = local_values
            [start_col + MULTIPLY_BY_01_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values
                [start_col + MULTIPLY_BY_01_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
            local_values
                [start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T3_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                + i
                + 12],
            local_values
                [start_col + MULTIPLY_BY_01_T2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_T3_CALC_OFFSET,
        bit_selector,
    );

    // X
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + MULTIPLY_BY_01_T3_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + MULTIPLY_BY_01_T3_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + MULTIPLY_BY_01_X_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_X_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_B1_OFFSET + i + 12],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 24],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 36],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for i in 0..12 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T4_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values[start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
        );
        let c3 = builder.mul_extension(sub_tmp3, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T5_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
        );
        let c4 = builder.mul_extension(sub_tmp4, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_01_T6_CALC_OFFSET,
        bit_selector,
    );

    // T7
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T6_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_T7_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values[start_col
                + MULTIPLY_BY_01_T7_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCE_TOTAL
                + RANGE_CHECK_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_0_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + MULTIPLY_BY_01_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_1_OFFSET
                + FP_SUBTRACTION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_Y_CALC_OFFSET,
        bit_selector,
    );

    // T8
    for i in 0..24 {
        let mul_tmp =
            local_values[start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_INPUT_OFFSET + i + 48],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            local_values[start_col + MULTIPLY_BY_01_B0_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_01_T8_CALC_OFFSET,
        bit_selector,
    );

    // Z
    for i in 0..12 {
        let sub_tmp1 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c1 = builder.mul_extension(
            sub_tmp1,
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_X_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c2 = builder.mul_extension(
            sub_tmp2,
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);

        let sub_tmp3 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c3 = builder.mul_extension(
            sub_tmp3,
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_0_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c3);
        yield_constr.constraint(builder, c);

        let sub_tmp4 = builder.sub_extension(
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_Y_OFFSET
                + i],
            local_values
                [start_col + MULTIPLY_BY_01_T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
        );
        let c4 = builder.mul_extension(
            sub_tmp4,
            local_values[start_col
                + MULTIPLY_BY_01_Z_CALC_OFFSET
                + FP2_ADDITION_1_OFFSET
                + FP_ADDITION_CHECK_OFFSET],
        );
        let c = builder.mul_extension(bit_selector_val, c4);
        yield_constr.constraint(builder, c);
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_01_Z_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints for [forbenius_map](super::native::Fp6::forbenius_map) function.
///
///  Constraints both input and power across this and next row, wherever selector is set to on. Constraint the divisor and remainder with power for `power == divisor*6 + remainder`. Constraints the bit decomposition as `remainder == bit0 + bit1*2 + bit2*4`. Selects the forbenius constant using mupliplexer logic. Then constraints fp2 forbenius map, multiplication, reduction and range check operations.
pub fn add_fp6_forbenius_map_constraints<
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
    for i in 0..24 * 3 {
        yield_constr.constraint_transition(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i]
                    - next_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i]),
        );
    }
    yield_constr.constraint_transition(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]
                - next_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]),
    );
    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values[start_col + FP6_FORBENIUS_MAP_DIV_OFFSET]
                * FE::from_canonical_usize(6)
                + local_values[start_col + FP6_FORBENIUS_MAP_REM_OFFSET]
                - local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]),
    );
    let bit0 = local_values[start_col + FP6_FORBENIUS_MAP_BIT0_OFFSET];
    let bit1 = local_values[start_col + FP6_FORBENIUS_MAP_BIT1_OFFSET];
    let bit2 = local_values[start_col + FP6_FORBENIUS_MAP_BIT2_OFFSET];
    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (bit0 + bit1 * FE::TWO + bit2 * FE::from_canonical_usize(4)
                - local_values[start_col + FP6_FORBENIUS_MAP_REM_OFFSET]),
    );
    let forbenius_coefficients_1 = Fp6::forbenius_coefficients_1()
        .iter()
        .map(|fp2| fp2.get_u32_slice().concat().try_into().unwrap())
        .collect::<Vec<[u32; 24]>>();
    let forbenius_coefficients_2 = Fp6::forbenius_coefficients_2()
        .iter()
        .map(|fp2| fp2.get_u32_slice().concat().try_into().unwrap())
        .collect::<Vec<[u32; 24]>>();
    let y1 = (0..24)
        .map(|i| {
            (P::ONES - bit0)
                * (P::ONES - bit1)
                * FE::from_canonical_u32(forbenius_coefficients_1[0][i])
                + (bit0) * (P::ONES - bit1) * FE::from_canonical_u32(forbenius_coefficients_1[1][i])
                + (P::ONES - bit0) * (bit1) * FE::from_canonical_u32(forbenius_coefficients_1[2][i])
                + (bit0) * (bit1) * FE::from_canonical_u32(forbenius_coefficients_1[3][i])
        })
        .collect::<Vec<P>>();
    let y2 = (0..24)
        .map(|i| {
            (P::ONES - bit0)
                * (P::ONES - bit1)
                * FE::from_canonical_u32(forbenius_coefficients_2[0][i])
                + (bit0) * (P::ONES - bit1) * FE::from_canonical_u32(forbenius_coefficients_2[1][i])
                + (P::ONES - bit0) * (bit1) * FE::from_canonical_u32(forbenius_coefficients_2[2][i])
                + (bit0) * (bit1) * FE::from_canonical_u32(forbenius_coefficients_2[3][i])
        })
        .collect::<Vec<P>>();

    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values
                [start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values
                [start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_POW_OFFSET]
                - local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]),
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP6_FORBENIUS_MAP_X_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_X_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_INPUT_OFFSET
                    + i]
                    - local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i]),
        );
    }
    add_fp2_forbenius_map_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET,
        bit_selector,
    );

    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values
                [start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values
                [start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_POW_OFFSET]
                - local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]),
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP6_FORBENIUS_MAP_T0_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_T0_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_INPUT_OFFSET
                    + i]
                    - local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i + 24]),
        );
    }
    add_fp2_forbenius_map_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP6_FORBENIUS_MAP_T0_CALC_OFFSET
                        + FP2_FORBENIUS_MAP_INPUT_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_Y_CALC_OFFSET
                    + FP2_FP2_X_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + FP6_FORBENIUS_MAP_T0_CALC_OFFSET
                        + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - y1[i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_Y_CALC_OFFSET
                    + FP2_FP2_Y_INPUT_OFFSET
                    + i
                    + 12]
                    - y1[i + 12]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET,
        bit_selector,
    );

    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values
                [start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values
                [start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_POW_OFFSET]
                - local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]),
    );
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP6_FORBENIUS_MAP_T1_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_T1_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_INPUT_OFFSET
                    + i]
                    - local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i + 48]),
        );
    }
    add_fp2_forbenius_map_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - local_values[start_col
                        + FP6_FORBENIUS_MAP_T1_CALC_OFFSET
                        + FP2_FORBENIUS_MAP_INPUT_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_Z_CALC_OFFSET
                    + FP2_FP2_X_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + FP6_FORBENIUS_MAP_T1_CALC_OFFSET
                        + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                        + FP_MULTIPLICATION_TOTAL_COLUMNS
                        + REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values
                    [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - y2[i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP6_FORBENIUS_MAP_Z_CALC_OFFSET
                    + FP2_FP2_Y_INPUT_OFFSET
                    + i
                    + 12]
                    - y2[i + 12]),
        );
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_fp6_forbenius_map_constraints_ext_circuit<
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
        local_values[start_col + FP6_FORBENIUS_MAP_SELECTOR_OFFSET],
    );

    for i in 0..24 * 3 {
        let c = builder.sub_extension(
            local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i],
            next_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);
    }
    let c = builder.sub_extension(
        local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET],
        next_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint_transition(builder, c);

    let six = builder.constant_extension(F::Extension::from_canonical_u32(6));
    let c = builder.mul_extension(local_values[start_col + FP6_FORBENIUS_MAP_DIV_OFFSET], six);
    let c = builder.add_extension(c, local_values[start_col + FP6_FORBENIUS_MAP_REM_OFFSET]);
    let c = builder.sub_extension(c, local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET]);
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    let bit0 = local_values[start_col + FP6_FORBENIUS_MAP_BIT0_OFFSET];
    let bit1 = local_values[start_col + FP6_FORBENIUS_MAP_BIT1_OFFSET];
    let bit2 = local_values[start_col + FP6_FORBENIUS_MAP_BIT2_OFFSET];

    let one = builder.constant_extension(F::Extension::ONE);
    let one_bit0 = builder.sub_extension(one, bit0);
    let one_bit1 = builder.sub_extension(one, bit1);

    let two = builder.constant_extension(F::Extension::TWO);
    let four = builder.constant_extension(F::Extension::from_canonical_u32(4));
    let mul1 = builder.mul_extension(bit1, two);
    let mul2 = builder.mul_extension(bit2, four);
    let c = builder.add_extension(bit0, mul1);
    let c = builder.add_extension(c, mul2);
    let c = builder.sub_extension(c, local_values[start_col + FP6_FORBENIUS_MAP_REM_OFFSET]);
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    let forbenius_coefficients_1 = Fp6::forbenius_coefficients_1()
        .iter()
        .map(|fp2| fp2.get_u32_slice().concat().try_into().unwrap())
        .collect::<Vec<[u32; 24]>>();
    let forbenius_coefficients_2 = Fp6::forbenius_coefficients_2()
        .iter()
        .map(|fp2| fp2.get_u32_slice().concat().try_into().unwrap())
        .collect::<Vec<[u32; 24]>>();
    let y1 = (0..24)
        .map(|i| {
            let const1 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_1[0][i],
            ));
            let const2 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_1[1][i],
            ));
            let const3 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_1[2][i],
            ));
            let const4 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_1[3][i],
            ));

            let bit = builder.mul_extension(one_bit0, one_bit1);
            let mul1 = builder.mul_extension(bit, const1);

            let bit = builder.mul_extension(bit0, one_bit1);
            let mul2 = builder.mul_extension(bit, const2);

            let bit = builder.mul_extension(one_bit0, bit1);
            let mul3 = builder.mul_extension(bit, const3);

            let bit = builder.mul_extension(bit0, bit1);
            let mul4 = builder.mul_extension(bit, const4);

            let c = builder.add_extension(mul1, mul2);
            let c = builder.add_extension(c, mul3);
            let c = builder.add_extension(c, mul4);
            c
        })
        .collect::<Vec<ExtensionTarget<D>>>();
    let y2 = (0..24)
        .map(|i| {
            let const1 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_2[0][i],
            ));
            let const2 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_2[1][i],
            ));
            let const3 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_2[2][i],
            ));
            let const4 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients_2[3][i],
            ));

            let bit = builder.mul_extension(one_bit0, one_bit1);
            let mul1 = builder.mul_extension(bit, const1);

            let bit = builder.mul_extension(bit0, one_bit1);
            let mul2 = builder.mul_extension(bit, const2);

            let bit = builder.mul_extension(one_bit0, bit1);
            let mul3 = builder.mul_extension(bit, const3);

            let bit = builder.mul_extension(bit0, bit1);
            let mul4 = builder.mul_extension(bit, const4);

            let c = builder.add_extension(mul1, mul2);
            let c = builder.add_extension(c, mul3);
            let c = builder.add_extension(c, mul4);
            c
        })
        .collect::<Vec<ExtensionTarget<D>>>();

    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values
            [start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_SELECTOR_OFFSET],
    );
    let c = builder.sub_extension(
        local_values[start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_POW_OFFSET],
        local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);
    for i in 0..24 {
        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
            local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_forbenius_map_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_FORBENIUS_MAP_X_CALC_OFFSET,
        bit_selector,
    );

    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values
            [start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_SELECTOR_OFFSET],
    );
    let c = builder.sub_extension(
        local_values[start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_POW_OFFSET],
        local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);
    for i in 0..24 {
        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
            local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i + 24],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_forbenius_map_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values
                [start_col + FP6_FORBENIUS_MAP_T0_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
            local_values[start_col
                + FP6_FORBENIUS_MAP_T0_CALC_OFFSET
                + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            y1[i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
            y1[i + 12],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_FORBENIUS_MAP_Y_CALC_OFFSET,
        bit_selector,
    );

    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values
            [start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_SELECTOR_OFFSET],
    );
    let c = builder.sub_extension(
        local_values[start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_POW_OFFSET],
        local_values[start_col + FP6_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);
    for i in 0..24 {
        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
            local_values[start_col + FP6_FORBENIUS_MAP_INPUT_OFFSET + i + 48],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_forbenius_map_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
            local_values
                [start_col + FP6_FORBENIUS_MAP_T1_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
            local_values[start_col
                + FP6_FORBENIUS_MAP_T1_CALC_OFFSET
                + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                + FP_MULTIPLICATION_TOTAL_COLUMNS
                + REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
            y2[i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values
                [start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
            y2[i + 12],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP6_FORBENIUS_MAP_Z_CALC_OFFSET,
        bit_selector,
    );
}
