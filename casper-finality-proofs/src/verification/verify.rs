use std::str::FromStr;

use ark_bls12_381::G2Affine;
use itertools::Itertools;
use num_bigint::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, packed::PackedField, types::Field64},
    plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{BigUintTarget, CircuitBuilderBiguint},
            u32::gadgets::arithmetic_u32::CircuitBuilderU32,
        },
        vars::{BoolVariable, ByteVariable, BytesVariable},
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

use super::proofs::{
    calc_pairing_precomp::PairingPrecompStark, ecc_aggregate::ECCAggStark,
    proofs::calc_pairing_precomp_proof,
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

type MlStark = MillerLoopStark<F, D>;
type FeStark = FinalExponentiateStark<F, D>;
type ECAggStark = ECCAggStark<F, D>;
type PPStark = PairingPrecompStark<F, D>;

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

pub fn verify_calc_precompire(q_x: Fp2, q_y: Fp2, q_z: Fp2) -> ProofTuple<F, C, D> {
    let (stark_pp, proof_pp, config_pp) = calc_pairing_precomp_proof::<F, C, D>(q_x, q_y, q_z);
    let recursive_pp = recursive_proof::<F, C, PPStark, C, D>(stark_pp, proof_pp, &config_pp, true);

    recursive_pp
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
    first_ml_proof: &ProofTuple<F, C, D>,
    second_ml_proof: &ProofTuple<F, C, D>,
    g1_generator: &PointG1Target,
    signature: &PointG2Target,
    public_key: &PointG1Target,
    message: &PointG2Target,
) {
    let first_ml_pub_inputs = &first_ml_proof.0.public_inputs;
    let second_ml_pub_inputs = &second_ml_proof.0.public_inputs;

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

    let first_ml_r = fp12_as_fp_limbs(first_ml_pub_inputs.to_vec(), 4920);
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

    builder.api.connect_biguint(&message[0][0], &g2_x_input_c0);
    builder.api.connect_biguint(&message[0][1], &g2_x_input_c1);
    builder.api.connect_biguint(&message[1][0], &g2_y_input_c0);
    builder.api.connect_biguint(&message[1][1], &g2_y_input_c1);

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

pub fn generate_biguint_target_from_f<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits_f: Vec<GoldilocksField>,
) -> BigUintTarget {
    println!("bits_f.len(): {:?}", bits_f.len());
    builder.api.constant_biguint(&BigUint::new(
        bits_f
            .iter()
            .map(|x| (x.0 % GoldilocksField::ORDER) as u32)
            .collect(),
    ))
}

pub fn generate_bits_from_g2_limbs<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits_f: Vec<GoldilocksField>,
) -> Vec<Vec<BoolVariable>> {
    let curr_bigu_t = generate_biguint_target_from_f(builder, bits_f); // abstract in one fn
    let mut res: Vec<Vec<BoolVariable>> = Vec::new();
    let bits_of_all_limbs: Vec<[BoolVariable; 32]> = curr_bigu_t
        .limbs
        .iter()
        .map(|f| (builder.api.u32_to_bits_le(*f)).map(|f| BoolVariable::from(f)))
        .collect();

    for bits_of_limb in bits_of_all_limbs {
        res.push(bits_of_limb.to_vec());
    }

    res.clone()
}

pub fn transform_bits_of_biguint_target_in_bytes(
    limbs_in_bits: Vec<Vec<BoolVariable>>,
) -> Vec<ByteVariable> {
    limbs_in_bits
        .iter()
        .flat_map(|limb| {
            limb.iter()
                .copied()
                .chunks(8)
                .into_iter()
                .map(|byte_bits| {
                    let x = ByteVariable(
                        byte_bits
                            .collect_vec()
                            .into_iter()
                            .rev()
                            .collect_vec()
                            .try_into()
                            .unwrap(),
                    );
                    x
                })
                .collect_vec()
        })
        .collect_vec()
}

// 0 - 24
// 24 - 48
// 48 - 72
//
pub fn calculate_bytes_g2_from_public_inputs<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits_f: Vec<GoldilocksField>,
) -> Vec<ByteVariable> {
    let reversed_bits_of_public_inputs = bits_f.iter().map(|f| *f).rev().collect_vec();
    let bits = generate_bits_from_g2_limbs(builder, reversed_bits_of_public_inputs); // returns passed Fp in LE bits
    transform_bits_of_biguint_target_in_bytes(bits)
}

pub fn compute_public_inputs_from_pairing_precompire<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits_f: Vec<GoldilocksField>,
) -> Vec<ByteVariable> {
    let mut final_bytes: Vec<ByteVariable> = Vec::new();
    let g2_x_c0 = calculate_bytes_g2_from_public_inputs(builder, bits_f[72..84].to_vec());
    let g2_x_c1 = calculate_bytes_g2_from_public_inputs(builder, bits_f[84..96].to_vec());
    let g2_y_c0 = calculate_bytes_g2_from_public_inputs(builder, bits_f[96..108].to_vec());
    let g2_y_c1 = calculate_bytes_g2_from_public_inputs(builder, bits_f[108..120].to_vec());

    final_bytes.extend(g2_x_c0);
    final_bytes.extend(g2_x_c1);
    final_bytes.extend(g2_y_c0);
    final_bytes.extend(g2_y_c1);

    final_bytes
}

// pub fn from_field_elements_to_bytevariable_slice(
//     field_vec: Vec<GoldilocksField>,
// ) -> Vec<ByteVariable> {
//     let z: Vec<ByteVariable> = field_vec
//         .iter()
//         .flat_map(|limb| {
//             limb.as_slice()
//                 .iter()
//                 .copied()
//                 .chunks(12)
//                 .into_iter()
//                 .map(|byte_bits| ByteVariable(byte_bits.collect_vec().try_into().unwrap()))
//                 .collect_vec()
//         })
//         .collect_vec();

//     z
// }

///
///
///
///
///

pub fn compute_pairing_precompile(g2_curve_point: G2Affine) -> ProofTuple<F, C, D> {
    verify_calc_precompire(
        Fp2([
            Fp::get_fp_from_biguint(g2_curve_point.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2_curve_point.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(g2_curve_point.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2_curve_point.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    )
}
#[cfg(test)]
mod tests {
    use std::{str::FromStr, time::Instant};

    use ark_bls12_381::{Fr, G1Affine, G2Affine};
    use ark_ec::AffineRepr;
    use ark_std::UniformRand;
    use num_bigint::BigUint;
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        plonk::config::{GenericConfig, PoseidonGoldilocksConfig},
    };
    use plonky2x::frontend::{
        builder::DefaultBuilder, uint::num::biguint::CircuitBuilderBiguint, vars::ByteVariable,
    };

    use super::{calc_ell_coeffs_and_generate_g2_point, compute_pairing_precompile};
    use crate::verification::{
        aggregation::hash_to_curve::hash_to_curve,
        curves::{
            g1::{generate_new_g1_point_target, PointG1Target},
            g2::PointG2Target,
        },
        proofs::{
            miller_loop::MillerLoopStark,
            proofs::{get_proof_public_inputs, ProofTuple},
        },
        utils::native_bls::{Fp, Fp2},
        verify::{
            compute_public_inputs_from_pairing_precompire, verify_bls_signatures,
            verify_calc_precompire,
        },
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

        fn compute_miller_loops(g1: G1Affine, signature: G2Affine) -> ProofTuple<F, C, D> {
            verify_miller_loop(
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
            )
        }

        fn compute_pairing_precompile(g2_curve_point: G2Affine) -> ProofTuple<F, C, D> {
            verify_calc_precompire(
                Fp2([
                    Fp::get_fp_from_biguint(
                        g2_curve_point.x.c0.to_string().parse::<BigUint>().unwrap(),
                    ),
                    Fp::get_fp_from_biguint(
                        g2_curve_point.x.c1.to_string().parse::<BigUint>().unwrap(),
                    ),
                ]),
                Fp2([
                    Fp::get_fp_from_biguint(
                        g2_curve_point.y.c0.to_string().parse::<BigUint>().unwrap(),
                    ),
                    Fp::get_fp_from_biguint(
                        g2_curve_point.y.c1.to_string().parse::<BigUint>().unwrap(),
                    ),
                ]),
                Fp2([
                    Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
                    Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
                ]),
            )
        }

        // MESSAGE in bytes
        // let bytes_message = [
        //     103, 140, 163, 210, 238, 252, 75, 8, 227, 27, 60, 229, 125, 150, 241, 222, 217, 156,
        //     178, 17, 14, 199, 15, 172, 94, 179, 249, 0, 197, 206, 104, 200, 165, 253, 55, 147, 171,
        //     191, 118, 189, 133, 138, 2, 22, 237, 6, 62, 10, 68, 105, 208, 102, 66, 70, 170, 114,
        //     194, 80, 215, 5, 63, 95, 202, 1, 99, 153, 67, 115, 7, 122, 235, 255, 142, 44, 3, 65,
        //     190, 166, 218, 72, 230, 196, 24, 88, 146, 193, 211, 90, 37, 173, 71, 152, 21, 226, 89,
        //     79, 239, 81, 149, 135, 188, 51, 52, 116, 26, 30, 126, 31, 35, 240, 201, 101, 33, 61,
        //     220, 192, 86, 47, 214, 243, 224, 136, 50, 56, 42, 233, 148, 244, 203, 198, 195, 120,
        //     36, 221, 181, 53, 160, 58, 167, 131, 216, 183, 83, 232, 151, 87, 46, 54, 128, 123, 231,
        //     212, 130, 19, 28, 96, 108, 111, 137, 154, 40, 184, 74, 69, 100, 64, 177, 98, 248, 32,
        //     12, 97, 49, 187, 39, 159, 168, 247, 29, 246, 209, 110, 77, 73, 20, 23, 174, 143, 93,
        //     92, 162, 48, 134, 119, 213, 139, 234, 205, 91, 113, 204, 121, 57, 4, 41, 180, 144, 76,
        //     107, 59, 176, 43, 11, 127, 34, 38, 164, 9, 141, 78, 245, 175, 145, 112, 129, 109, 18,
        //     250, 85, 16, 124, 182, 242, 158, 84, 219, 13, 207, 186, 82, 157, 132, 225, 236, 45,
        //     185, 228, 161, 169, 106, 25, 155, 251, 254, 223,
        // ];
        // .iter()
        // .map(|b| {
        //     let b_v = builder.constant(GoldilocksField::from_canonical_u8(*b));
        //     ByteVariable::from_variable(&mut builder, b_v)
        // })
        // .collect::<Vec<ByteVariable>>();

        /* Test purposes */
        let rng = &mut ark_std::rand::thread_rng();
        let g1 = G1Affine::generator();
        let sk: Fr = Fr::rand(rng);
        let pk = Into::<G1Affine>::into(g1 * sk);
        let message = G2Affine::rand(rng);
        let signature = Into::<G2Affine>::into(message * sk);

        let g2_f: Vec<GoldilocksField> =
            get_proof_public_inputs(compute_pairing_precompile(message));
        let message_in_bytes = compute_public_inputs_from_pairing_precompire(&mut builder, g2_f);
        let message_as_g2_point = hash_to_curve(&mut builder, &message_in_bytes);

        /* Prepare data for verification */
        let first_ml_proof = compute_miller_loops(g1, signature);
        let second_ml_proof = compute_miller_loops(pk, message);
        let g1_generator = generate_new_g1_point_target(&mut builder, g1);
        let signature: PointG2Target =
            calc_ell_coeffs_and_generate_g2_point(&mut builder, signature);
        let public_key: PointG1Target = generate_new_g1_point_target(&mut builder, pk);
        let message: PointG2Target = calc_ell_coeffs_and_generate_g2_point(&mut builder, message);

        verify_bls_signatures(
            &mut builder,
            &first_ml_proof,
            &second_ml_proof,
            &g1_generator,
            &signature,
            &public_key,
            &message_as_g2_point,
        );

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        let s = Instant::now();
        // Generate a proof.
        let (proof, output) = circuit.prove(&input);
        println!("Time to generate a proof {:?}", s.elapsed());
        // Verify proof.
        let s = Instant::now();
        circuit.verify(&proof, &input, &output);
        println!("Time to erify proof {:?}", s.elapsed());
    }

    #[test]
    fn test_message_in_bytes() {
        let mut builder = DefaultBuilder::new();
        let rng = &mut ark_std::rand::thread_rng();

        // let message = G2Affine::from_random_bytes(&[
        //     64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // ])
        // .unwrap();
        // println!("passes");

        // let temp = [
        //     64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // ];
        // let expected_message_in_bytes = temp
        //     .iter()
        //     .map(|b| {
        //         let b_v = builder.constant(GoldilocksField::from_canonical_u8(*b));
        //         ByteVariable::from_variable(&mut builder, b_v)
        //     })
        //     .collect::<Vec<ByteVariable>>();

        let message = G2Affine::rand(rng);
        let message_already_working_way: PointG2Target =
            calc_ell_coeffs_and_generate_g2_point(&mut builder, message);

        let g2_proof = compute_pairing_precompile(message);
        let g2_f: Vec<GoldilocksField> = get_proof_public_inputs(g2_proof);
        // RIGHT
        let input_htc_message_in_bytes =
            compute_public_inputs_from_pairing_precompire(&mut builder, g2_f);

        // CHECK IF HASH TO CURVE INPUT IS CORRECT
        // let _are_equal = input_htc_message_in_bytes
        //     .iter()
        //     .zip(expected_message_in_bytes.iter())
        //     .all(|(a, b)| {
        //         builder.assert_is_equal(*a, *b);
        //         true
        //     });
        //assert!(are_equal);

        let message_as_g2_point = hash_to_curve(&mut builder, &input_htc_message_in_bytes);

        builder.api.connect_biguint(
            &message_already_working_way[0][0],
            &message_as_g2_point[0][0],
        );
        builder.api.connect_biguint(
            &message_already_working_way[0][1],
            &message_as_g2_point[0][1],
        );
        builder.api.connect_biguint(
            &message_already_working_way[1][0],
            &message_as_g2_point[1][0],
        );
        builder.api.connect_biguint(
            &message_already_working_way[1][1],
            &message_as_g2_point[1][1],
        );

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        let s = Instant::now();
        // Generate a proof.
        let (proof, output) = circuit.prove(&input);
        println!("Time to generate a proof {:?}", s.elapsed());
        // Verify proof.
        let s = Instant::now();
        circuit.verify(&proof, &input, &output);
        println!("Time to erify proof {:?}", s.elapsed());
    }
}
