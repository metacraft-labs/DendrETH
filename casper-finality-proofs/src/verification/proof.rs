use std::time::Instant;

use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, plonk::config::GenericConfig,
    util::timing::TimingTree,
};
use starky::{
    config::StarkConfig, proof::StarkProofWithPublicInputs, prover::prove,
    util::trace_rows_to_poly_values, verifier::verify_stark_proof,
};

use crate::verification::{
    miller_loop::{self, MillerLoopStark},
    native,
};

use super::native::{Fp, Fp2};

fn miller_loop_main<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    x: Fp,
    y: Fp,
    q_x: Fp2,
    q_y: Fp2,
    q_z: Fp2,
) {
    let config = StarkConfig::standard_fast_config();
    let stark = MillerLoopStark::<F, D>::new(1024);
    let ell_coeffs = native::calc_pairing_precomp(q_x, q_y, q_z);
    let res = native::miller_loop(x, y, q_x, q_y, q_z);
    let mut public_inputs = Vec::<F>::new();
    for e in x.0.iter() {
        public_inputs.push(F::from_canonical_u32(*e));
    }
    for e in y.0.iter() {
        public_inputs.push(F::from_canonical_u32(*e));
    }
    for coeff in ell_coeffs.iter() {
        for f2 in coeff.iter() {
            for f in f2.0.iter() {
                for e in f.0.iter() {
                    public_inputs.push(F::from_canonical_u32(*e));
                }
            }
        }
    }
    for f in res.0.iter() {
        for e in f.0.iter() {
            public_inputs.push(F::from_canonical_u32(*e));
        }
    }
    assert_eq!(public_inputs.len(), miller_loop::PUBLIC_INPUTS);
    let s = Instant::now();
    let trace = stark.generate_trace(x, y, ell_coeffs);
    let trace_poly_values = trace_rows_to_poly_values(trace);
    let proof = prove::<F, C, MillerLoopStark<F, D>, D>(
        stark,
        &config,
        trace_poly_values,
        &public_inputs,
        &mut TimingTree::default(),
    )
    .unwrap();
    println!("Time taken for miller_loop stark proof {:?}", s.elapsed());
    verify_stark_proof(stark, proof.clone(), &config).unwrap();
    //(stark, proof, config)
}
