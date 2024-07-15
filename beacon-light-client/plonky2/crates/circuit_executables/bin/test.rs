#![feature(sync_unsafe_cell)]

use std::{cmp::min, fs, sync::Arc};

use anyhow::Result;
use circuit::Circuit;
use circuit_executables::{
    crud::proof_storage::{file_proof_storage::FileStorage, proof_storage::ProofStorage},
    utils::{parse_config_file, CommandLineOptionsBuilder},
};
use circuits::validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel;
use clap::Arg;
use futures::future::join_all;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};
use tokio::sync::Mutex;

#[tokio::main]
pub async fn main() -> Result<()> {
    let config = parse_config_file("../../common_config.json".to_owned())?;

    let matches = CommandLineOptionsBuilder::new("commitment_mapper")
        .with_redis_options(&config.redis_host, config.redis_port, &config.redis_auth)
        .with_work_queue_options()
        .with_proof_storage_options()
        .arg(
            Arg::with_name("range_start")
                .long("range-start")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("range_end")
                .long("range-end")
                .takes_value(true),
        )
        .get_matches();

    let validators_count = 1450498;

    let range_start = matches.value_of("range_start").unwrap().parse::<usize>()?;

    let range_end = matches
        .value_of("range_end")
        .unwrap_or(&(validators_count - 1).to_string())
        .parse::<usize>()?;

    let range_end = min(range_end, validators_count);

    assert!(range_start <= range_end);
    let total_proofs_to_check_count = range_end - range_start + 1;

    let (_target_fl, data_fl) = ValidatorsCommitmentMapperFirstLevel::build(&());

    println!("Processing range [{range_start}, {range_end}]");

    let slot = 9332704;

    let non_verifiable_indices = Arc::new(Mutex::new(Vec::new()));
    let missing_indices = Arc::new(Mutex::new(Vec::new()));
    let data_fl = Arc::new(data_fl);
    let checked_proofs_count = Arc::new(Mutex::new(0));

    let proof_dir_string = matches.value_of("folder_name").unwrap().to_owned();
    let proof_dir_str: &'static str = Box::leak(proof_dir_string.into_boxed_str());

    let futures = (range_start..min(range_end + 1, validators_count)).map(|validator_index| {
        let non_verifiable_indices_arc = non_verifiable_indices.clone();
        let missing_indices_arc = missing_indices.clone();
        let data_fl = data_fl.clone();
        let checked_proofs_count_arc = checked_proofs_count.clone();

        tokio::spawn(async move {
            let mut proof_storage = FileStorage::new(proof_dir_str.to_owned());

            let proof_bytes_result = {
                let gindex = 2usize.pow(40) + validator_index;
                let proof_key = format!("validator_proof_storage:{gindex}:{slot}");
                proof_storage.get_proof(proof_key.to_string()).await
            };

            match proof_bytes_result {
                Ok(proof_bytes) => {
                    let proof = ProofWithPublicInputs::<
                        GoldilocksField,
                        PoseidonGoldilocksConfig,
                        2,
                    >::from_bytes(proof_bytes, &data_fl.common).unwrap();

                    if data_fl.verify(proof).is_err() {
                        let mut non_verifiable_indices = non_verifiable_indices_arc.lock().await;
                        non_verifiable_indices.push(validator_index);
                    }
                }
                Err(_) => {
                    let mut missing_indices = missing_indices_arc.lock().await;
                    missing_indices.push(validator_index);
                }
            };

            let mut checked_proofs_count = checked_proofs_count_arc.lock().await;
            *checked_proofs_count += 1;

            if *checked_proofs_count % 10000 == 0 {
                println!("Checked: {checked_proofs_count}/{total_proofs_to_check_count}");
            }
        })
    });

    join_all(futures).await;

    let verifiable_proofs_count =
        total_proofs_to_check_count - non_verifiable_indices.lock().await.len();

    println!("Stats for range [{range_start}, {range_end}]");
    println!("Verifiable proofs count: {verifiable_proofs_count}/{total_proofs_to_check_count}");

    let mut non_verifiable_indices =
        Mutex::into_inner(Arc::try_unwrap(non_verifiable_indices).unwrap());
    non_verifiable_indices.sort();

    let mut missing_indices = Mutex::into_inner(Arc::try_unwrap(missing_indices).unwrap());
    missing_indices.sort();

    println!("non_verifiable_indices: {non_verifiable_indices:?}");
    println!("missing_indices: {missing_indices:?}");

    fs::create_dir_all("output").unwrap();

    fs::write(
        format!("output/non_verifiable_indices_{range_start}_{range_end}_{slot}.txt"),
        format!("{:#?}", non_verifiable_indices),
    )
    .unwrap();

    fs::write(
        format!("output/missing_indices_{range_start}_{range_end}_{slot}.txt"),
        format!("{:#?}", missing_indices),
    )
    .unwrap();

    Ok(())
}
