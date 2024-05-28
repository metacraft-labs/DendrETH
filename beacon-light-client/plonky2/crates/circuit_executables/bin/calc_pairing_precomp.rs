use std::{marker::PhantomData, str::FromStr, time::Instant};

use ark_bls12_381::G2Affine;
use ark_std::UniformRand;
use circuit::SerdeCircuitTarget;
use circuit_executables::{constants::SERIALIZED_CIRCUITS_DIR, crud::common::read_from_file};
use circuits::bls_verification::build_stark_proof_verifier::RecursiveStarkTargets;
use num_bigint::BigUint;
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
    aggregate_proof::calc_pairing_precomp,
    native::{Fp, Fp2},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

const CIRCUIT_NAME: &str = "pairing_precomp";

fn main_thread() {
    println!("Starting to deserialize circuit");

    let circuit_data_bytes = read_from_file(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/{CIRCUIT_NAME}.plonky2_circuit"
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

    let targets = get_targets().unwrap();

    println!("Deserialized circuit");
    let rng = &mut ark_std::rand::thread_rng();
    let g2 = G2Affine::rand(rng);

    println!("Starting Pairing precomp Proving");

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

    println!("Pairing precomp Done {:?}", s.elapsed());

    let mut pw = PartialWitness::new();
    starky::recursive_verifier::set_stark_proof_with_pis_target(
        &mut pw,
        &targets.proof,
        &proof_pp,
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

fn get_targets() -> Result<RecursiveStarkTargets, anyhow::Error> {
    let target_bytes = read_from_file(&format!(
        "{}/{}.plonky2_targets",
        SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME
    ))?;

    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(RecursiveStarkTargets::deserialize(&mut target_buffer).unwrap())
}
