use circuit::SerdeCircuitTarget;
use std::{marker::PhantomData, time::Instant};

use ark_std::UniformRand;
use circuit_executables::{crud::common::read_from_file, utils::CommandLineOptionsBuilder};
use circuits::bls_verification::build_stark_proof_verifier::RecursiveStarkTargets;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{
        circuit_data::CircuitData,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
    util::serialization::Buffer,
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use starky_bls12_381::{
    aggregate_proof::final_exponentiate_main,
    native::{Fp, Fp12},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

const CIRCUIT_NAME: &str = "final_exponentiate_circuit";

fn main_thread() {
    let matches = CommandLineOptionsBuilder::new("final_exponentiate")
        .with_serialized_circuits_dir()
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    println!("Starting to deserialize circuit");

    let circuit_data_bytes = read_from_file(&format!(
        "{serialized_circuits_dir}/{CIRCUIT_NAME}.plonky2_circuit"
    ))
    .unwrap();

    let circuit_data = CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        &circuit_data_bytes,
        &CustomGateSerializer,
        &CustomGeneratorSerializer {
            _phantom: PhantomData::<PoseidonGoldilocksConfig>,
        },
    )
    .unwrap();

    let targets = get_targets(serialized_circuits_dir).unwrap();

    println!("Deserialized circuit");

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

    let (_, proof_final_exp, _) = final_exponentiate_main::<F, C, D>(fp12);

    println!("Final exponetiate stark proof done in {:?}", s.elapsed());

    let mut pw = PartialWitness::new();
    starky::recursive_verifier::set_stark_proof_with_pis_target(
        &mut pw,
        &targets.proof,
        &proof_final_exp,
        targets.zero,
    );

    println!("Starting to generate proof");

    let s = Instant::now();
    let proof = circuit_data.prove(pw).unwrap();
    println!("time taken for plonky2 recursive proof {:?}", s.elapsed());

    let _ = circuit_data.verify(proof.clone());
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

fn get_targets(dir: &str) -> Result<RecursiveStarkTargets, anyhow::Error> {
    let target_bytes = read_from_file(&format!("{dir}/{CIRCUIT_NAME}.plonky2_targets",))?;

    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(RecursiveStarkTargets::deserialize(&mut target_buffer).unwrap())
}
