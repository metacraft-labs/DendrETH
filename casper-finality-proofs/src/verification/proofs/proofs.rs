use std::time::Instant;

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        circuit_data::{CommonCircuitData, VerifierOnlyCircuitData},
        config::GenericConfig,
        proof::ProofWithPublicInputs,
    },
    util::{log2_ceil, timing::TimingTree},
};
use plonky2x::prelude::Field;
use plonky2x::{
    backend::circuit::{DefaultParameters, PlonkParameters},
    utils::proof::ProofWithPublicInputsTargetUtils,
};
use starky::{
    config::StarkConfig, prover::prove, util::trace_rows_to_poly_values,
    verifier::verify_stark_proof,
};

use crate::verification::{
    proofs::{
        calc_pairing_precomp, ecc_aggregate,
        final_exponentiate::{self, FinalExponentiateStark},
        miller_loop::{self, MillerLoopStark},
    },
    utils::native_bls::{self, Fp, Fp12, Fp2},
};

use super::{calc_pairing_precomp::PairingPrecompStark, ecc_aggregate::ECCAggStark};

pub fn calc_pairing_precomp_proof<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    x: Fp2,
    y: Fp2,
    z: Fp2,
) -> (
    PairingPrecompStark<F, D>,
    starky::proof::StarkProofWithPublicInputs<F, C, D>,
    StarkConfig,
) {
    let mut config = StarkConfig::standard_fast_config();
    config.fri_config.rate_bits = 2;
    let stark = PairingPrecompStark::<F, D>::new(1024);
    let trace = stark.generate_trace(x.get_u32_slice(), y.get_u32_slice(), z.get_u32_slice());
    let ell_coeffs = native_bls::calc_pairing_precomp(x, y, z);
    let mut public_inputs = Vec::new();
    for e in x.get_u32_slice().concat().iter() {
        public_inputs.push(F::from_canonical_u32(e.clone()));
    }
    for e in y.get_u32_slice().concat().iter() {
        public_inputs.push(F::from_canonical_u32(e.clone()));
    }
    for e in z.get_u32_slice().concat().iter() {
        public_inputs.push(F::from_canonical_u32(e.clone()));
    }
    for cs in ell_coeffs.iter() {
        for fp2 in cs.iter() {
            for fp in fp2.0.iter() {
                for e in fp.0.iter() {
                    public_inputs.push(F::from_canonical_u32(*e));
                }
            }
        }
    }
    assert_eq!(public_inputs.len(), calc_pairing_precomp::PUBLIC_INPUTS);
    let trace_poly_values = trace_rows_to_poly_values(trace);
    let t = Instant::now();
    let proof = prove::<F, C, PairingPrecompStark<F, D>, D>(
        stark,
        &config,
        trace_poly_values,
        &public_inputs,
        &mut TimingTree::default(),
    )
    .unwrap();
    println!(
        "Time taken for calc_pairing_precomp stark proof {:?}",
        t.elapsed()
    );
    verify_stark_proof(stark, proof.clone(), &config).unwrap();
    (stark, proof, config)
}

pub fn miller_loop_main<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    x: Fp,
    y: Fp,
    q_x: Fp2,
    q_y: Fp2,
    q_z: Fp2,
) -> (
    MillerLoopStark<F, D>,
    starky::proof::StarkProofWithPublicInputs<F, C, D>,
    StarkConfig,
) {
    let config = StarkConfig::standard_fast_config();
    let stark = MillerLoopStark::<F, D>::new(1024);
    let ell_coeffs = native_bls::calc_pairing_precomp(q_x, q_y, q_z);
    let res = native_bls::miller_loop(x, y, q_x, q_y, q_z);
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
    (stark, proof, config)
}

pub fn final_exponentiate_main<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    x: Fp12,
) -> (
    FinalExponentiateStark<F, D>,
    starky::proof::StarkProofWithPublicInputs<F, C, D>,
    StarkConfig,
) {
    let mut config = StarkConfig::standard_fast_config();
    config.fri_config.rate_bits = 2;
    let stark = FinalExponentiateStark::<F, D>::new(8192);
    let s = Instant::now();
    let mut public_inputs = Vec::<F>::new();
    for e in x.get_u32_slice().concat().iter() {
        public_inputs.push(F::from_canonical_u32(*e));
    }
    for e in x.final_exponentiate().get_u32_slice().concat().iter() {
        public_inputs.push(F::from_canonical_u32(*e));
    }
    assert_eq!(public_inputs.len(), final_exponentiate::PUBLIC_INPUTS);
    let trace = stark.generate_trace(x);
    let trace_poly_values = trace_rows_to_poly_values(trace);
    let proof = prove::<F, C, FinalExponentiateStark<F, D>, D>(
        stark,
        &config,
        trace_poly_values,
        &public_inputs,
        &mut TimingTree::default(),
    )
    .unwrap();
    println!(
        "Time taken for final_exponentiate stark proof {:?}",
        s.elapsed()
    );
    verify_stark_proof(stark, proof.clone(), &config).unwrap();
    (stark, proof, config)
}

pub fn ec_aggregate_main<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    points: Vec<[Fp; 2]>,
    res: [Fp; 2],
    bits: Vec<bool>,
) -> (
    ECCAggStark<F, D>,
    starky::proof::StarkProofWithPublicInputs<F, C, D>,
    StarkConfig,
) {
    let mut config = StarkConfig::standard_fast_config();
    config.fri_config.rate_bits = 2;
    let num_rows = 1 << log2_ceil((points.len() - 1) * 12);
    let stark = ECCAggStark::<F, D>::new(num_rows);
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
    assert_eq!(public_inputs.len(), ecc_aggregate::PUBLIC_INPUTS);
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
    (stark, proof, config)
}

pub fn recursive_proof<
    F: plonky2::hash::hash_types::RichField + plonky2::field::extension::Extendable<D>,
    C: GenericConfig<D, F = F>,
    S: starky::stark::Stark<F, D> + Copy,
    InnerC: GenericConfig<D, F = F>,
    const D: usize,
>(
    stark: S,
    inner_proof: starky::proof::StarkProofWithPublicInputs<F, InnerC, D>,
    inner_config: &StarkConfig,
    print_gate_counts: bool,
) -> ProofTuple<F, C, D>
where
    InnerC::Hasher: plonky2::plonk::config::AlgebraicHasher<F>,
{
    let circuit_config = plonky2::plonk::circuit_data::CircuitConfig::standard_recursion_config();
    let mut builder = plonky2::plonk::circuit_builder::CircuitBuilder::<F, D>::new(circuit_config);
    let mut pw = plonky2::iop::witness::PartialWitness::new();
    let degree_bits = inner_proof.proof.recover_degree_bits(inner_config);
    let pt = starky::recursive_verifier::add_virtual_stark_proof_with_pis(
        &mut builder,
        &stark,
        inner_config,
        degree_bits,
        0,
        0,
    );
    builder.register_public_inputs(&pt.public_inputs);
    let zero = builder.zero();
    starky::recursive_verifier::set_stark_proof_with_pis_target(&mut pw, &pt, &inner_proof, zero);
    starky::recursive_verifier::verify_stark_proof_circuit::<F, InnerC, S, D>(
        &mut builder,
        stark,
        pt,
        inner_config,
    );

    if print_gate_counts {
        builder.print_gate_counts(0);
    }

    let data = builder.build::<C>();
    let s = Instant::now();
    let proof = data.prove(pw).unwrap();
    println!("time taken for plonky2 recursive proof {:?}", s.elapsed());
    data.verify(proof.clone()).unwrap();
    (proof, data.verifier_only, data.common)
}

pub fn get_proof_public_inputs<
    F: plonky2::hash::hash_types::RichField + plonky2::field::extension::Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    proof: ProofTuple<F, C, D>,
) -> Vec<F> {
    proof.0.public_inputs
}

pub type ProofTuple<F, C, const D: usize> = (
    ProofWithPublicInputs<F, C, D>,
    VerifierOnlyCircuitData<C, D>,
    CommonCircuitData<F, D>,
);
