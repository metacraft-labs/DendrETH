use circuit_executables::{
    cached_circuit_build::serialize_recursive_circuit_single_level,
    utils::CommandLineOptionsBuilder,
};
use circuits::validators_commitment_mapper::inner_level::ValidatorsCommitmentMapperInnerLevel;

use anyhow::Result;

use circuit::Circuit;
use circuits::validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel;
use clap::Arg;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "commitment_mapper";

fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("commitment_mapper_circuit_data_generation")
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
        .with_serialized_circuits_dir()
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let level = match matches.value_of("circuit_level").unwrap() {
        "all" => None,
        x => Some(x.parse::<usize>().unwrap()),
    };

    let (validators_balance_verification_targets, first_level_data) =
        ValidatorsCommitmentMapperFirstLevel::build(&());

    if level == None || level == Some(0) {
        serialize_recursive_circuit_single_level(
            &validators_balance_verification_targets,
            &first_level_data,
            serialized_circuits_dir,
            CIRCUIT_NAME,
            0,
        );
    }

    if level == Some(0) {
        return Ok(());
    }

    let mut prev_circuit_data = first_level_data;

    for current_level in 1..=40 {
        let (targets, data) = ValidatorsCommitmentMapperInnerLevel::build(&prev_circuit_data);

        if level == Some(current_level) || level == None {
            serialize_recursive_circuit_single_level(
                &targets,
                &data,
                serialized_circuits_dir,
                CIRCUIT_NAME,
                current_level,
            );

            println!("Wrote circuit and targets for level {current_level} in '{serialized_circuits_dir}'",);
        }

        if level == Some(current_level) {
            return Ok(());
        }

        prev_circuit_data = data;
    }

    Ok(())
}
