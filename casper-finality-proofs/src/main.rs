use std::{str::FromStr, time::Instant};

use ark_bls12_381::{Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use num_bigint::BigUint;
use plonky2::{
    iop::witness::PartialWitness,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};

use casper_finality_proofs::verification::{
    g1_ec_point::PointG1Target,
    g2_ec_point::PointG2Target,
    native::{Fp, Fp2},
    verify::{calc_ell_coeffs_and_generate_g2_point, verify_miller_loop},
};

fn main_thread() {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    use ark_std::UniformRand;

    let circuit_config = plonky2::plonk::circuit_data::CircuitConfig::standard_recursion_config();

    /* Test purposes */
    let rng = &mut ark_std::rand::thread_rng();
    let g1 = G1Affine::generator();
    let sk: Fr = Fr::rand(rng);
    let pk = Into::<G1Affine>::into(g1 * sk);
    let message = G2Affine::rand(rng);
    let signature = Into::<G2Affine>::into(message * sk);
    /* Test purposes */
    let first_ml_proof = verify_miller_loop(
        Fp::get_fp_from_biguint(g1.x.to_string().parse::<BigUint>().unwrap()),
        Fp::get_fp_from_biguint(g1.y.to_string().parse::<BigUint>().unwrap()),
        Fp2([
            Fp::get_fp_from_biguint(signature.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(signature.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(signature.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(signature.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    );

    // let second_ml_proof = verify_miller_loop(
    //     Fp::get_fp_from_biguint(pk.x.to_string().parse::<BigUint>().unwrap()),
    //     Fp::get_fp_from_biguint(pk.y.to_string().parse::<BigUint>().unwrap()),
    //     Fp2([
    //         Fp::get_fp_from_biguint(message.x.c0.to_string().parse::<BigUint>().unwrap()),
    //         Fp::get_fp_from_biguint(message.x.c1.to_string().parse::<BigUint>().unwrap()),
    //     ]),
    //     Fp2([
    //         Fp::get_fp_from_biguint(message.y.c0.to_string().parse::<BigUint>().unwrap()),
    //         Fp::get_fp_from_biguint(message.y.c1.to_string().parse::<BigUint>().unwrap()),
    //     ]),
    //     Fp2([
    //         Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
    //         Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
    //     ]),
    // );

    // G1 GENERATOR POINT
    // let g1_generator: PointG1Target = [
    //     builder.constant_biguint(&g1.x.to_string().parse::<BigUint>().unwrap()),
    //     builder.constant_biguint(&g1.y.to_string().parse::<BigUint>().unwrap()),
    // ];

    // SIGNATURE
    // let signature: PointG2Target = calc_ell_coeffs_and_generate_g2_point(&mut builder, signature);

    // // PUBLIC KEY
    // let public_key: PointG1Target = [
    //     builder.constant_biguint(&pk.x.to_string().parse::<BigUint>().unwrap()),
    //     builder.constant_biguint(&pk.y.to_string().parse::<BigUint>().unwrap()),
    // ];

    // // MESSAGE
    // let message: PointG2Target = calc_ell_coeffs_and_generate_g2_point(&mut builder, message);

    // verify_proofs(
    //     &mut builder,
    //     first_ml_proof,
    //     // second_ml_proof,
    //     &g1_generator,
    //     &signature,
    //     //&public_key,
    //     //&message,
    // );

    // let now = Instant::now();
    // let pw = PartialWitness::new();
    // let data = builder.build::<C>();
    // let _proof = data.prove(pw);
    // println!("time: {:?}", now.elapsed());
}

pub fn main() {
    std::thread::Builder::new()
        .spawn(|| {
            main_thread();
        })
        .unwrap()
        .join()
        .unwrap();
}
