use plonky2::{field::extension::Extendable, hash::hash_types::RichField};

pub fn assign_u32_12<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    row: usize,
    start_col: usize,
    val: [u32; 12],
) {
    for i in 0..12 {
        trace[row][start_col + i] = F::from_canonical_u32(val[i]);
    }
}

pub fn assign_u32_in_series<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    row: usize,
    start_col: usize,
    val: &[u32],
) {
    for i in 0..val.len() {
        trace[row][start_col + i] = F::from_canonical_u32(val[i]);
    }
}

pub fn assign_cols_from_prev<F: RichField + Extendable<D>, const D: usize, const C: usize>(
    trace: &mut Vec<[F; C]>,
    row: usize,
    start_col: usize,
    num_cols: usize,
) {
    assert!(row >= 1);
    for i in start_col..start_col + num_cols {
        trace[row][start_col + i] = trace[row - 1][start_col + i];
    }
}
