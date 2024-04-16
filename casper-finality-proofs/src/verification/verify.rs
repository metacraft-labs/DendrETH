use std::str::FromStr;

use ark_bls12_381::G2Affine;
use num_bigint::BigUint;
use plonky2::plonk::{
    circuit_builder::CircuitBuilder,
    config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2x::frontend::uint::num::biguint::{BigUintTarget, CircuitBuilderBiguint};

use crate::verification::native::{calc_pairing_precomp, Fp, Fp12, Fp2};

use super::{
    final_exponentiate::FinalExponentiateStark,
    g1_ec_point::PointG1Target,
    g2_ec_point::PointG2Target,
    miller_loop::MillerLoopStark,
    proofs::{final_exponentiate_main, miller_loop_main, recursive_proof, ProofTuple},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

type MlStark = MillerLoopStark<F, D>;
type FeStark = FinalExponentiateStark<F, D>;

// pub fn calculate_signature<F: RichField + Extendable<D>, const D: usize>(
//     builder: &mut CircuitBuilder<F, D>,
//     msg: &[Target],
//     secret_key: &Fp2Target,
// ) -> PointG2Target {
//     let hm_as_g2_point = hash_to_curve(builder, msg);
//     let signature = g2_scalar_mul(builder, &hm_as_g2_point, secret_key);

//     signature
// }

pub fn verify_miller_loop(x: Fp, y: Fp, q_x: Fp2, q_y: Fp2, q_z: Fp2) -> ProofTuple<F, C, D> {
    let (stark_ml, proof_ml, config_ml) = miller_loop_main::<F, C, D>(x, y, q_x, q_y, q_z);
    let recursive_ml = recursive_proof::<F, C, MlStark, C, D>(stark_ml, proof_ml, &config_ml, true);

    recursive_ml
}

pub fn verify_final_exponentiation(f: Fp12) -> ProofTuple<F, C, D> {
    let (stark_final_exp, proof_final_exp, config_final_exp) =
        final_exponentiate_main::<F, C, D>(f);
    let recursive_final_exp = recursive_proof::<F, C, FeStark, C, D>(
        stark_final_exp,
        proof_final_exp,
        &config_final_exp,
        true,
    );

    recursive_final_exp
}

fn fp12_as_biguint_target(
    builder: &mut CircuitBuilder<F, D>,
    f_inputs: Vec<F>,
    i: usize,
) -> Vec<BigUintTarget> {
    let mut f = Vec::new();
    let mut i = i;
    for _ in 0..12 {
        f.push(builder.constant_biguint(&BigUint::new(
            f_inputs[i..i + 12].iter().map(|x| x.0 as u32).collect(),
        )));
        i += 12;
    }

    f
}

fn fp12_as_fp_limbs(f_inputs: Vec<F>, i: usize) -> Vec<Fp> {
    let mut f = Vec::new();
    let mut i = i;
    for _ in 0..12 {
        f.push(Fp::get_fp_from_biguint(BigUint::new(
            f_inputs[i..i + 12].iter().map(|x| x.0 as u32).collect(),
        )));
        i += 12;
    }

    f
}

fn vec_limbs_to_fixed_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn calc_ell_coeffs_and_generate_g2_point(
    builder: &mut CircuitBuilder<F, D>,
    g2_point: G2Affine,
) -> PointG2Target {
    let ell_coeffs = calc_pairing_precomp(
        Fp2([
            Fp::get_fp_from_biguint(g2_point.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2_point.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(g2_point.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2_point.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    );

    [
        [
            builder.constant_biguint(&ell_coeffs[0][0].0[0].to_biguint()),
            builder.constant_biguint(&ell_coeffs[0][0].0[1].to_biguint()),
        ],
        [
            builder.constant_biguint(&ell_coeffs[0][1].0[0].to_biguint()),
            builder.constant_biguint(&ell_coeffs[0][1].0[1].to_biguint()),
        ],
    ]
}

pub fn verify_proofs(
    builder: &mut CircuitBuilder<F, D>,
    first_ml_proof: ProofTuple<F, C, D>,
    second_ml_proof: ProofTuple<F, C, D>,
    g1_generator: &PointG1Target,
    signature: &PointG2Target,
    public_key: &PointG1Target,
    hm_g2: &PointG2Target,
) {
    let first_ml_pub_inputs = first_ml_proof.0.public_inputs;
    let second_ml_pub_inputs = second_ml_proof.0.public_inputs;

    // FIRST MILLER LOOP
    let g1_x_input = builder.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[0..12]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g1_y_input = builder.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[12..24]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    let g2_x_input_c0 = builder.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[24..36]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_x_input_c1 = builder.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[36..48]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c0 = builder.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[48..60]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c1 = builder.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[60..72]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    builder.connect_biguint(&g1_generator[0], &g1_x_input);
    builder.connect_biguint(&g1_generator[1], &g1_y_input);

    builder.connect_biguint(&signature[0][0], &g2_x_input_c0);
    builder.connect_biguint(&signature[0][1], &g2_x_input_c1);
    builder.connect_biguint(&signature[1][0], &g2_y_input_c0);
    builder.connect_biguint(&signature[1][1], &g2_y_input_c1);

    let first_ml_r = fp12_as_fp_limbs(first_ml_pub_inputs, 4920);
    let (_, proof_final_exp, _) = final_exponentiate_main::<F, C, D>(Fp12(
        vec_limbs_to_fixed_array::<Fp, 12>(first_ml_r.clone()),
    ));
    let first_fin_exp_pub_inputs = proof_final_exp.public_inputs;
    let first_fin_exp_pub_inputs = fp12_as_biguint_target(builder, first_fin_exp_pub_inputs, 144);

    // SECOND MILLER LOOP
    let g1_x_input = builder.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[0..12]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g1_y_input = builder.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[12..24]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    let g2_x_input_c0 = builder.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[24..36]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_x_input_c1 = builder.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[36..48]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c0 = builder.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[48..60]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c1 = builder.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[60..72]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    builder.connect_biguint(&public_key[0], &g1_x_input);
    builder.connect_biguint(&public_key[1], &g1_y_input);

    builder.connect_biguint(&hm_g2[0][0], &g2_x_input_c0);
    builder.connect_biguint(&hm_g2[0][1], &g2_x_input_c1);
    builder.connect_biguint(&hm_g2[1][0], &g2_y_input_c0);
    builder.connect_biguint(&hm_g2[1][1], &g2_y_input_c1);

    let second_ml_r = fp12_as_fp_limbs(second_ml_pub_inputs.clone(), 4920);

    let (_, proof_final_exp, _) =
        final_exponentiate_main::<F, C, D>(Fp12(vec_limbs_to_fixed_array::<Fp, 12>(second_ml_r)));
    let second_fin_exp_pub_inputs = proof_final_exp.public_inputs;
    let second_fin_exp_pub_inputs = fp12_as_biguint_target(builder, second_fin_exp_pub_inputs, 144);

    for i in 0..12 {
        builder.connect_biguint(&first_fin_exp_pub_inputs[i], &second_fin_exp_pub_inputs[i]);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ark_bls12_381::{Fr, G1Affine, G2Affine};
    use ark_ec::AffineRepr;
    use ark_std::UniformRand;
    use num_bigint::BigUint;
    use plonky2::{
        iop::witness::PartialWitness,
        plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
    };
    use plonky2x::frontend::uint::num::biguint::CircuitBuilderBiguint;

    use crate::verification::{
        g1_ec_point::PointG1Target,
        g2_ec_point::PointG2Target,
        miller_loop::MillerLoopStark,
        native::{Fp, Fp2},
        verify::calc_ell_coeffs_and_generate_g2_point,
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type _MlStark = MillerLoopStark<F, D>;

    use super::{verify_miller_loop, verify_proofs};

    #[test]
    fn test_verify_proofs() {
        let circuit_config =
            plonky2::plonk::circuit_data::CircuitConfig::standard_recursion_config();
        let mut builder =
            plonky2::plonk::circuit_builder::CircuitBuilder::<F, D>::new(circuit_config);

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

        let second_ml_proof = verify_miller_loop(
            Fp::get_fp_from_biguint(pk.x.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(pk.y.to_string().parse::<BigUint>().unwrap()),
            Fp2([
                Fp::get_fp_from_biguint(message.x.c0.to_string().parse::<BigUint>().unwrap()),
                Fp::get_fp_from_biguint(message.x.c1.to_string().parse::<BigUint>().unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(message.y.c0.to_string().parse::<BigUint>().unwrap()),
                Fp::get_fp_from_biguint(message.y.c1.to_string().parse::<BigUint>().unwrap()),
            ]),
            Fp2([
                Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
                Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
            ]),
        );

        // G1 GENERATOR POINT
        let g1_generator: PointG1Target = [
            builder.constant_biguint(&g1.x.to_string().parse::<BigUint>().unwrap()),
            builder.constant_biguint(&g1.y.to_string().parse::<BigUint>().unwrap()),
        ];

        // SIGNATURE
        let signature: PointG2Target =
            calc_ell_coeffs_and_generate_g2_point(&mut builder, signature);

        // PUBLIC KEY
        let public_key: PointG1Target = [
            builder.constant_biguint(&pk.x.to_string().parse::<BigUint>().unwrap()),
            builder.constant_biguint(&pk.y.to_string().parse::<BigUint>().unwrap()),
        ];

        // MESSAGE
        let message: PointG2Target = calc_ell_coeffs_and_generate_g2_point(&mut builder, message);

        verify_proofs(
            // verify or verify_bls_signature
            &mut builder,
            first_ml_proof,
            second_ml_proof,
            &g1_generator,
            &signature,
            &public_key,
            &message,
        );

        let pw = PartialWitness::new();
        let data = builder.build::<C>();
        let _proof = data.prove(pw);
    }
}
