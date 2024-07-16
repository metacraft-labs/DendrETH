use circuit_executables::cached_circuit_build::SERIALIZED_CIRCUITS_DIR;
use circuits::validators_commitment_mapper::inner_level::ValidatorsCommitmentMapperInnerLevel;
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use std::{fs, marker::PhantomData};

use anyhow::Result;

use circuit::{Circuit, CircuitTargetType, SerdeCircuitTarget};
use circuits::validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel;
use clap::{App, Arg};

use jemallocator::Jemalloc;
use plonky2::plonk::config::PoseidonGoldilocksConfig;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}

const CIRCUIT_NAME: &str = "commitment_mapper";

fn main() -> Result<()> {
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
        ValidatorsCommitmentMapperFirstLevel::build(&());

    let gate_serializer = CustomGateSerializer;

    let generator_serializer = CustomGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    fs::create_dir_all(SERIALIZED_CIRCUITS_DIR).unwrap();

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
        let (targets, data) = ValidatorsCommitmentMapperInnerLevel::build(&prev_circuit_data);

        if level == Some(i) || level == None {
            let circuit_bytes = data
                .to_bytes(&gate_serializer, &generator_serializer)
                .unwrap();

            write_to_file(
                &format!(
                    "{}/{}_{}.plonky2_circuit",
                    SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME, i
                ),
                &circuit_bytes,
            )
            .unwrap();

            let inner_level_targets = targets.serialize().unwrap();

            write_to_file(
                &format!(
                    "{}/{}_{}.plonky2_targets",
                    SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME, i
                ),
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
    gate_serializer: &CustomGateSerializer,
    generator_serializer: &CustomGeneratorSerializer<PoseidonGoldilocksConfig, 2>,
    validator_commitment_targets: CircuitTargetType<ValidatorsCommitmentMapperFirstLevel>,
) {
    let circuit_bytes = first_level_data
        .to_bytes(gate_serializer, generator_serializer)
        .unwrap();

    write_to_file(
        &format!(
            "{}/{}_{}.plonky2_circuit",
            SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME, 0
        ),
        &circuit_bytes,
    )
    .unwrap();

    let validator_commitment_targets_bytes = validator_commitment_targets.serialize().unwrap();

    write_to_file(
        &format!(
            "{}/{}_{}.plonky2_targets",
            SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME, 0
        ),
        &validator_commitment_targets_bytes,
    )
    .unwrap();
}
