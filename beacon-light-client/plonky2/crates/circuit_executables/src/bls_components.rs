use anyhow::Result;
use ark_bls12_381::{Fq, Fq2, G1Affine, G1Projective, G2Affine};
use ark_ec::Group;
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

const CIRCUIT_DIR: &str = "circuits";
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

impl BlsComponents {
    fn remove_first_two_chars(&mut self) {
        self.input.pubkey = self.input.pubkey.chars().skip(2).collect();
        self.input.signature = self.input.signature.chars().skip(2).collect();
        self.input.message = self.input.message.chars().skip(2).collect();
    }
}

pub fn read_yaml_files_from_directory<P: AsRef<Path>>(
    dir: P,
) -> Result<Vec<BlsComponents>, Box<dyn std::error::Error>> {
    let mut components = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension() == Some(std::ffi::OsStr::new("yaml")) {
            let config = read_yaml_file(entry.path())?;
            components.push(config);
        }
    }

    Ok(components)
}

fn read_yaml_file<P: AsRef<Path>>(path: P) -> Result<BlsComponents, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(path)?;
    let mut components: BlsComponents = serde_yaml::from_str(&file_content)?;
    components.remove_first_two_chars();
    Ok(components)
}

pub async fn bls12_381_components_proofs(
    bls_components: BlsComponents,
) -> Result<ProofWithPublicInputs<F, C, D>> {
    let message_g2 = hash_to_curve_g2(
        &hex::decode(bls_components.input.message.clone()).unwrap(),
        DST.as_bytes(),
    );
    let message_g2 = convert_ecp2_to_g2affine(message_g2);

    let pubkey_g1 = G1Affine::deserialize_compressed(
        &*hex::decode(bls_components.input.pubkey.clone()).unwrap(),
    )
    .unwrap();
    let signature_g2 = G2Affine::deserialize_compressed(
        &*hex::decode(bls_components.input.signature.clone()).unwrap(),
    )
    .unwrap();
    let g1 = G1Projective::generator();
    let neg_g1 = g1.neg();

    let miller_loop1 = miller_loop(
        Fp::get_fp_from_biguint(pubkey_g1.x.to_string().parse::<BigUint>().unwrap()),
        Fp::get_fp_from_biguint(pubkey_g1.y.to_string().parse::<BigUint>().unwrap()),
        Fp2([
            Fp::get_fp_from_biguint(message_g2.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(message_g2.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(message_g2.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(message_g2.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    );

    let miller_loop2 = miller_loop(
        Fp::get_fp_from_biguint(neg_g1.x.to_string().parse::<BigUint>().unwrap()),
        Fp::get_fp_from_biguint(neg_g1.y.to_string().parse::<BigUint>().unwrap()),
        Fp2([
            Fp::get_fp_from_biguint(signature_g2.x.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(signature_g2.x.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(signature_g2.y.c0.to_string().parse::<BigUint>().unwrap()),
            Fp::get_fp_from_biguint(signature_g2.y.c1.to_string().parse::<BigUint>().unwrap()),
        ]),
        Fp2([
            Fp::get_fp_from_biguint(BigUint::from_str("1").unwrap()),
            Fp::get_fp_from_biguint(BigUint::from_str("0").unwrap()),
        ]),
    );

    let fp12_mull = miller_loop1 * miller_loop2;
    // PROVING HAPPENS HERE
    let (pp1, pp2) = handle_pairing_precomp(&message_g2, &signature_g2).await;

    let (ml1, ml2) =
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
    let mut pw = PartialWitness::<F>::new();

    pw.set_target_arr(
        &targets.pubkey,
        &hex::decode(bls_components.input.pubkey)
            .unwrap()
            .iter()
            .map(|x| F::from_canonical_usize(*x as usize))
            .collect::<Vec<F>>(),
    );
    pw.set_target_arr(
        &targets.sig,
        &hex::decode(bls_components.input.signature)
            .unwrap()
            .iter()
            .map(|x| F::from_canonical_usize(*x as usize))
            .collect::<Vec<F>>(),
    );
    pw.set_target_arr(
        &targets.msg,
        &hex::decode(bls_components.input.message)
            .unwrap()
            .iter()
            .map(|x| F::from_canonical_usize(*x as usize))
            .collect::<Vec<F>>(),
    );

    pw.set_proof_with_pis_target(&targets.pt_pp1, &pp1);
    pw.set_proof_with_pis_target(&targets.pt_pp2, &pp2);
    pw.set_proof_with_pis_target(&targets.pt_ml1, &ml1);
    pw.set_proof_with_pis_target(&targets.pt_ml2, &ml2);
    pw.set_proof_with_pis_target(&targets.pt_fp12m, &fp12_mul_proof);
    pw.set_proof_with_pis_target(&targets.pt_fe, &final_exp_proof);

    println!("Starting proof generation");

    let proof = circuit_data.prove(pw).unwrap();

    Ok(proof)
}

async fn handle_final_exponentiation(fp12_mull: &Fp12) -> ProofWithPublicInputs<F, C, D> {
    let final_exp_circuit_data =
        load_circuit_data_starky(&format!("{CIRCUIT_DIR}/final_exponentiate_circuit"));

    let final_exp_targets =
        get_recursive_stark_targets(&format!("{CIRCUIT_DIR}/final_exponentiate_circuit")).unwrap();

    let final_exp_proof =
        generate_final_exponentiate(&fp12_mull, &final_exp_targets, &final_exp_circuit_data);

    final_exp_proof
}

async fn handle_fp12_mul(
    miller_loop1: &Fp12,
    miller_loop2: &Fp12,
) -> ProofWithPublicInputs<F, C, D> {
    let fp12_mul_circuit_data = load_circuit_data_starky(&format!("{CIRCUIT_DIR}/fp12_mul"));

    let fp12_mul_targets = get_recursive_stark_targets(&format!("{CIRCUIT_DIR}/fp12_mul")).unwrap();

    let fp12_mul_proof = generate_fp12_mul_proof(
        &miller_loop1,
        &miller_loop2,
        &fp12_mul_targets,
        &fp12_mul_circuit_data,
    );

    fp12_mul_proof
}

async fn handle_miller_loop(
    pubkey_g1: &G1Affine,
    message_g2: &G2Affine,
    neg_g1: &G1Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<F, C, D>,
    ProofWithPublicInputs<F, C, D>,
) {
    let miller_loop_circuit_data = load_circuit_data_starky(&format!("{CIRCUIT_DIR}/miller_loop"));

    let miller_loop_targets =
        get_recursive_stark_targets(&format!("{CIRCUIT_DIR}/miller_loop")).unwrap();

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

async fn handle_pairing_precomp(
    message_g2: &G2Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<F, C, D>,
    ProofWithPublicInputs<F, C, D>,
) {
    let pairing_precomp_circuit_data =
        load_circuit_data_starky(&format!("{CIRCUIT_DIR}/pairing_precomp"));

    let pairing_precomp_targets =
        get_recursive_stark_targets(&format!("{CIRCUIT_DIR}/pairing_precomp")).unwrap();

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

fn convert_ecp2_to_g2affine(ecp2_point: ECP2) -> G2Affine {
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

fn convert_big_to_fq(big: Big) -> Fq {
    let bytes = &hex::decode(big.to_string()).unwrap();
    Fq::from_be_bytes_mod_order(bytes)
}

#[cfg(test)]
pub mod tests {
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig},
    };

    use super::{bls12_381_components_proofs, read_yaml_files_from_directory};

    const D: usize = 2;
    type F = GoldilocksField;

    #[tokio::test]
    async fn test_bls12_381_components_proofs_with_verify_eth_cases() {
        let eth_tests_directory_path = "../scripts/bls12-381-tests/eth_tests/bls/verify";
        let bls_components_with_verify_eth_tests_cases =
            read_yaml_files_from_directory(eth_tests_directory_path).unwrap();
        let standard_recursion_config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

        for i in 0..bls_components_with_verify_eth_tests_cases.len() {
            let current_eth_verify_test = &bls_components_with_verify_eth_tests_cases[i];
            let proof = bls12_381_components_proofs((*current_eth_verify_test).clone())
                .await
                .unwrap();

            println!("Proof generated");

            println!(
                "Is valid signature {}",
                proof.public_inputs[proof.public_inputs.len() - 1]
            );

            let proof_t = builder.constant(proof.public_inputs[proof.public_inputs.len() - 1]);
            builder.assert_one(proof_t);
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn test_bls12_381_components_proofs_with_verify_eth_cases_should_panic() {
        let eth_tests_directory_path = "../scripts/bls12-381-tests/eth_tests/bls/verify";
        let bls_components_with_verify_eth_tests_cases =
            read_yaml_files_from_directory(eth_tests_directory_path).unwrap();
        let standard_recursion_config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

        for i in 0..bls_components_with_verify_eth_tests_cases.len() {
            let current_eth_verify_test = &bls_components_with_verify_eth_tests_cases[i];
            let proof = bls12_381_components_proofs((*current_eth_verify_test).clone())
                .await
                .unwrap();

            println!("Proof generated");

            println!(
                "Is valid signature {}",
                proof.public_inputs[proof.public_inputs.len() - 1]
            );

            let proof_t = builder.constant(proof.public_inputs[proof.public_inputs.len() - 1]);
            builder.assert_one(proof_t);
        }
    }
}
