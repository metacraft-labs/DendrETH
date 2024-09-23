use anyhow::Result;
use ark_bls12_381::{Fq, Fq2, G1Affine, G1Projective, G2Affine};
use ark_ec::Group;
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;
use circuit::SerdeCircuitTarget;
use circuit_executables::{
    crud::{
        common::{get_recursive_stark_targets, load_circuit_data_starky, read_from_file},
        proof_storage::proof_storage::RedisBlobStorage,
    },
    provers::{
        generate_final_exponentiate, generate_fp12_mul_proof, generate_miller_loop_proof,
        generate_pairing_precomp_proof,
    },
    utils::CommandLineOptionsBuilder,
};
use circuits::bls_verification::bls12_381_circuit::BlsCircuitTargets;
use futures_lite::future;
use num_bigint::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
    util::serialization::Buffer,
};
use snowbridge_amcl::bls381::{big::Big, bls381::utils::hash_to_curve_g2, ecp2::ECP2};
use starky_bls12_381::native::{miller_loop, Fp, Fp12, Fp2};
use std::{ops::Neg, str::FromStr};

async fn async_main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("bls12_381_components_proofs")
        .with_serialized_circuits_dir()
        .with_proof_storage_config()
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
    let pubkey = "b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73";
    let signature = "b735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275";
    let msg = "5bb03392c9c8a8b92c840338f619bb060b109b254c9ab75d4dddc6d00932bce3";

    let message_g2 = hash_to_curve_g2(&hex::decode(msg).unwrap(), DST.as_bytes());
    let message_g2 = convert_ecp2_to_g2affine(message_g2);

    let pubkey_g1 = G1Affine::deserialize_compressed(&*hex::decode(pubkey).unwrap()).unwrap();
    let signature_g2 = G2Affine::deserialize_compressed(&*hex::decode(signature).unwrap()).unwrap();
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

    let storage_config_filepath = matches.get_one::<String>("proof_storage_cfg").unwrap();
    let mut storage =
        RedisBlobStorage::from_file(&storage_config_filepath, "bls-verification").await?;

    let (pp1, pp2) =
        handle_pairing_precomp(serialized_circuits_dir, &message_g2, &signature_g2).await;

    let (ml1, ml2) = handle_miller_loop(
        serialized_circuits_dir,
        &pubkey_g1,
        &message_g2,
        &neg_g1.into(),
        &signature_g2,
    )
    .await;

    let fp12_mul_proof =
        handle_fp12_mul(serialized_circuits_dir, &miller_loop1, &miller_loop2).await;

    let final_exp_proof = handle_final_exponentiation(serialized_circuits_dir, &fp12_mull).await;

    let circuit_data = load_circuit_data_starky(&format!("{serialized_circuits_dir}/bls12_381"));
    let target_bytes = read_from_file(&format!(
        "{}/{}.plonky2_targets",
        serialized_circuits_dir, "bls12_381"
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    let targets = BlsCircuitTargets::deserialize(&mut target_buffer).unwrap();
    let mut pw = PartialWitness::<GoldilocksField>::new();

    pw.set_target_arr(
        &targets.pubkey,
        &hex::decode(pubkey)
            .unwrap()
            .iter()
            .map(|x| GoldilocksField::from_canonical_usize(*x as usize))
            .collect::<Vec<GoldilocksField>>(),
    );
    pw.set_target_arr(
        &targets.sig,
        &hex::decode(signature)
            .unwrap()
            .iter()
            .map(|x| GoldilocksField::from_canonical_usize(*x as usize))
            .collect::<Vec<GoldilocksField>>(),
    );
    pw.set_target_arr(
        &targets.msg,
        &hex::decode(msg)
            .unwrap()
            .iter()
            .map(|x| GoldilocksField::from_canonical_usize(*x as usize))
            .collect::<Vec<GoldilocksField>>(),
    );

    pw.set_proof_with_pis_target(&targets.pt_pp1, &pp1);
    pw.set_proof_with_pis_target(&targets.pt_pp2, &pp2);
    pw.set_proof_with_pis_target(&targets.pt_ml1, &ml1);
    pw.set_proof_with_pis_target(&targets.pt_ml2, &ml2);
    pw.set_proof_with_pis_target(&targets.pt_fp12m, &fp12_mul_proof);
    pw.set_proof_with_pis_target(&targets.pt_fe, &final_exp_proof);

    println!("Starting proof generation");

    let proof = circuit_data.prove(pw).unwrap();

    println!("Proof generated");

    println!(
        "Is valid signature {}",
        proof.public_inputs[proof.public_inputs.len() - 1]
    );

    storage
        .blob
        .set_proof("bls12_381_proof".to_string(), &proof.to_bytes())
        .await?;

    Ok(())
}

async fn handle_final_exponentiation(
    serialized_circuits_dir: &str,
    fp12_mull: &Fp12,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    let final_exp_circuit_data = load_circuit_data_starky(&format!(
        "{serialized_circuits_dir}/final_exponentiate_circuit"
    ));

    let final_exp_targets = get_recursive_stark_targets(&format!(
        "{serialized_circuits_dir}/final_exponentiate_circuit"
    ))
    .unwrap();

    let final_exp_proof =
        generate_final_exponentiate(&fp12_mull, &final_exp_targets, &final_exp_circuit_data);

    final_exp_proof
}

async fn handle_fp12_mul(
    serialized_circuits_dir: &str,
    miller_loop1: &Fp12,
    miller_loop2: &Fp12,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    let fp12_mul_circuit_data =
        load_circuit_data_starky(&format!("{serialized_circuits_dir}/fp12_mul"));

    let fp12_mul_targets =
        get_recursive_stark_targets(&format!("{serialized_circuits_dir}/fp12_mul")).unwrap();

    let fp12_mul_proof = generate_fp12_mul_proof(
        &miller_loop1,
        &miller_loop2,
        &fp12_mul_targets,
        &fp12_mul_circuit_data,
    );

    fp12_mul_proof
}

async fn handle_miller_loop(
    serialized_circuits_dir: &str,
    pubkey_g1: &G1Affine,
    message_g2: &G2Affine,
    neg_g1: &G1Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) {
    let miller_loop_circuit_data =
        load_circuit_data_starky(&format!("{serialized_circuits_dir}/miller_loop"));

    let miller_loop_targets =
        get_recursive_stark_targets(&format!("{serialized_circuits_dir}/miller_loop")).unwrap();

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

fn main() {
    let _ = std::thread::Builder::new()
        .spawn(|| future::block_on(async_main()))
        .unwrap()
        .join()
        .unwrap();
}

async fn handle_pairing_precomp(
    serialized_circuits_dir: &str,
    message_g2: &G2Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) {
    let pairing_precomp_circuit_data =
        load_circuit_data_starky(&format!("{serialized_circuits_dir}/pairing_precomp"));

    let pairing_precomp_targets =
        get_recursive_stark_targets(&format!("{serialized_circuits_dir}/pairing_precomp")).unwrap();

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
