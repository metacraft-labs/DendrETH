use circuit::SerdeCircuitTarget;
use circuit_executables::cached_circuit_build::SERIALIZED_CIRCUITS_DIR;
use std::{fs, marker::PhantomData, str::FromStr, time::Instant};

use ark_bls12_381::G2Affine;
use ark_std::UniformRand;
use circuits::bls_verification::build_stark_proof_verifier::build_stark_proof_verifier;
use num_bigint::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use starky_bls12_381::{
    aggregate_proof::calc_pairing_precomp,
    calc_pairing_precomp::PairingPrecompStark,
    native::{Fp, Fp2},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;
type PpStark = PairingPrecompStark<F, D>;

fn main_thread() {
    let rng = &mut ark_std::rand::thread_rng();
    let g2 = G2Affine::rand(rng);

    println!("Starting Pairing precomp Proving");

    let s = Instant::now();

    let (stark_pp, proof_pp, config_pp) = calc_pairing_precomp::<F, C, D>(
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

    let s = Instant::now();

    let (recursive_stark_targets, data) =
        build_stark_proof_verifier::<GoldilocksField, PoseidonGoldilocksConfig, PpStark>(
            stark_pp, &proof_pp, &config_pp,
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

    let common_circuit_bytes = data.common.to_bytes(&CustomGateSerializer).unwrap();

    fs::write(
        format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp.plonky2_common_data"),
        &common_circuit_bytes,
    )
    .unwrap();

    fs::write(
        format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp.plonky2_circuit"),
        &circuit_bytes,
    )
    .unwrap();

    let targets = recursive_stark_targets.serialize().unwrap();

    fs::write(
        format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp.plonky2_targets"),
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
