use std::{str::FromStr, time::Instant};

use ark_bls12_381::{G1Affine, G2Affine};
use circuits::{
    bls_verification::build_stark_proof_verifier::RecursiveStarkTargets,
    common_targets::BasicRecursiveInnerCircuitTarget,
};
use num_bigint::BigUint;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_data::CircuitData,
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputs,
    },
};

use anyhow::Result;
use starky_bls12_381::{
    aggregate_proof::{
        calc_pairing_precomp, final_exponentiate_main, fp12_mul_main, miller_loop_main,
    },
    native::{Fp, Fp12, Fp2},
};

pub fn prove_inner_level(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_target: &BasicRecursiveInnerCircuitTarget,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    let inner_proof1 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof1_bytes,
            &circuit_data.common,
        )?;

    let inner_proof2 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof2_bytes,
            &circuit_data.common,
        )?;

    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&inner_circuit_target.proof1, &inner_proof1);
    pw.set_proof_with_pis_target(&inner_circuit_target.proof2, &inner_proof2);

    Ok(inner_circuit_data.prove(pw)?)
}

// TODO: Don't hard code the D
pub fn prove_inner_level2<F: RichField + Extendable<2>, C: GenericConfig<2, F = F>>(
    proof1: &ProofWithPublicInputs<F, C, 2>,
    proof2: &ProofWithPublicInputs<F, C, 2>,
    inner_circuit_target: &BasicRecursiveInnerCircuitTarget,
    inner_circuit_data: &CircuitData<F, C, 2>,
) -> Result<ProofWithPublicInputs<F, C, 2>>
where
    <C as GenericConfig<2>>::Hasher: AlgebraicHasher<F>,
{
    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&inner_circuit_target.proof1, proof1);
    pw.set_proof_with_pis_target(&inner_circuit_target.proof2, proof2);

    Ok(inner_circuit_data.prove(pw)?)
}

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub fn generate_pairing_precomp_proof(
    g2: &G2Affine,
    pairing_precomp_targets: &RecursiveStarkTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    println!("Starting Pairing precomp starky proof");

    let s = Instant::now();
    let (_, proof_pp, _) = calc_pairing_precomp::<F, C, D>(
        Fp2([
            Fp::get_fp_from_biguint(g2.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(g2.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    );

    println!("Pairing precomp starky proof done {:?}", s.elapsed());

    let mut pw = PartialWitness::new();
    starky::recursive_verifier::set_stark_proof_with_pis_target(
        &mut pw,
        &pairing_precomp_targets.proof,
        &proof_pp,
        pairing_precomp_targets.zero,
    );

    println!("Starting to generate plonky2 proof");
    let s = Instant::now();
    let proof = circuit_data.prove(pw).unwrap();
    println!("time taken for plonky2 recursive proof {:?}", s.elapsed());

    proof
}

pub fn generate_miller_loop_proof(
    g1: &G1Affine,
    g2: &G2Affine,
    miller_loop_targets: &RecursiveStarkTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    println!("Starting Miller Loop Proving");

    let s = Instant::now();

    let (_, proof_ml, _) = miller_loop_main::<F, C, D>(
        Fp::get_fp_from_biguint(g1.x.to_string().parse::<BigUint>().unwrap()),
        Fp::get_fp_from_biguint(g1.y.to_string().parse::<BigUint>().unwrap()),
        Fp2([
            Fp::get_fp_from_biguint(g2.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(g2.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    );

    println!("Miller Loop Proving Done {:?}", s.elapsed());

    let mut pw = PartialWitness::new();
    starky::recursive_verifier::set_stark_proof_with_pis_target(
        &mut pw,
        &miller_loop_targets.proof,
        &proof_ml,
        miller_loop_targets.zero,
    );

    println!("Starting to generate proof");

    let s = Instant::now();
    let proof = circuit_data.prove(pw).unwrap();
    println!("time taken for plonky2 recursive proof {:?}", s.elapsed());

    proof
}

pub fn generate_fp12_mul_proof(
    miller_loop1: &Fp12,
    miller_loop2: &Fp12,
    fp12_mul_targets: &RecursiveStarkTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    let s = Instant::now();
    println!("Starting FP12 Mul Proving");
    let (_, proof_fp12_mul, _) = fp12_mul_main::<F, C, D>(*miller_loop1, *miller_loop2);
    println!("FP12 Mul Proving Done {:?}", s.elapsed());

    let mut pw = PartialWitness::new();
    starky::recursive_verifier::set_stark_proof_with_pis_target(
        &mut pw,
        &fp12_mul_targets.proof,
        &proof_fp12_mul,
        fp12_mul_targets.zero,
    );

    println!("Starting to generate proof");

    let s = Instant::now();
    let proof = circuit_data.prove(pw).unwrap();
    println!("time taken for plonky2 recursive proof {:?}", s.elapsed());

    proof
}

pub fn generate_final_exponentiate(
    fp12_mul: &Fp12,
    final_exponentiate_targets: &RecursiveStarkTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    println!("Final exponetiate stark proof started");

    let s = Instant::now();

    let (_, proof_final_exp, _) = final_exponentiate_main::<F, C, D>(*fp12_mul);

    println!("Final exponetiate stark proof done in {:?}", s.elapsed());

    let mut pw = PartialWitness::new();
    starky::recursive_verifier::set_stark_proof_with_pis_target(
        &mut pw,
        &final_exponentiate_targets.proof,
        &proof_final_exp,
        final_exponentiate_targets.zero,
    );

    println!("Starting to generate proof");

    let s = Instant::now();
    let proof = circuit_data.prove(pw).unwrap();
    println!("time taken for plonky2 recursive proof {:?}", s.elapsed());

    proof
}
