use anyhow::Result;
use ark_bls12_381::{Fq, Fq2, G1Affine, G1Projective, G2Affine};
use ark_ec::{CurveGroup, Group};
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;
use circuit::SerdeCircuitTarget;
use circuits::bls_verification::bls12_381_circuit::BlsCircuitTargets;
use num::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
    util::serialization::Buffer,
};
use serde::Deserialize;
use snowbridge_amcl::bls381::{big::Big, bls381::utils::hash_to_curve_g2, ecp2::ECP2};
use starky_bls12_381::native::{miller_loop, Fp, Fp12, Fp2};
use std::{fs, ops::Neg, path::Path, str::FromStr};

use crate::{
    constants::SERIALIZED_CIRCUITS_DIR,
    crud::common::{get_recursive_stark_targets, load_circuit_data_starky, read_from_file},
    provers::{
        generate_final_exponentiate, generate_fp12_mul_proof, generate_miller_loop_proof,
        generate_pairing_precomp_proof,
    },
};

const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;

#[derive(Clone, Debug, Deserialize)]
pub struct Input {
    pub pubkey: String,
    pub signature: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlsComponents {
    pub input: Input,
    pub output: bool,
}

pub struct BlsProofs {
    pub pairing_prec_proof1: ProofWithPublicInputs<F, C, D>,
    pub pairing_prec_proof2: ProofWithPublicInputs<F, C, D>,
    pub miller_loop_proof2: ProofWithPublicInputs<F, C, D>,
    pub miller_loop_proof1: ProofWithPublicInputs<F, C, D>,
    pub fp12_mul_proof: ProofWithPublicInputs<F, C, D>,
    pub final_exp_proof: ProofWithPublicInputs<F, C, D>,
}

impl BlsComponents {
    fn remove_first_two_chars(&mut self) {
        self.input.pubkey = self.input.pubkey.chars().skip(2).collect();
        self.input.signature = self.input.signature.chars().skip(2).collect();
        self.input.message = self.input.message.chars().skip(2).collect();
    }
}

pub async fn bls12_381_components_proofs(
    components: &BlsComponents,
) -> Result<ProofWithPublicInputs<F, C, D>> {
    let message_g2 = convert_ecp2_to_g2affine(hash_to_curve_g2(
        &hex::decode(&components.input.message).unwrap(),
        DST.as_bytes(),
    ));
    let signature_g2 = G2Affine::deserialize_compressed_unchecked(
        &*hex::decode(&components.input.signature).unwrap(),
    )
    .unwrap();
    let pubkey_g1 = G1Affine::deserialize_compressed_unchecked(
        &*hex::decode(&components.input.pubkey).unwrap(),
    )
    .unwrap();
    let neg_g1 = G1Projective::generator().neg();

    let miller_loop1 = compute_native_miller_loop_from(pubkey_g1, message_g2);

    let miller_loop2 = compute_native_miller_loop_from(neg_g1.into_affine(), signature_g2);

    let fp12_mull = miller_loop1 * miller_loop2;

    // PROVING HAPPENS HERE
    let (pairing_prec_proof1, pairing_prec_proof2) =
        handle_pairing_precomp(&message_g2, &signature_g2).await;

    let (miller_loop_proof1, miller_loop_proof2) =
        handle_miller_loop(&pubkey_g1, &message_g2, &neg_g1.into(), &signature_g2).await;

    let fp12_mul_proof = handle_fp12_mul(&miller_loop1, &miller_loop2).await;

    let final_exp_proof = handle_final_exponentiation(&fp12_mull).await;

    let circuit_data = load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/bls12_381"));
    let target_bytes = read_from_file(&format!(
        "{}/{}.plonky2_targets",
        SERIALIZED_CIRCUITS_DIR, "bls12_381"
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    let targets = BlsCircuitTargets::deserialize(&mut target_buffer).unwrap();
    let proofs = BlsProofs {
        pairing_prec_proof1,
        pairing_prec_proof2,
        miller_loop_proof1,
        miller_loop_proof2,
        fp12_mul_proof,
        final_exp_proof,
    };

    let mut pw = PartialWitness::<F>::new();
    set_bls_witness(&mut pw, &targets, &components, &proofs);

    println!("Starting proof generation");

    let proof = circuit_data.prove(pw).unwrap();

    Ok(proof)
}

pub async fn handle_final_exponentiation(fp12_mull: &Fp12) -> ProofWithPublicInputs<F, C, D> {
    let final_exp_circuit_data = load_circuit_data_starky(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/final_exponentiate_circuit"
    ));

    let final_exp_targets = get_recursive_stark_targets(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/final_exponentiate_circuit"
    ))
    .unwrap();

    let final_exp_proof =
        generate_final_exponentiate(&fp12_mull, &final_exp_targets, &final_exp_circuit_data);

    final_exp_proof
}

pub async fn handle_fp12_mul(
    miller_loop1: &Fp12,
    miller_loop2: &Fp12,
) -> ProofWithPublicInputs<F, C, D> {
    let fp12_mul_circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/fp12_mul"));

    let fp12_mul_targets =
        get_recursive_stark_targets(&format!("{SERIALIZED_CIRCUITS_DIR}/fp12_mul")).unwrap();

    let fp12_mul_proof = generate_fp12_mul_proof(
        &miller_loop1,
        &miller_loop2,
        &fp12_mul_targets,
        &fp12_mul_circuit_data,
    );

    fp12_mul_proof
}

pub async fn handle_miller_loop(
    pubkey_g1: &G1Affine,
    message_g2: &G2Affine,
    neg_g1: &G1Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<F, C, D>,
    ProofWithPublicInputs<F, C, D>,
) {
    let miller_loop_circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop"));

    let miller_loop_targets =
        get_recursive_stark_targets(&format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop")).unwrap();

    let ml1 = generate_miller_loop_proof(
        &pubkey_g1,
        &message_g2,
        &miller_loop_targets,
        &miller_loop_circuit_data,
    );

    let ml2 = generate_miller_loop_proof(
        &neg_g1,
        &signature_g2,
        &miller_loop_targets,
        &miller_loop_circuit_data,
    );

    (ml1, ml2)
}

pub async fn handle_pairing_precomp(
    message_g2: &G2Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<F, C, D>,
    ProofWithPublicInputs<F, C, D>,
) {
    let pairing_precomp_circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp"));

    let pairing_precomp_targets =
        get_recursive_stark_targets(&format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp")).unwrap();

    let pp1 = generate_pairing_precomp_proof(
        &message_g2,
        &pairing_precomp_targets,
        &pairing_precomp_circuit_data,
    );

    let pp2 = generate_pairing_precomp_proof(
        &signature_g2,
        &pairing_precomp_targets,
        &pairing_precomp_circuit_data,
    );

    (pp1, pp2)
}

pub fn read_yaml_file<P: AsRef<Path>>(
    path: P,
) -> Result<BlsComponents, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(path)?;
    let mut components: BlsComponents = serde_yaml::from_str(&file_content)?;
    components.remove_first_two_chars();
    Ok(components)
}

pub fn set_bls_witness(
    pw: &mut PartialWitness<F>,
    targets: &BlsCircuitTargets,
    components: &BlsComponents,
    proofs: &BlsProofs,
) {
    pw.set_target_arr(
        &targets.pubkey,
        &hex::decode(&components.input.pubkey)
            .unwrap()
            .iter()
            .map(|x| F::from_canonical_usize(*x as usize))
            .collect::<Vec<F>>(),
    );
    pw.set_target_arr(
        &targets.sig,
        &hex::decode(&components.input.signature)
            .unwrap()
            .iter()
            .map(|x| F::from_canonical_usize(*x as usize))
            .collect::<Vec<F>>(),
    );
    pw.set_target_arr(
        &targets.msg,
        &hex::decode(&components.input.message)
            .unwrap()
            .iter()
            .map(|x| F::from_canonical_usize(*x as usize))
            .collect::<Vec<F>>(),
    );

    pw.set_proof_with_pis_target(&targets.pt_pp1, &proofs.pairing_prec_proof1);
    pw.set_proof_with_pis_target(&targets.pt_pp2, &proofs.pairing_prec_proof2);
    pw.set_proof_with_pis_target(&targets.pt_ml1, &proofs.miller_loop_proof1);
    pw.set_proof_with_pis_target(&targets.pt_ml2, &proofs.miller_loop_proof2);
    pw.set_proof_with_pis_target(&targets.pt_fp12m, &proofs.fp12_mul_proof);
    pw.set_proof_with_pis_target(&targets.pt_fe, &proofs.final_exp_proof);
}

pub fn compute_native_miller_loop_from(
    g1_affine_point: G1Affine,
    g2_affine_point: G2Affine,
) -> Fp12 {
    miller_loop(
        Fp::get_fp_from_biguint(g1_affine_point.x.to_string().parse::<BigUint>().unwrap()),
        Fp::get_fp_from_biguint(g1_affine_point.y.to_string().parse::<BigUint>().unwrap()),
        Fp2([
            Fp::get_fp_from_biguint(g2_affine_point.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2_affine_point.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(g2_affine_point.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(g2_affine_point.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    )
}

pub fn convert_ecp2_to_g2affine(ecp2_point: ECP2) -> G2Affine {
    let x = Fq2::new(
        convert_big_to_fq(ecp2_point.getpx().geta()),
        convert_big_to_fq(ecp2_point.getpx().getb()),
    );

    let y = Fq2::new(
        convert_big_to_fq(ecp2_point.getpy().geta()),
        convert_big_to_fq(ecp2_point.getpy().getb()),
    );

    G2Affine::new(x, y)
}

pub fn convert_big_to_fq(big: Big) -> Fq {
    let bytes = &hex::decode(big.to_string()).unwrap();
    Fq::from_be_bytes_mod_order(bytes)
}

#[cfg(test)]
pub mod tests {
    use std::env;

    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig},
    };

    use super::{bls12_381_components_proofs, read_yaml_file};

    const PATH_TO_VERIFY_ETH_TEST_CASES: &str = "scripts/bls12-381-tests/eth_tests/bls/verify";
    const D: usize = 2;
    type F = GoldilocksField;

    #[tokio::test]
    async fn test_bls12_381_components_proofs_with_verify_eth_cases() {
        let args: Vec<String> = env::args().collect();
        if args.len() < 3 {
            panic!("Expected a file path as argument");
        }

        let current_file_path = format!("{}/{}", PATH_TO_VERIFY_ETH_TEST_CASES, &args[3]);
        println!("current file path is: {:?}", &current_file_path);
        let bls_components =
            read_yaml_file(format!("{}/{}", PATH_TO_VERIFY_ETH_TEST_CASES, &args[3])).unwrap();
        let standard_recursion_config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

        println!("current pubkey: {:?}", bls_components.input.pubkey);
        println!("current signature: {:?}", bls_components.input.signature);
        println!("current message: {:?}", bls_components.input.message);
        let proof = bls12_381_components_proofs(&bls_components).await.unwrap();

        println!(
            "Is valid signature {}",
            proof.public_inputs[proof.public_inputs.len() - 1]
        );

        let proof_t = builder.constant(proof.public_inputs[proof.public_inputs.len() - 1]);
        if bls_components.output {
            builder.assert_one(proof_t);
        } else {
            builder.assert_zero(proof_t);
        }

        println!(
            "test case is VALID for: pubkey: {:?}, signature: {:?} and message: {:?}",
            bls_components.input.pubkey,
            bls_components.input.signature,
            bls_components.input.message,
        );
    }

    #[tokio::test]
    async fn test_bls12_381_at_infinity_case() {
        let file_path = "verify_infinity_pubkey_and_infinity_signature.yaml";
        let bls_components =
            read_yaml_file(format!("{}/{}", PATH_TO_VERIFY_ETH_TEST_CASES, file_path)).unwrap();
        let standard_recursion_config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

        println!("current pubkey: {:?}", bls_components.input.pubkey);
        println!("current signature: {:?}", bls_components.input.signature);
        println!("current message: {:?}", bls_components.input.message);
        let proof = bls12_381_components_proofs(&bls_components).await.unwrap();

        println!(
            "Is valid signature {}",
            proof.public_inputs[proof.public_inputs.len() - 1]
        );

        let proof_t = builder.constant(proof.public_inputs[proof.public_inputs.len() - 1]);
        if bls_components.output {
            builder.assert_one(proof_t);
        } else {
            builder.assert_zero(proof_t);
        }

        println!(
            "test case is VALID for: pubkey: {:?}, signature: {:?} and message: {:?}",
            bls_components.input.pubkey,
            bls_components.input.signature,
            bls_components.input.message,
        );
    }

    #[tokio::test]
    async fn test_bls12_381_one_privkey() {
        let file_path = "verifycase_one_privkey_47117849458281be.yaml";
        let bls_components =
            read_yaml_file(format!("{}/{}", PATH_TO_VERIFY_ETH_TEST_CASES, file_path)).unwrap();
        let standard_recursion_config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

        println!("current pubkey: {:?}", bls_components.input.pubkey);
        println!("current signature: {:?}", bls_components.input.signature);
        println!("current message: {:?}", bls_components.input.message);
        let proof = bls12_381_components_proofs(&bls_components).await.unwrap();

        println!(
            "Is valid signature {}",
            proof.public_inputs[proof.public_inputs.len() - 1]
        );

        let proof_t = builder.constant(proof.public_inputs[proof.public_inputs.len() - 1]);
        if bls_components.output {
            builder.assert_one(proof_t);
        } else {
            builder.assert_zero(proof_t);
        }

        println!(
            "test case is VALID for: pubkey: {:?}, signature: {:?} and message: {:?}",
            bls_components.input.pubkey,
            bls_components.input.signature,
            bls_components.input.message,
        );
    }
}
