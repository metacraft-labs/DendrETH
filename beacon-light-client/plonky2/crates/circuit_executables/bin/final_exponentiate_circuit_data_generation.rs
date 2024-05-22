use std::{fs, marker::PhantomData, time::Instant};

use ark_std::UniformRand;
use circuits::{
    build_stark_proof_verifier::build_stark_proof_verifier,
    serialization::targets_serialization::WriteTargets,
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use starky::{config::StarkConfig, proof::StarkProofWithPublicInputs};
use starky_bls12_381::{
    aggregate_proof::final_exponentiate_main,
    final_exponentiate::FinalExponentiateStark,
    native::{Fp, Fp12},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;
type FeStark = FinalExponentiateStark<F, D>;

fn main_thread() {
    // let (validators_balance_verification_targets, first_level_data) = build_final_exponentiate();
    let rng = &mut ark_std::rand::thread_rng();

    let fq = ark_bls12_381::Fq12::rand(rng);

    let fp12 = Fp12([
        Fp::get_fp_from_biguint(fq.c0.c0.c0.0.into()),
        Fp::get_fp_from_biguint(fq.c0.c0.c1.0.into()),
        Fp::get_fp_from_biguint(fq.c0.c1.c0.0.into()),
        Fp::get_fp_from_biguint(fq.c0.c1.c1.0.into()),
        Fp::get_fp_from_biguint(fq.c0.c2.c0.0.into()),
        Fp::get_fp_from_biguint(fq.c0.c2.c1.0.into()),
        Fp::get_fp_from_biguint(fq.c1.c0.c0.0.into()),
        Fp::get_fp_from_biguint(fq.c1.c0.c1.0.into()),
        Fp::get_fp_from_biguint(fq.c1.c1.c0.0.into()),
        Fp::get_fp_from_biguint(fq.c1.c1.c1.0.into()),
        Fp::get_fp_from_biguint(fq.c1.c2.c0.0.into()),
        Fp::get_fp_from_biguint(fq.c1.c2.c1.0.into()),
    ]);

    println!("Final exponetiate stark proof started");

    let s = Instant::now();

    let (stark_final_exp, proof_final_exp, config_final_exp): (
        FeStark,
        StarkProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
        StarkConfig,
    ) = final_exponentiate_main::<F, C, D>(fp12);

    println!("Final exponetiate stark proof done in {:?}", s.elapsed());

    let s = Instant::now();

    let (recursive_stark_targets, data) =
        build_stark_proof_verifier::<GoldilocksField, PoseidonGoldilocksConfig, FeStark>(
            stark_final_exp,
            &proof_final_exp,
            &config_final_exp,
        );

    println!(
        "time taken for building plonky2 recursive circuit data {:?}",
        s.elapsed()
    );

    println!("Starting serialization");

    let s = Instant::now();

    let circuit_bytes = data
        .to_bytes(
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<PoseidonGoldilocksConfig>,
            },
        )
        .unwrap();

    fs::write(
        "circuits/final_exponentiate_circuit.plonky2_circuit",
        &circuit_bytes,
    )
    .unwrap();

    let common_circuit_bytes = data.common.to_bytes(&CustomGateSerializer).unwrap();

    fs::write(
        "circuits/final_exponentiate_circuit.plonky2_common_data",
        &common_circuit_bytes,
    )
    .unwrap();

    let targets = recursive_stark_targets.write_targets().unwrap();

    fs::write(
        "circuits/final_exponentiate_circuit.plonky2_targets",
        &targets,
    )
    .unwrap();

    println!("time taken for serialization {:?}", s.elapsed());
}

fn main() {
    std::thread::Builder::new()
        .spawn(|| {
            main_thread();
        })
        .unwrap()
        .join()
        .unwrap();
}
