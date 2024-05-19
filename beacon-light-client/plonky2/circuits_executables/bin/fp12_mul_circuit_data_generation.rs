use std::{fs, marker::PhantomData, time::Instant};

use ark_std::UniformRand;
use circuits::{
    build_stark_proof_verifier::build_stark_proof_verifier, targets_serialization::WriteTargets,
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use starky_bls12_381::{
    aggregate_proof::fp12_mul_main,
    fp12_mul::FP12MulStark,
    native::{Fp, Fp12},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

type Fp12MulStark = FP12MulStark<F, D>;

fn main_thread() {
    let rng = &mut ark_std::rand::thread_rng();

    let fq = ark_bls12_381::Fq12::rand(rng);

    let fp12_1 = Fp12([
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

    let fq = ark_bls12_381::Fq12::rand(rng);

    let fp12_2 = Fp12([
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

    let s = Instant::now();
    println!("Starting FP12 Mul Proving");
    let (stark_fp12_mul, proof_fp12_mul, config_fp12_mul) =
        fp12_mul_main::<F, C, D>(fp12_1, fp12_2);
    println!("FP12 Mul Proving Done {:?}", s.elapsed());

    let s = Instant::now();

    let (recursive_stark_targets, data) = build_stark_proof_verifier::<
        GoldilocksField,
        PoseidonGoldilocksConfig,
        Fp12MulStark,
    >(stark_fp12_mul, &proof_fp12_mul, &config_fp12_mul);

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

    fs::write("circuits/fp12_mul.plonky2_circuit", &circuit_bytes).unwrap();

    let common_data_bytes = data.common.to_bytes(&CustomGateSerializer).unwrap();

    fs::write("circuits/fp12_mul.plonky2_common_data", &common_data_bytes).unwrap();

    let targets = recursive_stark_targets.write_targets().unwrap();

    fs::write("circuits/fp12_mul.plonky2_targets", &targets).unwrap();

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
