use anyhow::Result;
use ark_bls12_381::{G1Affine, G1Projective, G2Affine};
use ark_ec::{CurveGroup, Group};
use ark_serialize::CanonicalDeserialize;
use circuit::SerdeCircuitTarget;
use circuit_executables::{
    bls_components::{
        compute_native_miller_loop_from, convert_ecp2_to_g2affine, handle_final_exponentiation,
        handle_fp12_mul, handle_miller_loop, handle_pairing_precomp, set_bls_witness,
        BlsComponents, BlsProofs, Input,
    },
    constants::SERIALIZED_CIRCUITS_DIR,
    crud::{
        common::{load_circuit_data_starky, read_from_file},
        proof_storage::proof_storage::create_proof_storage,
    },
    utils::CommandLineOptionsBuilder,
};
use circuits::bls_verification::bls12_381_circuit::BlsCircuitTargets;
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField, iop::witness::PartialWitness,
    util::serialization::Buffer,
};
use snowbridge_amcl::bls381::bls381::utils::hash_to_curve_g2;
use std::ops::Neg;

async fn async_main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("bls12_381_components_proofs")
        .with_proof_storage_options()
        .get_matches();
    const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
    let pubkey = "b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73";
    let signature = "b735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275";
    let msg = "5bb03392c9c8a8b92c840338f619bb060b109b254c9ab75d4dddc6d00932bce3";

    let message_g2 = convert_ecp2_to_g2affine(hash_to_curve_g2(
        &hex::decode(&msg).unwrap(),
        DST.as_bytes(),
    ));
    let pubkey_g1 =
        G1Affine::deserialize_compressed_unchecked(&*hex::decode(pubkey).unwrap()).unwrap();
    let signature_g2 =
        G2Affine::deserialize_compressed_unchecked(&*hex::decode(signature).unwrap()).unwrap();
    let g1 = G1Projective::generator();
    let neg_g1 = g1.neg();

    let miller_loop1 = compute_native_miller_loop_from(pubkey_g1, message_g2);

    let miller_loop2 = compute_native_miller_loop_from(neg_g1.into_affine(), signature_g2);

    let fp12_mull = miller_loop1 * miller_loop2;
    // PROVING HAPPENS HERE

    let mut proof_storage = create_proof_storage(&matches).await;
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
    let mut pw = PartialWitness::<GoldilocksField>::new();

    set_bls_witness(
        &mut pw,
        &targets,
        &BlsComponents {
            input: Input {
                pubkey: pubkey.to_string(),
                signature: signature.to_string(),
                message: msg.to_string(),
            },
            output: true,
        },
        &BlsProofs {
            pairing_prec_proof1,
            pairing_prec_proof2,
            miller_loop_proof2,
            miller_loop_proof1,
            fp12_mul_proof,
            final_exp_proof,
        },
    );

    println!("Starting proof generation");

    let proof = circuit_data.prove(pw).unwrap();

    println!("Proof generated");

    println!(
        "Is valid signature {}",
        proof.public_inputs[proof.public_inputs.len() - 1]
    );

    proof_storage
        .set_proof("bls12_381_proof".to_string(), &proof.to_bytes())
        .await?;

    Ok(())
}

fn main() {
    let _ = std::thread::Builder::new()
        .spawn(|| future::block_on(async_main()))
        .unwrap()
        .join()
        .unwrap();
}
