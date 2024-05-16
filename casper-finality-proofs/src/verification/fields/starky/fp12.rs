use crate::verification::{
    fields::starky::{fp::*, fp2::*,fp6::*},
    utils::{
        native_bls::{
            fp4_square, get_bls_12_381_parameter, mul_by_nonresidue, Fp, Fp12, Fp2, Fp6
        },
        starky_utils::*,
    },
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

// MultiplyBy014
/*
    These trace offsets are for multiplyBy014 (super::native::Fp12::multiplyBy014) function. The Ti's are defined in the native function definition. It needs 12 rows.
*/
pub const MULTIPLY_BY_014_SELECTOR_OFFSET: usize = 0;
pub const MULTIPLY_BY_014_INPUT_OFFSET: usize = MULTIPLY_BY_014_SELECTOR_OFFSET + 1;
pub const MULTIPLY_BY_014_O0_OFFSET: usize = MULTIPLY_BY_014_INPUT_OFFSET + 24 * 3 * 2;
pub const MULTIPLY_BY_014_O1_OFFSET: usize = MULTIPLY_BY_014_O0_OFFSET + 24;
pub const MULTIPLY_BY_014_O4_OFFSET: usize = MULTIPLY_BY_014_O1_OFFSET + 24;
pub const MULTIPLY_BY_014_T0_CALC_OFFSET: usize = MULTIPLY_BY_014_O4_OFFSET + 24;
pub const MULTIPLY_BY_014_T1_CALC_OFFSET: usize =
    MULTIPLY_BY_014_T0_CALC_OFFSET + MULTIPLY_BY_01_TOTAL;
pub const MULTIPLY_BY_014_T2_CALC_OFFSET: usize =
    MULTIPLY_BY_014_T1_CALC_OFFSET + MULTIPLY_BY_1_TOTAL;
pub const MULTIPLY_BY_014_X_CALC_OFFSET: usize =
    MULTIPLY_BY_014_T2_CALC_OFFSET + FP6_NON_RESIDUE_MUL_TOTAL;
pub const MULTIPLY_BY_014_T3_CALC_OFFSET: usize = MULTIPLY_BY_014_X_CALC_OFFSET
    + FP6_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const MULTIPLY_BY_014_T4_CALC_OFFSET: usize = MULTIPLY_BY_014_T3_CALC_OFFSET
    + FP6_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const MULTIPLY_BY_014_T5_CALC_OFFSET: usize = MULTIPLY_BY_014_T4_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const MULTIPLY_BY_014_T6_CALC_OFFSET: usize =
    MULTIPLY_BY_014_T5_CALC_OFFSET + MULTIPLY_BY_01_TOTAL;
pub const MULTIPLY_BY_014_Y_CALC_OFFSET: usize = MULTIPLY_BY_014_T6_CALC_OFFSET
    + FP6_ADDITION_TOTAL
    + FP6_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const MULTIPLY_BY_014_TOTAL: usize = MULTIPLY_BY_014_Y_CALC_OFFSET
    + FP6_ADDITION_TOTAL
    + FP6_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;

// FP12 multiplication offsets
/*
    These trace offsets are for fp12 multiplication. It needs 12 rows. The Ti's are defined in (super::native::mul_fp_12).
*/
pub const FP12_MUL_SELECTOR_OFFSET: usize = 0;
pub const FP12_MUL_X_INPUT_OFFSET: usize = FP12_MUL_SELECTOR_OFFSET + 1;
pub const FP12_MUL_Y_INPUT_OFFSET: usize = FP12_MUL_X_INPUT_OFFSET + 24 * 3 * 2;
pub const FP12_MUL_T0_CALC_OFFSET: usize = FP12_MUL_Y_INPUT_OFFSET + 24 * 3 * 2;
pub const FP12_MUL_T1_CALC_OFFSET: usize = FP12_MUL_T0_CALC_OFFSET + FP6_MUL_TOTAL_COLUMNS;
pub const FP12_MUL_T2_CALC_OFFSET: usize = FP12_MUL_T1_CALC_OFFSET + FP6_MUL_TOTAL_COLUMNS;
pub const FP12_MUL_X_CALC_OFFSET: usize = FP12_MUL_T2_CALC_OFFSET + FP6_NON_RESIDUE_MUL_TOTAL;
pub const FP12_MUL_T3_CALC_OFFSET: usize =
    FP12_MUL_X_CALC_OFFSET + FP6_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const FP12_MUL_T4_CALC_OFFSET: usize =
    FP12_MUL_T3_CALC_OFFSET + FP6_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const FP12_MUL_T5_CALC_OFFSET: usize =
    FP12_MUL_T4_CALC_OFFSET + FP6_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const FP12_MUL_T6_CALC_OFFSET: usize = FP12_MUL_T5_CALC_OFFSET + FP6_MUL_TOTAL_COLUMNS;
pub const FP12_MUL_Y_CALC_OFFSET: usize = FP12_MUL_T6_CALC_OFFSET
    + FP6_ADDITION_TOTAL
    + FP6_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;
pub const FP12_MUL_TOTAL_COLUMNS: usize = FP12_MUL_Y_CALC_OFFSET
    + FP6_ADDITION_TOTAL
    + FP6_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 6;

// Cyclotomic square offsets
/*
    These trace offsets are for cyclotomicSquare function (super::native::Fp12::cyclotomicSquare). It needs 12 rows. The Ti's are defined in native function.
*/
pub const CYCLOTOMIC_SQ_SELECTOR_OFFSET: usize = 0;
pub const CYCLOTOMIC_SQ_INPUT_OFFSET: usize = CYCLOTOMIC_SQ_SELECTOR_OFFSET + 1;
pub const CYCLOTOMIC_SQ_T0_CALC_OFFSET: usize = CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 3 * 2;
pub const CYCLOTOMIC_SQ_T1_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T2_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T3_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T4_CALC_OFFSET: usize =
    CYCLOTOMIC_SQ_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_TOTAL;
pub const CYCLOTOMIC_SQ_T5_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T4_CALC_OFFSET
    + FP2_SUBTRACTION_TOTAL
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_C0_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T6_CALC_OFFSET: usize = CYCLOTOMIC_SQ_C0_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_T7_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T6_CALC_OFFSET
    + FP2_SUBTRACTION_TOTAL
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_C1_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T8_CALC_OFFSET: usize = CYCLOTOMIC_SQ_C1_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_T9_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T8_CALC_OFFSET
    + FP2_SUBTRACTION_TOTAL
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_C2_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T10_CALC_OFFSET: usize = CYCLOTOMIC_SQ_C2_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_T11_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T10_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_C3_CALC_OFFSET: usize =
    CYCLOTOMIC_SQ_T11_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T12_CALC_OFFSET: usize = CYCLOTOMIC_SQ_C3_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_T13_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T12_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_C4_CALC_OFFSET: usize =
    CYCLOTOMIC_SQ_T13_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_T14_CALC_OFFSET: usize = CYCLOTOMIC_SQ_C4_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_T15_CALC_OFFSET: usize = CYCLOTOMIC_SQ_T14_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const CYCLOTOMIC_SQ_C5_CALC_OFFSET: usize =
    CYCLOTOMIC_SQ_T15_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const CYCLOTOMIC_SQ_TOTAL_COLUMNS: usize = CYCLOTOMIC_SQ_C5_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;

// Cyclotomic exponent offsets
/*
    These offsets are for cyclotomicExponent (super::native::Fp12::cyclotomicExponent) function. Needs 12*70 rows. The offsets are defined such that each 0 bit of the bls12-381 parameter takes 12 rows (one operation, cyclotomicSquare) and each 1 bit takes 12*2 rows (two operations, cyclotomicSquare and fp12 multiplication).
    CYCLOTOMIC_EXP_START_ROW -> selector which is 1 for the first row of the trace.
    FIRST_ROW_SELECTOR_OFFSET -> selector which is 1 for the starting row for each operation. Hence, every 12th row, it is set 1.
    RES_ROW_SELECTOR_OFFSET -> selector which is 1 for the row which contains the final result of cyclotomicExponent.
    BIT1_SELECTOR_OFFSET -> selector which is 1 for each 1 bit of bls12-381 parameter. It is set 1 for 12 rows continous rows.
    INPUT_OFFSET -> offset where input for the function is set.
    Z_OFFSET -> offset where result of the previous computation is stored.
    Z_CYCLOTOMIC_SQ_OFFSET -> offset containing the computation for cyclotomicSquare function.
    Z_MUL_INPUT_OFFSET -> offset containing the computation for fp12 multiplication.

    Z_CYCLOTMIC_SQ_OFFSET and Z_MUL_INPUT_OFFSET are equal because both the operations are never done in the same rows. In a single row, either cyclotomic square is being computed or fp12 multiplication is being computed.
*/
pub const CYCLOTOMIC_EXP_SELECTOR_OFFSET: usize = 0;
pub const CYCLOTOMIC_EXP_START_ROW: usize = CYCLOTOMIC_EXP_SELECTOR_OFFSET + 1;
pub const FIRST_ROW_SELECTOR_OFFSET: usize = CYCLOTOMIC_EXP_START_ROW + 1;
pub const BIT1_SELECTOR_OFFSET: usize = FIRST_ROW_SELECTOR_OFFSET + 1;
pub const RES_ROW_SELECTOR_OFFSET: usize = BIT1_SELECTOR_OFFSET + 1;
pub const INPUT_OFFSET: usize = RES_ROW_SELECTOR_OFFSET + 1;
pub const Z_OFFSET: usize = INPUT_OFFSET + 24 * 3 * 2;
pub const Z_CYCLOTOMIC_SQ_OFFSET: usize = Z_OFFSET + 24 * 3 * 2;
pub const Z_MUL_INPUT_OFFSET: usize = Z_OFFSET + 24 * 3 * 2;
pub const CYCLOTOMIC_EXP_TOTAL_COLUMNS: usize = Z_MUL_INPUT_OFFSET + FP12_MUL_TOTAL_COLUMNS;

// Forbenius map Fp12
/*
    These trace offsets are for forbenius_map (super::native::Fp12::forbenius_map) function. It needs 12 rows.
    FP12_FORBENIUS_MAP_DIV_OFFSET -> offset which stores integer division power/12.
    FP12_FORBENIUS_MAP_REM_OFFSET -> offset which stores power%12.
    FP12_FORBENIUS_MAP_BIT0_OFFSET, FP12_FORBENIUS_MAP_BIT1_OFFSET, FP12_FORBENIUS_MAP_BIT2_OFFSET, FP12_FORBENIUS_MAP_BIT3_OFFSET -> offsets which store the bit decomposition of remainder (power%12).
*/
pub const FP12_FORBENIUS_MAP_SELECTOR_OFFSET: usize = 0;
pub const FP12_FORBENIUS_MAP_INPUT_OFFSET: usize = FP12_FORBENIUS_MAP_SELECTOR_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_POW_OFFSET: usize = FP12_FORBENIUS_MAP_INPUT_OFFSET + 24 * 3 * 2;
pub const FP12_FORBENIUS_MAP_DIV_OFFSET: usize = FP12_FORBENIUS_MAP_POW_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_REM_OFFSET: usize = FP12_FORBENIUS_MAP_DIV_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_BIT0_OFFSET: usize = FP12_FORBENIUS_MAP_REM_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_BIT1_OFFSET: usize = FP12_FORBENIUS_MAP_BIT0_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_BIT2_OFFSET: usize = FP12_FORBENIUS_MAP_BIT1_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_BIT3_OFFSET: usize = FP12_FORBENIUS_MAP_BIT2_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_R0_CALC_OFFSET: usize = FP12_FORBENIUS_MAP_BIT3_OFFSET + 1;
pub const FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET: usize =
    FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_TOTAL_COLUMNS;
pub const FP12_FORBENIUS_MAP_C0_CALC_OFFSET: usize =
    FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + FP6_FORBENIUS_MAP_TOTAL_COLUMNS;
pub const FP12_FORBENIUS_MAP_C1_CALC_OFFSET: usize =
    FP12_FORBENIUS_MAP_C0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP12_FORBENIUS_MAP_C2_CALC_OFFSET: usize =
    FP12_FORBENIUS_MAP_C1_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const FP12_FORBENIUS_MAP_TOTAL_COLUMNS: usize =
    FP12_FORBENIUS_MAP_C2_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;

// Fp12 conjugate
/*
    These trace offsets are for fp12 conjugate (super::native::Fp12::conjugate). It needs 1 row.
*/
pub const FP12_CONJUGATE_INPUT_OFFSET: usize = 0;
pub const FP12_CONJUGATE_OUTPUT_OFFSET: usize = FP12_CONJUGATE_INPUT_OFFSET + 24 * 3 * 2;
pub const FP12_CONJUGATE_ADDITIION_OFFSET: usize = FP12_CONJUGATE_OUTPUT_OFFSET + 24 * 3 * 2;
pub const FP12_CONJUGATE_TOTAL: usize = FP12_CONJUGATE_ADDITIION_OFFSET + FP6_ADDITION_TOTAL;

/// Fills trace of [multiplyBy014](super::native::Fp12::multiplyBy014) function. Input is 12\*12 limbs and three 12\*2 limbs. Needs 12 rows.
pub fn fill_trace_multiply_by_014<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp12,
    o0: &Fp2,
    o1: &Fp2,
    o4: &Fp2,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        for i in 0..12 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_014_INPUT_OFFSET + i * 12,
                &x.0[i].0,
            );
        }
        for i in 0..2 {
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_014_O0_OFFSET + i * 12,
                &o0.0[i].0,
            );
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_014_O1_OFFSET + i * 12,
                &o1.0[i].0,
            );
            assign_u32_in_series(
                trace,
                row,
                start_col + MULTIPLY_BY_014_O4_OFFSET + i * 12,
                &o4.0[i].0,
            );
        }
        trace[row][start_col + MULTIPLY_BY_014_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + MULTIPLY_BY_014_SELECTOR_OFFSET] = F::ZERO;

    let c0 = Fp6(x.0[..6].try_into().unwrap());
    let c1 = Fp6(x.0[6..].try_into().unwrap());

    let t0 = c0.multiply_by_01(*o0, *o1);
    fill_trace_multiply_by_01(
        trace,
        &c0,
        o0,
        o1,
        start_row,
        end_row,
        start_col + MULTIPLY_BY_014_T0_CALC_OFFSET,
    );
    let t1 = c1.multiply_by_1(*o4);
    fill_trace_multiply_by_1(
        trace,
        &c1,
        o4,
        start_row,
        end_row,
        start_col + MULTIPLY_BY_014_T1_CALC_OFFSET,
    );
    let t2 = mul_by_nonresidue(t1.0);
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication_fp6(
            trace,
            &t1,
            row,
            start_col + MULTIPLY_BY_014_T2_CALC_OFFSET,
        );
    }
    let _x = t2 + t0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction_fp6(
            trace,
            &t2,
            &t0,
            row,
            start_col + MULTIPLY_BY_014_X_CALC_OFFSET,
        );
    }

    let t3 = c0 + c1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction_fp6(
            trace,
            &c0,
            &c1,
            row,
            start_col + MULTIPLY_BY_014_T3_CALC_OFFSET,
        );
    }
    let t4 = (*o1) + (*o4);
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &o1.get_u32_slice(),
            &o4.get_u32_slice(),
            row,
            start_col + MULTIPLY_BY_014_T4_CALC_OFFSET,
        );
    }
    let t5 = t3.multiply_by_01(*o0, t4);
    fill_trace_multiply_by_01(
        trace,
        &t3,
        o0,
        &t4,
        start_row,
        end_row,
        start_col + MULTIPLY_BY_014_T5_CALC_OFFSET,
    );
    let t6 = t5 - t0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction_fp6(
            trace,
            &t5,
            &t0,
            row,
            start_col + MULTIPLY_BY_014_T6_CALC_OFFSET,
        );
    }
    let _y = t6 - t1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction_fp6(
            trace,
            &t6,
            &t1,
            row,
            start_col + MULTIPLY_BY_014_Y_CALC_OFFSET,
        );
    }
}

/// Fills stark trace for fp12 multiplication. Inputs are 12*12 limbs each. Needs 12 rows.
pub fn fill_trace_fp12_multiplication<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp12,
    y: &Fp12,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        for i in 0..12 {
            assign_u32_in_series(
                trace,
                row,
                start_col + FP12_MUL_X_INPUT_OFFSET + i * 12,
                &x.0[i].0,
            );
            assign_u32_in_series(
                trace,
                row,
                start_col + FP12_MUL_Y_INPUT_OFFSET + i * 12,
                &y.0[i].0,
            );
        }
        trace[row][start_col + FP12_MUL_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + FP12_MUL_SELECTOR_OFFSET] = F::ZERO;
    let (c0, c1) = (
        Fp6(x.0[0..6].try_into().unwrap()),
        Fp6(x.0[6..12].try_into().unwrap()),
    );
    let (r0, r1) = (
        Fp6(y.0[0..6].try_into().unwrap()),
        Fp6(y.0[6..12].try_into().unwrap()),
    );
    let t0 = c0 * r0;
    fill_trace_fp6_multiplication(
        trace,
        &c0,
        &r0,
        start_row,
        end_row,
        start_col + FP12_MUL_T0_CALC_OFFSET,
    );
    let t1 = c1 * r1;
    fill_trace_fp6_multiplication(
        trace,
        &c1,
        &r1,
        start_row,
        end_row,
        start_col + FP12_MUL_T1_CALC_OFFSET,
    );
    let t2 = mul_by_nonresidue(t1.0);
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication_fp6(
            trace,
            &t1,
            row,
            start_col + FP12_MUL_T2_CALC_OFFSET,
        );
    }
    let _x = t0 + t2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction_fp6(
            trace,
            &t0,
            &t2,
            row,
            start_col + FP12_MUL_X_CALC_OFFSET,
        );
    }

    let t3 = c0 + c1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction_fp6(
            trace,
            &c0,
            &c1,
            row,
            start_col + FP12_MUL_T3_CALC_OFFSET,
        );
    }
    let t4 = r0 + r1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction_fp6(
            trace,
            &r0,
            &r1,
            row,
            start_col + FP12_MUL_T4_CALC_OFFSET,
        );
    }
    let t5 = t3 * t4;
    fill_trace_fp6_multiplication(
        trace,
        &t3,
        &t4,
        start_row,
        end_row,
        start_col + FP12_MUL_T5_CALC_OFFSET,
    );
    let t6 = t5 - t0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction_fp6(
            trace,
            &t5,
            &t0,
            row,
            start_col + FP12_MUL_T6_CALC_OFFSET,
        );
    }
    let _y = t6 - t1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction_fp6(
            trace,
            &t6,
            &t1,
            row,
            start_col + FP12_MUL_Y_CALC_OFFSET,
        );
    }
}

/// Fills trace of [cyclotomicSquare](super::native::Fp12::cyclotomicSquare) function. Input is 12*12 limbs. Needs 12 rows.
pub fn fill_trace_cyclotomic_sq<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp12,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + CYCLOTOMIC_SQ_INPUT_OFFSET,
            &x.get_u32_slice().concat(),
        );
        trace[row][start_col + CYCLOTOMIC_SQ_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + CYCLOTOMIC_SQ_SELECTOR_OFFSET] = F::ZERO;
    let c0c0 = Fp2(x.0[0..2].try_into().unwrap());
    let c0c1 = Fp2(x.0[2..4].try_into().unwrap());
    let c0c2 = Fp2(x.0[4..6].try_into().unwrap());
    let c1c0 = Fp2(x.0[6..8].try_into().unwrap());
    let c1c1 = Fp2(x.0[8..10].try_into().unwrap());
    let c1c2 = Fp2(x.0[10..12].try_into().unwrap());
    let two = Fp::get_fp_from_biguint(BigUint::from(2 as u32));

    let t0 = fp4_square(c0c0, c1c1);
    fill_trace_fp4_sq(
        trace,
        &c0c0,
        &c1c1,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET,
    );

    let t1 = fp4_square(c1c0, c0c2);
    fill_trace_fp4_sq(
        trace,
        &c1c0,
        &c0c2,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET,
    );

    let t2 = fp4_square(c0c1, c1c2);
    fill_trace_fp4_sq(
        trace,
        &c0c1,
        &c1c2,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET,
    );

    let t3 = t2.1.mul_by_nonresidue();
    for row in start_row..end_row + 1 {
        fill_trace_non_residue_multiplication(
            trace,
            &t2.1.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET,
        );
    }

    let t4 = t0.0 - c0c0;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t0.0.get_u32_slice(),
            &c0c0.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T4_CALC_OFFSET,
        );
    }
    let t5 = t4 * two;
    fill_trace_fp2_fp_mul(
        trace,
        &t4.get_u32_slice(),
        &two.0,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET,
    );
    let _c0 = t5 + t0.0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t5.get_u32_slice(),
            &t0.0.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_C0_CALC_OFFSET,
        );
    }

    let t6 = t1.0 - c0c1;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t1.0.get_u32_slice(),
            &c0c1.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T6_CALC_OFFSET,
        );
    }
    let t7 = t6 * two;
    fill_trace_fp2_fp_mul(
        trace,
        &t6.get_u32_slice(),
        &two.0,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET,
    );
    let _c1 = t7 + t1.0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t7.get_u32_slice(),
            &t1.0.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_C1_CALC_OFFSET,
        );
    }

    let t8 = t2.0 - c0c2;
    for row in start_row..end_row + 1 {
        fill_trace_subtraction_with_reduction(
            trace,
            &t2.0.get_u32_slice(),
            &c0c2.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T8_CALC_OFFSET,
        );
    }
    let t9 = t8 * two;
    fill_trace_fp2_fp_mul(
        trace,
        &t8.get_u32_slice(),
        &two.0,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET,
    );
    let _c2 = t9 + t2.0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t9.get_u32_slice(),
            &t2.0.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_C2_CALC_OFFSET,
        );
    }

    let t10 = t3 + c1c0;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t3.get_u32_slice(),
            &c1c0.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T10_CALC_OFFSET,
        );
    }
    let t11 = t10 * two;
    fill_trace_fp2_fp_mul(
        trace,
        &t10.get_u32_slice(),
        &two.0,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET,
    );
    let _c3 = t11 + t3;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t11.get_u32_slice(),
            &t3.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_C3_CALC_OFFSET,
        );
    }

    let t12 = t0.1 + c1c1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t0.1.get_u32_slice(),
            &c1c1.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T12_CALC_OFFSET,
        );
    }
    let t13 = t12 * two;
    fill_trace_fp2_fp_mul(
        trace,
        &t12.get_u32_slice(),
        &two.0,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET,
    );
    let _c4 = t13 + t0.1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t13.get_u32_slice(),
            &t0.1.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_C4_CALC_OFFSET,
        );
    }

    let t14 = t1.1 + c1c2;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t1.1.get_u32_slice(),
            &c1c2.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_T14_CALC_OFFSET,
        );
    }
    let t15 = t14 * two;
    fill_trace_fp2_fp_mul(
        trace,
        &t14.get_u32_slice(),
        &two.0,
        start_row,
        end_row,
        start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET,
    );
    let _c5 = t15 + t1.1;
    for row in start_row..end_row + 1 {
        fill_trace_addition_with_reduction(
            trace,
            &t15.get_u32_slice(),
            &t1.1.get_u32_slice(),
            row,
            start_col + CYCLOTOMIC_SQ_C5_CALC_OFFSET,
        );
    }
}

/// Fills trace of [cyclotomicExponent](super::native::Fp12::cyclotocmicExponent) function. Input is 12\*12 limbs. Needs 12\*70 rows. For each bit 0 of bls12-381 parameter, fills the trace for cyclotomicSquare computation. For each bit 1 of the bls12-381 parameter, fills trace for cyclotomic square computation in 12 rows, then fills the trace for fp12 multiplication computation in the next 12 rows and also sets `trace[row][start_col + BIT1_SELECTOR_OFFSET]` to 1 for these rows. After going through all bits of the bls12-381 parameter, fills the result in the next row's Z_OFFSET, while also setting RES_ROW_SELECTOR to 1.
pub fn fill_trace_cyclotomic_exp<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp12,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + INPUT_OFFSET,
            &x.get_u32_slice().concat(),
        );
        trace[row][start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET] = F::ONE;
    }
    trace[end_row][start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET] = F::ZERO;
    trace[start_row][start_col + CYCLOTOMIC_EXP_START_ROW] = F::ONE;
    let mut z = Fp12::one();
    let mut i = get_bls_12_381_parameter().bits() - 1;
    let mut bitone = false;
    assert_eq!(end_row + 1 - start_row, 70 * 12 + 1);

    for j in 0..70 {
        let s_row = start_row + j * 12;
        let e_row = s_row + 11;
        for row in s_row..e_row + 1 {
            if bitone {
                trace[row][start_col + BIT1_SELECTOR_OFFSET] = F::ONE;
            }
            assign_u32_in_series(
                trace,
                row,
                start_col + Z_OFFSET,
                &z.get_u32_slice().concat(),
            );
        }
        trace[s_row][start_col + FIRST_ROW_SELECTOR_OFFSET] = F::ONE;
        if bitone {
            fill_trace_fp12_multiplication(
                trace,
                &z,
                &x,
                s_row,
                e_row,
                start_col + Z_MUL_INPUT_OFFSET,
            );
            z = z * (*x);
        } else {
            fill_trace_cyclotomic_sq(trace, &z, s_row, e_row, start_col + Z_CYCLOTOMIC_SQ_OFFSET);
            z = z.cyclotomic_square();
        }
        if get_bls_12_381_parameter().bit(i) && !bitone {
            bitone = true;
        } else if j < 69 {
            i -= 1;
            bitone = false;
        }
    }
    trace[start_row + 70 * 12][start_col + RES_ROW_SELECTOR_OFFSET] = F::ONE;
    assign_u32_in_series(
        trace,
        start_row + 70 * 12,
        start_col + Z_OFFSET,
        &z.get_u32_slice().concat(),
    );
}

/// Fills trace of [forbenius_map](super::native::Fp12::forbenius_map) function. Input is 12*12 limbs and usize. Needs 12 rows.
pub fn fill_trace_fp12_forbenius_map<
    F: RichField + Extendable<D>,
    const D: usize,
    const C: usize,
>(
    trace: &mut Vec<[F; C]>,
    x: &Fp12,
    pow: usize,
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    let div = pow / 12;
    let rem = pow % 12;
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET,
            &x.get_u32_slice().concat(),
        );
        trace[row][start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET] = F::ONE;
        trace[row][start_col + FP12_FORBENIUS_MAP_POW_OFFSET] = F::from_canonical_usize(pow);
        trace[row][start_col + FP12_FORBENIUS_MAP_DIV_OFFSET] = F::from_canonical_usize(div);
        trace[row][start_col + FP12_FORBENIUS_MAP_REM_OFFSET] = F::from_canonical_usize(rem);
        trace[row][start_col + FP12_FORBENIUS_MAP_BIT0_OFFSET] = F::from_canonical_usize(rem & 1);
        trace[row][start_col + FP12_FORBENIUS_MAP_BIT1_OFFSET] =
            F::from_canonical_usize((rem >> 1) & 1);
        trace[row][start_col + FP12_FORBENIUS_MAP_BIT2_OFFSET] =
            F::from_canonical_usize((rem >> 2) & 1);
        trace[row][start_col + FP12_FORBENIUS_MAP_BIT3_OFFSET] = F::from_canonical_usize(rem >> 3);
    }
    trace[end_row][start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET] = F::ZERO;
    let r0 = Fp6(x.0[0..6].to_vec().try_into().unwrap());
    let r1 = Fp6(x.0[6..12].to_vec().try_into().unwrap());
    let _x = r0.forbenius_map(pow);
    fill_trace_fp6_forbenius_map(
        trace,
        &r0,
        pow,
        start_row,
        end_row,
        start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET,
    );
    let c0c1c2 = r1.forbenius_map(pow);
    fill_trace_fp6_forbenius_map(
        trace,
        &r1,
        pow,
        start_row,
        end_row,
        start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET,
    );
    let c0 = Fp2(c0c1c2.0[0..2].to_vec().try_into().unwrap());
    let c1 = Fp2(c0c1c2.0[2..4].to_vec().try_into().unwrap());
    let c2 = Fp2(c0c1c2.0[4..6].to_vec().try_into().unwrap());
    let forbenius_coefficients = Fp12::forbenius_coefficients();
    let coeff = forbenius_coefficients[pow % 12];
    generate_trace_fp2_mul(
        trace,
        c0.get_u32_slice(),
        coeff.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP12_FORBENIUS_MAP_C0_CALC_OFFSET,
    );
    generate_trace_fp2_mul(
        trace,
        c1.get_u32_slice(),
        coeff.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP12_FORBENIUS_MAP_C1_CALC_OFFSET,
    );
    generate_trace_fp2_mul(
        trace,
        c2.get_u32_slice(),
        coeff.get_u32_slice(),
        start_row,
        end_row,
        start_col + FP12_FORBENIUS_MAP_C2_CALC_OFFSET,
    );
}

/// Fill trace of [conjugate](super::native::Fp12::conjugate) function. Input is 12*12 limbs. Needs 1 row.
pub fn fill_trace_fp12_conjugate<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp12,
    row: usize,
    start_col: usize,
) {
    assign_u32_in_series(
        trace,
        row,
        start_col + FP12_CONJUGATE_INPUT_OFFSET,
        &x.get_u32_slice().concat(),
    );
    let conjugate = x.conjugate();
    assign_u32_in_series(
        trace,
        row,
        start_col + FP12_CONJUGATE_OUTPUT_OFFSET,
        &conjugate.get_u32_slice().concat(),
    );
    let x_fp6 = Fp6(x.0[6..12].try_into().unwrap());
    let conjugat_fp6 = Fp6(conjugate.0[6..12].try_into().unwrap());
    fill_trace_addition_fp6(
        trace,
        &x_fp6.get_u32_slice(),
        &conjugat_fp6.get_u32_slice(),
        row,
        start_col + FP12_CONJUGATE_ADDITIION_OFFSET,
    );
}

/// Constraints [multiplyBy014](super::native::Fp12::multiplyBy014) function.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints all the Ti's (defined in the native function) accordinng to their respective operations.
pub fn add_multiply_by_014_constraints<
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

    for i in 0..12 {
        for j in 0..12 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i]
                        - next_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i]),
            );
        }
        for j in 0..2 {
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i]
                        - next_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i]),
            );
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i]
                        - next_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i]),
            );
            yield_constr.constraint_transition(
                bit_selector_val
                    * local_values[start_col + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i]
                        - next_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i]),
            );
        }
    }

    // T0
    for i in 0..12 {
        for j in 0..6 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + MULTIPLY_BY_01_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i]),
            );
        }
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + MULTIPLY_BY_01_B0_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + MULTIPLY_BY_01_B1_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i]),
            );
        }
    }
    add_multiply_by_01_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..12 {
        for j in 0..6 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T1_CALC_OFFSET
                        + MULTIPLY_BY_1_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T1_CALC_OFFSET
                        + MULTIPLY_BY_1_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values
                            [start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i + 24 * 3]),
            );
        }
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T1_CALC_OFFSET
                        + MULTIPLY_BY_1_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T1_CALC_OFFSET
                        + MULTIPLY_BY_1_B1_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i]),
            );
        }
    }
    add_multiply_by_1_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for j in 0..2 {
        let (x_offset, yz_offset) = if j == 0 {
            (FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET, Z1_REDUCE_OFFSET)
        } else {
            (FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET, Z2_REDUCE_OFFSET)
        };
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                        + i
                        + j * 12]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T1_CALC_OFFSET
                            + MULTIPLY_BY_1_X_CALC_OFFSET
                            + x_offset
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                        + i
                        + j * 12
                        + 24]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T1_CALC_OFFSET
                            + MULTIPLY_BY_1_Y_CALC_OFFSET
                            + yz_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                        + i
                        + j * 12
                        + 48]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T1_CALC_OFFSET
                            + MULTIPLY_BY_1_Z_CALC_OFFSET
                            + yz_offset
                            + REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_non_residue_multiplication_fp6_constraints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T2_CALC_OFFSET,
        bit_selector,
    );

    // X
    for j in 0..6 {
        let (addition_offset, x_offset, y_offset) = if j == 0 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_NON_RESIDUE_MUL_C2
                    + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
                MULTIPLY_BY_01_X_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        } else if j == 1 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_NON_RESIDUE_MUL_C2
                    + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
                MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL,
            )
        } else if j == 2 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET,
                MULTIPLY_BY_01_Y_CALC_OFFSET + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
            )
        } else if j == 3 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 12,
                MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL,
            )
        } else if j == 4 {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 24,
                MULTIPLY_BY_01_Z_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 36,
                MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL,
            )
        };
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_T2_CALC_OFFSET + x_offset + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T0_CALC_OFFSET
                            + y_offset
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_X_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for j in 0..3 {
        let mut addition_offset = if j == 0 {
            FP6_ADDITION_0_OFFSET
        } else if j == 1 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        for k in 0..2 {
            addition_offset += if k == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            for i in 0..12 {
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col
                            + MULTIPLY_BY_014_T3_CALC_OFFSET
                            + addition_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + MULTIPLY_BY_014_T3_CALC_OFFSET
                            + addition_offset
                            + FP_ADDITION_X_OFFSET
                            + i]
                            - local_values
                                [start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 24 + k * 12 + i]),
                );
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col
                            + MULTIPLY_BY_014_T3_CALC_OFFSET
                            + addition_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + MULTIPLY_BY_014_T3_CALC_OFFSET
                            + addition_offset
                            + FP_ADDITION_Y_OFFSET
                            + i]
                            - local_values[start_col
                                + MULTIPLY_BY_014_INPUT_OFFSET
                                + j * 24
                                + k * 12
                                + i
                                + 24 * 3]),
                );
            }
        }
    }
    add_addition_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T3_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for j in 0..2 {
        let addition_offset = if j == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T4_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T4_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T4_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T4_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..12 {
        for j in 0..6 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + MULTIPLY_BY_01_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T3_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + MULTIPLY_BY_01_B0_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + MULTIPLY_BY_01_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + MULTIPLY_BY_01_B1_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T4_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_multiply_by_01_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for j in 0..3 {
        let (mut addition_offset, mut subtraction_offset, input_offset) = if j == 0 {
            (
                FP6_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_01_X_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        } else if j == 1 {
            (
                FP6_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_01_Y_CALC_OFFSET + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET,
                FP6_SUBTRACTION_2_OFFSET,
                MULTIPLY_BY_01_Z_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        };
        for k in 0..2 {
            addition_offset += if k == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            subtraction_offset += if k == 0 {
                FP2_SUBTRACTION_0_OFFSET
            } else {
                FP2_SUBTRACTION_1_OFFSET
            };
            for i in 0..12 {
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col
                            + MULTIPLY_BY_014_T6_CALC_OFFSET
                            + addition_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + MULTIPLY_BY_014_T6_CALC_OFFSET
                            + addition_offset
                            + FP_ADDITION_X_OFFSET
                            + i]
                            - local_values[start_col
                                + MULTIPLY_BY_014_T5_CALC_OFFSET
                                + input_offset
                                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col
                            + MULTIPLY_BY_014_T6_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + subtraction_offset
                            + FP_SUBTRACTION_CHECK_OFFSET]
                        * (local_values[start_col
                            + MULTIPLY_BY_014_T6_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + subtraction_offset
                            + FP_SUBTRACTION_Y_OFFSET
                            + i]
                            - local_values[start_col
                                + MULTIPLY_BY_014_T0_CALC_OFFSET
                                + input_offset
                                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
            }
        }
    }
    add_subtraction_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_T6_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for j in 0..6 {
        let (addition_offset, subtraction_offset, input_offset) = if j == 0 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET + FP2_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_1_X_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
            )
        } else if j == 1 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_0_OFFSET + FP2_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_1_X_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
            )
        } else if j == 2 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_1_OFFSET + FP2_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_1_Y_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        } else if j == 3 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET + FP2_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_1_Y_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        } else if j == 4 {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_2_OFFSET + FP2_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_1_Z_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_2_OFFSET + FP2_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_1_Z_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        };
        for i in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + MULTIPLY_BY_014_T6_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + subtraction_offset
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + subtraction_offset
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [start_col + MULTIPLY_BY_014_T1_CALC_OFFSET + input_offset + i]),
            );
        }
    }
    add_subtraction_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + MULTIPLY_BY_014_Y_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_multiply_by_014_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        let mul_tmp = local_values[start_col + MULTIPLY_BY_014_SELECTOR_OFFSET];
        for j in 0..12 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i],
                next_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint_transition(builder, c);
        }
        for j in 0..2 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i],
                next_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint_transition(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i],
                next_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint_transition(builder, c);

            let sub_tmp3 = builder.sub_extension(
                local_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i],
                next_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i],
            );
            let c3 = builder.mul_extension(sub_tmp3, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c3);
            yield_constr.constraint_transition(builder, c);
        }
    }

    // T0
    for i in 0..12 {
        let mul_tmp = local_values
            [start_col + MULTIPLY_BY_014_T0_CALC_OFFSET + MULTIPLY_BY_01_SELECTOR_OFFSET];
        for j in 0..6 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T0_CALC_OFFSET
                    + MULTIPLY_BY_01_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
        for j in 0..2 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T0_CALC_OFFSET
                    + MULTIPLY_BY_01_B0_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T0_CALC_OFFSET
                    + MULTIPLY_BY_01_B1_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_multiply_by_01_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_014_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..12 {
        let mul_tmp = local_values
            [start_col + MULTIPLY_BY_014_T1_CALC_OFFSET + MULTIPLY_BY_1_SELECTOR_OFFSET];
        for j in 0..6 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T1_CALC_OFFSET
                    + MULTIPLY_BY_1_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i + 24 * 3],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
        for j in 0..2 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T1_CALC_OFFSET
                    + MULTIPLY_BY_1_B1_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_multiply_by_1_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_014_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for j in 0..2 {
        let (x_offset, yz_offset) = if j == 0 {
            (FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET, Z1_REDUCE_OFFSET)
        } else {
            (FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET, Z2_REDUCE_OFFSET)
        };
        for i in 0..12 {
            let mul_tmp = local_values
                [start_col + MULTIPLY_BY_014_T2_CALC_OFFSET + FP6_NON_RESIDUE_MUL_CHECK_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T2_CALC_OFFSET
                    + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + j * 12],
                local_values[start_col
                    + MULTIPLY_BY_014_T1_CALC_OFFSET
                    + MULTIPLY_BY_1_X_CALC_OFFSET
                    + x_offset
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T2_CALC_OFFSET
                    + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + j * 12
                    + 24],
                local_values[start_col
                    + MULTIPLY_BY_014_T1_CALC_OFFSET
                    + MULTIPLY_BY_1_Y_CALC_OFFSET
                    + yz_offset
                    + REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);

            let sub_tmp3 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T2_CALC_OFFSET
                    + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + j * 12
                    + 48],
                local_values[start_col
                    + MULTIPLY_BY_014_T1_CALC_OFFSET
                    + MULTIPLY_BY_1_Z_CALC_OFFSET
                    + yz_offset
                    + REDUCED_OFFSET
                    + i],
            );
            let c3 = builder.mul_extension(sub_tmp3, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c3);
            yield_constr.constraint(builder, c);
        }
    }
    add_non_residue_multiplication_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_014_T2_CALC_OFFSET,
        bit_selector,
    );

    // X
    for j in 0..6 {
        let (addition_offset, x_offset, y_offset) = if j == 0 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_NON_RESIDUE_MUL_C2
                    + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
                MULTIPLY_BY_01_X_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        } else if j == 1 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_NON_RESIDUE_MUL_C2
                    + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
                MULTIPLY_BY_01_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL,
            )
        } else if j == 2 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET,
                MULTIPLY_BY_01_Y_CALC_OFFSET + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
            )
        } else if j == 3 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 12,
                MULTIPLY_BY_01_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL,
            )
        } else if j == 4 {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 24,
                MULTIPLY_BY_01_Z_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_NON_RESIDUE_MUL_INPUT_OFFSET + 36,
                MULTIPLY_BY_01_Z_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL,
            )
        };
        for i in 0..12 {
            let mul_tmp = local_values[start_col
                + MULTIPLY_BY_014_X_CALC_OFFSET
                + addition_offset
                + FP_ADDITION_CHECK_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_X_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col + MULTIPLY_BY_014_T2_CALC_OFFSET + x_offset + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_X_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + MULTIPLY_BY_014_T0_CALC_OFFSET
                    + y_offset
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_014_X_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for j in 0..3 {
        let mut addition_offset = if j == 0 {
            FP6_ADDITION_0_OFFSET
        } else if j == 1 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        for k in 0..2 {
            addition_offset += if k == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            for i in 0..12 {
                let mul_tmp = local_values[start_col
                    + MULTIPLY_BY_014_T3_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_CHECK_OFFSET];

                let sub_tmp1 = builder.sub_extension(
                    local_values[start_col
                        + MULTIPLY_BY_014_T3_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_X_OFFSET
                        + i],
                    local_values[start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 24 + k * 12 + i],
                );
                let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
                let c = builder.mul_extension(bit_selector_val, c1);
                yield_constr.constraint(builder, c);

                let sub_tmp2 = builder.sub_extension(
                    local_values[start_col
                        + MULTIPLY_BY_014_T3_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_Y_OFFSET
                        + i],
                    local_values
                        [start_col + MULTIPLY_BY_014_INPUT_OFFSET + j * 24 + k * 12 + i + 24 * 3],
                );
                let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
                let c = builder.mul_extension(bit_selector_val, c2);
                yield_constr.constraint(builder, c);
            }
        }
    }
    add_addition_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_014_T3_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for j in 0..2 {
        let addition_offset = if j == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for i in 0..12 {
            let mul_tmp = local_values[start_col
                + MULTIPLY_BY_014_T4_CALC_OFFSET
                + addition_offset
                + FP_ADDITION_CHECK_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T4_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T4_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_014_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..12 {
        let mul_tmp = local_values
            [start_col + MULTIPLY_BY_014_T5_CALC_OFFSET + MULTIPLY_BY_01_SELECTOR_OFFSET];
        for j in 0..6 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T5_CALC_OFFSET
                    + MULTIPLY_BY_01_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col
                    + MULTIPLY_BY_014_T3_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
        for j in 0..2 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T5_CALC_OFFSET
                    + MULTIPLY_BY_01_B0_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_T5_CALC_OFFSET
                    + MULTIPLY_BY_01_B1_OFFSET
                    + j * 12
                    + i],
                local_values[start_col
                    + MULTIPLY_BY_014_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_multiply_by_01_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + MULTIPLY_BY_014_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for j in 0..3 {
        let (mut addition_offset, mut subtraction_offset, input_offset) = if j == 0 {
            (
                FP6_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_01_X_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        } else if j == 1 {
            (
                FP6_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_01_Y_CALC_OFFSET + FP2_ADDITION_TOTAL + FP2_SUBTRACTION_TOTAL,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET,
                FP6_SUBTRACTION_2_OFFSET,
                MULTIPLY_BY_01_Z_CALC_OFFSET + FP2_ADDITION_TOTAL,
            )
        };
        for k in 0..2 {
            addition_offset += if k == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            subtraction_offset += if k == 0 {
                FP2_SUBTRACTION_0_OFFSET
            } else {
                FP2_SUBTRACTION_1_OFFSET
            };
            for i in 0..12 {
                let sub_tmp1 = builder.sub_extension(
                    local_values[start_col
                        + MULTIPLY_BY_014_T6_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_X_OFFSET
                        + i],
                    local_values[start_col
                        + MULTIPLY_BY_014_T5_CALC_OFFSET
                        + input_offset
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c1 = builder.mul_extension(
                    sub_tmp1,
                    local_values[start_col
                        + MULTIPLY_BY_014_T6_CALC_OFFSET
                        + addition_offset
                        + FP_ADDITION_CHECK_OFFSET],
                );
                let c = builder.mul_extension(bit_selector_val, c1);
                yield_constr.constraint(builder, c);

                let sub_tmp2 = builder.sub_extension(
                    local_values[start_col
                        + MULTIPLY_BY_014_T6_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + subtraction_offset
                        + FP_SUBTRACTION_Y_OFFSET
                        + i],
                    local_values[start_col
                        + MULTIPLY_BY_014_T0_CALC_OFFSET
                        + input_offset
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c2 = builder.mul_extension(
                    sub_tmp2,
                    local_values[start_col
                        + MULTIPLY_BY_014_T6_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + subtraction_offset
                        + FP_SUBTRACTION_CHECK_OFFSET],
                );
                let c = builder.mul_extension(bit_selector_val, c2);
                yield_constr.constraint(builder, c);
            }
        }
    }
    add_subtraction_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_014_T6_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for j in 0..6 {
        let (addition_offset, subtraction_offset, input_offset) = if j == 0 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET + FP2_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_1_X_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
            )
        } else if j == 1 {
            (
                FP6_ADDITION_0_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_0_OFFSET + FP2_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_1_X_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                    + FP_SINGLE_REDUCED_OFFSET,
            )
        } else if j == 2 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_1_OFFSET + FP2_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_1_Y_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        } else if j == 3 {
            (
                FP6_ADDITION_1_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET + FP2_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_1_Y_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        } else if j == 4 {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_2_OFFSET + FP2_SUBTRACTION_0_OFFSET,
                MULTIPLY_BY_1_Z_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET + FP2_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_2_OFFSET + FP2_SUBTRACTION_1_OFFSET,
                MULTIPLY_BY_1_Z_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET,
            )
        };
        for i in 0..12 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_Y_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + MULTIPLY_BY_014_T6_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(
                sub_tmp1,
                local_values[start_col
                    + MULTIPLY_BY_014_Y_CALC_OFFSET
                    + addition_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + MULTIPLY_BY_014_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + subtraction_offset
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[start_col + MULTIPLY_BY_014_T1_CALC_OFFSET + input_offset + i],
            );
            let c2 = builder.mul_extension(
                sub_tmp2,
                local_values[start_col
                    + MULTIPLY_BY_014_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + subtraction_offset
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_subtraction_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + MULTIPLY_BY_014_Y_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints fp12 multiplication.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints all the Ti's (defined in the [function](super::native::mul_fp_12)) accordinng to their respective operations.
pub fn add_fp12_multiplication_constraints<
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

    for i in 0..24 * 3 * 2 {
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP12_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i]
                    - next_values[start_col + FP12_MUL_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint_transition(
            bit_selector_val
                * local_values[start_col + FP12_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i]
                    - next_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i]),
        );
    }

    // T0
    for i in 0..24 * 3 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_X_INPUT_OFFSET + i]
                    - local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i]),
        );
    }
    add_fp6_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_MUL_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..24 * 3 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_X_INPUT_OFFSET + i]
                    - local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i + 24 * 3]),
        );
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_Y_INPUT_OFFSET + i]
                    - local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i + 24 * 3]),
        );
    }
    add_fp6_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_MUL_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for i in 0..6 {
        let fp2_offset = if i < 2 {
            FP6_MUL_X_CALC_OFFSET
        } else if i < 4 {
            FP6_MUL_Y_CALC_OFFSET
        } else {
            FP6_MUL_Z_CALC_OFFSET
        };
        let fp_offset = i % 2;
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values
                        [start_col + FP12_MUL_T2_CALC_OFFSET + FP6_NON_RESIDUE_MUL_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                        + i * 12
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T1_CALC_OFFSET
                            + fp2_offset
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * fp_offset
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
        }
    }
    add_non_residue_multiplication_fp6_constraints(
        local_values,
        yield_constr,
        start_col + FP12_MUL_T2_CALC_OFFSET,
        bit_selector,
    );

    // X
    for i in 0..6 {
        let (fp2_offset_l, fp2_offset_r) = if i < 2 {
            (FP6_ADDITION_0_OFFSET, FP6_MUL_X_CALC_OFFSET)
        } else if i < 4 {
            (FP6_ADDITION_1_OFFSET, FP6_MUL_Y_CALC_OFFSET)
        } else {
            (FP6_ADDITION_2_OFFSET, FP6_MUL_Z_CALC_OFFSET)
        };
        let (fp_offset, num_redn) = if i % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, 0)
        } else {
            (FP2_ADDITION_1_OFFSET, 1)
        };
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_X_CALC_OFFSET
                        + fp2_offset_l
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_X_CALC_OFFSET
                        + fp2_offset_l
                        + fp_offset
                        + FP_ADDITION_X_OFFSET
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T0_CALC_OFFSET
                            + fp2_offset_r
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
            if i < 2 {
                let y_offset = if i == 0 {
                    FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                } else {
                    FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                };
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col
                            + FP12_MUL_X_CALC_OFFSET
                            + fp2_offset_l
                            + fp_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + FP12_MUL_X_CALC_OFFSET
                            + fp2_offset_l
                            + fp_offset
                            + FP_ADDITION_Y_OFFSET
                            + j]
                            - local_values[start_col
                                + FP12_MUL_T2_CALC_OFFSET
                                + FP6_NON_RESIDUE_MUL_C2
                                + y_offset
                                + FP_SINGLE_REDUCED_OFFSET
                                + j]),
                )
            } else {
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col
                            + FP12_MUL_X_CALC_OFFSET
                            + fp2_offset_l
                            + fp_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + FP12_MUL_X_CALC_OFFSET
                            + fp2_offset_l
                            + fp_offset
                            + FP_ADDITION_Y_OFFSET
                            + j]
                            - local_values[start_col
                                + FP12_MUL_T2_CALC_OFFSET
                                + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                                + (i - 2) * 12
                                + j]),
                );
            }
        }
    }
    add_addition_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + FP12_MUL_X_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for i in 0..6 {
        let fp2_offset = if i < 2 {
            FP6_ADDITION_0_OFFSET
        } else if i < 4 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        let fp_offset = if i % 2 == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_T3_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T3_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_X_OFFSET
                        + j]
                        - local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i * 12 + j]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_T3_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T3_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_Y_OFFSET
                        + j]
                        - local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i * 12 + j + 24 * 3]),
            );
        }
    }
    add_addition_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + FP12_MUL_T3_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for i in 0..6 {
        let fp2_offset = if i < 2 {
            FP6_ADDITION_0_OFFSET
        } else if i < 4 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        let fp_offset = if i % 2 == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_T4_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T4_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_X_OFFSET
                        + j]
                        - local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i * 12 + j]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_T4_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T4_CALC_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_Y_OFFSET
                        + j]
                        - local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i * 12 + j + 24 * 3]),
            );
        }
    }
    add_addition_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + FP12_MUL_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..6 {
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP12_MUL_T5_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T5_CALC_OFFSET
                        + FP6_MUL_X_INPUT_OFFSET
                        + i * 12
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T3_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + FP12_MUL_T5_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T5_CALC_OFFSET
                        + FP6_MUL_Y_INPUT_OFFSET
                        + i * 12
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T4_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
        }
    }
    add_fp6_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_MUL_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for i in 0..6 {
        let (fp2_offset_lx, fp2_offset_ly, fp2_offset_r) = if i < 2 {
            (
                FP6_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET,
                FP6_MUL_X_CALC_OFFSET,
            )
        } else if i < 4 {
            (
                FP6_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET,
                FP6_MUL_Y_CALC_OFFSET,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET,
                FP6_SUBTRACTION_2_OFFSET,
                FP6_MUL_Z_CALC_OFFSET,
            )
        };
        let (fp_offset_x, fp_offset_y, num_redn) = if i % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET, 0)
        } else {
            (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET, 1)
        };
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_T6_CALC_OFFSET
                        + fp2_offset_lx
                        + fp_offset_x
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T6_CALC_OFFSET
                        + fp2_offset_lx
                        + fp_offset_x
                        + FP_ADDITION_X_OFFSET
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T5_CALC_OFFSET
                            + fp2_offset_r
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_T6_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + fp2_offset_ly
                        + fp_offset_y
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_T6_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + fp2_offset_ly
                        + fp_offset_y
                        + FP_SUBTRACTION_Y_OFFSET
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T0_CALC_OFFSET
                            + fp2_offset_r
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
        }
    }
    add_subtraction_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + FP12_MUL_T6_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for i in 0..6 {
        let (fp2_offset_lx, fp2_offset_ly, fp2_offset_r) = if i < 2 {
            (
                FP6_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET,
                FP6_MUL_X_CALC_OFFSET,
            )
        } else if i < 4 {
            (
                FP6_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET,
                FP6_MUL_Y_CALC_OFFSET,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET,
                FP6_SUBTRACTION_2_OFFSET,
                FP6_MUL_Z_CALC_OFFSET,
            )
        };
        let (fp_offset_x, fp_offset_y, num_redn) = if i % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET, 0)
        } else {
            (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET, 1)
        };
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_Y_CALC_OFFSET
                        + fp2_offset_lx
                        + fp_offset_x
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_Y_CALC_OFFSET
                        + fp2_offset_lx
                        + fp_offset_x
                        + FP_ADDITION_X_OFFSET
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T6_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col
                        + FP12_MUL_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + fp2_offset_ly
                        + fp_offset_y
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + FP12_MUL_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + fp2_offset_ly
                        + fp_offset_y
                        + FP_SUBTRACTION_Y_OFFSET
                        + j]
                        - local_values[start_col
                            + FP12_MUL_T1_CALC_OFFSET
                            + fp2_offset_r
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                            + FP_SINGLE_REDUCED_OFFSET
                            + j]),
            );
        }
    }
    add_subtraction_with_reduction_constranints_fp6(
        local_values,
        yield_constr,
        start_col + FP12_MUL_Y_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_fp12_multiplication_constraints_ext_circuit<
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

    for i in 0..24 * 3 * 2 {
        let mul_tmp = local_values[start_col + FP12_MUL_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i],
            next_values[start_col + FP12_MUL_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint_transition(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i],
            next_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint_transition(builder, c);
    }

    // T0
    for i in 0..24 * 3 {
        let mul_tmp = local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_X_INPUT_OFFSET + i],
            local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP12_MUL_T0_CALC_OFFSET + FP6_MUL_Y_INPUT_OFFSET + i],
            local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp6_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_MUL_T0_CALC_OFFSET,
        bit_selector,
    );

    // T1
    for i in 0..24 * 3 {
        let mul_tmp = local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET];

        let sub_tmp1 = builder.sub_extension(
            local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_X_INPUT_OFFSET + i],
            local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i + 24 * 3],
        );
        let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint(builder, c);

        let sub_tmp2 = builder.sub_extension(
            local_values[start_col + FP12_MUL_T1_CALC_OFFSET + FP6_MUL_Y_INPUT_OFFSET + i],
            local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i + 24 * 3],
        );
        let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint(builder, c);
    }
    add_fp6_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_MUL_T1_CALC_OFFSET,
        bit_selector,
    );

    // T2
    for i in 0..6 {
        let fp2_offset = if i < 2 {
            FP6_MUL_X_CALC_OFFSET
        } else if i < 4 {
            FP6_MUL_Y_CALC_OFFSET
        } else {
            FP6_MUL_Z_CALC_OFFSET
        };
        let fp_offset = i % 2;
        for j in 0..12 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T2_CALC_OFFSET
                    + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i * 12
                    + j],
                local_values[start_col
                    + FP12_MUL_T1_CALC_OFFSET
                    + fp2_offset
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * fp_offset
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c = builder.mul_extension(
                sub_tmp,
                local_values
                    [start_col + FP12_MUL_T2_CALC_OFFSET + FP6_NON_RESIDUE_MUL_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_non_residue_multiplication_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_MUL_T2_CALC_OFFSET,
        bit_selector,
    );

    // X
    for i in 0..6 {
        let (fp2_offset_l, fp2_offset_r) = if i < 2 {
            (FP6_ADDITION_0_OFFSET, FP6_MUL_X_CALC_OFFSET)
        } else if i < 4 {
            (FP6_ADDITION_1_OFFSET, FP6_MUL_Y_CALC_OFFSET)
        } else {
            (FP6_ADDITION_2_OFFSET, FP6_MUL_Z_CALC_OFFSET)
        };
        let (fp_offset, num_redn) = if i % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, 0)
        } else {
            (FP2_ADDITION_1_OFFSET, 1)
        };
        for j in 0..12 {
            let mul_tmp = local_values[start_col
                + FP12_MUL_X_CALC_OFFSET
                + fp2_offset_l
                + fp_offset
                + FP_ADDITION_CHECK_OFFSET];

            let sub_tmp = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_X_CALC_OFFSET
                    + fp2_offset_l
                    + fp_offset
                    + FP_ADDITION_X_OFFSET
                    + j],
                local_values[start_col
                    + FP12_MUL_T0_CALC_OFFSET
                    + fp2_offset_r
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);

            if i < 2 {
                let y_offset = if i == 0 {
                    FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET
                } else {
                    FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET
                };

                let sub_tmp = builder.sub_extension(
                    local_values[start_col
                        + FP12_MUL_X_CALC_OFFSET
                        + fp2_offset_l
                        + fp_offset
                        + FP_ADDITION_Y_OFFSET
                        + j],
                    local_values[start_col
                        + FP12_MUL_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_C2
                        + y_offset
                        + FP_SINGLE_REDUCED_OFFSET
                        + j],
                );
                let c = builder.mul_extension(sub_tmp, mul_tmp);
                let c = builder.mul_extension(bit_selector_val, c);
                yield_constr.constraint(builder, c);
            } else {
                let sub_tmp = builder.sub_extension(
                    local_values[start_col
                        + FP12_MUL_X_CALC_OFFSET
                        + fp2_offset_l
                        + fp_offset
                        + FP_ADDITION_Y_OFFSET
                        + j],
                    local_values[start_col
                        + FP12_MUL_T2_CALC_OFFSET
                        + FP6_NON_RESIDUE_MUL_INPUT_OFFSET
                        + (i - 2) * 12
                        + j],
                );
                let c = builder.mul_extension(sub_tmp, mul_tmp);
                let c = builder.mul_extension(bit_selector_val, c);
                yield_constr.constraint(builder, c);
            }
        }
    }
    add_addition_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_MUL_X_CALC_OFFSET,
        bit_selector,
    );

    // T3
    for i in 0..6 {
        let fp2_offset = if i < 2 {
            FP6_ADDITION_0_OFFSET
        } else if i < 4 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        let fp_offset = if i % 2 == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for j in 0..12 {
            let mul_tmp = local_values[start_col
                + FP12_MUL_T3_CALC_OFFSET
                + fp2_offset
                + fp_offset
                + FP_ADDITION_CHECK_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T3_CALC_OFFSET
                    + fp2_offset
                    + fp_offset
                    + FP_ADDITION_X_OFFSET
                    + j],
                local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i * 12 + j],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T3_CALC_OFFSET
                    + fp2_offset
                    + fp_offset
                    + FP_ADDITION_Y_OFFSET
                    + j],
                local_values[start_col + FP12_MUL_X_INPUT_OFFSET + i * 12 + j + 24 * 3],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_MUL_T3_CALC_OFFSET,
        bit_selector,
    );

    // T4
    for i in 0..6 {
        let fp2_offset = if i < 2 {
            FP6_ADDITION_0_OFFSET
        } else if i < 4 {
            FP6_ADDITION_1_OFFSET
        } else {
            FP6_ADDITION_2_OFFSET
        };
        let fp_offset = if i % 2 == 0 {
            FP2_ADDITION_0_OFFSET
        } else {
            FP2_ADDITION_1_OFFSET
        };
        for j in 0..12 {
            let mul_tmp = local_values[start_col
                + FP12_MUL_T4_CALC_OFFSET
                + fp2_offset
                + fp_offset
                + FP_ADDITION_CHECK_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T4_CALC_OFFSET
                    + fp2_offset
                    + fp_offset
                    + FP_ADDITION_X_OFFSET
                    + j],
                local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i * 12 + j],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T4_CALC_OFFSET
                    + fp2_offset
                    + fp_offset
                    + FP_ADDITION_Y_OFFSET
                    + j],
                local_values[start_col + FP12_MUL_Y_INPUT_OFFSET + i * 12 + j + 24 * 3],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_MUL_T4_CALC_OFFSET,
        bit_selector,
    );

    // T5
    for i in 0..6 {
        for j in 0..12 {
            let mul_tmp =
                local_values[start_col + FP12_MUL_T5_CALC_OFFSET + FP6_MUL_SELECTOR_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values
                    [start_col + FP12_MUL_T5_CALC_OFFSET + FP6_MUL_X_INPUT_OFFSET + i * 12 + j],
                local_values[start_col
                    + FP12_MUL_T3_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values
                    [start_col + FP12_MUL_T5_CALC_OFFSET + FP6_MUL_Y_INPUT_OFFSET + i * 12 + j],
                local_values[start_col
                    + FP12_MUL_T4_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp6_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_MUL_T5_CALC_OFFSET,
        bit_selector,
    );

    // T6
    for i in 0..6 {
        let (fp2_offset_lx, fp2_offset_ly, fp2_offset_r) = if i < 2 {
            (
                FP6_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET,
                FP6_MUL_X_CALC_OFFSET,
            )
        } else if i < 4 {
            (
                FP6_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET,
                FP6_MUL_Y_CALC_OFFSET,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET,
                FP6_SUBTRACTION_2_OFFSET,
                FP6_MUL_Z_CALC_OFFSET,
            )
        };
        let (fp_offset_x, fp_offset_y, num_redn) = if i % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET, 0)
        } else {
            (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET, 1)
        };
        for j in 0..12 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T6_CALC_OFFSET
                    + fp2_offset_lx
                    + fp_offset_x
                    + FP_ADDITION_X_OFFSET
                    + j],
                local_values[start_col
                    + FP12_MUL_T5_CALC_OFFSET
                    + fp2_offset_r
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c1 = builder.mul_extension(
                sub_tmp1,
                local_values[start_col
                    + FP12_MUL_T6_CALC_OFFSET
                    + fp2_offset_lx
                    + fp_offset_x
                    + FP_ADDITION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_T6_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + fp2_offset_ly
                    + fp_offset_y
                    + FP_SUBTRACTION_Y_OFFSET
                    + j],
                local_values[start_col
                    + FP12_MUL_T0_CALC_OFFSET
                    + fp2_offset_r
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c2 = builder.mul_extension(
                sub_tmp2,
                local_values[start_col
                    + FP12_MUL_T6_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + fp2_offset_ly
                    + fp_offset_y
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_subtraction_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_MUL_T6_CALC_OFFSET,
        bit_selector,
    );

    // Y
    for i in 0..6 {
        let (fp2_offset_lx, fp2_offset_ly, fp2_offset_r) = if i < 2 {
            (
                FP6_ADDITION_0_OFFSET,
                FP6_SUBTRACTION_0_OFFSET,
                FP6_MUL_X_CALC_OFFSET,
            )
        } else if i < 4 {
            (
                FP6_ADDITION_1_OFFSET,
                FP6_SUBTRACTION_1_OFFSET,
                FP6_MUL_Y_CALC_OFFSET,
            )
        } else {
            (
                FP6_ADDITION_2_OFFSET,
                FP6_SUBTRACTION_2_OFFSET,
                FP6_MUL_Z_CALC_OFFSET,
            )
        };
        let (fp_offset_x, fp_offset_y, num_redn) = if i % 2 == 0 {
            (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET, 0)
        } else {
            (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET, 1)
        };
        for j in 0..12 {
            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_Y_CALC_OFFSET
                    + fp2_offset_lx
                    + fp_offset_x
                    + FP_ADDITION_X_OFFSET
                    + j],
                local_values[start_col
                    + FP12_MUL_T6_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * i
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c1 = builder.mul_extension(
                sub_tmp1,
                local_values[start_col
                    + FP12_MUL_Y_CALC_OFFSET
                    + fp2_offset_lx
                    + fp_offset_x
                    + FP_ADDITION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + FP12_MUL_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + fp2_offset_ly
                    + fp_offset_y
                    + FP_SUBTRACTION_Y_OFFSET
                    + j],
                local_values[start_col
                    + FP12_MUL_T1_CALC_OFFSET
                    + fp2_offset_r
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * num_redn
                    + FP_SINGLE_REDUCED_OFFSET
                    + j],
            );
            let c2 = builder.mul_extension(
                sub_tmp2,
                local_values[start_col
                    + FP12_MUL_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + fp2_offset_ly
                    + fp_offset_y
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
    }
    add_subtraction_with_reduction_constraints_fp6_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_MUL_Y_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints for [cyclotomicSquare](super::native::Fp12::cyclotomicSquare) function.
///
/// Constraints inputs across this and next row, wherever selector is set to on. Constraints all the Ti's (defined in the native function) accordinng to their respective operations.
pub fn add_cyclotomic_sq_constraints<
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
    for i in 0..24 * 3 * 2 {
        yield_constr.constraint_transition(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_SELECTOR_OFFSET]
                * (local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + i]
                    - next_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + i]),
        );
    }

    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_INPUT_X_OFFSET + i]
                    - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_INPUT_Y_OFFSET + i]
                    - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 4 + i]),
        );
    }
    add_fp4_sq_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_INPUT_X_OFFSET + i]
                    - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 3 + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_INPUT_Y_OFFSET + i]
                    - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 2 + i]),
        );
    }
    add_fp4_sq_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_INPUT_X_OFFSET + i]
                    - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 1 + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_INPUT_Y_OFFSET + i]
                    - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 5 + i]),
        );
    }
    add_fp4_sq_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i]
                    - local_values[start_col
                        + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                        + FP4_SQ_Y_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET]
                * (local_values[start_col
                    + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                    + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                    + i
                    + 12]
                    - local_values[start_col
                        + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                        + FP4_SQ_Y_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]),
        );
    }
    add_non_residue_multiplication_constraints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, fp_sub_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                            + FP4_SQ_X_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + fp_sub_offset
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + fp_sub_offset
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i]),
            )
        }
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T4_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T5_CALC_OFFSET
                        + FP2_FP_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        let val = if i == 0 { FE::TWO } else { FE::ZERO };
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                    - val),
        );
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T5_CALC_OFFSET
                            + x_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                            + FP4_SQ_X_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_C0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, fp_sub_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                            + FP4_SQ_X_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + fp_sub_offset
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + fp_sub_offset
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24]),
            )
        }
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T6_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T7_CALC_OFFSET
                        + FP2_FP_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        let val = if i == 0 { FE::TWO } else { FE::ZERO };
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                    - val),
        );
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T7_CALC_OFFSET
                            + x_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                            + FP4_SQ_X_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_C1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, fp_sub_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                            + FP4_SQ_X_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + fp_sub_offset
                        + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + fp_sub_offset
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 2]),
            )
        }
    }
    add_subtraction_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T8_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T9_CALC_OFFSET
                        + FP2_FP_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        let val = if i == 0 { FE::TWO } else { FE::ZERO };
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                    - val),
        );
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T9_CALC_OFFSET
                            + x_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                            + FP4_SQ_X_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_C2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, x_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                            + x_offset
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values
                            [start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 3]),
            )
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T10_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T11_CALC_OFFSET
                        + FP2_FP_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        let val = if i == 0 { FE::TWO } else { FE::ZERO };
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                    - val),
        );
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, y_offset, fp_add_offset) = if j == 0 {
                (
                    X0_Y_REDUCE_OFFSET,
                    FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET,
                    FP2_ADDITION_0_OFFSET,
                )
            } else {
                (
                    X1_Y_REDUCE_OFFSET,
                    FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET,
                    FP2_ADDITION_1_OFFSET,
                )
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T11_CALC_OFFSET
                            + x_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                            + y_offset
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_C3_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let fp_add_offset = if j == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                            + FP4_SQ_Y_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values
                            [start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 4]),
            )
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T12_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T13_CALC_OFFSET
                        + FP2_FP_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        let val = if i == 0 { FE::TWO } else { FE::ZERO };
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                    - val),
        );
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T13_CALC_OFFSET
                            + x_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                            + FP4_SQ_Y_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_C4_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let fp_add_offset = if j == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                            + FP4_SQ_Y_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values
                            [start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 5]),
            )
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T14_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_T15_CALC_OFFSET
                        + FP2_FP_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
        let val = if i == 0 { FE::TWO } else { FE::ZERO };
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values
                    [start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values
                    [start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                    - val),
        );
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_X_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T15_CALC_OFFSET
                            + x_offset
                            + REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values[start_col
                        + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[start_col
                        + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i]
                        - local_values[start_col
                            + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                            + FP4_SQ_Y_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }
    add_addition_with_reduction_constranints(
        local_values,
        yield_constr,
        start_col + CYCLOTOMIC_SQ_C5_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_cyclotomic_sq_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..24 * 3 * 2 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_SELECTOR_OFFSET],
        );
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + i],
            next_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_INPUT_X_OFFSET + i],
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET + FP4_SQ_INPUT_Y_OFFSET + i],
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 4 + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp4_sq_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_INPUT_X_OFFSET + i],
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 3 + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET + FP4_SQ_INPUT_Y_OFFSET + i],
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 2 + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp4_sq_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..24 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_INPUT_X_OFFSET + i],
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 1 + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET + FP4_SQ_INPUT_Y_OFFSET + i],
            local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + 24 * 5 + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp4_sq_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values
                [start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_CHECK_OFFSET],
        );

        let c = builder.sub_extension(
            local_values
                [start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET + FP2_NON_RESIDUE_MUL_INPUT_OFFSET + i],
            local_values[start_col
                + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col
                + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                + FP2_NON_RESIDUE_MUL_INPUT_OFFSET
                + i
                + 12],
            local_values[start_col
                + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                + FP4_SQ_Y_CALC_OFFSET
                + FP2_ADDITION_TOTAL
                + FP2_SUBTRACTION_TOTAL
                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL)
                + FP_SINGLE_REDUCED_OFFSET
                + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_non_residue_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T3_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, fp_sub_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
            };
            let tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );
            let tmp2 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + fp_sub_offset
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp1, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + fp_sub_offset
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i],
            );
            let c = builder.mul_extension(tmp2, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T4_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        for j in 0..2 {
            let c = builder.sub_extension(
                local_values
                    [start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + j * 12 + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T4_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
        let val = if i == 0 {
            builder.constant_extension(F::Extension::TWO)
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
            val,
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values
                    [start_col + CYCLOTOMIC_SQ_T5_CALC_OFFSET + x_offset + REDUCED_OFFSET + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C0_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_C0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, fp_sub_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
            };
            let tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );
            let tmp2 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + fp_sub_offset
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp1, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + fp_sub_offset
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24],
            );
            let c = builder.mul_extension(tmp2, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T6_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        for j in 0..2 {
            let c = builder.sub_extension(
                local_values
                    [start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + j * 12 + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
        let val = if i == 0 {
            builder.constant_extension(F::Extension::TWO)
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
            val,
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values
                    [start_col + CYCLOTOMIC_SQ_T7_CALC_OFFSET + x_offset + REDUCED_OFFSET + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C1_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_C1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, fp_sub_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
            };
            let tmp1 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );
            let tmp2 = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + fp_sub_offset
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp1, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + fp_sub_offset
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 2],
            );
            let c = builder.mul_extension(tmp2, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_subtraction_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T8_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        for j in 0..2 {
            let c = builder.sub_extension(
                local_values
                    [start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + j * 12 + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T8_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
        let val = if i == 0 {
            builder.constant_extension(F::Extension::TWO)
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
            val,
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values
                    [start_col + CYCLOTOMIC_SQ_T9_CALC_OFFSET + x_offset + REDUCED_OFFSET + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C2_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T2_CALC_OFFSET
                    + FP4_SQ_X_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_C2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (fp_add_offset, x_offset) = if j == 0 {
                (FP2_ADDITION_0_OFFSET, FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET)
            } else {
                (FP2_ADDITION_1_OFFSET, FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET)
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                    + x_offset
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 3],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T10_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        for j in 0..2 {
            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T11_CALC_OFFSET
                    + FP2_FP_X_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T10_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
        let val = if i == 0 {
            builder.constant_extension(F::Extension::TWO)
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
            val,
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, y_offset, fp_add_offset) = if j == 0 {
                (
                    X0_Y_REDUCE_OFFSET,
                    FP2_NON_RESIDUE_MUL_Z0_REDUCE_OFFSET,
                    FP2_ADDITION_0_OFFSET,
                )
            } else {
                (
                    X1_Y_REDUCE_OFFSET,
                    FP2_NON_RESIDUE_MUL_Z1_REDUCE_OFFSET,
                    FP2_ADDITION_1_OFFSET,
                )
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values
                    [start_col + CYCLOTOMIC_SQ_T11_CALC_OFFSET + x_offset + REDUCED_OFFSET + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C3_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T3_CALC_OFFSET
                    + y_offset
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_C3_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let fp_add_offset = if j == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 4],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T12_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        for j in 0..2 {
            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T13_CALC_OFFSET
                    + FP2_FP_X_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T12_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
        let val = if i == 0 {
            builder.constant_extension(F::Extension::TWO)
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
            val,
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values
                    [start_col + CYCLOTOMIC_SQ_T13_CALC_OFFSET + x_offset + REDUCED_OFFSET + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C4_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T0_CALC_OFFSET
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_C4_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let fp_add_offset = if j == 0 {
                FP2_ADDITION_0_OFFSET
            } else {
                FP2_ADDITION_1_OFFSET
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col + CYCLOTOMIC_SQ_INPUT_OFFSET + j * 12 + i + 24 * 5],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_T14_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        let tmp = builder.mul_extension(
            bit_selector_val,
            local_values[start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
        );
        for j in 0..2 {
            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_T15_CALC_OFFSET
                    + FP2_FP_X_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
        let val = if i == 0 {
            builder.constant_extension(F::Extension::TWO)
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let c = builder.sub_extension(
            local_values[start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
            val,
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let (x_offset, fp_add_offset) = if j == 0 {
                (X0_Y_REDUCE_OFFSET, FP2_ADDITION_0_OFFSET)
            } else {
                (X1_Y_REDUCE_OFFSET, FP2_ADDITION_1_OFFSET)
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values[start_col
                    + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_CHECK_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_X_OFFSET
                    + i],
                local_values
                    [start_col + CYCLOTOMIC_SQ_T15_CALC_OFFSET + x_offset + REDUCED_OFFSET + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + CYCLOTOMIC_SQ_C5_CALC_OFFSET
                    + fp_add_offset
                    + FP_ADDITION_Y_OFFSET
                    + i],
                local_values[start_col
                    + CYCLOTOMIC_SQ_T1_CALC_OFFSET
                    + FP4_SQ_Y_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_addition_with_reduction_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + CYCLOTOMIC_SQ_C5_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints for [cyclotomicExponent](super::native::Fp12::cyclotocmicExponent) function.
///
/// Constraints inputs across this and next row, wherever selector is set to on. When `CYCLOTOMIC_EXP_START_ROW` is set, constraints z to be 1. Creates two `bit_selector` values from `BIT1_SELECTOR`. Constraints cyclotomicSquare function with `bit0` and constraints fp12 multiplication with `bit1`. What it does is switch on the constraints of cyclotomicSquare when `BIT1_SELECTOR` is off and switch on the constraints of fp12 multiplication when `BIT1_SELECTOR` is on. When `FIRST_ROW_SELECTOR` is on in the next row, constraints z value of the next row with result of cyclotmicSquare function and `bit0` of current row and constraints z value of the next row with result of fp12 multiplication and `bit1` of current row.
pub fn add_cyclotomic_exp_constraints<
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
    op_selector: Option<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    for i in 0..24 * 3 * 2 {
        yield_constr.constraint_transition(
            op_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET]
                * (local_values[start_col + INPUT_OFFSET + i]
                    - next_values[start_col + INPUT_OFFSET + i]),
        );
    }
    for i in 0..24 * 3 * 2 {
        let val = if i == 0 { P::ONES } else { P::ZEROS };
        yield_constr.constraint(
            op_selector.unwrap_or(P::ONES)
                * local_values[start_col + CYCLOTOMIC_EXP_START_ROW]
                * (local_values[start_col + Z_OFFSET + i] - val),
        );
    }

    let bit1 = (local_values[start_col + BIT1_SELECTOR_OFFSET]) * op_selector.unwrap_or(P::ONES);
    let bit0 =
        (P::ONES - local_values[start_col + BIT1_SELECTOR_OFFSET]) * op_selector.unwrap_or(P::ONES);

    for i in 0..12 {
        for j in 0..6 {
            let c_offset = if j == 0 {
                CYCLOTOMIC_SQ_C0_CALC_OFFSET
            } else if j == 1 {
                CYCLOTOMIC_SQ_C1_CALC_OFFSET
            } else if j == 2 {
                CYCLOTOMIC_SQ_C2_CALC_OFFSET
            } else if j == 3 {
                CYCLOTOMIC_SQ_C3_CALC_OFFSET
            } else if j == 4 {
                CYCLOTOMIC_SQ_C4_CALC_OFFSET
            } else {
                CYCLOTOMIC_SQ_C5_CALC_OFFSET
            };
            for k in 0..2 {
                yield_constr.constraint_transition(
                    bit0 * local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET]
                        * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[start_col + Z_OFFSET + j * 24 + k * 12 + i]
                            - local_values[start_col
                                + Z_CYCLOTOMIC_SQ_OFFSET
                                + c_offset
                                + FP2_ADDITION_TOTAL
                                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
            }
        }
    }

    for i in 0..12 {
        for j in 0..6 {
            yield_constr.constraint_transition(
                bit1 * local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET]
                    * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                    * (next_values[start_col + Z_OFFSET + j * 12 + i]
                        - local_values[start_col
                            + Z_MUL_INPUT_OFFSET
                            + FP12_MUL_X_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint_transition(
                bit1 * local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET]
                    * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                    * (next_values[start_col + Z_OFFSET + j * 12 + i + 24 * 3]
                        - local_values[start_col
                            + Z_MUL_INPUT_OFFSET
                            + FP12_MUL_Y_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }

    for i in 0..24 * 3 * 2 {
        yield_constr.constraint(
            bit0 * local_values[start_col + Z_CYCLOTOMIC_SQ_OFFSET + CYCLOTOMIC_SQ_SELECTOR_OFFSET]
                * (local_values
                    [start_col + Z_CYCLOTOMIC_SQ_OFFSET + CYCLOTOMIC_SQ_INPUT_OFFSET + i]
                    - local_values[start_col + Z_OFFSET + i]),
        );
    }
    add_cyclotomic_sq_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Z_CYCLOTOMIC_SQ_OFFSET,
        Some(bit0),
    );

    for i in 0..24 * 3 * 2 {
        yield_constr.constraint(
            bit1 * local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_X_INPUT_OFFSET + i]
                    - local_values[start_col + Z_OFFSET + i]),
        );
        yield_constr.constraint(
            bit1 * local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_Y_INPUT_OFFSET + i]
                    - local_values[start_col + INPUT_OFFSET + i]),
        );
    }
    add_fp12_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + Z_MUL_INPUT_OFFSET,
        Some(bit1),
    );

    for i in 0..12 {
        for j in 0..6 {
            let c_offset = if j == 0 {
                CYCLOTOMIC_SQ_C0_CALC_OFFSET
            } else if j == 1 {
                CYCLOTOMIC_SQ_C1_CALC_OFFSET
            } else if j == 2 {
                CYCLOTOMIC_SQ_C2_CALC_OFFSET
            } else if j == 3 {
                CYCLOTOMIC_SQ_C3_CALC_OFFSET
            } else if j == 4 {
                CYCLOTOMIC_SQ_C4_CALC_OFFSET
            } else {
                CYCLOTOMIC_SQ_C5_CALC_OFFSET
            };
            for k in 0..2 {
                yield_constr.constraint_transition(
                    op_selector.unwrap_or(P::ONES)
                        * local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET]
                        * next_values[start_col + RES_ROW_SELECTOR_OFFSET]
                        * (next_values[start_col + Z_OFFSET + j * 24 + k * 12 + i]
                            - local_values[start_col
                                + Z_CYCLOTOMIC_SQ_OFFSET
                                + c_offset
                                + FP2_ADDITION_TOTAL
                                + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
            }
        }
    }
}

pub fn add_cyclotomic_exp_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    op_selector: Option<ExtensionTarget<D>>,
) {
    let one = builder.constant_extension(F::Extension::ONE);
    let op_selector_val = op_selector.unwrap_or(one);

    for i in 0..24 * 3 * 2 {
        let tmp = builder.mul_extension(
            op_selector_val,
            local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + INPUT_OFFSET + i],
            next_values[start_col + INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);
    }

    for i in 0..24 * 3 * 2 {
        let val = if i == 0 {
            one
        } else {
            builder.constant_extension(F::Extension::ZERO)
        };
        let tmp = builder.mul_extension(
            op_selector_val,
            local_values[start_col + CYCLOTOMIC_EXP_START_ROW],
        );

        let c = builder.sub_extension(local_values[start_col + Z_OFFSET + i], val);
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }

    let bit1 = builder.mul_extension(
        op_selector_val,
        local_values[start_col + BIT1_SELECTOR_OFFSET],
    );
    let bit0 = builder.sub_extension(one, local_values[start_col + BIT1_SELECTOR_OFFSET]);
    let bit0 = builder.mul_extension(op_selector_val, bit0);

    for i in 0..12 {
        for j in 0..6 {
            let c_offset = if j == 0 {
                CYCLOTOMIC_SQ_C0_CALC_OFFSET
            } else if j == 1 {
                CYCLOTOMIC_SQ_C1_CALC_OFFSET
            } else if j == 2 {
                CYCLOTOMIC_SQ_C2_CALC_OFFSET
            } else if j == 3 {
                CYCLOTOMIC_SQ_C3_CALC_OFFSET
            } else if j == 4 {
                CYCLOTOMIC_SQ_C4_CALC_OFFSET
            } else {
                CYCLOTOMIC_SQ_C5_CALC_OFFSET
            };
            let mul = builder.mul_extension(
                bit0,
                local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET],
            );
            let mul =
                builder.mul_extension(mul, next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]);
            for k in 0..2 {
                let c = builder.sub_extension(
                    next_values[start_col + Z_OFFSET + j * 24 + k * 12 + i],
                    local_values[start_col
                        + Z_CYCLOTOMIC_SQ_OFFSET
                        + c_offset
                        + FP2_ADDITION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c = builder.mul_extension(mul, c);
                yield_constr.constraint_transition(builder, c);
            }
        }
    }

    for i in 0..12 {
        let mul = builder.mul_extension(
            bit1,
            local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET],
        );
        let mul = builder.mul_extension(mul, next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]);
        for j in 0..6 {
            let c = builder.sub_extension(
                next_values[start_col + Z_OFFSET + j * 12 + i],
                local_values[start_col
                    + Z_MUL_INPUT_OFFSET
                    + FP12_MUL_X_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(mul, c);
            yield_constr.constraint_transition(builder, c);

            let c = builder.sub_extension(
                next_values[start_col + Z_OFFSET + j * 12 + i + 24 * 3],
                local_values[start_col
                    + Z_MUL_INPUT_OFFSET
                    + FP12_MUL_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c = builder.mul_extension(mul, c);
            yield_constr.constraint_transition(builder, c);
        }
    }

    for i in 0..24 * 3 * 2 {
        let tmp = builder.mul_extension(
            bit0,
            local_values[start_col + Z_CYCLOTOMIC_SQ_OFFSET + CYCLOTOMIC_SQ_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + Z_CYCLOTOMIC_SQ_OFFSET + CYCLOTOMIC_SQ_INPUT_OFFSET + i],
            local_values[start_col + Z_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_cyclotomic_sq_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Z_CYCLOTOMIC_SQ_OFFSET,
        Some(bit0),
    );

    for i in 0..24 * 3 * 2 {
        let tmp = builder.mul_extension(
            bit1,
            local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_SELECTOR_OFFSET],
        );

        let c = builder.sub_extension(
            local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_X_INPUT_OFFSET + i],
            local_values[start_col + Z_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);

        let c = builder.sub_extension(
            local_values[start_col + Z_MUL_INPUT_OFFSET + FP12_MUL_Y_INPUT_OFFSET + i],
            local_values[start_col + INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp12_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + Z_MUL_INPUT_OFFSET,
        Some(bit1),
    );

    for i in 0..12 {
        let mul = builder.mul_extension(
            op_selector_val,
            local_values[start_col + CYCLOTOMIC_EXP_SELECTOR_OFFSET],
        );
        let mul = builder.mul_extension(mul, next_values[start_col + RES_ROW_SELECTOR_OFFSET]);
        for j in 0..6 {
            let c_offset = if j == 0 {
                CYCLOTOMIC_SQ_C0_CALC_OFFSET
            } else if j == 1 {
                CYCLOTOMIC_SQ_C1_CALC_OFFSET
            } else if j == 2 {
                CYCLOTOMIC_SQ_C2_CALC_OFFSET
            } else if j == 3 {
                CYCLOTOMIC_SQ_C3_CALC_OFFSET
            } else if j == 4 {
                CYCLOTOMIC_SQ_C4_CALC_OFFSET
            } else {
                CYCLOTOMIC_SQ_C5_CALC_OFFSET
            };
            for k in 0..2 {
                let c = builder.sub_extension(
                    next_values[start_col + Z_OFFSET + j * 24 + k * 12 + i],
                    local_values[start_col
                        + Z_CYCLOTOMIC_SQ_OFFSET
                        + c_offset
                        + FP2_ADDITION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * k
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c = builder.mul_extension(mul, c);
                yield_constr.constraint_transition(builder, c);
            }
        }
    }
}

/// Constraints for [forbenius_map](super::native::Fp12::forbenius_map) function.
///
///  Constraints both input and power across this and next row, wherever selector is set to on. Constraint the divisor and remainder with power for `power == divisor*12 + remainder`. Constraints the bit decomposition as `remainder == bit0 + bit1*2 + bit2*4 + bit3*8`. Selects the forbenius constant using mupliplexer logic. Then constraints fp6 forbenius map, multiplication, reduction and range check operations.
pub fn add_fp12_forbenius_map_constraints<
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
    for i in 0..24 * 3 * 2 {
        yield_constr.constraint_transition(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i]
                    - next_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i]),
        );
    }
    yield_constr.constraint_transition(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET]
                - next_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET]),
    );
    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values[start_col + FP12_FORBENIUS_MAP_DIV_OFFSET]
                * FE::from_canonical_usize(12)
                + local_values[start_col + FP12_FORBENIUS_MAP_REM_OFFSET]
                - local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET]),
    );
    let bit0 = local_values[start_col + FP12_FORBENIUS_MAP_BIT0_OFFSET];
    let bit1 = local_values[start_col + FP12_FORBENIUS_MAP_BIT1_OFFSET];
    let bit2 = local_values[start_col + FP12_FORBENIUS_MAP_BIT2_OFFSET];
    let bit3 = local_values[start_col + FP12_FORBENIUS_MAP_BIT3_OFFSET];
    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (bit0
                + bit1 * FE::TWO
                + bit2 * FE::from_canonical_usize(4)
                + bit3 * FE::from_canonical_usize(8)
                - local_values[start_col + FP12_FORBENIUS_MAP_REM_OFFSET]),
    );
    let forbenius_coefficients = Fp12::forbenius_coefficients()
        .iter()
        .map(|fp2| fp2.get_u32_slice().concat().try_into().unwrap())
        .collect::<Vec<[u32; 24]>>();
    let y = (0..24)
        .map(|i| {
            (P::ONES - bit0)
                * (P::ONES - bit1)
                * (P::ONES - bit2)
                * FE::from_canonical_u32(forbenius_coefficients[0][i])
                + (bit0)
                    * (P::ONES - bit1)
                    * (P::ONES - bit2)
                    * FE::from_canonical_u32(forbenius_coefficients[1][i])
                + (P::ONES - bit0)
                    * (bit1)
                    * (P::ONES - bit2)
                    * FE::from_canonical_u32(forbenius_coefficients[2][i])
                + (bit0)
                    * (bit1)
                    * (P::ONES - bit2)
                    * FE::from_canonical_u32(forbenius_coefficients[3][i])
                + (P::ONES - bit0)
                    * (P::ONES - bit1)
                    * (bit2)
                    * FE::from_canonical_u32(forbenius_coefficients[4][i])
                + (bit0)
                    * (P::ONES - bit1)
                    * (bit2)
                    * FE::from_canonical_u32(forbenius_coefficients[5][i])
                + (P::ONES - bit0)
                    * (bit1)
                    * (bit2)
                    * FE::from_canonical_u32(forbenius_coefficients[6][i])
        })
        .collect::<Vec<P>>();
    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values
                [start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values
                [start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_POW_OFFSET]
                - local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET]),
    );
    for i in 0..24 * 3 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP12_FORBENIUS_MAP_R0_CALC_OFFSET
                    + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP12_FORBENIUS_MAP_R0_CALC_OFFSET
                    + FP6_FORBENIUS_MAP_INPUT_OFFSET
                    + i]
                    - local_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i]),
        );
    }
    add_fp6_forbenius_map_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET,
        bit_selector,
    );

    yield_constr.constraint(
        bit_selector.unwrap_or(P::ONES)
            * local_values[start_col
                + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET
                + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
            * (local_values
                [start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + FP6_FORBENIUS_MAP_POW_OFFSET]
                - local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET]),
    );
    for i in 0..24 * 3 {
        yield_constr.constraint(
            bit_selector.unwrap_or(P::ONES)
                * local_values[start_col
                    + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET
                    + FP6_FORBENIUS_MAP_SELECTOR_OFFSET]
                * (local_values[start_col
                    + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET
                    + FP6_FORBENIUS_MAP_INPUT_OFFSET
                    + i]
                    - local_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i + 24 * 3]),
        );
    }
    add_fp6_forbenius_map_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let offset = if j == 0 {
                FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET
            } else {
                FP6_FORBENIUS_MAP_X_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCED_OFFSET
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + FP12_FORBENIUS_MAP_C0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_FORBENIUS_MAP_C0_CALC_OFFSET
                        + FP2_FP2_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values
                            [start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + offset + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + FP12_FORBENIUS_MAP_C0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_FORBENIUS_MAP_C0_CALC_OFFSET
                        + FP2_FP2_Y_INPUT_OFFSET
                        + j * 12
                        + i]
                        - y[j * 12 + i]),
            );
        }
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_FORBENIUS_MAP_C0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let offset = if j == 0 {
                FP6_FORBENIUS_MAP_Y_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else {
                FP6_FORBENIUS_MAP_Y_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + FP12_FORBENIUS_MAP_C1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_FORBENIUS_MAP_C1_CALC_OFFSET
                        + FP2_FP2_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values
                            [start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + offset + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + FP12_FORBENIUS_MAP_C1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_FORBENIUS_MAP_C1_CALC_OFFSET
                        + FP2_FP2_Y_INPUT_OFFSET
                        + j * 12
                        + i]
                        - y[j * 12 + i]),
            );
        }
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_FORBENIUS_MAP_C1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let offset = if j == 0 {
                FP6_FORBENIUS_MAP_Z_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else {
                FP6_FORBENIUS_MAP_Z_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            };
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + FP12_FORBENIUS_MAP_C2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_FORBENIUS_MAP_C2_CALC_OFFSET
                        + FP2_FP2_X_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values
                            [start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + offset + i]),
            );
            yield_constr.constraint(
                bit_selector.unwrap_or(P::ONES)
                    * local_values
                        [start_col + FP12_FORBENIUS_MAP_C2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + FP12_FORBENIUS_MAP_C2_CALC_OFFSET
                        + FP2_FP2_Y_INPUT_OFFSET
                        + j * 12
                        + i]
                        - y[j * 12 + i]),
            );
        }
    }
    add_fp2_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + FP12_FORBENIUS_MAP_C2_CALC_OFFSET,
        bit_selector,
    );
}

pub fn add_fp12_forbenius_map_constraints_ext_circuit<
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
    let one = builder.constant_extension(F::Extension::ONE);
    let bit_selector_val = bit_selector.unwrap_or(one);

    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values[start_col + FP12_FORBENIUS_MAP_SELECTOR_OFFSET],
    );

    for i in 0..24 * 3 * 2 {
        let c = builder.sub_extension(
            local_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i],
            next_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint_transition(builder, c);
    }

    let c = builder.sub_extension(
        local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET],
        next_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint_transition(builder, c);

    let twelve = builder.constant_extension(F::Extension::from_canonical_u32(12));
    let c = builder.mul_extension(
        local_values[start_col + FP12_FORBENIUS_MAP_DIV_OFFSET],
        twelve,
    );
    let c = builder.add_extension(c, local_values[start_col + FP12_FORBENIUS_MAP_REM_OFFSET]);
    let c = builder.sub_extension(c, local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET]);
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    let bit0 = local_values[start_col + FP12_FORBENIUS_MAP_BIT0_OFFSET];
    let bit1 = local_values[start_col + FP12_FORBENIUS_MAP_BIT1_OFFSET];
    let bit2 = local_values[start_col + FP12_FORBENIUS_MAP_BIT2_OFFSET];
    let bit3 = local_values[start_col + FP12_FORBENIUS_MAP_BIT3_OFFSET];
    let one_bit0 = builder.sub_extension(one, bit0);
    let one_bit1 = builder.sub_extension(one, bit1);
    let one_bit2 = builder.sub_extension(one, bit2);

    let two = builder.constant_extension(F::Extension::TWO);
    let four = builder.constant_extension(F::Extension::from_canonical_u32(4));
    let eight = builder.constant_extension(F::Extension::from_canonical_u32(8));
    let c = builder.mul_add_extension(bit1, two, bit0);
    let c = builder.mul_add_extension(bit2, four, c);
    let c = builder.mul_add_extension(bit3, eight, c);
    let c = builder.sub_extension(c, local_values[start_col + FP12_FORBENIUS_MAP_REM_OFFSET]);
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    let forbenius_coefficients = Fp12::forbenius_coefficients()
        .iter()
        .map(|fp2| fp2.get_u32_slice().concat().try_into().unwrap())
        .collect::<Vec<[u32; 24]>>();
    let y = (0..24)
        .map(|i| {
            let fc0 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[0][i],
            ));
            let fc1 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[1][i],
            ));
            let fc2 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[2][i],
            ));
            let fc3 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[3][i],
            ));
            let fc4 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[4][i],
            ));
            let fc5 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[5][i],
            ));
            let fc6 = builder.constant_extension(F::Extension::from_canonical_u32(
                forbenius_coefficients[6][i],
            ));

            let val_zero = builder.mul_many_extension([one_bit0, one_bit1, one_bit2, fc0]);
            let val_one = builder.mul_many_extension([bit0, one_bit1, one_bit2, fc1]);
            let val_two = builder.mul_many_extension([one_bit0, bit1, one_bit2, fc2]);
            let val_three = builder.mul_many_extension([bit0, bit1, one_bit2, fc3]);
            let val_four = builder.mul_many_extension([one_bit0, one_bit1, bit2, fc4]);
            let val_five = builder.mul_many_extension([bit0, one_bit1, bit2, fc5]);
            let val_six = builder.mul_many_extension([one_bit0, bit1, bit2, fc6]);

            let c = builder.add_many_extension([
                val_zero, val_one, val_two, val_three, val_four, val_five, val_six,
            ]);
            c
        })
        .collect::<Vec<ExtensionTarget<D>>>();

    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values
            [start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_SELECTOR_OFFSET],
    );

    let c = builder.sub_extension(
        local_values[start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET + FP6_FORBENIUS_MAP_POW_OFFSET],
        local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    for i in 0..24 * 3 {
        let c = builder.sub_extension(
            local_values[start_col
                + FP12_FORBENIUS_MAP_R0_CALC_OFFSET
                + FP6_FORBENIUS_MAP_INPUT_OFFSET
                + i],
            local_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp6_forbenius_map_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_FORBENIUS_MAP_R0_CALC_OFFSET,
        bit_selector,
    );

    let tmp = builder.mul_extension(
        bit_selector_val,
        local_values
            [start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + FP6_FORBENIUS_MAP_SELECTOR_OFFSET],
    );

    let c = builder.sub_extension(
        local_values
            [start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + FP6_FORBENIUS_MAP_POW_OFFSET],
        local_values[start_col + FP12_FORBENIUS_MAP_POW_OFFSET],
    );
    let c = builder.mul_extension(tmp, c);
    yield_constr.constraint(builder, c);

    for i in 0..24 * 3 {
        let c = builder.sub_extension(
            local_values[start_col
                + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET
                + FP6_FORBENIUS_MAP_INPUT_OFFSET
                + i],
            local_values[start_col + FP12_FORBENIUS_MAP_INPUT_OFFSET + i + 24 * 3],
        );
        let c = builder.mul_extension(tmp, c);
        yield_constr.constraint(builder, c);
    }
    add_fp6_forbenius_map_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let offset = if j == 0 {
                FP6_FORBENIUS_MAP_X_CALC_OFFSET + FP2_FORBENIUS_MAP_INPUT_OFFSET
            } else {
                FP6_FORBENIUS_MAP_X_CALC_OFFSET
                    + FP2_FORBENIUS_MAP_T0_CALC_OFFSET
                    + FP_MULTIPLICATION_TOTAL_COLUMNS
                    + REDUCED_OFFSET
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values
                    [start_col + FP12_FORBENIUS_MAP_C0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + FP12_FORBENIUS_MAP_C0_CALC_OFFSET
                    + FP2_FP2_X_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + offset + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + FP12_FORBENIUS_MAP_C0_CALC_OFFSET
                    + FP2_FP2_Y_INPUT_OFFSET
                    + j * 12
                    + i],
                y[j * 12 + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_FORBENIUS_MAP_C0_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let offset = if j == 0 {
                FP6_FORBENIUS_MAP_Y_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else {
                FP6_FORBENIUS_MAP_Y_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values
                    [start_col + FP12_FORBENIUS_MAP_C1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + FP12_FORBENIUS_MAP_C1_CALC_OFFSET
                    + FP2_FP2_X_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + offset + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + FP12_FORBENIUS_MAP_C1_CALC_OFFSET
                    + FP2_FP2_Y_INPUT_OFFSET
                    + j * 12
                    + i],
                y[j * 12 + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_FORBENIUS_MAP_C1_CALC_OFFSET,
        bit_selector,
    );

    for i in 0..12 {
        for j in 0..2 {
            let offset = if j == 0 {
                FP6_FORBENIUS_MAP_Z_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET
            } else {
                FP6_FORBENIUS_MAP_Z_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET
            };
            let tmp = builder.mul_extension(
                bit_selector_val,
                local_values
                    [start_col + FP12_FORBENIUS_MAP_C2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let c = builder.sub_extension(
                local_values[start_col
                    + FP12_FORBENIUS_MAP_C2_CALC_OFFSET
                    + FP2_FP2_X_INPUT_OFFSET
                    + j * 12
                    + i],
                local_values[start_col + FP12_FORBENIUS_MAP_C0C1C2_CALC_OFFSET + offset + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);

            let c = builder.sub_extension(
                local_values[start_col
                    + FP12_FORBENIUS_MAP_C2_CALC_OFFSET
                    + FP2_FP2_Y_INPUT_OFFSET
                    + j * 12
                    + i],
                y[j * 12 + i],
            );
            let c = builder.mul_extension(tmp, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp2_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + FP12_FORBENIUS_MAP_C2_CALC_OFFSET,
        bit_selector,
    );
}

/// Constraints for [conjugate](super::native::Fp12::conjugate) function.
pub fn add_fp12_conjugate_constraints<
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
                    bit_selector.unwrap_or(P::ONES)
                        * local_values[start_col
                            + FP12_CONJUGATE_ADDITIION_OFFSET
                            + fp2_offset
                            + fp_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + FP12_CONJUGATE_ADDITIION_OFFSET
                            + fp2_offset
                            + fp_offset
                            + FP_ADDITION_X_OFFSET
                            + i]
                            - local_values[start_col
                                + FP12_CONJUGATE_INPUT_OFFSET
                                + (j + 3) * 24
                                + k * 12
                                + i]),
                );
                yield_constr.constraint(
                    bit_selector.unwrap_or(P::ONES)
                        * local_values[start_col
                            + FP12_CONJUGATE_ADDITIION_OFFSET
                            + fp2_offset
                            + fp_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + FP12_CONJUGATE_ADDITIION_OFFSET
                            + fp2_offset
                            + fp_offset
                            + FP_ADDITION_Y_OFFSET
                            + i]
                            - local_values[start_col
                                + FP12_CONJUGATE_OUTPUT_OFFSET
                                + (j + 3) * 24
                                + k * 12
                                + i]),
                );
            }
        }
    }
    add_negate_fp6_constraints(
        local_values,
        yield_constr,
        start_col + FP12_CONJUGATE_ADDITIION_OFFSET,
        bit_selector,
    );
}

pub fn add_fp12_conjugate_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

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
                let tmp = builder.mul_extension(
                    bit_selector_val,
                    local_values[start_col
                        + FP12_CONJUGATE_ADDITIION_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET],
                );

                let c = builder.sub_extension(
                    local_values[start_col
                        + FP12_CONJUGATE_ADDITIION_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_X_OFFSET
                        + i],
                    local_values
                        [start_col + FP12_CONJUGATE_INPUT_OFFSET + (j + 3) * 24 + k * 12 + i],
                );
                let c = builder.mul_extension(tmp, c);
                yield_constr.constraint(builder, c);

                let c = builder.sub_extension(
                    local_values[start_col
                        + FP12_CONJUGATE_ADDITIION_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_Y_OFFSET
                        + i],
                    local_values
                        [start_col + FP12_CONJUGATE_OUTPUT_OFFSET + (j + 3) * 24 + k * 12 + i],
                );
                let c = builder.mul_extension(tmp, c);
                yield_constr.constraint(builder, c);
            }
        }
    }
    add_negate_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + FP12_CONJUGATE_ADDITIION_OFFSET,
        bit_selector,
    );
}
