use anyhow::Result;
use circuits::{
    build_balance_inner_level_circuit::build_inner_level_circuit,
    build_validator_balance_circuit::build_validator_balance_circuit,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    targets_serialization::WriteTargets,
};
use num::clamp;
use std::{fs, marker::PhantomData, path::Path};

use clap::{App, Arg};
use futures_lite::future;

use jemallocator::Jemalloc;
use plonky2::plonk::config::PoseidonGoldilocksConfig;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "balance_verification";

fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}

fn main() -> Result<(), String> {
    future::block_on(async_main())
}

pub async fn async_main() -> Result<(), String> {
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
    let level_str = matches.value_of("circuit_level").unwrap();
    let level = match level_str {
        "all" => None,
        x => Some(x.parse::<usize>().unwrap()),
    };

    let (validators_balance_verification_targets, first_level_data) =
        build_validator_balance_circuit(8);

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    if level != None && level.unwrap() > 37 {
        return Err(String::from(format!(
            "Supplied level {} is larger than the maximum allowed level 37",
            level.unwrap()
        )));
    }

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
    let max_level = if level == None {
        37
    } else {
        clamp(level.unwrap(), 1, 37)
    };
    for i in 1..=max_level {
        let (targets, data) = build_inner_level_circuit::<1>(&prev_circuit_data);
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

    let mut exists = false;
    for i in 1..=max_level {
        if Path::new(&format!(
            "{}/{}_{}.plonky2_circuit",
            CIRCUIT_DIR, CIRCUIT_NAME, i
        ))
        .exists()
            || Path::new(&format!(
                "{}/{}_{}.plonky2_targets",
                CIRCUIT_DIR, CIRCUIT_NAME, i
            ))
            .exists()
        {
            exists = true;
            break;
        }
    }
    if !exists {
        return Err(String::from(format!(
            "No plonky2 output created. Level used was: {}",
            level_str
        )));
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
    validators_balance_verification_targets: circuits::validator_balance_circuit::ValidatorBalanceVerificationTargets<1>,
) {
    let circuit_bytes = first_level_data
        .to_bytes(gate_serializer, generator_serializer)
        .unwrap();

    write_to_file(
        &format!("{}/{}_0.plonky2_circuit", CIRCUIT_DIR, CIRCUIT_NAME),
        &circuit_bytes,
    )
    .unwrap();

    let validator_balance_verification_targets_bytes = validators_balance_verification_targets
        .write_targets()
        .unwrap();

    write_to_file(
        &format!("{}/{}_0.plonky2_targets", CIRCUIT_DIR, CIRCUIT_NAME),
        &validator_balance_verification_targets_bytes,
    )
    .unwrap();
}
