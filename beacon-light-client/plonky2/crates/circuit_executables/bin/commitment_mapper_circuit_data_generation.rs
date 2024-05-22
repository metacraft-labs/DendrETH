use std::{fs, marker::PhantomData};

use anyhow::Result;
use circuits::{
    build_commitment_mapper_first_level_circuit::build_commitment_mapper_first_level_circuit,
    build_commitment_mapper_inner_level_circuit::build_commitment_mapper_inner_circuit,
    serialization::generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    serialization::targets_serialization::WriteTargets,
    validator_commitment_mapper::ValidatorCommitmentTargets,
};

use clap::{App, Arg};
use futures_lite::future;

use jemallocator::Jemalloc;
use plonky2::plonk::config::PoseidonGoldilocksConfig;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "commitment_mapper";

fn main() -> Result<()> {
    future::block_on(async_main())
}

pub async fn async_main() -> Result<()> {
    let matches = App::new("")
        .arg(
            Arg::with_name("circuit_level")
                .short('l')
                .long("level")
                .value_name("LEVEL")
                .help("Sets the circuit level")
                .takes_value(true)
                .default_value("all")
                .validator(|x| {
                    if x == "all" || x.parse::<usize>().is_ok() {
                        Ok(())
                    } else {
                        Err(String::from("The level must be a number or 'all'"))
                    }
                }),
        )
        .get_matches();

    let level = match matches.value_of("circuit_level").unwrap() {
        "all" => None,
        x => Some(x.parse::<usize>().unwrap()),
    };

    let (validators_balance_verification_targets, first_level_data) =
        build_commitment_mapper_first_level_circuit();

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    fs::create_dir_all(CIRCUIT_DIR).unwrap();

    if level == None || level == Some(0) {
        write_first_level_circuit(
            &first_level_data,
            &gate_serializer,
            &generator_serializer,
            validators_balance_verification_targets,
        );
    }

    if level == Some(0) {
        return Ok(());
    }

    let mut prev_circuit_data = first_level_data;

    for i in 1..41 {
        let (targets, data) = build_commitment_mapper_inner_circuit(&prev_circuit_data);

        if level == Some(i) || level == None {
            let circuit_bytes = data
                .to_bytes(&gate_serializer, &generator_serializer)
                .unwrap();

            write_to_file(
                &format!("{}/{}_{}.plonky2_circuit", CIRCUIT_DIR, CIRCUIT_NAME, i),
                &circuit_bytes,
            )
            .unwrap();

            let inner_level_targets = targets.write_targets().unwrap();

            write_to_file(
                &format!("{}/{}_{}.plonky2_targets", CIRCUIT_DIR, CIRCUIT_NAME, i),
                &inner_level_targets,
            )
            .unwrap();
        }

        if level == Some(i) {
            return Ok(());
        }

        prev_circuit_data = data;
    }

    Ok(())
}

fn write_first_level_circuit(
    first_level_data: &plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    gate_serializer: &DendrETHGateSerializer,
    generator_serializer: &DendrETHGeneratorSerializer<PoseidonGoldilocksConfig, 2>,
    validator_commitment_targets: ValidatorCommitmentTargets,
) {
    let circuit_bytes = first_level_data
        .to_bytes(gate_serializer, generator_serializer)
        .unwrap();

    write_to_file(
        &format!("{}/{}_{}.plonky2_circuit", CIRCUIT_DIR, CIRCUIT_NAME, 0),
        &circuit_bytes,
    )
    .unwrap();

    let validator_commitment_targets_bytes = validator_commitment_targets.write_targets().unwrap();

    write_to_file(
        &format!("{}/{}_{}.plonky2_targets", CIRCUIT_DIR, CIRCUIT_NAME, 0),
        &validator_commitment_targets_bytes,
    )
    .unwrap();
}
