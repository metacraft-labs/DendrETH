use std::{fs, marker::PhantomData, ops::RangeInclusive, path::Path, process};
use num::clamp;
use anyhow::Result;
use circuits::{
    build_balance_inner_level_circuit::build_inner_level_circuit,
    build_validator_balance_circuit::build_validator_balance_circuit,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer}, targets_serialization::WriteTargets,
};

use clap::{App, Arg};
use futures_lite::future;

use jemallocator::Jemalloc;
use plonky2::{plonk::config::PoseidonGoldilocksConfig};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}

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
    let level_str = matches.value_of("circuit_level").unwrap();
    let level = match matches.value_of("circuit_level").unwrap() {
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
        eprintln!("\x1b[31mError: Supplied level {} is larger than the maximum allowed level 38\x1b[0m", level.unwrap());
        process::exit(1);
    }

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
    let level_range = if level == None { 1..=37 } else { RangeInclusive::new(1,clamp(level.unwrap(),1,37)) };
    for i in level_range {
        let (targets, data) = build_inner_level_circuit(&prev_circuit_data);
        println!("{}", i);
        if level == Some(i) || level == None {
            let circuit_bytes = data
                .to_bytes(&gate_serializer, &generator_serializer)
                .unwrap();

            write_to_file(&format!("{}.plonky2_circuit", i), &circuit_bytes).unwrap();

            let inner_level_targets = targets.write_targets().unwrap();

            write_to_file(&format!("{}.plonky2_targets", i), &inner_level_targets).unwrap();
        }

        if level == Some(i) {
            return Ok(());
        }

        prev_circuit_data = data;
    }

    let mut exists = false;
    for i in 1..=37 {
        if Path::new(&format!("{}.plonky2_circuit",i)).exists() || Path::new(&format!("{}.plonky2_targets",i)).exists() {
            exists = true;
        }
    }
    if !exists {
        eprintln!("\x1b[31mError: No plonky2 output created. Level used was: {}\x1b[0m", level_str);
        process::exit(1);
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
    validators_balance_verification_targets: circuits::validator_balance_circuit::ValidatorBalanceVerificationTargets,
) {
    let circuit_bytes = first_level_data
        .to_bytes(gate_serializer, generator_serializer)
        .unwrap();

    write_to_file(&format!("{}.plonky2_circuit", 0), &circuit_bytes).unwrap();

    let validator_balance_verification_targets_bytes = validators_balance_verification_targets.write_targets().unwrap();

    write_to_file(
        &format!("{}.plonky2_targets", 0),
        &validator_balance_verification_targets_bytes,
    )
    .unwrap();
}
