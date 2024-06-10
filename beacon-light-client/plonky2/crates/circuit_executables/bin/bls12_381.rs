use anyhow::Result;
use circuit::SerdeCircuitTarget;
use circuit_executables::{
    bls_components::{set_bls_witness, BlsComponents, BlsProofs, Input},
    constants::SERIALIZED_CIRCUITS_DIR,
    crud::{
        common::{load_circuit_data_starky, load_common_circuit_data_starky, read_from_file},
        proof_storage::proof_storage::{create_proof_storage, ProofStorage},
    },
    utils::CommandLineOptionsBuilder,
};
use circuits::bls_verification::bls12_381_circuit::BlsCircuitTargets;
use futures_lite::future;

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
    util::serialization::Buffer,
};

const CIRCUIT_NAME: &str = "bls12_381";

async fn async_main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("bls12_381_components_proofs")
        .with_proof_storage_options()
        .get_matches();

    let circuit_data =
        load_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/{CIRCUIT_NAME}"));
    let target_bytes = read_from_file(&format!(
        "{}/{}.plonky2_targets",
        SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    let targets = BlsCircuitTargets::deserialize(&mut target_buffer).unwrap();

    let pubkey = "a491d1b0ecd9bb917989f0e74f0dea0422eac4a873e5e2644f368dffb9a6e20fd6e10c1b77654d067c0618f6e5a7f79a";
    let signature = "882730e5d03f6b42c3abc26d3372625034e1d871b65a8a6b900a56dae22da98abbe1b68f85e49fe7652a55ec3d0591c20767677e33e5cbb1207315c41a9ac03be39c2e7668edc043d6cb1d9fd93033caa8a1c5b0e84bedaeb6c64972503a43eb";
    let msg = "5656565656565656565656565656565656565656565656565656565656565656";

    let mut proof_storage = create_proof_storage(&matches).await;

    let (pairing_prec_proof1, pairing_prec_proof2) =
        get_pairing_precomp_proofs(&mut proof_storage).await;
    let (miller_loop_proof1, miller_loop_proof2) = get_miller_loop_proofs(&mut proof_storage).await;
    let fp12_mul_proof = get_fp12_mul_proof(&mut proof_storage).await;
    let final_exp_proof = get_final_exp_proof(&mut proof_storage).await;

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
    Ok(())
}

async fn get_final_exp_proof(
    proof_storage: &mut Box<dyn ProofStorage>,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    let final_exp_circuit_data = load_common_circuit_data_starky(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/final_exponentiate_circuit"
    ));
    let final_exp_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            (proof_storage.get_proof("final_exp_proof".to_string()).await).unwrap(),
            &final_exp_circuit_data,
        )
        .unwrap();

    final_exp_proof
}

async fn get_fp12_mul_proof(
    proof_storage: &mut Box<dyn ProofStorage>,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    let fp12_mul_circuit_data =
        load_common_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/fp12_mul"));
    let fp12_mul_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            (proof_storage.get_proof("fp12_mul_proof".to_string()).await).unwrap(),
            &fp12_mul_circuit_data,
        )
        .unwrap();

    fp12_mul_proof
}

async fn get_miller_loop_proofs(
    proof_storage: &mut Box<dyn ProofStorage>,
) -> (
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) {
    let miller_loop_circuit_data =
        load_common_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/miller_loop"));
    let ml1 = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        (proof_storage
            .get_proof("miller_loop_proof_1".to_string())
            .await)
            .unwrap(),
        &miller_loop_circuit_data,
    )
    .unwrap();
    let ml2 = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        (proof_storage
            .get_proof("miller_loop_proof_2".to_string())
            .await)
            .unwrap(),
        &miller_loop_circuit_data,
    )
    .unwrap();

    (ml1, ml2)
}

async fn get_pairing_precomp_proofs(
    proof_storage: &mut Box<dyn ProofStorage>,
) -> (
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) {
    let pairing_precomp_circuit_data =
        load_common_circuit_data_starky(&format!("{SERIALIZED_CIRCUITS_DIR}/pairing_precomp"));
    let pp1 = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        (proof_storage
            .get_proof("pairing_precomp_proof1".to_string())
            .await)
            .unwrap(),
        &pairing_precomp_circuit_data,
    )
    .unwrap();
    let pp2 = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        (proof_storage
            .get_proof("pairing_precomp_proof2".to_string())
            .await)
            .unwrap(),
        &pairing_precomp_circuit_data,
    )
    .unwrap();

    (pp1, pp2)
}

fn main() -> Result<()> {
    future::block_on(async_main())
}
