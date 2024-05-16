use std::cmp::min;

use crate::verification::{
    fields::starky::{
        fp::*,
        fp12::*,
        fp2::*,
        fp6::*,
    },
    utils::{
        native_bls::{get_bls_12_381_parameter, Fp, Fp12, Fp2, Fp6},
        starky_utils::*,
    },
};

use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::{
    field::{
        extension::{Extendable, FieldExtension},
        packed::PackedField,
        types::Field,
    },
    hash::hash_types::RichField,
    iop::ext_target::ExtensionTarget,
};
use starky::{
    constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer},
    evaluation_frame::{StarkEvaluationFrame, StarkFrame},
    stark::Stark,
};

// Miller loop offsets
/*
    These trace offsets are for the miller_loop function (super::native::miller_loop). It takes 12*68 rows. The MSB of bls12-381 parameter is not used.
    FIRST_BIT_SELECTOR_OFFSET -> selector which is set 1 when the trace is for the first bit inside the loop.
    LAST_BIT_SELECTOR_OFFSET -> selector which is set 1 when the trace is for the last bit inside the loop.
    FIRST_ROW_SELECTOR_OFFSET -> selector which is 1 for the starting row for each operation. Hence, every 12th row, it is set 1.
    BIT1_SELECTOR_OFFSET -> selector which is 1 for each 1 bit of bls12-381 parameter. It is set 1 for 12 rows continous rows.
    PX_OFFSET -> offset where Px is set (defined in native function definition).
    PY_OFFSET -> offset where Py is set (defined in native function definition).
    ELL_COEFFS_INDEX_OFFSET -> offset which stores which index of the `ell_coeffs` array the trace is currently on. Total 68 selectors, one for each possible index of ell_coeffs.
    ELL_COEFFS_OFFSET -> offset which stores the `ell_coeffs` used in the current row computation.
    F12_OFFSET -> offset which stores the result of the current miller loop computation.
    O1_CALC_OFFSET -> offset which calculates `ell_coeffs[1]`*Px.
    O4_CALC_OFFSET -> offset which calculates `ell_coeffs[2]`*Py.
    F12_MUL_BY_014_OFFSET -> offset for multiplyBy014 function computation.
    F12_SQ_CALC_OFFSET -> offset for f12*f12 computation.
    MILLER_LOOP_RES_OFFSET -> offset which stores the result of miller_loop function.
    RES_CONJUGATE_OFFSET -> offset which stores the computation of conjugate of miller loop result. (used to match f12 value after the last loop of computation).
*/
pub const FIRST_BIT_SELECTOR_OFFSET: usize = 0;
pub const LAST_BIT_SELECTOR_OFFSET: usize = FIRST_BIT_SELECTOR_OFFSET + 1;
pub const FIRST_ROW_SELECTOR_OFFSET: usize = LAST_BIT_SELECTOR_OFFSET + 1;
pub const BIT1_SELECTOR_OFFSET: usize = FIRST_ROW_SELECTOR_OFFSET + 1;
pub const PX_OFFSET: usize = BIT1_SELECTOR_OFFSET + 1;
pub const PY_OFFSET: usize = PX_OFFSET + 12;
pub const ELL_COEFFS_INDEX_OFFEST: usize = PY_OFFSET + 12;
pub const ELL_COEFFS_OFFSET: usize = ELL_COEFFS_INDEX_OFFEST + 68;
pub const F12_OFFSET: usize = ELL_COEFFS_OFFSET + 24 * 3;
pub const O1_CALC_OFFSET: usize = F12_OFFSET + 24 * 3 * 2;
pub const O4_CALC_OFFSET: usize = O1_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const F12_MUL_BY_014_OFFSET: usize = O4_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const F12_SQ_CALC_OFFSET: usize = F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_TOTAL;
pub const MILLER_LOOP_RES_OFFSET: usize = F12_SQ_CALC_OFFSET + FP12_MUL_TOTAL_COLUMNS;
pub const RES_CONJUGATE_OFFSET: usize = MILLER_LOOP_RES_OFFSET + 24 * 3 * 2;
pub const MILLER_LOOP_TOTAL: usize = RES_CONJUGATE_OFFSET + FP6_ADDITION_TOTAL;

pub const TOTAL_COLUMNS: usize = MILLER_LOOP_TOTAL;
pub const COLUMNS: usize = TOTAL_COLUMNS;

/*
    The public inputs for this stark are the x, y inputs to the miller_loop function followed by the array of `ell_coeffs` resulted from calc_pairing_precomp, then the final result of miller_loop.
*/

pub const PIS_PX_OFFSET: usize = 0;
pub const PIS_PY_OFFSET: usize = PIS_PX_OFFSET + 12;
pub const PIS_ELL_COEFFS_OFFSET: usize = PIS_PY_OFFSET + 12;
pub const PIS_RES_OFFSET: usize = PIS_ELL_COEFFS_OFFSET + 68 * 24 * 3;
pub const PUBLIC_INPUTS: usize = PIS_RES_OFFSET + 24 * 3 * 2;

// A (Fp) * B (Fp) => C (Fp)
#[derive(Clone, Copy)]
pub struct MillerLoopStark<F: RichField + Extendable<D>, const D: usize> {
    num_rows: usize,
    _f: std::marker::PhantomData<F>,
}

/// Fills the trace of [miller_loop](super::native::miller_loop) function. Inputs are two 12 limbs and `ell_coeffs` array computed from `calc_pairing_precomp`. The values of Px and Py are filled across all rows in the trace. `FIRST_BIT_SELECTOR_OFFSET` is set 1 for the first loop computation. Sets the `ELL_COEFFS_INDEX` for the corresponding index. Sets the corresponding `ell_coeff` for the current row of computation. Fills the F12 trace, starting with 1 and then updates after each loop of computation. Fills trace for O1 and O4 calculations. Sets `FIRST_ROW_SELECTOR` to 1 for starting row of the operation. Fills the trace for multiplyBy014 caluclations and Fp12 multiplication calculations. Then fills the trace with miller_loop result, followed by conjugate computation for miller loop result.
pub fn fill_trace_miller_loop<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    x: &Fp,
    y: &Fp,
    ell_coeffs: &[[Fp2; 3]],
    start_row: usize,
    end_row: usize,
    start_col: usize,
) {
    for row in start_row..end_row + 1 {
        assign_u32_in_series(trace, row, start_col + PX_OFFSET, &x.0);
        assign_u32_in_series(trace, row, start_col + PY_OFFSET, &y.0);
    }
    let mut f12 = Fp12::one();
    let mut i = get_bls_12_381_parameter().bits() - 2;
    let mut bitone = false;
    // for j in 0..ell_coeffs.len() {
    for j in 0..min((end_row + 1 - start_row) / 12, ell_coeffs.len()) {
        let s_row = start_row + j * 12;
        let e_row = start_row + (j + 1) * 12 - 1;
        for row in s_row..e_row + 1 {
            if j == 0 {
                trace[row][start_col + FIRST_BIT_SELECTOR_OFFSET] = F::ONE;
            }
            if i == 0 {
                trace[row][start_col + LAST_BIT_SELECTOR_OFFSET] = F::ONE;
            }
            if bitone {
                trace[row][start_col + BIT1_SELECTOR_OFFSET] = F::ONE;
            }
            trace[row][start_col + ELL_COEFFS_INDEX_OFFEST + j] = F::ONE;
            for k in 0..3 {
                assign_u32_in_series(
                    trace,
                    row,
                    start_col + ELL_COEFFS_OFFSET + k * 24,
                    &ell_coeffs[j][k].get_u32_slice().concat(),
                );
            }
            assign_u32_in_series(
                trace,
                row,
                start_col + F12_OFFSET,
                &f12.get_u32_slice().concat(),
            );
        }
        if j != 0 {
            trace[s_row][start_col + FIRST_ROW_SELECTOR_OFFSET] = F::ONE;
        }
        let e = ell_coeffs[j];
        fill_trace_fp2_fp_mul(
            trace,
            &e[1].get_u32_slice(),
            &x.0,
            s_row,
            e_row,
            start_col + O1_CALC_OFFSET,
        );
        let o1 = e[1] * (*x);
        fill_trace_fp2_fp_mul(
            trace,
            &e[2].get_u32_slice(),
            &y.0,
            s_row,
            e_row,
            start_col + O4_CALC_OFFSET,
        );
        let o4 = e[2] * (*y);
        fill_trace_multiply_by_014(
            trace,
            &f12,
            &e[0],
            &o1,
            &o4,
            s_row,
            e_row,
            start_col + F12_MUL_BY_014_OFFSET,
        );
        f12 = f12.multiply_by_014(e[0], o1, o4);
        fill_trace_fp12_multiplication(
            trace,
            &f12,
            &f12,
            s_row,
            e_row,
            start_col + F12_SQ_CALC_OFFSET,
        );
        let f12_sq = f12 * f12;
        if get_bls_12_381_parameter().bit(i) && !bitone {
            bitone = true;
        } else if j < ell_coeffs.len() - 1 {
            f12 = f12_sq;
            i -= 1;
            bitone = false;
        }
    }
    f12 = f12.conjugate();
    for row in start_row..end_row + 1 {
        assign_u32_in_series(
            trace,
            row,
            start_col + MILLER_LOOP_RES_OFFSET,
            &f12.get_u32_slice().concat(),
        );
    }
    for row in start_row..end_row + 1 {
        fill_trace_negate_fp6(
            trace,
            &Fp6(f12.0[6..].try_into().unwrap()),
            row,
            start_col + RES_CONJUGATE_OFFSET,
        );
    }
    // assert_eq!(i, 0);
}

// Implement trace generator
impl<F: RichField + Extendable<D>, const D: usize> MillerLoopStark<F, D> {
    pub fn new(num_rows: usize) -> Self {
        Self {
            num_rows,
            _f: std::marker::PhantomData,
        }
    }

    pub fn generate_trace(
        &self,
        x: Fp,
        y: Fp,
        ell_coeffs: Vec<[Fp2; 3]>,
    ) -> Vec<[F; TOTAL_COLUMNS]> {
        let mut trace = vec![[F::ZERO; TOTAL_COLUMNS]; self.num_rows];
        fill_trace_miller_loop(&mut trace, &x, &y, &ell_coeffs, 0, self.num_rows - 1, 0);
        trace
        // let start_col = 0;
        // for row in 0..self.num_rows-1 {
        //     let local_values = self.trace[row];
        //     let next_values = self.trace[row+1];
        //     println!(
        //         "{} * (1 - {}) * ({} - {}) = {}",
        //         next_values[start_col + FIRST_ROW_SELECTOR_OFFSET],
        //         next_values[start_col + BIT1_SELECTOR_OFFSET],
        //         next_values[start_col + F12_OFFSET],
        //         local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_X_CALC_OFFSET + FP6_ADDITION_TOTAL + FP_SINGLE_REDUCED_OFFSET],
        //         next_values[start_col + FIRST_ROW_SELECTOR_OFFSET] *
        //         (F::ONE - next_values[start_col + BIT1_SELECTOR_OFFSET]) *
        //         (next_values[start_col + F12_OFFSET] -
        //         local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_X_CALC_OFFSET + FP6_ADDITION_TOTAL + FP_SINGLE_REDUCED_OFFSET])
        //     );
        // }
    }
}

/// The constraints of this stark are as follows:
/// * Constraint Px and Py to be same across all rows.
/// * Constraints F12 to 1 when `FIRST_BIT_SELECTOR` is set 1.
/// * Constraints next row F12 to result of current row multiplyBy014 when next row `FIRST_ROW_SELECTOR` is set 1 and next row `BIT1_SELECTOR` is set 1.
/// * Constraints next row F12 to result of current row fp12 multiplication when next row `FIRST_ROW_SELECTOR` is set 1 and next row `BIT1_SELECTOR` is set 0.
/// * Constraints O1 computation with Px and current `ell_coeffs[1]`.
/// * Constraints O4 computation with Py and current `ell_coeffs[2]`.
/// * Constraints multiplyBy014 computation with F12, `ell_coeffs[0]`, O1 and O4.
/// * Constrants fp12 multiplication for F12*F12.
/// * Constraints result conjugate computation with miller loop res.
/// * Constraints the result of conjugate computation with F12 when `LAST_BIT_SELECTOR` is set 1.
pub fn add_miller_loop_constraints<
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
        yield_constr.constraint_transition(
            local_values[start_col + PX_OFFSET + i] - next_values[start_col + PX_OFFSET + i],
        );
        yield_constr.constraint_transition(
            local_values[start_col + PY_OFFSET + i] - next_values[start_col + PY_OFFSET + i],
        );
    }
    for i in 0..24 * 3 * 2 {
        if i == 0 {
            yield_constr.constraint(
                local_values[start_col + FIRST_BIT_SELECTOR_OFFSET]
                    * (local_values[start_col + F12_OFFSET + i] - P::ONES),
            );
        } else {
            yield_constr.constraint(
                local_values[start_col + FIRST_BIT_SELECTOR_OFFSET]
                    * local_values[start_col + F12_OFFSET + i],
            );
        }
    }

    for i in 0..12 {
        for j in 0..6 {
            yield_constr.constraint(
                bit_selector_val
                    * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                    * next_values[start_col + BIT1_SELECTOR_OFFSET]
                    * (next_values[start_col + F12_OFFSET + j * 12 + i]
                        - local_values[start_col
                            + F12_MUL_BY_014_OFFSET
                            + MULTIPLY_BY_014_X_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                    * next_values[start_col + BIT1_SELECTOR_OFFSET]
                    * (next_values[start_col + F12_OFFSET + j * 12 + i + 24 * 3]
                        - local_values[start_col
                            + F12_MUL_BY_014_OFFSET
                            + MULTIPLY_BY_014_Y_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                    * (P::ONES - next_values[start_col + BIT1_SELECTOR_OFFSET])
                    * (next_values[start_col + F12_OFFSET + j * 12 + i]
                        - local_values[start_col
                            + F12_SQ_CALC_OFFSET
                            + FP12_MUL_X_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * next_values[start_col + FIRST_ROW_SELECTOR_OFFSET]
                    * (P::ONES - next_values[start_col + BIT1_SELECTOR_OFFSET])
                    * (next_values[start_col + F12_OFFSET + j * 12 + i + 24 * 3]
                        - local_values[start_col
                            + F12_SQ_CALC_OFFSET
                            + FP12_MUL_Y_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }
    }

    // O1
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + O1_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + O1_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                    - local_values[start_col + ELL_COEFFS_OFFSET + 24 + i]),
        );
        if i < 12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + O1_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col + O1_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                        - local_values[start_col + PX_OFFSET + i]),
            );
        }
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + O1_CALC_OFFSET,
        bit_selector,
    );

    // O4
    for i in 0..24 {
        yield_constr.constraint(
            bit_selector_val
                * local_values[start_col + O4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                * (local_values[start_col + O4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                    - local_values[start_col + ELL_COEFFS_OFFSET + 48 + i]),
        );
        if i < 12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + O4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col + O4_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i]
                        - local_values[start_col + PY_OFFSET + i]),
            );
        }
    }
    add_fp2_fp_mul_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + O4_CALC_OFFSET,
        bit_selector,
    );

    // f12 multiply by 014
    for i in 0..12 {
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values
                        [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_INPUT_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + F12_OFFSET + j * 12 + i]),
            );
        }
        for j in 0..2 {
            let z_offset = if j == 0 {
                X0_Y_REDUCE_OFFSET
            } else {
                X1_Y_REDUCE_OFFSET
            };
            yield_constr.constraint(
                bit_selector_val
                    * local_values
                        [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_O0_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + ELL_COEFFS_OFFSET + j * 12 + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values
                        [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_O1_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + O1_CALC_OFFSET + z_offset + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values
                        [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_O4_OFFSET
                        + j * 12
                        + i]
                        - local_values[start_col + O4_CALC_OFFSET + z_offset + REDUCED_OFFSET + i]),
            );
        }
    }
    add_multiply_by_014_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + F12_MUL_BY_014_OFFSET,
        bit_selector,
    );

    // f12 * f12
    for i in 0..12 {
        for j in 0..6 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]
                        - local_values[start_col
                            + F12_SQ_CALC_OFFSET
                            + FP12_MUL_X_INPUT_OFFSET
                            + j * 12
                            + i]),
            );
            yield_constr.constraint(
                bit_selector_val
                    * local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_SELECTOR_OFFSET]
                    * (local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + FP6_SUBTRACTION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]
                        - local_values[start_col
                            + F12_SQ_CALC_OFFSET
                            + FP12_MUL_X_INPUT_OFFSET
                            + j * 12
                            + i
                            + 24 * 3]),
            );
        }
        for j in 0..12 {
            yield_constr.constraint(
                bit_selector_val
                    * local_values
                        [start_col + F12_SQ_CALC_OFFSET + FP12_MUL_X_INPUT_OFFSET + j * 12 + i]
                    - local_values
                        [start_col + F12_SQ_CALC_OFFSET + FP12_MUL_Y_INPUT_OFFSET + j * 12 + i],
            );
        }
    }
    add_fp12_multiplication_constraints(
        local_values,
        next_values,
        yield_constr,
        start_col + F12_SQ_CALC_OFFSET,
        bit_selector,
    );

    // RES conjugate
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
                        * local_values[start_col
                            + RES_CONJUGATE_OFFSET
                            + fp2_offset
                            + fp_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + RES_CONJUGATE_OFFSET
                            + fp2_offset
                            + fp_offset
                            + FP_ADDITION_X_OFFSET
                            + i]
                            - local_values
                                [start_col + MILLER_LOOP_RES_OFFSET + (j + 3) * 24 + k * 12 + i]),
                );
            }
        }
    }
    add_negate_fp6_constraints(
        local_values,
        yield_constr,
        start_col + RES_CONJUGATE_OFFSET,
        bit_selector,
    );

    // RES with last bit result
    for i in 0..12 {
        for j in 0..3 {
            let (fp2_add_offset, fp2_sub_offset) = if j == 0 {
                (FP6_ADDITION_0_OFFSET, FP6_SUBTRACTION_0_OFFSET)
            } else if j == 1 {
                (FP6_ADDITION_1_OFFSET, FP6_SUBTRACTION_1_OFFSET)
            } else {
                (FP6_ADDITION_2_OFFSET, FP6_SUBTRACTION_2_OFFSET)
            };
            for k in 0..2 {
                let (fp_add_offset, fp_sub_offset) = if k == 0 {
                    (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
                } else {
                    (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
                };
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col + LAST_BIT_SELECTOR_OFFSET]
                        * local_values[start_col
                            + F12_MUL_BY_014_OFFSET
                            + MULTIPLY_BY_014_X_CALC_OFFSET
                            + fp2_add_offset
                            + fp_add_offset
                            + FP_ADDITION_CHECK_OFFSET]
                        * (local_values[start_col
                            + F12_MUL_BY_014_OFFSET
                            + MULTIPLY_BY_014_X_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * (j * 2 + k)
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - local_values
                                [start_col + MILLER_LOOP_RES_OFFSET + j * 24 + k * 12 + i]),
                );
                yield_constr.constraint(
                    bit_selector_val
                        * local_values[start_col + LAST_BIT_SELECTOR_OFFSET]
                        * local_values[start_col
                            + F12_MUL_BY_014_OFFSET
                            + MULTIPLY_BY_014_Y_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + fp2_sub_offset
                            + fp_sub_offset
                            + FP_SUBTRACTION_CHECK_OFFSET]
                        * (local_values[start_col
                            + F12_MUL_BY_014_OFFSET
                            + MULTIPLY_BY_014_Y_CALC_OFFSET
                            + FP6_ADDITION_TOTAL
                            + FP6_SUBTRACTION_TOTAL
                            + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * (j * 2 + k)
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - local_values[start_col
                                + RES_CONJUGATE_OFFSET
                                + fp2_add_offset
                                + fp_add_offset
                                + FP_ADDITION_Y_OFFSET
                                + i]),
                );
            }
        }
    }
}
pub fn add_miller_loop_constraints_ext_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    local_values: &[ExtensionTarget<D>],
    next_values: &[ExtensionTarget<D>],
    start_col: usize,
    bit_selector: Option<ExtensionTarget<D>>,
) {
    let bit_selector_val = bit_selector.unwrap_or(builder.constant_extension(F::Extension::ONE));

    for i in 0..12 {
        let c1 = builder.sub_extension(
            local_values[start_col + PX_OFFSET + i],
            next_values[start_col + PX_OFFSET + i],
        );
        let c = builder.mul_extension(bit_selector_val, c1);
        yield_constr.constraint_transition(builder, c);

        let c2 = builder.sub_extension(
            local_values[start_col + PY_OFFSET + i],
            next_values[start_col + PY_OFFSET + i],
        );
        let c = builder.mul_extension(bit_selector_val, c2);
        yield_constr.constraint_transition(builder, c);
    }
    for i in 0..24 * 3 * 2 {
        let one = builder.constant_extension(F::Extension::ONE);
        let mul_tmp = local_values[start_col + FIRST_BIT_SELECTOR_OFFSET];
        if i == 0 {
            let sub_tmp = builder.sub_extension(local_values[start_col + F12_OFFSET + i], one);
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        } else {
            let c = builder.mul_extension(mul_tmp, local_values[start_col + F12_OFFSET + i]);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
    }

    for i in 0..12 {
        for j in 0..6 {
            let one = builder.constant_extension(F::Extension::ONE);

            let mul_tmp1 = builder.mul_extension(
                next_values[start_col + FIRST_ROW_SELECTOR_OFFSET],
                next_values[start_col + BIT1_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                next_values[start_col + F12_OFFSET + j * 12 + i],
                local_values[start_col
                    + F12_MUL_BY_014_OFFSET
                    + MULTIPLY_BY_014_X_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp1);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                next_values[start_col + F12_OFFSET + j * 12 + i + 24 * 3],
                local_values[start_col
                    + F12_MUL_BY_014_OFFSET
                    + MULTIPLY_BY_014_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp1);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);

            let sub_tmp = builder.sub_extension(one, next_values[start_col + BIT1_SELECTOR_OFFSET]);
            let mul_tmp2 =
                builder.mul_extension(next_values[start_col + FIRST_ROW_SELECTOR_OFFSET], sub_tmp);

            let sub_tmp3 = builder.sub_extension(
                next_values[start_col + F12_OFFSET + j * 12 + i],
                local_values[start_col
                    + F12_SQ_CALC_OFFSET
                    + FP12_MUL_X_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c3 = builder.mul_extension(sub_tmp3, mul_tmp2);
            let c = builder.mul_extension(bit_selector_val, c3);
            yield_constr.constraint(builder, c);

            let sub_tmp4 = builder.sub_extension(
                next_values[start_col + F12_OFFSET + j * 12 + i + 24 * 3],
                local_values[start_col
                    + F12_SQ_CALC_OFFSET
                    + FP12_MUL_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(sub_tmp4, mul_tmp2);
            let c = builder.mul_extension(bit_selector_val, c4);
            yield_constr.constraint(builder, c);
        }
    }

    // O1
    for i in 0..24 {
        let mul_tmp = local_values[start_col + O1_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET];
        let sub_tmp = builder.sub_extension(
            local_values[start_col + O1_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
            local_values[start_col + ELL_COEFFS_OFFSET + 24 + i],
        );
        let c = builder.mul_extension(sub_tmp, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c);
        yield_constr.constraint(builder, c);
        if i < 12 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col + O1_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
                local_values[start_col + PX_OFFSET + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + O1_CALC_OFFSET,
        bit_selector,
    );

    // O4
    for i in 0..24 {
        let mul_tmp = local_values[start_col + O4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET];
        let sub_tmp = builder.sub_extension(
            local_values[start_col + O4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
            local_values[start_col + ELL_COEFFS_OFFSET + 48 + i],
        );
        let c = builder.mul_extension(sub_tmp, mul_tmp);
        let c = builder.mul_extension(bit_selector_val, c);
        yield_constr.constraint(builder, c);
        if i < 12 {
            let sub_tmp = builder.sub_extension(
                local_values[start_col + O4_CALC_OFFSET + FP2_FP_Y_INPUT_OFFSET + i],
                local_values[start_col + PY_OFFSET + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp2_fp_mul_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + O4_CALC_OFFSET,
        bit_selector,
    );

    // f12 multiply by 014
    for i in 0..12 {
        let mul_tmp =
            local_values[start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_SELECTOR_OFFSET];
        for j in 0..12 {
            let sub_tmp = builder.sub_extension(
                local_values
                    [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_INPUT_OFFSET + j * 12 + i],
                local_values[start_col + F12_OFFSET + j * 12 + i],
            );
            let c = builder.mul_extension(sub_tmp, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
        for j in 0..2 {
            let z_offset = if j == 0 {
                X0_Y_REDUCE_OFFSET
            } else {
                X1_Y_REDUCE_OFFSET
            };

            let sub_tmp1 = builder.sub_extension(
                local_values
                    [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_O0_OFFSET + j * 12 + i],
                local_values[start_col + ELL_COEFFS_OFFSET + j * 12 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values
                    [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_O1_OFFSET + j * 12 + i],
                local_values[start_col + O1_CALC_OFFSET + z_offset + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);

            let sub_tmp3 = builder.sub_extension(
                local_values
                    [start_col + F12_MUL_BY_014_OFFSET + MULTIPLY_BY_014_O4_OFFSET + j * 12 + i],
                local_values[start_col + O4_CALC_OFFSET + z_offset + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(sub_tmp3, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c3);
            yield_constr.constraint(builder, c);
        }
    }
    add_multiply_by_014_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + F12_MUL_BY_014_OFFSET,
        bit_selector,
    );

    // f12 * f12
    for i in 0..12 {
        for j in 0..6 {
            let mul_tmp = local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_SELECTOR_OFFSET];

            let sub_tmp1 = builder.sub_extension(
                local_values[start_col
                    + F12_MUL_BY_014_OFFSET
                    + MULTIPLY_BY_014_X_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
                local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_X_INPUT_OFFSET + j * 12 + i],
            );
            let c1 = builder.mul_extension(sub_tmp1, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c1);
            yield_constr.constraint(builder, c);

            let sub_tmp2 = builder.sub_extension(
                local_values[start_col
                    + F12_MUL_BY_014_OFFSET
                    + MULTIPLY_BY_014_Y_CALC_OFFSET
                    + FP6_ADDITION_TOTAL
                    + FP6_SUBTRACTION_TOTAL
                    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * j
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
                local_values[start_col
                    + F12_SQ_CALC_OFFSET
                    + FP12_MUL_X_INPUT_OFFSET
                    + j * 12
                    + i
                    + 24 * 3],
            );
            let c2 = builder.mul_extension(sub_tmp2, mul_tmp);
            let c = builder.mul_extension(bit_selector_val, c2);
            yield_constr.constraint(builder, c);
        }
        for j in 0..12 {
            let c = builder.sub_extension(
                local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_X_INPUT_OFFSET + j * 12 + i],
                local_values[start_col + F12_SQ_CALC_OFFSET + FP12_MUL_Y_INPUT_OFFSET + j * 12 + i],
            );
            let c = builder.mul_extension(bit_selector_val, c);
            yield_constr.constraint(builder, c);
        }
    }
    add_fp12_multiplication_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        next_values,
        start_col + F12_SQ_CALC_OFFSET,
        bit_selector,
    );

    // RES conjugate
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
                let sub_tmp = builder.sub_extension(
                    local_values[start_col
                        + RES_CONJUGATE_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_X_OFFSET
                        + i],
                    local_values[start_col + MILLER_LOOP_RES_OFFSET + (j + 3) * 24 + k * 12 + i],
                );
                let c = builder.mul_extension(
                    sub_tmp,
                    local_values[start_col
                        + RES_CONJUGATE_OFFSET
                        + fp2_offset
                        + fp_offset
                        + FP_ADDITION_CHECK_OFFSET],
                );
                let c = builder.mul_extension(bit_selector_val, c);
                yield_constr.constraint(builder, c);
            }
        }
    }
    add_negate_fp6_constraints_ext_circuit(
        builder,
        yield_constr,
        local_values,
        start_col + RES_CONJUGATE_OFFSET,
        bit_selector,
    );

    // RES with last bit result
    for i in 0..12 {
        for j in 0..3 {
            let (fp2_add_offset, fp2_sub_offset) = if j == 0 {
                (FP6_ADDITION_0_OFFSET, FP6_SUBTRACTION_0_OFFSET)
            } else if j == 1 {
                (FP6_ADDITION_1_OFFSET, FP6_SUBTRACTION_1_OFFSET)
            } else {
                (FP6_ADDITION_2_OFFSET, FP6_SUBTRACTION_2_OFFSET)
            };
            for k in 0..2 {
                let (fp_add_offset, fp_sub_offset) = if k == 0 {
                    (FP2_ADDITION_0_OFFSET, FP2_SUBTRACTION_0_OFFSET)
                } else {
                    (FP2_ADDITION_1_OFFSET, FP2_SUBTRACTION_1_OFFSET)
                };

                let mul_tmp1 = builder.mul_extension(
                    local_values[start_col + LAST_BIT_SELECTOR_OFFSET],
                    local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + fp2_add_offset
                        + fp_add_offset
                        + FP_ADDITION_CHECK_OFFSET],
                );
                let sub_tmp1 = builder.sub_extension(
                    local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_X_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * (j * 2 + k)
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    local_values[start_col + MILLER_LOOP_RES_OFFSET + j * 24 + k * 12 + i],
                );
                let c1 = builder.mul_extension(sub_tmp1, mul_tmp1);
                let c = builder.mul_extension(bit_selector_val, c1);
                yield_constr.constraint(builder, c);

                let mul_tmp2 = builder.mul_extension(
                    local_values[start_col + LAST_BIT_SELECTOR_OFFSET],
                    local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + fp2_sub_offset
                        + fp_sub_offset
                        + FP_SUBTRACTION_CHECK_OFFSET],
                );
                let sub_tmp2 = builder.sub_extension(
                    local_values[start_col
                        + F12_MUL_BY_014_OFFSET
                        + MULTIPLY_BY_014_Y_CALC_OFFSET
                        + FP6_ADDITION_TOTAL
                        + FP6_SUBTRACTION_TOTAL
                        + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * (j * 2 + k)
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    local_values[start_col
                        + RES_CONJUGATE_OFFSET
                        + fp2_add_offset
                        + fp_add_offset
                        + FP_ADDITION_Y_OFFSET
                        + i],
                );
                let c2 = builder.mul_extension(sub_tmp2, mul_tmp2);
                let c = builder.mul_extension(bit_selector_val, c2);
                yield_constr.constraint(builder, c);
            }
        }
    }
}

/*
    Constraints for miller loop stark:
    * Constraint Px with public input x
    * Constraint Py with public input y
    * Constraint current row `ell_coeff` along with `ELL_COEFFS_INDEX` with public inputs `ell_coeffs`.
    * Constrain `MILLER_LOOP_RES` with public inputs result
    * Constraints for miller loop computation.
*/

// Implement constraint generator without Stark trait

pub fn traitless_eval_packed_generic<
    F: RichField + Extendable<D>,
    const D: usize,
    FE,
    P,
    const D2: usize,
    EvalFrame: StarkEvaluationFrame<P, FE>,
>(
    vars: StarkFrame<P, P::Scalar, COLUMNS, PUBLIC_INPUTS>,
    yield_constr: &mut ConstraintConsumer<P>,
) where
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
{
    let local_values = vars.get_local_values();
    let next_values = vars.get_next_values();
    let public_inputs = vars.get_public_inputs();

    // ----
    for i in 0..12 {
        yield_constr.constraint(local_values[PX_OFFSET + i] - public_inputs[PIS_PX_OFFSET + i]);
        yield_constr.constraint(local_values[PY_OFFSET + i] - public_inputs[PIS_PY_OFFSET + i]);
    }
    for i in 0..68 {
        for j in 0..24 * 3 {
            yield_constr.constraint(
                local_values[ELL_COEFFS_INDEX_OFFEST + i]
                    * (local_values[ELL_COEFFS_OFFSET + j]
                        - public_inputs[PIS_ELL_COEFFS_OFFSET + i * 24 * 3 + j]),
            );
        }
    }
    for i in 0..24 * 3 * 2 {
        yield_constr.constraint(
            local_values[MILLER_LOOP_RES_OFFSET + i] - public_inputs[PIS_RES_OFFSET + i],
        );
    }
    add_miller_loop_constraints(local_values, next_values, yield_constr, 0, None);
}

// Implement constraint generator
impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for MillerLoopStark<F, D> {
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

        // ----
        for i in 0..12 {
            yield_constr.constraint(local_values[PX_OFFSET + i] - public_inputs[PIS_PX_OFFSET + i]);
            yield_constr.constraint(local_values[PY_OFFSET + i] - public_inputs[PIS_PY_OFFSET + i]);
        }
        for i in 0..68 {
            for j in 0..24 * 3 {
                yield_constr.constraint(
                    local_values[ELL_COEFFS_INDEX_OFFEST + i]
                        * (local_values[ELL_COEFFS_OFFSET + j]
                            - public_inputs[PIS_ELL_COEFFS_OFFSET + i * 24 * 3 + j]),
                );
            }
        }
        for i in 0..24 * 3 * 2 {
            yield_constr.constraint(
                local_values[MILLER_LOOP_RES_OFFSET + i] - public_inputs[PIS_RES_OFFSET + i],
            );
        }
        add_miller_loop_constraints(local_values, next_values, yield_constr, 0, None);
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
            let c1 = builder.sub_extension(
                local_values[PX_OFFSET + i],
                public_inputs[PIS_PX_OFFSET + i],
            );
            yield_constr.constraint(builder, c1);

            let c2 = builder.sub_extension(
                local_values[PY_OFFSET + i],
                public_inputs[PIS_PY_OFFSET + i],
            );
            yield_constr.constraint(builder, c2);
        }
        for i in 0..68 {
            for j in 0..24 * 3 {
                let sub_tmp = builder.sub_extension(
                    local_values[ELL_COEFFS_OFFSET + j],
                    public_inputs[PIS_ELL_COEFFS_OFFSET + i * 24 * 3 + j],
                );
                let c = builder.mul_extension(local_values[ELL_COEFFS_INDEX_OFFEST + i], sub_tmp);
                yield_constr.constraint(builder, c);
            }
        }
        for i in 0..24 * 3 * 2 {
            let c = builder.sub_extension(
                local_values[MILLER_LOOP_RES_OFFSET + i],
                public_inputs[PIS_RES_OFFSET + i],
            );
            yield_constr.constraint(builder, c);
        }
        add_miller_loop_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            0,
            None,
        );
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}
