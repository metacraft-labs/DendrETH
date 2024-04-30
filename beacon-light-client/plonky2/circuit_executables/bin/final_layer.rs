use circuit_executables::{
    crud::{
        common::{
            fetch_final_layer_input, fetch_proof, fetch_proof_balances, load_circuit_data,
            save_final_proof,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    utils::{parse_config_file, CommandLineOptionsBuilder},
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{
    circuit_input_common::{BalanceProof, SetPWValues, ValidatorProof},
    final_layer::build_final_circuit::build_final_circuit,
    serialization::generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    utils::utils::bits_to_bytes,
};
use colored::Colorize;
use std::{fs, marker::PhantomData, println, time::Instant};

use anyhow::Result;
use futures_lite::future;
use num_traits::ToPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};

const CIRCUIT_DIR: &str = "circuits";

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("final_layer")
        .with_redis_options(&common_config.redis_host, common_config.redis_port)
        .with_proof_storage_options()
        .with_protocol_options()
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let mut proof_storage = create_proof_storage(&matches).await;

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!(
        "{}",
        format!("Redis connection took: {:?}", elapsed).yellow()
    );

    let balance_data =
        load_circuit_data(&format!("{}/balance_verification_37", CIRCUIT_DIR)).unwrap();
    let commitment_data =
        load_circuit_data(&format!("{}/commitment_mapper_40", CIRCUIT_DIR)).unwrap();

    let (circuit_targets, circuit_data) = build_final_circuit::<1>(&balance_data, &commitment_data);

    let protocol = matches.value_of("protocol").unwrap();

    let final_input_data = fetch_final_layer_input(&mut con, protocol).await?;

    let mut pw: PartialWitness<GoldilocksField> = PartialWitness::new();

    circuit_targets.set_pw_values(&mut pw, &final_input_data);

    let balance_proof: BalanceProof = fetch_proof_balances(&mut con, protocol, 37, 0).await?;
    let balance_proof_bytes = proof_storage.get_proof(balance_proof.proof_key).await?;

    let balance_final_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            balance_proof_bytes,
            &balance_data.common,
        )?;

    pw.set_proof_with_pis_target(
        &circuit_targets.balance_circuit_targets.proof,
        &balance_final_proof,
    );

    pw.set_cap_target(
        &circuit_targets
            .balance_circuit_targets
            .verifier_circuit_target
            .constants_sigmas_cap,
        &balance_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        circuit_targets
            .balance_circuit_targets
            .verifier_circuit_target
            .circuit_digest,
        balance_data.verifier_only.circuit_digest,
    );

    let commitment_proof: ValidatorProof =
        fetch_proof(&mut con, 1, final_input_data.slot.to_u64().unwrap()).await?;

    let commitment_proof_bytes = proof_storage.get_proof(commitment_proof.proof_key).await?;

    let commitment_final_proof = ProofWithPublicInputs::<
        GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >::from_bytes(commitment_proof_bytes, &commitment_data.common)?;

    pw.set_proof_with_pis_target(
        &circuit_targets.commitment_mapper_circuit_targets.proof,
        &commitment_final_proof,
    );

    pw.set_cap_target(
        &circuit_targets
            .commitment_mapper_circuit_targets
            .verifier_circuit_target
            .constants_sigmas_cap,
        &commitment_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        circuit_targets
            .commitment_mapper_circuit_targets
            .verifier_circuit_target
            .circuit_digest,
        commitment_data.verifier_only.circuit_digest,
    );

    let proof = circuit_data.prove(pw)?;

    save_final_proof(
        &mut con,
        protocol.to_string(),
        &proof,
        hex::encode(bits_to_bytes(&final_input_data.block_root)),
        balance_proof.withdrawal_credentials,
        balance_proof.range_total_value,
        balance_proof.number_of_non_activated_validators,
        balance_proof.number_of_active_validators,
        balance_proof.number_of_non_activated_validators,
    )
    .await?;

    println!(
        "{}",
        format!(
            "Proof size: {}",
            proof.to_bytes().len().to_string().magenta()
        )
        .blue()
        .bold()
    );

    println!("{}", "Final proof saved!".blue().bold());

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let circuit_data_bytes = circuit_data
        .to_bytes(&gate_serializer, &generator_serializer)
        .unwrap();

    fs::write("circuits/final_layer.plonky2_circuit", circuit_data_bytes).unwrap();

    println!("{}", "Circuit data saved!".blue().bold());

    println!("{}", "Running wrapper...".blue().bold());

    wrap_final_layer_in_poseidon_bn_128(con, false, circuit_data, proof, protocol.to_string())
        .await?;

    println!("{}", "Wrapper finished!".blue().bold());

    Ok(())
}
