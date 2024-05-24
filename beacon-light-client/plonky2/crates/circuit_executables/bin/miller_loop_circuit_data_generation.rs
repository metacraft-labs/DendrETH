use std::{fs, marker::PhantomData, str::FromStr, time::Instant};

use ark_bls12_381::{G1Affine, G2Affine};
use ark_std::UniformRand;
use circuit::SerdeCircuitTarget;
use circuit_executables::constants::SERIALIZED_CIRCUITS_DIR;
use circuits::bls_verification::build_stark_proof_verifier::build_stark_proof_verifier;
use num_bigint::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use starky_bls12_381::{
    aggregate_proof::miller_loop_main,
    miller_loop::MillerLoopStark,
    native::{Fp, Fp2},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;
type MlStark = MillerLoopStark<F, D>;

fn main_thread() {
    let rng = &mut ark_std::rand::thread_rng();
    let g1 = G1Affine::rand(rng);
    let g2 = G2Affine::rand(rng);

    println!("Starting Miller Loop Proving");

    let s = Instant::now();

    let (stark_ml, proof_ml, config_ml) = miller_loop_main::<F, C, D>(
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

    let s = Instant::now();

    let (recursive_stark_targets, data) =
        build_stark_proof_verifier::<GoldilocksField, PoseidonGoldilocksConfig, MlStark>(
            stark_ml, &proof_ml, &config_ml,
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
        format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop.plonky2_circuit"),
        &circuit_bytes,
    )
    .unwrap();

    let common_circuit_bytes = data.common.to_bytes(&CustomGateSerializer).unwrap();

    fs::write(
        format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop.plonky2_common_data"),
        &common_circuit_bytes,
    )
    .unwrap();

    let targets = recursive_stark_targets.serialize().unwrap();

    fs::write(
        format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop.plonky2_targets"),
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
