use std::str::FromStr;

use ark_bls12_381::G2Affine;
use num_bigint::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::biguint::{BigUintTarget, CircuitBuilderBiguint},
        vars::ByteVariable,
    },
};

use crate::verification::{
    curves::{g1::PointG1Target, g2::PointG2Target},
    proofs::{
        final_exponentiate::FinalExponentiateStark,
        miller_loop::MillerLoopStark,
        proofs::{
            ec_aggregate_main, final_exponentiate_main, miller_loop_main, recursive_proof,
            ProofTuple,
        },
    },
    utils::native_bls::{calc_pairing_precomp, Fp, Fp12, Fp2},
};

use super::{aggregation::hash_to_curve::hash_to_curve, proofs::ecc_aggregate::ECCAggStark};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

type MlStark = MillerLoopStark<F, D>;
type FeStark = FinalExponentiateStark<F, D>;
type ECAggStark = ECCAggStark<F, D>;

pub fn verify_pubkeys_aggregation(
    points: Vec<[Fp; 2]>,
    res: [Fp; 2],
    bits: Vec<bool>,
) -> ProofTuple<F, C, D> {
    let (stark_ec, proof_ec, config_ec) = ec_aggregate_main::<F, C, D>(points, res, bits.clone());
    let recursive_ec =
        recursive_proof::<F, C, ECCAggStark<F, D>, C, D>(stark_ec, proof_ec, &config_ec, true);

    recursive_ec
}

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

fn fp12_as_biguint_target<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    f_inputs: Vec<F>,
    i: usize,
) -> Vec<BigUintTarget> {
    let mut f = Vec::new();
    let mut i = i;
    for _ in 0..12 {
        f.push(builder.api.constant_biguint(&BigUint::new(
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

pub fn calc_ell_coeffs_and_generate_g2_point<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
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
            builder
                .api
                .constant_biguint(&ell_coeffs[0][0].0[0].to_biguint()),
            builder
                .api
                .constant_biguint(&ell_coeffs[0][0].0[1].to_biguint()),
        ],
        [
            builder
                .api
                .constant_biguint(&ell_coeffs[0][1].0[0].to_biguint()),
            builder
                .api
                .constant_biguint(&ell_coeffs[0][1].0[1].to_biguint()),
        ],
    ]
}

pub fn verify_bls_signatures(
    builder: &mut CircuitBuilder<impl PlonkParameters<D>, D>,
    first_ml_proof: ProofTuple<F, C, D>,
    second_ml_proof: ProofTuple<F, C, D>,
    g1_generator: &PointG1Target,
    signature: &PointG2Target,
    public_key: &PointG1Target,
    hm_g2: &[ByteVariable],
) {
    let hm_g2 = hash_to_curve(builder, hm_g2);
    let first_ml_pub_inputs = first_ml_proof.0.public_inputs;
    let second_ml_pub_inputs = second_ml_proof.0.public_inputs;

    // FIRST MILLER LOOP
    let g1_x_input = builder.api.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[0..12]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g1_y_input = builder.api.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[12..24]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    let g2_x_input_c0 = builder.api.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[24..36]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_x_input_c1 = builder.api.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[36..48]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c0 = builder.api.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[48..60]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c1 = builder.api.constant_biguint(&BigUint::new(
        first_ml_pub_inputs[60..72]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    builder.api.connect_biguint(&g1_generator[0], &g1_x_input);
    builder.api.connect_biguint(&g1_generator[1], &g1_y_input);

    builder
        .api
        .connect_biguint(&signature[0][0], &g2_x_input_c0);
    builder
        .api
        .connect_biguint(&signature[0][1], &g2_x_input_c1);
    builder
        .api
        .connect_biguint(&signature[1][0], &g2_y_input_c0);
    builder
        .api
        .connect_biguint(&signature[1][1], &g2_y_input_c1);

    let first_ml_r = fp12_as_fp_limbs(first_ml_pub_inputs, 4920);
    let (_, proof_final_exp, _) = final_exponentiate_main::<F, C, D>(Fp12(
        vec_limbs_to_fixed_array::<Fp, 12>(first_ml_r.clone()),
    ));
    let first_fin_exp_pub_inputs = proof_final_exp.public_inputs;
    let first_fin_exp_pub_inputs = fp12_as_biguint_target(builder, first_fin_exp_pub_inputs, 144);

    // SECOND MILLER LOOP
    let g1_x_input = builder.api.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[0..12]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g1_y_input = builder.api.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[12..24]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    let g2_x_input_c0 = builder.api.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[24..36]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_x_input_c1 = builder.api.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[36..48]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c0 = builder.api.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[48..60]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));
    let g2_y_input_c1 = builder.api.constant_biguint(&BigUint::new(
        second_ml_pub_inputs[60..72]
            .iter()
            .map(|x| x.0 as u32)
            .collect(),
    ));

    builder.api.connect_biguint(&public_key[0], &g1_x_input);
    builder.api.connect_biguint(&public_key[1], &g1_y_input);

    builder.api.connect_biguint(&hm_g2[0][0], &g2_x_input_c0);
    builder.api.connect_biguint(&hm_g2[0][1], &g2_x_input_c1);
    builder.api.connect_biguint(&hm_g2[1][0], &g2_y_input_c0);
    builder.api.connect_biguint(&hm_g2[1][1], &g2_y_input_c1);

    let second_ml_r = fp12_as_fp_limbs(second_ml_pub_inputs.clone(), 4920);

    let (_, proof_final_exp, _) =
        final_exponentiate_main::<F, C, D>(Fp12(vec_limbs_to_fixed_array::<Fp, 12>(second_ml_r)));
    let second_fin_exp_pub_inputs = proof_final_exp.public_inputs;
    let second_fin_exp_pub_inputs = fp12_as_biguint_target(builder, second_fin_exp_pub_inputs, 144);

    for i in 0..12 {
        builder
            .api
            .connect_biguint(&first_fin_exp_pub_inputs[i], &second_fin_exp_pub_inputs[i]);
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
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::witness::PartialWitness,
        plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
    };
    use plonky2x::frontend::{
        builder::DefaultBuilder, uint::num::biguint::CircuitBuilderBiguint, vars::ByteVariable,
    };

    use super::{calc_ell_coeffs_and_generate_g2_point, verify_pubkeys_aggregation};
    use crate::verification::{
        curves::{
            g1::{g1_ecc_aggregate, PointG1Target},
            g2::PointG2Target,
        },
        proofs::miller_loop::MillerLoopStark,
        utils::native_bls::{Fp, Fp2},
        verify::verify_bls_signatures,
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type _MlStark = MillerLoopStark<F, D>;

    use super::verify_miller_loop;

    #[test]
    fn test_verify_proofs() {
        use jemallocator::Jemalloc;

        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
        let mut builder = DefaultBuilder::new();

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
            builder
                .api
                .constant_biguint(&g1.x.to_string().parse::<BigUint>().unwrap()),
            builder
                .api
                .constant_biguint(&g1.y.to_string().parse::<BigUint>().unwrap()),
        ];

        // SIGNATURE
        let signature: PointG2Target =
            calc_ell_coeffs_and_generate_g2_point(&mut builder, signature);

        // PUBLIC KEY
        let public_key: PointG1Target = [
            builder
                .api
                .constant_biguint(&pk.x.to_string().parse::<BigUint>().unwrap()),
            builder
                .api
                .constant_biguint(&pk.y.to_string().parse::<BigUint>().unwrap()),
        ];

        // MESSAGE
        // let message: PointG2Target = calc_ell_coeffs_and_generate_g2_point(&mut builder, message);

        // MESSAGE in bytes
        let message = [
            103, 140, 163, 210, 238, 252, 75, 8, 227, 27, 60, 229, 125, 150, 241, 222, 217, 156,
            178, 17, 14, 199, 15, 172, 94, 179, 249, 0, 197, 206, 104, 200, 165, 253, 55, 147, 171,
            191, 118, 189, 133, 138, 2, 22, 237, 6, 62, 10, 68, 105, 208, 102, 66, 70, 170, 114,
            194, 80, 215, 5, 63, 95, 202, 1, 99, 153, 67, 115, 7, 122, 235, 255, 142, 44, 3, 65,
            190, 166, 218, 72, 230, 196, 24, 88, 146, 193, 211, 90, 37, 173, 71, 152, 21, 226, 89,
            79, 239, 81, 149, 135, 188, 51, 52, 116, 26, 30, 126, 31, 35, 240, 201, 101, 33, 61,
            220, 192, 86, 47, 214, 243, 224, 136, 50, 56, 42, 233, 148, 244, 203, 198, 195, 120,
            36, 221, 181, 53, 160, 58, 167, 131, 216, 183, 83, 232, 151, 87, 46, 54, 128, 123, 231,
            212, 130, 19, 28, 96, 108, 111, 137, 154, 40, 184, 74, 69, 100, 64, 177, 98, 248, 32,
            12, 97, 49, 187, 39, 159, 168, 247, 29, 246, 209, 110, 77, 73, 20, 23, 174, 143, 93,
            92, 162, 48, 134, 119, 213, 139, 234, 205, 91, 113, 204, 121, 57, 4, 41, 180, 144, 76,
            107, 59, 176, 43, 11, 127, 34, 38, 164, 9, 141, 78, 245, 175, 145, 112, 129, 109, 18,
            250, 85, 16, 124, 182, 242, 158, 84, 219, 13, 207, 186, 82, 157, 132, 225, 236, 45,
            185, 228, 161, 169, 106, 25, 155, 251, 254, 223,
        ]
        .iter()
        .map(|b| {
            let b_v = builder.constant(GoldilocksField::from_canonical_u8(*b));
            ByteVariable::from_variable(&mut builder, b_v)
        })
        .collect::<Vec<ByteVariable>>();

        verify_bls_signatures(
            &mut builder,
            first_ml_proof,
            second_ml_proof,
            &g1_generator,
            &signature,
            &public_key,
            &message,
        );

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);
    }

    #[test]
    fn test_pubkeys_aggregation() {
        let circuit_config =
            plonky2::plonk::circuit_data::CircuitConfig::standard_recursion_config();
        let mut builder =
            plonky2::plonk::circuit_builder::CircuitBuilder::<F, D>::new(circuit_config);

        let a_bigu = BigUint::from_str(
                "1216495682195235861952885506871698490232894470117269383940381148575524314493849307811227440691167647909822763414941"
            ).unwrap();
        let b_bigu = BigUint::from_str(
                "2153848155426317245700560287567131132765685008362732985860101000686875894603366983854567186180519945327668975076337"
            ).unwrap();

        let a_fp = Fp::get_fp_from_biguint(a_bigu.clone());
        let b_fp = Fp::get_fp_from_biguint(b_bigu.clone());

        let a_bigu_t = builder.constant_biguint(&a_bigu);
        let b_bigu_t = builder.constant_biguint(&b_bigu);

        let ec_proof = verify_pubkeys_aggregation(
            vec![[a_fp, b_fp], [a_fp, b_fp]],
            [a_fp, b_fp],
            vec![true, false],
        );
        let point = [a_bigu_t, b_bigu_t];
        g1_ecc_aggregate(&mut builder, point.clone(), point);

        // If we are going to check the pubkey ec point
        // let ec_proof_pub_inputs = ec_proof.0.public_inputs;

        // //
        // let g1_pk_point_x_input = builder.constant_biguint(&BigUint::new(
        //     ec_proof_pub_inputs[0..12]
        //         .iter()
        //         .map(|x| x.0 as u32)
        //         .collect(),
        // ));
        // let g1_pk_point_y_input = builder.constant_biguint(&BigUint::new(
        //     ec_proof_pub_inputs[12..24]
        //         .iter()
        //         .map(|x| x.0 as u32)
        //         .collect(),
        // ));
    }
}
