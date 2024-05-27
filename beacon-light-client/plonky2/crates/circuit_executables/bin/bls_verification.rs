#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use ark_bls12_381::{Fq, Fq2, G1Affine, G1Projective, G2Affine};
use ark_ec::Group;
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;
use aws_sdk_s3::primitives::event_stream::Message;
use circuit::{
    serde_circuit_target::deserialize_circuit_target, set_witness::SetWitness, CircuitTargetType,
    SerdeCircuitTarget,
};
use circuit_executables::{
    constants::{BLS_DST, SERIALIZED_CIRCUITS_DIR, VALIDATOR_REGISTRY_LIMIT},
    crud::{
        common::{
            delete_balance_verification_proof_dependencies, fetch_proofs_balances,
            fetch_validator_balance_input, get_recursive_stark_targets, load_circuit_data,
            load_circuit_data_starky, read_from_file, save_balance_proof,
        },
        proof_storage::proof_storage::{create_proof_storage, ProofStorage},
    },
    db_constants::DB_CONSTANTS,
    provers::{
        generate_final_exponentiate, generate_fp12_mul_proof, generate_miller_loop_proof,
        generate_pairing_precomp_proof, prove_inner_level,
    },
    utils::{
        parse_balance_verification_command_line_options, parse_bls_verification_command_line_options, parse_config_file, CommandLineOptionsBuilder
    },
};
use circuits::{
    bls_verification::bls12_381_circuit::BlsCircuitTargets,
    common_targets::BasicRecursiveInnerCircuitTarget,
    types::BalanceProof,
    withdrawal_credentials_balance_aggregator::{
        first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
        inner_level::WithdrawalCredentialsBalanceAggregatorInnerLevel,
    },
};
use colored::Colorize;
use num::BigUint;
use snowbridge_amcl::bls381::{big::Big, bls381::utils::hash_to_curve_g2, ecp2::ECP2};
use starky_bls12_381::native::{miller_loop, Fp, Fp12, Fp2};
use std::{
    ops::Neg,
    println,
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;

use futures_lite::future;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
    util::serialization::Buffer,
};

use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "bls_verification";

pub struct BLSData {
    dst: String,
    pubkey: String,
    signature: String,
    msg: String,
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("bls_verification")
        .with_redis_options(&common_config.redis_host, common_config.redis_port)
        .with_work_queue_options()
        .with_proof_storage_options()
        .get_matches();

    let config = parse_bls_verification_command_line_options(&matches);

    println!("{}", "Connecting to Redis...".yellow());
    let client = redis::Client::open(config.redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let mut proof_storage = create_proof_storage(&matches).await;

    println!("{}", "Loading circuit data...".yellow());
    let circuit_data = load_circuit_data(&format!(
        "{}/{}",
        SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME
    ))?;

    let protocol = matches.value_of("protocol").unwrap();

    let queue = WorkQueue::new(KeyPrefix::new(format!(
        "{}:{}",
        protocol, DB_CONSTANTS.bls_verification_queue
    )));

    println!(
        "{}",
        &format!(
            "{}:{}",
            protocol, DB_CONSTANTS.bls_verification_queue
        )
    );

    let start: Instant = Instant::now();
    process_queue(
        &mut con,
        proof_storage.as_mut(),
        &queue,
        config.stop_after,
        config.lease_for,
    )
    .await
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    stop_after: u64,
    lease_for: u64,
) -> Result<()> {
    loop {
        let queue_item = match queue
            .lease(
                con,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        {
            Some(item) => item,
            None => {
                println!("{}", "No tasks left in queue".bright_green().bold());

                return Ok(());
            }
        };

        if queue_item.data.is_empty() {
            println!("{}", "Skipping empty data task".yellow());
            queue.complete(con, &queue_item).await?;

            continue;
        }

        let data = prepare_bls_prover(queue_item).await;

        println!(
            "{}",
            format!(
                "Processing task for index {}...",
                data.pubkey.to_string().magenta()
            )
            .blue()
            .bold()
        );

        let proof = prove_bls(data).await;

        proof_storage
            .set_proof("bls12_381_proof".to_string(), &proof.unwrap().to_bytes())
            .await?;
    }
}

async fn prepare_bls_prover(queue_item: Item) -> BLSData {
    BLSData {
        dst: BLS_DST.to_string(),
        pubkey: std::str::from_utf8(&queue_item.data[0..8])
            .unwrap()
            .to_string(),
        signature: std::str::from_utf8(&queue_item.data[8..16])
            .unwrap()
            .to_string(),
        msg: std::str::from_utf8(&queue_item.data[16..24])
            .unwrap()
            .to_string(),
    }
}

async fn prove_bls(
    data: BLSData,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>, anyhow::Error> {
    let dst = data.dst.as_str();
    let pubkey = data.pubkey.as_str();
    let signature = data.signature.as_str();
    let msg = data.msg.as_str();

    let message_g2 = hash_to_curve_g2(&hex::decode(msg).unwrap(), dst.as_bytes());
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

    //TODO: From here

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

    Ok(proof)
}

async fn handle_final_exponentiation(
    fp12_mull: &Fp12,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
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

async fn handle_fp12_mul(
    miller_loop1: &Fp12,
    miller_loop2: &Fp12,
) -> ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> {
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

async fn handle_miller_loop(
    pubkey_g1: &G1Affine,
    message_g2: &G2Affine,
    neg_g1: &G1Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
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

async fn handle_pairing_precomp(
    message_g2: &G2Affine,
    signature_g2: &G2Affine,
) -> (
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
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
    Fq::from_be_bytes_mod_order(bytes) //TODO: Why?
}
