use std::{cmp::min, str::FromStr};

use num_bigint::BigUint;
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
    constraint_consumer::ConstraintConsumer,
    evaluation_frame::{StarkEvaluationFrame, StarkFrame},
    stark::Stark,
};

use crate::verification::utils::native_bls::{
    calc_precomp_stuff_loop0, calc_precomp_stuff_loop1, calc_qs, get_bls_12_381_parameter,
    get_u32_vec_from_literal, mod_inverse, modulus, Fp, Fp2,
};

use crate::verification::fields::starky::fp::*;
use crate::verification::fields::starky::fp2::*;
use crate::verification::utils::starky_utils::*;

/*
    These trace offsets are for the calc_pairing_precomp function (super::native::calc_pairing_precomp). It takes 12*68 rows. The offsets are defined such that each 0 bit of the bls12-381 parameter takes 12 rows (one operation) and each 1 bit takes 12*2 rows (two operations). The MSB of bls12-381 parameter is not used.
    Z_MULT_Z_INV_OFFSET -> offset for multiplication z(input) and z_inv. Required to verify that z*z_inv = 1.
    X_MULT_Z_INV_OFFSET -> offset for multiplication of x(input) and z_inv.
    Y_MULT_Z_INV_OFFSET -> offset for multiplication of y(input) and z_inv.
    QX_OFFSET -> offset where Qx is set (defined in native function definition).
    QY_OFFSET -> offset where Qy is set (defined in native function definition).
    QZ_OFFSET -> offset where Qz is set (defined in native function definition).
    FIRST_ROW_SELECTOR_OFFSET -> selector which is 1 for the starting row for each operation. Hence, every 12th row, it is set 1.
    FIRST_LOOP_SELECTOR_OFFSET -> selector which is set 1 when the trace is for the first computation inside the loop.
    BIT1_SELECTOR_OFFSET -> selector which is 1 for each 1 bit of bls12-381 parameter. It is set 1 for 12 rows continous rows.
    RX_OFFSET -> offset where Rx is set (defined in native function definition), updates after each loop of computation.
    RY_OFFSET -> offset where Ry is set (defined in native function definition), updates after each loop of computation.
    RZ_OFFSET -> offset where Rz is set (defined in native function definition), updates after each loop of computation.
    ELL_COEFFS_IDX_OFFSET -> offset which stores which index of the `ell_coeffs` array the trace is currently on. Total 68 selectors, one for each possible index of ell_coeffs.
    ---
    BIT0 OFFSETS -> offsets for computing `ell_coeffs` for 0 bit of bls12-381 parameter. Ti's and Xi's are the same as defined in the function definition.
    BIT1 OFFSETS -> offsets for computing `ell_coeffs` for 1 bit of bls12-381 parameter. Ti's and Xi's are the same as defined in the function definition.

    BIT0 and BIT1 offsets start on the same value because both the operations are never done in the same rows. In a single row, either bit 0 operations are being done, or bit 1 operations are being done.
*/

pub const Z_MULT_Z_INV_OFFSET: usize = 0;
pub const X_MULT_Z_INV_OFFSET: usize = Z_MULT_Z_INV_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const Y_MULT_Z_INV_OFFSET: usize = X_MULT_Z_INV_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;

pub const QX_OFFSET: usize = Y_MULT_Z_INV_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const QY_OFFSET: usize = QX_OFFSET + 24;
pub const QZ_OFFSET: usize = QY_OFFSET + 24;
pub const FIRST_LOOP_SELECTOR_OFFSET: usize = QZ_OFFSET + 24;
pub const FIRST_ROW_SELECTOR_OFFSET: usize = FIRST_LOOP_SELECTOR_OFFSET + 1;
pub const BIT1_SELECTOR_OFFSET: usize = FIRST_ROW_SELECTOR_OFFSET + 1;
pub const RX_OFFSET: usize = BIT1_SELECTOR_OFFSET + 1;
pub const RY_OFFSET: usize = RX_OFFSET + 24;
pub const RZ_OFFSET: usize = RY_OFFSET + 24;
pub const ELL_COEFFS_IDX_OFFSET: usize = RZ_OFFSET + 24;

// bit0 offsets
pub const T0_CALC_OFFSET: usize = ELL_COEFFS_IDX_OFFSET + 68;
pub const T1_CALC_OFFSET: usize = T0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const X0_CALC_OFFSET: usize = T1_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const T2_CALC_OFFSET: usize = X0_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const T3_CALC_OFFSET: usize = T2_CALC_OFFSET + MULTIPLY_B_TOTAL_COLUMS;
pub const X1_CALC_OFFSET: usize = T3_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const T4_CALC_OFFSET: usize = X1_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const X2_CALC_OFFSET: usize = T4_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const X3_CALC_OFFSET: usize = X2_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const X4_CALC_OFFSET: usize = X3_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const X5_CALC_OFFSET: usize = X4_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const X6_CALC_OFFSET: usize = X5_CALC_OFFSET + FP2_ADDITION_TOTAL;
pub const X7_CALC_OFFSET: usize = X6_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const X8_CALC_OFFSET: usize = X7_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const X9_CALC_OFFSET: usize = X8_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const X10_CALC_OFFSET: usize =
    X9_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const X11_CALC_OFFSET: usize = X10_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const X12_CALC_OFFSET: usize = X11_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const X13_CALC_OFFSET: usize = X12_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const NEW_RX_OFFSET: usize = X13_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const NEW_RY_OFFSET: usize = NEW_RX_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const NEW_RZ_OFFSET: usize = NEW_RY_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT0_TOTAL_COLUMNS: usize = NEW_RZ_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;

// bit1 offsets
pub const BIT1_T0_CALC_OFFSET: usize = ELL_COEFFS_IDX_OFFSET + 68;
pub const BIT1_T1_CALC_OFFSET: usize = BIT1_T0_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T2_CALC_OFFSET: usize = BIT1_T1_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_T3_CALC_OFFSET: usize = BIT1_T2_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T4_CALC_OFFSET: usize = BIT1_T3_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_T5_CALC_OFFSET: usize = BIT1_T4_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T6_CALC_OFFSET: usize = BIT1_T5_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T7_CALC_OFFSET: usize = BIT1_T6_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_T8_CALC_OFFSET: usize = BIT1_T7_CALC_OFFSET + FP2_ADDITION_TOTAL;
pub const BIT1_T9_CALC_OFFSET: usize = BIT1_T8_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T10_CALC_OFFSET: usize = BIT1_T9_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T11_CALC_OFFSET: usize = BIT1_T10_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T12_CALC_OFFSET: usize = BIT1_T11_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T13_CALC_OFFSET: usize = BIT1_T12_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_T14_CALC_OFFSET: usize = BIT1_T13_CALC_OFFSET + FP2_FP_TOTAL_COLUMNS;
pub const BIT1_T15_CALC_OFFSET: usize = BIT1_T14_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_T16_CALC_OFFSET: usize =
    BIT1_T15_CALC_OFFSET + FP2_ADDITION_TOTAL + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_T17_CALC_OFFSET: usize = BIT1_T16_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_T18_CALC_OFFSET: usize = BIT1_T17_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_RX_CALC_OFFSET: usize = BIT1_T18_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_RY_CALC_OFFSET: usize = BIT1_RX_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;
pub const BIT1_RZ_CALC_OFFSET: usize = BIT1_RY_CALC_OFFSET
    + FP2_ADDITION_TOTAL
    + FP2_SUBTRACTION_TOTAL
    + (FP_SINGLE_REDUCE_TOTAL + RANGE_CHECK_TOTAL) * 2;
pub const BIT1_TOTAL_COLUMNS: usize = BIT1_RZ_CALC_OFFSET + TOTAL_COLUMNS_FP2_MULTIPLICATION;

pub const TOTAL_COLUMNS: usize = BIT1_TOTAL_COLUMNS;
// pub const TOTAL_COLUMNS: usize = BIT0_TOTAL_COLUMNS;

pub const COLUMNS: usize = TOTAL_COLUMNS;
pub const PUBLIC_INPUTS: usize = 72 + 68 * 24 * 3;

/*
    The public inputs for this stark are the x, y and z inputs to the calc_pairing_precomp function followed by the output ell_coeffs array.
*/

pub const X0_PUBLIC_INPUTS_OFFSET: usize = 0;
pub const X1_PUBLIC_INPUTS_OFFSET: usize = 12;
pub const Y0_PUBLIC_INPUTS_OFFSET: usize = 24;
pub const Y1_PUBLIC_INPUTS_OFFSET: usize = 36;
pub const Z0_PUBLIC_INPUTS_OFFSET: usize = 48;
pub const Z1_PUBLIC_INPUTS_OFFSET: usize = 60;
pub const ELL_COEFFS_PUBLIC_INPUTS_OFFSET: usize = 72;

// A (Fp) * B (Fp) => C (Fp)
#[derive(Clone, Copy)]
pub struct PairingPrecompStark<F: RichField + Extendable<D>, const D: usize> {
    num_rows: usize,
    _f: std::marker::PhantomData<F>,
}

// Implement trace generator
impl<F: RichField + Extendable<D>, const D: usize> PairingPrecompStark<F, D> {
    pub fn new(num_rows: usize) -> Self {
        Self {
            num_rows: num_rows,
            _f: std::marker::PhantomData,
        }
    }

    /// Fills the trace of [calc_pairing_precomp](super::native::calc_pairing_precomp) function. Inputs are three 12\*2 limbs. The trace first has a multiplication of z and z_inv to verify the correctness of the inverse. Followed by operations of x\*z_inv and y\*z_inv. The values of Qx, Qy and Qz are filled across all rows in the trace. `FIRST_LOOP_SELECTOR` is set 1 for the first loop computation. Sets Rx, Ry and Rz values for the current loop, and the `ELL_COEFFS_IDX` for the corresponding index. Sets `FIRST_ROW_SELECTOR` to 1 for starting row of the operation. For each bit 0 of bls12-381 parameter, fills the trace for bit 0 computation. For each bit 1 of the bls12-381 parameter, fills trace for bit 0 computation in 12 rows, then fills the trace for bit 1 computation in the next 12 rows and also sets `BIT1_SELECTOR` to 1 for these rows.
    pub fn generate_trace(
        &self,
        x: [[u32; 12]; 2],
        y: [[u32; 12]; 2],
        z: [[u32; 12]; 2],
    ) -> Vec<[F; TOTAL_COLUMNS]> {
        let mut trace = vec![[F::ZERO; TOTAL_COLUMNS]; self.num_rows];
        let z_fp2 = Fp2([Fp(z[0]), Fp(z[1])]);
        let z_inv = z_fp2.invert();
        let z_inv_slice: [[u32; 12]; 2] = [z_inv.0[0].0, z_inv.0[1].0];
        generate_trace_fp2_mul(&mut trace, z, z_inv_slice, 0, self.num_rows - 1, 0);

        // Calculate ax = x * (z_inv)
        generate_trace_fp2_mul(
            &mut trace,
            x,
            z_inv_slice,
            0,
            self.num_rows - 1,
            X_MULT_Z_INV_OFFSET,
        );

        // Calculate ay = y * (z_inv)
        generate_trace_fp2_mul(
            &mut trace,
            y,
            z_inv_slice,
            0,
            self.num_rows - 1,
            Y_MULT_Z_INV_OFFSET,
        );

        let (qx, qy, qz) = calc_qs(
            Fp2([Fp(x[0]), Fp(x[1])]),
            Fp2([Fp(y[0]), Fp(y[1])]),
            Fp2([Fp(z[0]), Fp(z[1])]),
        );

        // Fill qx, qy, qz for all rows
        for row in 0..self.num_rows {
            for i in 0..12 {
                trace[row][QX_OFFSET + i] = F::from_canonical_u64(qx.0[0].0[i] as u64); //trace[0][X_MULT_Z_INV_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i];
                trace[row][QX_OFFSET + i + 12] = F::from_canonical_u64(qx.0[1].0[i] as u64); //trace[0][X_MULT_Z_INV_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i];
                trace[row][QY_OFFSET + i] = F::from_canonical_u64(qy.0[0].0[i] as u64); //trace[0][Y_MULT_Z_INV_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i];
                trace[row][QY_OFFSET + i + 12] = F::from_canonical_u64(qy.0[1].0[i] as u64); //trace[0][Y_MULT_Z_INV_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i];
                trace[row][QZ_OFFSET + i] = F::ZERO;
                trace[row][QZ_OFFSET + i + 12] = F::ZERO;
            }
            trace[row][QZ_OFFSET] = F::ONE;
        }

        let (mut rx, mut ry, mut rz) = (qx, qy, qz);
        let mut bit_pos = 62;
        let mut bit1 = false;
        let num_coeffs = 68;
        for n in 0..(self.num_rows / 12 + 1) {
            let start_row = n * 12;
            let end_row = (n + 1) * 12;
            for row in start_row..min(end_row, self.num_rows) {
                if n == 0 {
                    trace[row][FIRST_LOOP_SELECTOR_OFFSET] = F::ONE;
                }
                assign_u32_in_series(&mut trace, row, RX_OFFSET, &rx.get_u32_slice()[0]);
                assign_u32_in_series(&mut trace, row, RX_OFFSET + 12, &rx.get_u32_slice()[1]);
                assign_u32_in_series(&mut trace, row, RY_OFFSET, &ry.get_u32_slice()[0]);
                assign_u32_in_series(&mut trace, row, RY_OFFSET + 12, &ry.get_u32_slice()[1]);
                assign_u32_in_series(&mut trace, row, RZ_OFFSET, &rz.get_u32_slice()[0]);
                assign_u32_in_series(&mut trace, row, RZ_OFFSET + 12, &rz.get_u32_slice()[1]);
                if bit1 {
                    trace[row][BIT1_SELECTOR_OFFSET] = F::ONE;
                }
                if n < num_coeffs {
                    trace[row][ELL_COEFFS_IDX_OFFSET + n] = F::ONE;
                }
            }
            trace[start_row][FIRST_ROW_SELECTOR_OFFSET] = F::ONE;
            if end_row > self.num_rows {
                break;
            }
            if !bit1 {
                // Loop 0
                let values_0 = calc_precomp_stuff_loop0(rx, ry, rz);
                // t0
                generate_trace_fp2_mul(
                    &mut trace,
                    ry.get_u32_slice(),
                    ry.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    T0_CALC_OFFSET,
                );
                // t1
                generate_trace_fp2_mul(
                    &mut trace,
                    rz.get_u32_slice(),
                    rz.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    T1_CALC_OFFSET,
                );
                // x0
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[4].get_u32_slice(),
                    &Fp::get_fp_from_biguint(BigUint::from_str("3").unwrap()).0,
                    start_row,
                    end_row - 1,
                    X0_CALC_OFFSET,
                );
                // t2
                fill_multiply_by_b_trace(
                    &mut trace,
                    &values_0[5].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    T2_CALC_OFFSET,
                );
                // t3
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[6].get_u32_slice(),
                    &Fp::get_fp_from_biguint(BigUint::from_str("3").unwrap()).0,
                    start_row,
                    end_row - 1,
                    T3_CALC_OFFSET,
                );
                // x1
                generate_trace_fp2_mul(
                    &mut trace,
                    ry.get_u32_slice(),
                    rz.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    X1_CALC_OFFSET,
                );
                // t4
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[8].get_u32_slice(),
                    &Fp::get_fp_from_biguint(BigUint::from_str("2").unwrap()).0,
                    start_row,
                    end_row - 1,
                    T4_CALC_OFFSET,
                );
                // x2
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_0[6].get_u32_slice(),
                        &values_0[3].get_u32_slice(),
                        row,
                        X2_CALC_OFFSET,
                    );
                }
                // x3
                generate_trace_fp2_mul(
                    &mut trace,
                    rx.get_u32_slice(),
                    rx.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    X3_CALC_OFFSET,
                );
                // x4
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[10].get_u32_slice(),
                    &Fp::get_fp_from_biguint(BigUint::from_str("3").unwrap()).0,
                    start_row,
                    end_row - 1,
                    X4_CALC_OFFSET,
                );
                // x5
                for row in start_row..end_row {
                    fill_trace_negate_fp2(
                        &mut trace,
                        &values_0[9].get_u32_slice(),
                        row,
                        X5_CALC_OFFSET,
                    );
                }
                // x6
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_0[3].get_u32_slice(),
                        &values_0[7].get_u32_slice(),
                        row,
                        X6_CALC_OFFSET,
                    );
                }
                // x7
                generate_trace_fp2_mul(
                    &mut trace,
                    rx.get_u32_slice(),
                    ry.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    X7_CALC_OFFSET,
                );
                // x8
                generate_trace_fp2_mul(
                    &mut trace,
                    values_0[14].get_u32_slice(),
                    values_0[15].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    X8_CALC_OFFSET,
                );
                // x9
                for row in start_row..end_row {
                    fill_trace_addition_with_reduction(
                        &mut trace,
                        &values_0[3].get_u32_slice(),
                        &values_0[7].get_u32_slice(),
                        row,
                        X9_CALC_OFFSET,
                    );
                }
                let k = get_u32_vec_from_literal(mod_inverse(BigUint::from(2 as u32), modulus()));
                // x10
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[17].get_u32_slice(),
                    &k,
                    start_row,
                    end_row - 1,
                    X10_CALC_OFFSET,
                );
                // x11
                generate_trace_fp2_mul(
                    &mut trace,
                    values_0[18].get_u32_slice(),
                    values_0[18].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    X11_CALC_OFFSET,
                );
                // x12
                generate_trace_fp2_mul(
                    &mut trace,
                    values_0[6].get_u32_slice(),
                    values_0[6].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    X12_CALC_OFFSET,
                );
                // x13
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[20].get_u32_slice(),
                    &Fp::get_fp_from_biguint(BigUint::from_str("3").unwrap()).0,
                    start_row,
                    end_row - 1,
                    X13_CALC_OFFSET,
                );
                // new rx
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_0[16].get_u32_slice(),
                    &k,
                    start_row,
                    end_row - 1,
                    NEW_RX_OFFSET,
                );
                // new ry
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_0[19].get_u32_slice(),
                        &values_0[21].get_u32_slice(),
                        row,
                        NEW_RY_OFFSET,
                    );
                }
                // new rz
                generate_trace_fp2_mul(
                    &mut trace,
                    values_0[3].get_u32_slice(),
                    values_0[9].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    NEW_RZ_OFFSET,
                );
                rx = values_0[0];
                ry = values_0[1];
                rz = values_0[2];

                bit1 = get_bls_12_381_parameter().bit(bit_pos);
                bit_pos = if bit1 {
                    bit_pos
                } else {
                    bit_pos.checked_sub(1).unwrap_or(0)
                }
            } else {
                let values_1 = calc_precomp_stuff_loop1(rx, ry, rz, qx, qy);
                // bit1_t0
                generate_trace_fp2_mul(
                    &mut trace,
                    qy.get_u32_slice(),
                    rz.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T0_CALC_OFFSET,
                );
                // bit1_t1
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &ry.get_u32_slice(),
                        &values_1[3].get_u32_slice(),
                        row,
                        BIT1_T1_CALC_OFFSET,
                    );
                }
                // bit1_t2
                generate_trace_fp2_mul(
                    &mut trace,
                    qx.get_u32_slice(),
                    rz.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T2_CALC_OFFSET,
                );
                // bit1_t3
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &rx.get_u32_slice(),
                        &values_1[5].get_u32_slice(),
                        row,
                        BIT1_T3_CALC_OFFSET,
                    );
                }
                // bit1_t4
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[4].get_u32_slice(),
                    qx.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T4_CALC_OFFSET,
                );
                // bit1_t5
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[6].get_u32_slice(),
                    qy.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T5_CALC_OFFSET,
                );
                // bit1_t6
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_1[7].get_u32_slice(),
                        &values_1[8].get_u32_slice(),
                        row,
                        BIT1_T6_CALC_OFFSET,
                    );
                }
                // bit1_t7
                for row in start_row..end_row {
                    fill_trace_negate_fp2(
                        &mut trace,
                        &values_1[4].get_u32_slice(),
                        row,
                        BIT1_T7_CALC_OFFSET,
                    );
                }
                // bit1_t8
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[6].get_u32_slice(),
                    values_1[6].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T8_CALC_OFFSET,
                );
                // bit1_t9
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[11].get_u32_slice(),
                    values_1[6].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T9_CALC_OFFSET,
                );
                // bit1_t10
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[11].get_u32_slice(),
                    rx.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T10_CALC_OFFSET,
                );
                // bit1_t11
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[4].get_u32_slice(),
                    values_1[4].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T11_CALC_OFFSET,
                );
                // bit1_t12
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[14].get_u32_slice(),
                    rz.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T12_CALC_OFFSET,
                );
                // bit1_t13
                fill_trace_fp2_fp_mul(
                    &mut trace,
                    &values_1[13].get_u32_slice(),
                    &Fp::get_fp_from_biguint(BigUint::from_str("2").unwrap()).0,
                    start_row,
                    end_row - 1,
                    BIT1_T13_CALC_OFFSET,
                );
                // bit1_t14
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_1[12].get_u32_slice(),
                        &values_1[16].get_u32_slice(),
                        row,
                        BIT1_T14_CALC_OFFSET,
                    );
                }
                // bit1_t15
                for row in start_row..end_row {
                    fill_trace_addition_with_reduction(
                        &mut trace,
                        &values_1[17].get_u32_slice(),
                        &values_1[15].get_u32_slice(),
                        row,
                        BIT1_T15_CALC_OFFSET,
                    );
                }
                // bit1_t16
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_1[13].get_u32_slice(),
                        &values_1[18].get_u32_slice(),
                        row,
                        BIT1_T16_CALC_OFFSET,
                    );
                }
                // bit1_t17
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[19].get_u32_slice(),
                    values_1[4].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T17_CALC_OFFSET,
                );
                // bit1_t18
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[12].get_u32_slice(),
                    ry.get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_T18_CALC_OFFSET,
                );
                // new rx
                generate_trace_fp2_mul(
                    &mut trace,
                    values_1[6].get_u32_slice(),
                    values_1[18].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_RX_CALC_OFFSET,
                );
                // new ry
                for row in start_row..end_row {
                    fill_trace_subtraction_with_reduction(
                        &mut trace,
                        &values_1[20].get_u32_slice(),
                        &values_1[21].get_u32_slice(),
                        row,
                        BIT1_RY_CALC_OFFSET,
                    );
                }
                // new rz
                generate_trace_fp2_mul(
                    &mut trace,
                    rz.get_u32_slice(),
                    values_1[12].get_u32_slice(),
                    start_row,
                    end_row - 1,
                    BIT1_RZ_CALC_OFFSET,
                );

                rx = values_1[0];
                ry = values_1[1];
                rz = values_1[2];
                bit1 = false;
                bit_pos = bit_pos.checked_sub(1).unwrap_or(0);
            }
        }
        trace
    }
}

/*
    The constraints of this stark are as follows:
    * Constraint the result of z*z_inv multiplication to be 1.
    * Constraint the x input of z*z_inv multiplication to public input z.
    * Constraint the x input of x*z_inv multiplication to public input x, and y input of x*z_inv to the y input of z*z_inv.
    * Constraint the x input of y*z_inv multiplication to public input y, and y input of y*z_inv to the y input of z*z_inv.
    * Constraint the result of x*z_inv multiplication to Qx, and constraint Qx to be same in all rows.
    * Constraint the result of y*z_inv multiplication to Qy, and constraint Qy to be same in all rows.
    * Constraint Qz to be 1, and constraint Qz to be same in all rows.
    * Creates two `bit_selector` values from `BIT1_SELECTOR`, `bit0` and `bit1`.
    * For `FIRST_LOOP_SELECTOR` set 1, constraints Rx, Ry, and Rz to Qx, Qy, and Qz.
    * For `FIRST_ROW_SELECTOR` set 1 in the next row, constraints Rx, Ry and Rz values of the next row with current row bit 0 operaion new_Rx, new_Ry, and new_Rz and `bit0` selector, as well as current row bit 1 operation new_Rx, new_Ry, and new_Rz and `bit1` selector.
    * Constraints public inputs with `ELL_COEFFS_IDX` selector and the result of bit 0 operation results and `bit0` selector, as well as the result of bit 1 operation results and `bit1` selector.
    * Constraints all operations for bit 0 computation with `bit0` selector, i.e., those constraints are only on when `BIT1_SELECTOR` is off.
    * Constraints all operations for bit 1 computation with `bit1` selector, i.e., those constraints are only on when `BIT1_SELECTOR` is on.
*/

// Implement constraint generator
impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for PairingPrecompStark<F, D> {
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

        //  ---- Constrain z * z_inv ---
        // Z = [X0, Y0]
        // Z_INV = [X1, Y1] // We dont need to public input constrain Z_INV
        // Z * Z_INV = [c1, c2] => [c1 => [1,..,0], c2 => [0,..,0]]
        for i in 0..12 {
            if i == 0 {
                yield_constr.constraint_first_row(
                    local_values[Z1_REDUCE_OFFSET + REDUCED_OFFSET + i] - FE::ONE,
                )
            } else {
                yield_constr
                    .constraint_first_row(local_values[Z1_REDUCE_OFFSET + REDUCED_OFFSET + i])
            }
            yield_constr.constraint_first_row(local_values[Z2_REDUCE_OFFSET + REDUCED_OFFSET + i])
        }
        // let match_inputs_z_mult_z_inv: Vec<P> = [
        //     &public_inputs[Z0_PUBLIC_INPUTS_OFFSET..Z0_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[Z0_PUBLIC_INPUTS_OFFSET..Z0_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[Z1_PUBLIC_INPUTS_OFFSET..Z1_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[Z1_PUBLIC_INPUTS_OFFSET..Z1_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     ].concat();
        for i in 0..12 {
            yield_constr.constraint_first_row(
                local_values[Z_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - public_inputs[Z0_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[Z_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + 12 + i]
                    - public_inputs[Z1_PUBLIC_INPUTS_OFFSET + i],
            );
        }
        add_fp2_mul_constraints(local_values, next_values, yield_constr, 0, None);

        // Constrain ax = x * z_inv
        // Constrain Z-inv matches with last Z_MULT_Z_INV
        // COnstraint X match with public input
        // let match_inputs_x_mult_z_inv: Vec<P> = [
        //     &public_inputs[X0_PUBLIC_INPUTS_OFFSET..X0_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[X0_PUBLIC_INPUTS_OFFSET..X0_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[X1_PUBLIC_INPUTS_OFFSET..X1_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[X1_PUBLIC_INPUTS_OFFSET..X1_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        // ].concat();

        for i in 0..12 {
            yield_constr.constraint_first_row(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - public_inputs[X0_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + 12 + i]
                    - public_inputs[X1_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values
                        [Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + 12 + i]
                    - local_values
                        [Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
        }
        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X_MULT_Z_INV_OFFSET,
            None,
        );

        // Constrain ay = y * z_inv
        // Constrain Z-inv matches with last Z_MULT_Z_INV
        // COnstraint Y match with public input
        // let match_inputs_y_mult_z_inv: Vec<P> = [
        //     &public_inputs[Y0_PUBLIC_INPUTS_OFFSET..Y0_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[Y0_PUBLIC_INPUTS_OFFSET..Y0_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[Y1_PUBLIC_INPUTS_OFFSET..Y1_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_1_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        //     &public_inputs[Y1_PUBLIC_INPUTS_OFFSET..Y1_PUBLIC_INPUTS_OFFSET+12].iter().map(|x| P::ZEROS + x.clone()).collect::<Vec<P>>(),
        //     &local_values[Z_MULT_Z_INV_OFFSET + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET..Z_MULT_Z_INV_OFFSET + X_1_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET+12],
        // ].concat();
        for i in 0..12 {
            yield_constr.constraint_first_row(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                    - public_inputs[Y0_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + 12 + i]
                    - public_inputs[Y1_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                    - local_values
                        [Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + 12 + i]
                    - local_values
                        [Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
        }
        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            Y_MULT_Z_INV_OFFSET,
            None,
        );

        // Constrain Qx, Qy, Qz
        for i in 0..12 {
            // Qx
            yield_constr.constraint_first_row(
                local_values[X_MULT_Z_INV_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]
                    - local_values[QX_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[X_MULT_Z_INV_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]
                    - local_values[QX_OFFSET + 12 + i],
            );
            // Qy
            yield_constr.constraint_first_row(
                local_values[Y_MULT_Z_INV_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]
                    - local_values[QY_OFFSET + i],
            );
            yield_constr.constraint_first_row(
                local_values[Y_MULT_Z_INV_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]
                    - local_values[QY_OFFSET + 12 + i],
            );
            if i == 0 {
                yield_constr.constraint_first_row(local_values[QZ_OFFSET + i] - FE::ONE);
            } else {
                yield_constr.constraint_first_row(local_values[QZ_OFFSET + i]);
            }
            yield_constr.constraint_first_row(local_values[QZ_OFFSET + 12 + i]);
        }
        for i in 0..24 {
            yield_constr
                .constraint_transition(local_values[QX_OFFSET + i] - next_values[QX_OFFSET + i]);
            yield_constr
                .constraint_transition(local_values[QY_OFFSET + i] - next_values[QY_OFFSET + i]);
            yield_constr
                .constraint_transition(local_values[QZ_OFFSET + i] - next_values[QZ_OFFSET + i]);
        }

        let bit1 = local_values[BIT1_SELECTOR_OFFSET];
        let bit0 = P::ONES - bit1;

        for i in 0..24 {
            yield_constr.constraint(
                local_values[FIRST_LOOP_SELECTOR_OFFSET]
                    * local_values[FIRST_ROW_SELECTOR_OFFSET]
                    * (local_values[RX_OFFSET + i] - local_values[QX_OFFSET + i]),
            );
            yield_constr.constraint(
                local_values[FIRST_LOOP_SELECTOR_OFFSET]
                    * local_values[FIRST_ROW_SELECTOR_OFFSET]
                    * (local_values[RY_OFFSET + i] - local_values[QY_OFFSET + i]),
            );
            yield_constr.constraint(
                local_values[FIRST_LOOP_SELECTOR_OFFSET]
                    * local_values[FIRST_ROW_SELECTOR_OFFSET]
                    * (local_values[RZ_OFFSET + i] - local_values[QZ_OFFSET + i]),
            );
            if i < 12 {
                yield_constr.constraint(
                    bit0 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RX_OFFSET + i]
                            - local_values
                                [NEW_RX_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
                );
                yield_constr.constraint(
                    bit0 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RY_OFFSET + i]
                            - local_values[NEW_RY_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
                yield_constr.constraint(
                    bit0 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RZ_OFFSET + i]
                            - local_values[NEW_RZ_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
                );
                yield_constr.constraint(
                    bit1 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RX_OFFSET + i]
                            - local_values
                                [BIT1_RX_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
                );
                yield_constr.constraint(
                    bit1 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RY_OFFSET + i]
                            - local_values[BIT1_RY_CALC_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
                yield_constr.constraint(
                    bit1 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RZ_OFFSET + i]
                            - local_values
                                [BIT1_RZ_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
                );
            } else {
                yield_constr.constraint(
                    bit0 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RX_OFFSET + i]
                            - local_values
                                [NEW_RX_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i - 12]),
                );
                yield_constr.constraint(
                    bit0 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RY_OFFSET + i]
                            - local_values[NEW_RY_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCE_TOTAL
                                + RANGE_CHECK_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i
                                - 12]),
                );
                yield_constr.constraint(
                    bit0 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RZ_OFFSET + i]
                            - local_values
                                [NEW_RZ_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12]),
                );
                yield_constr.constraint(
                    bit1 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RX_OFFSET + i]
                            - local_values
                                [BIT1_RX_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12]),
                );
                yield_constr.constraint(
                    bit1 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RY_OFFSET + i]
                            - local_values[BIT1_RY_CALC_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCE_TOTAL
                                + RANGE_CHECK_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i
                                - 12]),
                );
                yield_constr.constraint(
                    bit1 * (P::ONES - next_values[FIRST_LOOP_SELECTOR_OFFSET])
                        * next_values[FIRST_ROW_SELECTOR_OFFSET]
                        * (next_values[RZ_OFFSET + i]
                            - local_values
                                [BIT1_RZ_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12]),
                );
            }
            yield_constr.constraint_transition(
                (P::ONES - next_values[FIRST_ROW_SELECTOR_OFFSET])
                    * (local_values[RX_OFFSET + i] - next_values[RX_OFFSET + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[FIRST_ROW_SELECTOR_OFFSET])
                    * (local_values[RY_OFFSET + i] - next_values[RY_OFFSET + i]),
            );
            yield_constr.constraint_transition(
                (P::ONES - next_values[FIRST_ROW_SELECTOR_OFFSET])
                    * (local_values[RZ_OFFSET + i] - next_values[RZ_OFFSET + i]),
            );
        }

        for idx in 0..68 {
            for i in 0..12 {
                yield_constr.constraint(
                    bit0 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[X2_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i]),
                );
                yield_constr.constraint(
                    bit0 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[X2_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 12]),
                );
                yield_constr.constraint(
                    bit0 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[X4_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 24]),
                );
                yield_constr.constraint(
                    bit0 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[X4_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 36]),
                );
                yield_constr.constraint(
                    bit0 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values
                            [X5_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 48]),
                );
                yield_constr.constraint(
                    bit0 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values
                            [X5_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 60]),
                );

                yield_constr.constraint(
                    bit1 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[BIT1_T6_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i]),
                );
                yield_constr.constraint(
                    bit1 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[BIT1_T6_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 12]),
                );
                yield_constr.constraint(
                    bit1 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[BIT1_T7_CALC_OFFSET
                            + FP2_ADDITION_0_OFFSET
                            + FP_ADDITION_Y_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 24]),
                );
                yield_constr.constraint(
                    bit1 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[BIT1_T7_CALC_OFFSET
                            + FP2_ADDITION_1_OFFSET
                            + FP_ADDITION_Y_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 36]),
                );
                yield_constr.constraint(
                    bit1 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 48]),
                );
                yield_constr.constraint(
                    bit1 * local_values[ELL_COEFFS_IDX_OFFSET + idx]
                        * (local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]
                            - public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 60]),
                );
            }
        }

        // t0
        for i in 0..24 {
            yield_constr.constraint(
                bit0 * local_values[T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[RY_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RY_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            T0_CALC_OFFSET,
            Some(bit0),
        );

        // T1
        for i in 0..24 {
            yield_constr.constraint(
                bit0 * local_values[T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            T1_CALC_OFFSET,
            Some(bit0),
        );

        // X0
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X0_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X0_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values[T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X0_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X0_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values[T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            if i == 0 {
                yield_constr.constraint(
                    bit0 * local_values[X0_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[X0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET]
                            - FE::from_canonical_u32(3 as u32)),
                );
            } else {
                yield_constr.constraint(
                    bit0 * local_values[X0_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * local_values[X0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
            }
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X0_CALC_OFFSET,
            Some(bit0),
        );

        // T2
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[T2_CALC_OFFSET + MULTIPLY_B_SELECTOR_OFFSET]
                    * (local_values[X0_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]
                        - local_values[T2_CALC_OFFSET + MULTIPLY_B_X_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[T2_CALC_OFFSET + MULTIPLY_B_SELECTOR_OFFSET]
                    * (local_values[X0_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]
                        - local_values[T2_CALC_OFFSET + MULTIPLY_B_X_OFFSET + i + 12]),
            );
            // if i == 0 {
            //     yield_constr.constraint_first_row(
            //         local_values[T2_CALC_OFFSET + MULTIPLY_B_X0_B_MUL_OFFSET + Y_INPUT_OFFSET + i] -
            //         FE::from_canonical_u32(4 as u32)
            //     );
            // } else {
            //     yield_constr.constraint_first_row(
            //         local_values[T2_CALC_OFFSET + MULTIPLY_B_X0_B_MUL_OFFSET + Y_INPUT_OFFSET + i]
            //     );
            // }
            // if i == 0 {
            //     yield_constr.constraint_first_row(
            //         local_values[T2_CALC_OFFSET + MULTIPLY_B_X1_B_MUL_OFFSET + Y_INPUT_OFFSET + i] -
            //         FE::from_canonical_u32(4 as u32)
            //     );
            // } else {
            //     yield_constr.constraint_first_row(
            //         local_values[T2_CALC_OFFSET + MULTIPLY_B_X1_B_MUL_OFFSET + Y_INPUT_OFFSET + i]
            //     );
            // }
        }
        add_multiply_by_b_constraints(
            local_values,
            next_values,
            yield_constr,
            T2_CALC_OFFSET,
            Some(bit0),
        );

        // T3
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[T3_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[T3_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[T3_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[T3_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            if i == 0 {
                yield_constr.constraint(
                    bit0 * local_values[T3_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[T3_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET]
                            - FE::from_canonical_u32(3 as u32)),
                );
            } else {
                yield_constr.constraint(
                    bit0 * local_values[T3_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[T3_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
                );
            }
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            T3_CALC_OFFSET,
            Some(bit0),
        );

        // x1
        for i in 0..24 {
            yield_constr.constraint(
                bit0 * local_values[X1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[RY_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X1_CALC_OFFSET,
            Some(bit0),
        );

        // T4
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[T4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[T4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values[X1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[T4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[T4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values[X1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            if i == 0 {
                yield_constr.constraint(
                    bit0 * local_values[T4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET]
                            - FE::from_canonical_u32(2 as u32)),
                );
            } else {
                yield_constr.constraint(
                    bit0 * local_values[T4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
                );
            }
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            T4_CALC_OFFSET,
            Some(bit0),
        );

        // x2
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values
                    [X2_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X2_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [X2_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X2_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X2_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[X2_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X2_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[X2_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            X2_CALC_OFFSET,
            Some(bit0),
        );

        // x3
        for i in 0..24 {
            yield_constr.constraint(
                bit0 * local_values[X3_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X3_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[RX_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X3_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X3_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RX_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X3_CALC_OFFSET,
            Some(bit0),
        );

        // x4
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values[X3_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values[X3_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            if i == 0 {
                yield_constr.constraint(
                    bit0 * local_values[X4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[X4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET]
                            - FE::from_canonical_u32(3 as u32)),
                );
            } else {
                yield_constr.constraint(
                    bit0 * local_values[X4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[X4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
                );
            }
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X4_CALC_OFFSET,
            Some(bit0),
        );

        // x5
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values
                    [X5_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[T4_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]
                        - local_values
                            [X5_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [X5_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[T4_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]
                        - local_values
                            [X5_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]),
            );
        }
        add_negate_fp2_constraints(local_values, yield_constr, X5_CALC_OFFSET, Some(bit0));

        // x6
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values
                    [X6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [X6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[X6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[T3_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[X6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[T3_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            X6_CALC_OFFSET,
            Some(bit0),
        );

        // x7
        for i in 0..24 {
            yield_constr.constraint(
                bit0 * local_values[X7_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X7_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[RX_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X7_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X7_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RY_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X7_CALC_OFFSET,
            Some(bit0),
        );

        // x8
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[X6_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[X6_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[X7_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[X7_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X8_CALC_OFFSET,
            Some(bit0),
        );

        // x9
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values
                    [X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i]
                        - local_values[T3_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i]
                        - local_values[T3_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_addition_with_reduction_constranints(
            local_values,
            yield_constr,
            X9_CALC_OFFSET,
            Some(bit0),
        );

        let k = get_u32_vec_from_literal(mod_inverse(BigUint::from(2 as u32), modulus()));

        // x10
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X10_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X10_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values
                            [X9_CALC_OFFSET + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X10_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X10_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values[X9_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X10_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X10_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - FE::from_canonical_u32(k[i])),
            );
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X10_CALC_OFFSET,
            Some(bit0),
        );

        // x11
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[X10_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[X10_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[X10_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[X10_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X11_CALC_OFFSET,
            Some(bit0),
        );

        // x12
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[X12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values
                            [T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X12_CALC_OFFSET,
            Some(bit0),
        );

        // X13
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[X13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values[X12_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[X13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[X13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values[X12_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            if i == 0 {
                yield_constr.constraint(
                    bit0 * local_values[X13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[X13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET]
                            - FE::from_canonical_u32(3 as u32)),
                );
            } else {
                yield_constr.constraint(
                    bit0 * local_values[X13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[X13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
                );
            }
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            X13_CALC_OFFSET,
            Some(bit0),
        );

        // new Rx
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[NEW_RX_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[NEW_RX_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values[X8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RX_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[NEW_RX_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values[X8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RX_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[NEW_RX_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - FE::from_canonical_u32(k[i])),
            );
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            NEW_RX_OFFSET,
            Some(bit0),
        );

        // new Ry
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values
                    [NEW_RY_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [NEW_RY_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[X11_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values
                    [NEW_RY_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [NEW_RY_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[X11_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RY_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[NEW_RY_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[X13_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RY_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[NEW_RY_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[X13_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            NEW_RY_OFFSET,
            Some(bit0),
        );

        // new Rz
        for i in 0..12 {
            yield_constr.constraint(
                bit0 * local_values[NEW_RZ_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[NEW_RZ_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RZ_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[NEW_RZ_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RZ_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[NEW_RZ_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[T4_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit0 * local_values[NEW_RZ_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[NEW_RZ_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[T4_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            NEW_RZ_OFFSET,
            Some(bit0),
        );

        // bit1_t0
        for i in 0..24 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[QY_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T0_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t1
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T1_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T1_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[RY_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T1_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T1_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[RY_OFFSET + i + 12]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T1_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T1_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_T1_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t2
        for i in 0..24 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[QX_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T2_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t3
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T3_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T3_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[RX_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T3_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T3_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[RX_OFFSET + i + 12]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_T3_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t4
        for i in 0..24 {
            if i < 12 {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                        * (local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                            - local_values[BIT1_T1_CALC_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
            } else {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                        * (local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                            - local_values[BIT1_T1_CALC_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCE_TOTAL
                                + RANGE_CHECK_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i
                                - 12]),
                );
            }
            yield_constr.constraint(
                bit1 * local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[QX_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T4_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t5
        for i in 0..24 {
            if i < 12 {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                        * (local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                            - local_values[BIT1_T3_CALC_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i]),
                );
            } else {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                        * (local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                            - local_values[BIT1_T3_CALC_OFFSET
                                + FP2_ADDITION_TOTAL
                                + FP2_SUBTRACTION_TOTAL
                                + FP_SINGLE_REDUCE_TOTAL
                                + RANGE_CHECK_TOTAL
                                + FP_SINGLE_REDUCED_OFFSET
                                + i
                                - 12]),
                );
            }
            yield_constr.constraint(
                bit1 * local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[QY_OFFSET + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T5_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t6
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T4_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T4_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T5_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T5_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_T6_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t7
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T7_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[BIT1_T1_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]
                        - local_values[BIT1_T7_CALC_OFFSET
                            + FP2_ADDITION_0_OFFSET
                            + FP_ADDITION_X_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T7_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values[BIT1_T1_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i]
                        - local_values[BIT1_T7_CALC_OFFSET
                            + FP2_ADDITION_1_OFFSET
                            + FP_ADDITION_X_OFFSET
                            + i]),
            );
        }
        add_negate_fp2_constraints(local_values, yield_constr, BIT1_T7_CALC_OFFSET, Some(bit1));

        // bit1_t8
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T8_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t9
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values
                            [BIT1_T8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values
                            [BIT1_T8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T9_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t10
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values
                            [BIT1_T8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values
                            [BIT1_T8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RX_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[RX_OFFSET + i + 12]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T10_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t11
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[BIT1_T1_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T1_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[BIT1_T1_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T1_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T11_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t12
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values
                            [BIT1_T11_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values
                            [BIT1_T11_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[RZ_OFFSET + i + 12]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T12_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t13
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[BIT1_T13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i]
                        - local_values
                            [BIT1_T10_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                    * (local_values[BIT1_T13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i]
                        - local_values
                            [BIT1_T10_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            if i == 0 {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[BIT1_T13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET]
                            - FE::from_canonical_u32(2 as u32)),
                );
            } else {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET]
                        * (local_values[BIT1_T13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]),
                );
            }
        }

        add_fp2_fp_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T13_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t14
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T14_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T14_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T9_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T14_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T14_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T9_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T14_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T13_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T14_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T13_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_T14_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_T15
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[BIT1_T14_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values[BIT1_T14_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i]
                        - local_values
                            [BIT1_T12_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i]
                        - local_values
                            [BIT1_T12_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_addition_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_T15_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t16
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T16_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T16_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T10_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_T16_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_T16_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T10_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T16_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[BIT1_T15_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_T16_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values[BIT1_T15_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_T16_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t17
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[BIT1_T16_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T16_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[BIT1_T1_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T1_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T17_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t18
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values
                            [BIT1_T9_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values
                            [BIT1_T9_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[RY_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[RY_OFFSET + i + 12]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_T18_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_rx
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T3_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP2_SUBTRACTION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                        - local_values[BIT1_T15_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12]
                        - local_values[BIT1_T15_CALC_OFFSET
                            + FP2_ADDITION_TOTAL
                            + FP_SINGLE_REDUCE_TOTAL
                            + RANGE_CHECK_TOTAL
                            + FP_SINGLE_REDUCED_OFFSET
                            + i]),
            );
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_RX_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_ry
        for i in 0..12 {
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_RY_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_RY_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T17_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values
                    [BIT1_RY_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET]
                    * (local_values
                        [BIT1_RY_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i]
                        - local_values
                            [BIT1_T17_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_RY_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_RY_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_0_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T18_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
            yield_constr.constraint(
                bit1 * local_values[BIT1_RY_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET]
                    * (local_values[BIT1_RY_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_1_OFFSET
                        + FP_SUBTRACTION_Y_OFFSET
                        + i]
                        - local_values
                            [BIT1_T18_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]),
            );
        }

        add_subtraction_with_reduction_constranints(
            local_values,
            yield_constr,
            BIT1_RY_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_rz
        for i in 0..24 {
            yield_constr.constraint(
                bit1 * local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                    * (local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i]
                        - local_values[RZ_OFFSET + i]),
            );
            if i < 12 {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                        * (local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                            - local_values
                                [BIT1_T9_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i]),
                );
            } else {
                yield_constr.constraint(
                    bit1 * local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]
                        * (local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i]
                            - local_values
                                [BIT1_T9_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12]),
                );
            }
        }

        add_fp2_mul_constraints(
            local_values,
            next_values,
            yield_constr,
            BIT1_RZ_CALC_OFFSET,
            Some(bit1),
        );
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

        let one = builder.constant_extension(F::Extension::ONE);

        for i in 0..12 {
            if i == 0 {
                let c =
                    builder.sub_extension(local_values[Z1_REDUCE_OFFSET + REDUCED_OFFSET + i], one);
                yield_constr.constraint_first_row(builder, c);
            } else {
                yield_constr.constraint_first_row(
                    builder,
                    local_values[Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                );
            }

            yield_constr
                .constraint_first_row(builder, local_values[Z2_REDUCE_OFFSET + REDUCED_OFFSET + i]);
        }

        for i in 0..12 {
            let c1 = builder.sub_extension(
                local_values[Z_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                public_inputs[Z0_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c1);

            let c2 = builder.sub_extension(
                local_values[Z_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + 12 + i],
                public_inputs[Z1_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c2);
        }

        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            0,
            None,
        );

        for i in 0..12 {
            let c1 = builder.sub_extension(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                public_inputs[X0_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c1);

            let c2 = builder.sub_extension(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + 12 + i],
                public_inputs[X1_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c2);

            let c3 = builder.sub_extension(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values
                    [Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c3);

            let c4 = builder.sub_extension(
                local_values[X_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + 12 + i],
                local_values
                    [Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X_MULT_Z_INV_OFFSET,
            None,
        );

        for i in 0..12 {
            let c1 = builder.sub_extension(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                public_inputs[Y0_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c1);

            let c2 = builder.sub_extension(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_X_INPUT_OFFSET + 12 + i],
                public_inputs[Y1_PUBLIC_INPUTS_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c2);

            let c3 = builder.sub_extension(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values
                    [Z_MULT_Z_INV_OFFSET + X_0_Y_0_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c3);

            let c4 = builder.sub_extension(
                local_values[Y_MULT_Z_INV_OFFSET + FP2_FP2_Y_INPUT_OFFSET + 12 + i],
                local_values
                    [Z_MULT_Z_INV_OFFSET + X_0_Y_1_MULTIPLICATION_OFFSET + Y_INPUT_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            Y_MULT_Z_INV_OFFSET,
            None,
        );

        for i in 0..12 {
            let c1 = builder.sub_extension(
                local_values[X_MULT_Z_INV_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[QX_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c1);

            let c2 = builder.sub_extension(
                local_values[X_MULT_Z_INV_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[QX_OFFSET + 12 + i],
            );
            yield_constr.constraint_first_row(builder, c2);

            let c3 = builder.sub_extension(
                local_values[Y_MULT_Z_INV_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[QY_OFFSET + i],
            );
            yield_constr.constraint_first_row(builder, c3);

            let c4 = builder.sub_extension(
                local_values[Y_MULT_Z_INV_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[QY_OFFSET + 12 + i],
            );
            yield_constr.constraint_first_row(builder, c4);

            if i == 0 {
                let c = builder.sub_extension(local_values[QZ_OFFSET + i], one);
                yield_constr.constraint_first_row(builder, c);
            } else {
                yield_constr.constraint_first_row(builder, local_values[QZ_OFFSET + i]);
            }
            yield_constr.constraint_first_row(builder, local_values[QZ_OFFSET + 12 + i]);
        }
        for i in 0..24 {
            let c1 = builder.sub_extension(local_values[QX_OFFSET + i], next_values[QX_OFFSET + i]);
            yield_constr.constraint_transition(builder, c1);

            let c2 = builder.sub_extension(local_values[QY_OFFSET + i], next_values[QY_OFFSET + i]);
            yield_constr.constraint_transition(builder, c2);

            let c3 = builder.sub_extension(local_values[QZ_OFFSET + i], next_values[QZ_OFFSET + i]);
            yield_constr.constraint_transition(builder, c3);
        }

        let bit1 = local_values[BIT1_SELECTOR_OFFSET];
        let bit0 = builder.sub_extension(one, bit1);

        for i in 0..24 {
            let mul_tmp1 = builder.mul_extension(
                local_values[FIRST_LOOP_SELECTOR_OFFSET],
                local_values[FIRST_ROW_SELECTOR_OFFSET],
            );

            let sub_tmp1 =
                builder.sub_extension(local_values[RX_OFFSET + i], local_values[QX_OFFSET + i]);
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 =
                builder.sub_extension(local_values[RY_OFFSET + i], local_values[QY_OFFSET + i]);
            let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 =
                builder.sub_extension(local_values[RZ_OFFSET + i], local_values[QZ_OFFSET + i]);
            let c3 = builder.mul_extension(mul_tmp1, sub_tmp3);
            yield_constr.constraint(builder, c3);

            if i < 12 {
                let sub_tmp1 = builder.sub_extension(one, next_values[FIRST_LOOP_SELECTOR_OFFSET]);

                let mul_tmp1 = builder.mul_extension(bit0, sub_tmp1);
                let mul_tmp2 =
                    builder.mul_extension(mul_tmp1, next_values[FIRST_ROW_SELECTOR_OFFSET]);

                let sub_tmp2 = builder.sub_extension(
                    next_values[RX_OFFSET + i],
                    local_values[NEW_RX_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                );
                let c1 = builder.mul_extension(mul_tmp2, sub_tmp2);
                yield_constr.constraint(builder, c1);

                let sub_tmp3 = builder.sub_extension(
                    next_values[RY_OFFSET + i],
                    local_values[NEW_RY_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c2 = builder.mul_extension(mul_tmp2, sub_tmp3);
                yield_constr.constraint(builder, c2);

                let sub_tmp4 = builder.sub_extension(
                    next_values[RZ_OFFSET + i],
                    local_values[NEW_RZ_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                );
                let c3 = builder.mul_extension(mul_tmp2, sub_tmp4);
                yield_constr.constraint(builder, c3);

                let mul_tmp3 = builder.mul_extension(bit1, sub_tmp1);
                let mul_tmp4 =
                    builder.mul_extension(mul_tmp3, next_values[FIRST_ROW_SELECTOR_OFFSET]);

                let sub_tmp5 = builder.sub_extension(
                    next_values[RX_OFFSET + i],
                    local_values[BIT1_RX_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                );
                let c4 = builder.mul_extension(mul_tmp4, sub_tmp5);
                yield_constr.constraint(builder, c4);

                let sub_tmp6 = builder.sub_extension(
                    next_values[RY_OFFSET + i],
                    local_values[BIT1_RY_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c5 = builder.mul_extension(mul_tmp4, sub_tmp6);
                yield_constr.constraint(builder, c5);

                let sub_tmp7 = builder.sub_extension(
                    next_values[RZ_OFFSET + i],
                    local_values[BIT1_RZ_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                );
                let c6 = builder.mul_extension(mul_tmp4, sub_tmp7);
                yield_constr.constraint(builder, c6);
            } else {
                let sub_tmp1 = builder.sub_extension(one, next_values[FIRST_LOOP_SELECTOR_OFFSET]);
                let mul_tmp1 =
                    builder.mul_extension(sub_tmp1, next_values[FIRST_ROW_SELECTOR_OFFSET]);

                let mul_tmp2 = builder.mul_extension(bit0, mul_tmp1);

                let sub_tmp1 = builder.sub_extension(
                    next_values[RX_OFFSET + i],
                    local_values[NEW_RX_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i - 12],
                );
                let c1 = builder.mul_extension(mul_tmp2, sub_tmp1);
                yield_constr.constraint(builder, c1);

                let sub_tmp2 = builder.sub_extension(
                    next_values[RY_OFFSET + i],
                    local_values[NEW_RY_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i
                        - 12],
                );
                let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
                yield_constr.constraint(builder, c2);

                let sub_tmp3 = builder.sub_extension(
                    next_values[RZ_OFFSET + i],
                    local_values[NEW_RZ_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12],
                );
                let c3 = builder.mul_extension(mul_tmp2, sub_tmp3);
                yield_constr.constraint(builder, c3);

                let mul_tmp3 = builder.mul_extension(bit1, mul_tmp1);

                let sub_tmp4 = builder.sub_extension(
                    next_values[RX_OFFSET + i],
                    local_values[BIT1_RX_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12],
                );
                let c4 = builder.mul_extension(mul_tmp3, sub_tmp4);
                yield_constr.constraint(builder, c4);

                let sub_tmp5 = builder.sub_extension(
                    next_values[RY_OFFSET + i],
                    local_values[BIT1_RY_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i
                        - 12],
                );
                let c5 = builder.mul_extension(mul_tmp3, sub_tmp5);
                yield_constr.constraint(builder, c5);

                let sub_tmp6 = builder.sub_extension(
                    next_values[RZ_OFFSET + i],
                    local_values[BIT1_RZ_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12],
                );
                let c6 = builder.mul_extension(mul_tmp3, sub_tmp6);
                yield_constr.constraint(builder, c6);
            }

            let sub_tmp = builder.sub_extension(one, next_values[FIRST_ROW_SELECTOR_OFFSET]);

            let sub_tmp1 =
                builder.sub_extension(local_values[RX_OFFSET + i], next_values[RX_OFFSET + i]);
            let c1 = builder.mul_extension(sub_tmp, sub_tmp1);
            yield_constr.constraint_transition(builder, c1);

            let sub_tmp2 =
                builder.sub_extension(local_values[RY_OFFSET + i], next_values[RY_OFFSET + i]);
            let c2 = builder.mul_extension(sub_tmp, sub_tmp2);
            yield_constr.constraint_transition(builder, c2);

            let sub_tmp3 =
                builder.sub_extension(local_values[RZ_OFFSET + i], next_values[RZ_OFFSET + i]);
            let c3 = builder.mul_extension(sub_tmp, sub_tmp3);
            yield_constr.constraint_transition(builder, c3);
        }

        for idx in 0..68 {
            for i in 0..12 {
                let mul_tmp1 =
                    builder.mul_extension(bit0, local_values[ELL_COEFFS_IDX_OFFSET + idx]);

                let sub_tmp1 = builder.sub_extension(
                    local_values[X2_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i],
                );
                let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
                yield_constr.constraint(builder, c1);

                let sub_tmp2 = builder.sub_extension(
                    local_values[X2_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 12],
                );
                let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
                yield_constr.constraint(builder, c2);

                let sub_tmp3 = builder.sub_extension(
                    local_values[X4_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 24],
                );
                let c3 = builder.mul_extension(mul_tmp1, sub_tmp3);
                yield_constr.constraint(builder, c3);

                let sub_tmp4 = builder.sub_extension(
                    local_values[X4_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 36],
                );
                let c4 = builder.mul_extension(mul_tmp1, sub_tmp4);
                yield_constr.constraint(builder, c4);

                let sub_tmp5 = builder.sub_extension(
                    local_values[X5_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 48],
                );
                let c5 = builder.mul_extension(mul_tmp1, sub_tmp5);
                yield_constr.constraint(builder, c5);

                let sub_tmp6 = builder.sub_extension(
                    local_values[X5_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 60],
                );
                let c6 = builder.mul_extension(mul_tmp1, sub_tmp6);
                yield_constr.constraint(builder, c6);

                let mul_tmp2 =
                    builder.mul_extension(bit1, local_values[ELL_COEFFS_IDX_OFFSET + idx]);

                let sub_tmp7 = builder.sub_extension(
                    local_values[BIT1_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i],
                );
                let c7 = builder.mul_extension(mul_tmp2, sub_tmp7);
                yield_constr.constraint(builder, c7);

                let sub_tmp8 = builder.sub_extension(
                    local_values[BIT1_T6_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 12],
                );
                let c8 = builder.mul_extension(mul_tmp2, sub_tmp8);
                yield_constr.constraint(builder, c8);

                let sub_tmp9 = builder.sub_extension(
                    local_values
                        [BIT1_T7_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 24],
                );
                let c9 = builder.mul_extension(mul_tmp2, sub_tmp9);
                yield_constr.constraint(builder, c9);

                let sub_tmp10 = builder.sub_extension(
                    local_values
                        [BIT1_T7_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 36],
                );
                let c10 = builder.mul_extension(mul_tmp2, sub_tmp10);
                yield_constr.constraint(builder, c10);

                let sub_tmp11 = builder.sub_extension(
                    local_values[BIT1_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 48],
                );
                let c11 = builder.mul_extension(mul_tmp2, sub_tmp11);
                yield_constr.constraint(builder, c11);

                let sub_tmp12 = builder.sub_extension(
                    local_values[BIT1_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                    public_inputs[ELL_COEFFS_PUBLIC_INPUTS_OFFSET + idx * 72 + i + 60],
                );
                let c12 = builder.mul_extension(mul_tmp2, sub_tmp12);
                yield_constr.constraint(builder, c12);
            }
        }

        // T0
        for i in 0..24 {
            let mul_tmp1 =
                builder.mul_extension(bit0, local_values[T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[RY_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RY_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }

        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            T0_CALC_OFFSET,
            Some(bit0),
        );

        // T1
        for i in 0..24 {
            let mul_tmp1 =
                builder.mul_extension(bit0, local_values[T1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[T1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[T1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp1, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            T1_CALC_OFFSET,
            Some(bit0),
        );
        // X0
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[X0_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X0_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[T1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X0_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[T1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            if i == 0 {
                let constant =
                    builder.constant_extension(F::Extension::from_canonical_u32(3 as u32));
                let sub_tmp = builder.sub_extension(
                    local_values[X0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET],
                    constant,
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let c = builder.mul_extension(
                    mul_tmp,
                    local_values[X0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
                yield_constr.constraint(builder, c);
            }
        }

        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X0_CALC_OFFSET,
            Some(bit0),
        );

        // T2
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[T2_CALC_OFFSET + MULTIPLY_B_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X0_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_X_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X0_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_X_OFFSET + i + 12],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            //comments in the function not converted
        }

        add_multiply_by_b_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            T2_CALC_OFFSET,
            Some(bit0),
        );

        // T3
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[T3_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[T3_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[T3_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            if i == 0 {
                let constant =
                    builder.constant_extension(F::Extension::from_canonical_u32(3 as u32));
                let sub_tmp = builder.sub_extension(
                    local_values[T3_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET],
                    constant,
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let c = builder.mul_extension(
                    mul_tmp,
                    local_values[T3_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
                yield_constr.constraint(builder, c);
            }
        }

        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            T3_CALC_OFFSET,
            Some(bit0),
        );

        // x1

        for i in 0..24 {
            let mul_tmp =
                builder.mul_extension(bit0, local_values[X1_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[X1_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[RY_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X1_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }

        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X1_CALC_OFFSET,
            Some(bit0),
        );

        // T4
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[T4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[T4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[X1_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[T4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[X1_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            if i == 0 {
                let constant =
                    builder.constant_extension(F::Extension::from_canonical_u32(2 as u32));
                let sub_tmp = builder.sub_extension(
                    local_values[T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET],
                    constant,
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let c = builder.mul_extension(
                    mul_tmp,
                    local_values[T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
                yield_constr.constraint(builder, c);
            }
        }

        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            T4_CALC_OFFSET,
            Some(bit0),
        );

        // x2
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit0,
                local_values[X2_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[X2_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit0,
                local_values[X2_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values[X2_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit0,
                local_values[X2_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[X2_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit0,
                local_values[X2_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[X2_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }

        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            X2_CALC_OFFSET,
            Some(bit0),
        );

        //x3
        for i in 0..24 {
            let mul_tmp =
                builder.mul_extension(bit0, local_values[X3_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[X3_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[RX_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X3_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RX_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }

        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X3_CALC_OFFSET,
            Some(bit0),
        );

        //x4
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[X4_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[X3_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X4_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[X3_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);
            if i == 0 {
                let constant =
                    builder.constant_extension(F::Extension::from_canonical_u32(3 as u32));
                let sub_tmp = builder.sub_extension(
                    local_values[X4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET],
                    constant,
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let c = builder.mul_extension(
                    mul_tmp,
                    local_values[X4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
                yield_constr.constraint(builder, c);
            }
        }

        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X4_CALC_OFFSET,
            Some(bit0),
        );

        // x5
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit0,
                local_values[X5_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[T4_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[X5_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit0,
                local_values[X5_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values[T4_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
                local_values[X5_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }
        add_negate_fp2_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            X5_CALC_OFFSET,
            Some(bit0),
        );

        // x6
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit0,
                local_values[X6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[X6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit0,
                local_values[X6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values[X6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit0,
                local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[T3_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit0,
                local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[T3_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            X6_CALC_OFFSET,
            Some(bit0),
        );

        //x7
        for i in 0..24 {
            let mul_tmp =
                builder.mul_extension(bit0, local_values[X7_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[X7_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[RX_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X7_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RY_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X7_CALC_OFFSET,
            Some(bit0),
        );

        // x8
        for i in 0..12 {
            let mul_tmp =
                builder.mul_extension(bit0, local_values[X8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[X8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[X6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[X8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[X7_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[X8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[X7_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X8_CALC_OFFSET,
            Some(bit0),
        );

        //x9
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit0,
                local_values[X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                bit0,
                local_values[X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[X9_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i],
                local_values[T3_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp1, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[X9_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i],
                local_values[T3_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp2, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_addition_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            X9_CALC_OFFSET,
            Some(bit0),
        );

        let k = get_u32_vec_from_literal(mod_inverse(BigUint::from(2 as u32), modulus()));

        // x10
        for i in 0..12 {
            let lc = builder.constant_extension(F::Extension::from_canonical_u32(k[i]));

            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[X10_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X10_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[X9_CALC_OFFSET + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X10_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[X9_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[X10_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                lc,
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);
        }

        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X10_CALC_OFFSET,
            Some(bit0),
        );

        //x11
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[X11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[X10_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[X10_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[X11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[X10_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[X11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[X10_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X11_CALC_OFFSET,
            Some(bit0),
        );

        //x12
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[X12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[X12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z0_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[X12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[T2_CALC_OFFSET + MULTIPLY_B_Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X12_CALC_OFFSET,
            Some(bit0),
        );

        //x13

        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[X13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[X13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[X12_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[X13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[X12_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            if i == 0 {
                let constant =
                    builder.constant_extension(F::Extension::from_canonical_u32(3 as u32));
                let sub_tmp = builder.sub_extension(
                    local_values[X13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET],
                    constant,
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let c = builder.mul_extension(
                    mul_tmp,
                    local_values[X13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
                yield_constr.constraint(builder, c);
            }
        }
        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            X13_CALC_OFFSET,
            Some(bit0),
        );

        // new Rx
        for i in 0..12 {
            let lc = builder.constant_extension(F::Extension::from_canonical_u32(k[i]));
            let mul_tmp = builder.mul_extension(
                bit0,
                local_values[NEW_RX_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[NEW_RX_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[X8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[NEW_RX_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[X8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 =
                builder.sub_extension(local_values[NEW_RX_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i], lc);
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);
        }
        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            NEW_RX_OFFSET,
            Some(bit0),
        );

        // new Ry
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit0,
                local_values[NEW_RY_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[NEW_RY_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[X11_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit0,
                local_values[NEW_RY_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values[NEW_RY_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[X11_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit0,
                local_values[NEW_RY_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[NEW_RY_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[X13_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit0,
                local_values[NEW_RY_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[NEW_RY_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[X13_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            NEW_RY_OFFSET,
            Some(bit0),
        );

        // new Rz
        for i in 0..12 {
            let mul_tmp =
                builder.mul_extension(bit0, local_values[NEW_RZ_OFFSET + FP2_FP2_SELECTOR_OFFSET]);

            let sub_tmp1 = builder.sub_extension(
                local_values[NEW_RZ_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[NEW_RZ_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[NEW_RZ_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[T4_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[NEW_RZ_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[T4_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            NEW_RZ_OFFSET,
            Some(bit0),
        );

        // bit1_t0
        for i in 0..24 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[QY_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T0_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T0_CALC_OFFSET,
            Some(bit1),
        );

        //bit1_t1
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T1_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_T1_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[RY_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T1_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_T1_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[RY_OFFSET + i + 12],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit1,
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T0_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit1,
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T0_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }

        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T1_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t2
        for i in 0..24 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[QX_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T2_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }

        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T2_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t3
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T3_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_T3_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[RX_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T3_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_T3_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[RX_OFFSET + i + 12],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit1,
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T2_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit1,
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T2_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T3_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t4
        for i in 0..24 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );
            if i < 12 {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                    local_values[BIT1_T1_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                    local_values[BIT1_T1_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i
                        - 12],
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            }

            let sub_tmp = builder.sub_extension(
                local_values[BIT1_T4_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[QX_OFFSET + i],
            );
            let c = builder.mul_extension(mul_tmp, sub_tmp);
            yield_constr.constraint(builder, c);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T4_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t5
        for i in 0..24 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );
            if i < 12 {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                    local_values[BIT1_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i],
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                    local_values[BIT1_T3_CALC_OFFSET
                        + FP2_ADDITION_TOTAL
                        + FP2_SUBTRACTION_TOTAL
                        + FP_SINGLE_REDUCE_TOTAL
                        + RANGE_CHECK_TOTAL
                        + FP_SINGLE_REDUCED_OFFSET
                        + i
                        - 12],
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            }
            let sub_tmp = builder.sub_extension(
                local_values[BIT1_T5_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[QY_OFFSET + i],
            );
            let c = builder.mul_extension(mul_tmp, sub_tmp);
            yield_constr.constraint(builder, c);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T5_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t6
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_T6_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T4_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_T6_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T4_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit1,
                local_values[BIT1_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T5_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit1,
                local_values[BIT1_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T6_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T5_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T6_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t7
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T7_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
                local_values
                    [BIT1_T7_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T7_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
                local_values
                    [BIT1_T7_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);
        }
        add_negate_fp2_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T7_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t8
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T8_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T8_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t9
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T9_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T9_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t10
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T8_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T8_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RX_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T10_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[RX_OFFSET + i + 12],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T10_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t11
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T11_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T11_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t12
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T11_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T11_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T12_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[RZ_OFFSET + i + 12],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T12_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t13
        for i in 0..12 {
            let lc = builder.constant_extension(F::Extension::from_canonical_u32(2 as u32));

            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T13_CALC_OFFSET + FP2_FP_MUL_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + i],
                local_values[BIT1_T10_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T13_CALC_OFFSET + FP2_FP_X_INPUT_OFFSET + 12 + i],
                local_values[BIT1_T10_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            if i == 0 {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_T13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET],
                    lc,
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let c = builder.mul_extension(
                    mul_tmp,
                    local_values[BIT1_T13_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                );
                yield_constr.constraint(builder, c);
            }
        }
        add_fp2_fp_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T13_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t14
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T14_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_T14_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T9_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T14_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_T14_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T9_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit1,
                local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T13_CALC_OFFSET + X0_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit1,
                local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T13_CALC_OFFSET + X1_Y_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }

        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T14_CALC_OFFSET,
            Some(bit1),
        );

        //bit1_t15
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T14_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_Y_OFFSET + i],
                local_values[BIT1_T12_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp1, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_Y_OFFSET + i],
                local_values[BIT1_T12_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp2, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_addition_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T15_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t16
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T16_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_T16_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T10_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_T16_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_T16_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T10_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit1,
                local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit1,
                local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T15_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_T16_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t17
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T16_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T17_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[BIT1_T1_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T17_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_t18
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T9_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T9_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values[RY_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_T18_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[RY_OFFSET + i + 12],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_T18_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_rx
        for i in 0..12 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp1 = builder.sub_extension(
                local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c1 = builder.mul_extension(mul_tmp, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let sub_tmp2 = builder.sub_extension(
                local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i + 12],
                local_values[BIT1_T3_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c2 = builder.mul_extension(mul_tmp, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                local_values
                    [BIT1_T15_CALC_OFFSET + FP2_ADDITION_TOTAL + FP_SINGLE_REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_RX_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i + 12],
                local_values[BIT1_T15_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP_SINGLE_REDUCE_TOTAL
                    + RANGE_CHECK_TOTAL
                    + FP_SINGLE_REDUCED_OFFSET
                    + i],
            );
            let c4 = builder.mul_extension(mul_tmp, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_RX_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_ry
        for i in 0..12 {
            let mul_tmp1 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_RY_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp1 = builder.sub_extension(
                local_values
                    [BIT1_RY_CALC_OFFSET + FP2_ADDITION_0_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T17_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c1 = builder.mul_extension(mul_tmp1, sub_tmp1);
            yield_constr.constraint(builder, c1);

            let mul_tmp2 = builder.mul_extension(
                bit1,
                local_values
                    [BIT1_RY_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_CHECK_OFFSET],
            );
            let sub_tmp2 = builder.sub_extension(
                local_values
                    [BIT1_RY_CALC_OFFSET + FP2_ADDITION_1_OFFSET + FP_ADDITION_X_OFFSET + i],
                local_values[BIT1_T17_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c2 = builder.mul_extension(mul_tmp2, sub_tmp2);
            yield_constr.constraint(builder, c2);

            let mul_tmp3 = builder.mul_extension(
                bit1,
                local_values[BIT1_RY_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp3 = builder.sub_extension(
                local_values[BIT1_RY_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_0_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T18_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c3 = builder.mul_extension(mul_tmp3, sub_tmp3);
            yield_constr.constraint(builder, c3);

            let mul_tmp4 = builder.mul_extension(
                bit1,
                local_values[BIT1_RY_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_CHECK_OFFSET],
            );
            let sub_tmp4 = builder.sub_extension(
                local_values[BIT1_RY_CALC_OFFSET
                    + FP2_ADDITION_TOTAL
                    + FP2_SUBTRACTION_1_OFFSET
                    + FP_SUBTRACTION_Y_OFFSET
                    + i],
                local_values[BIT1_T18_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i],
            );
            let c4 = builder.mul_extension(mul_tmp4, sub_tmp4);
            yield_constr.constraint(builder, c4);
        }
        add_subtraction_with_reduction_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            BIT1_RY_CALC_OFFSET,
            Some(bit1),
        );

        // bit1_rz
        for i in 0..24 {
            let mul_tmp = builder.mul_extension(
                bit1,
                local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_SELECTOR_OFFSET],
            );

            let sub_tmp = builder.sub_extension(
                local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_X_INPUT_OFFSET + i],
                local_values[RZ_OFFSET + i],
            );
            let c = builder.mul_extension(mul_tmp, sub_tmp);
            yield_constr.constraint(builder, c);

            if i < 12 {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                    local_values[BIT1_T9_CALC_OFFSET + Z1_REDUCE_OFFSET + REDUCED_OFFSET + i],
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            } else {
                let sub_tmp = builder.sub_extension(
                    local_values[BIT1_RZ_CALC_OFFSET + FP2_FP2_Y_INPUT_OFFSET + i],
                    local_values[BIT1_T9_CALC_OFFSET + Z2_REDUCE_OFFSET + REDUCED_OFFSET + i - 12],
                );
                let c = builder.mul_extension(mul_tmp, sub_tmp);
                yield_constr.constraint(builder, c);
            }
        }
        add_fp2_mul_constraints_ext_circuit(
            builder,
            yield_constr,
            local_values,
            next_values,
            BIT1_RZ_CALC_OFFSET,
            Some(bit1),
        );
    }

    fn constraint_degree(&self) -> usize {
        4
    }
}
