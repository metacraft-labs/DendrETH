#![feature(generic_const_exprs)]
use anyhow::{bail, Result};
use circuit::Circuit;
use circuits::{
    deposits_accumulator_balance_aggregator::{
        build_balance_accumulator_inner_level,
        build_validator_balance_accumulator_circuit::build_validator_balance_accumulator_circuit,
        validator_balance_circuit_accumulator::ValidatorBalanceVerificationAccumulatorTargets,
    },
    serialization::{
        generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
        targets_serialization::WriteTargets,
    },
    withdrawal_credentials_balance_aggregator::{
        first_level::circuit::ValidatorBalanceVerificationTargets,
        inner_level_circuit::{build_inner_level_circuit, BalanceInnerCircuitTargets},
        WithdrawalCredentialsBalanceAggregatorFirstLevel,
    },
};
use num::clamp;
use std::{fs, marker::PhantomData};

use clap::{App, Arg};
use futures_lite::future;

use jemallocator::Jemalloc;
use plonky2::plonk::config::PoseidonGoldilocksConfig;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_DIR: &str = "circuits";

fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}

enum ValidatorBalanceTargets<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:,
{
    ValidatorBalanceFirstLevel(
        ValidatorBalanceVerificationTargets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>,
    ),
    ValidatorBalanceAccumulatorFirstLevel(ValidatorBalanceVerificationAccumulatorTargets),
    ValidatorBalanceInnerLevel(BalanceInnerCircuitTargets),
    ValidatorBalanceAccumulatorInnerLevel(
        build_balance_accumulator_inner_level::BalanceInnerCircuitTargets,
    ),
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
        .arg(
            Arg::with_name("number_of_validators")
                .long("number_of_validators")
                .value_name("number")
                .help("Sets the number of validators")
                .takes_value(true)
                .default_value("2"),
        )
        .arg(
            Arg::with_name("circuit_name")
                .long("circuit_name")
                .value_name("name")
                .help("Sets the circuit name")
                .takes_value(true)
                .default_value("balance_verification"),
        )
        .get_matches();

    let level = match matches.value_of("circuit_level").unwrap() {
        "all" => None,
        x => Some(x.parse::<usize>().unwrap()),
    };

    let circuit_name = matches.value_of("circuit_name").unwrap().to_owned();

    if circuit_name != "balance_verification" && circuit_name != "balance_accumulator" {
        bail!("Invalid circuit name. Specify \"balance_verification\" or \"balance_accumulator\"");
    }

    let validators_len = if circuit_name == "balance_accumulator" {
        2
    } else {
        8
    };

    let (validators_balance_verification_targets, first_level_data) = if circuit_name
        == "balance_accumulator"
    {
        println!("building accumulator");
        let (targets, data) = build_validator_balance_accumulator_circuit(validators_len);
        (
            ValidatorBalanceTargets::<8, 1>::ValidatorBalanceAccumulatorFirstLevel(targets),
            data,
        )
    } else {
        let (targets, data) = WithdrawalCredentialsBalanceAggregatorFirstLevel::<8, 1>::build(());
        (
            ValidatorBalanceTargets::ValidatorBalanceFirstLevel(targets),
            data,
        )
    };

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    if level != None && level.unwrap() > 37 {
        bail!(
            "Supplied level {} is larger than the maximum allowed level 37",
            level.unwrap()
        );
    }

    fs::create_dir_all(CIRCUIT_DIR).unwrap();

    if level == None || level == Some(0) {
        write_first_level_circuit(
            &first_level_data,
            &gate_serializer,
            &generator_serializer,
            validators_balance_verification_targets,
            &circuit_name,
        );
    }

    if level == Some(0) {
        return Ok(());
    }

    let max_level = if level == None {
        37
    } else {
        clamp(level.unwrap(), 1, 37)
    };

    let mut prev_circuit_data = first_level_data;

    for i in 1..=max_level {
        let (targets, data) = if circuit_name == "balance_accumulator" {
            let (targets, data) = build_balance_accumulator_inner_level::build_inner_level_circuit(
                &prev_circuit_data,
            );

            (
                ValidatorBalanceTargets::<8, 1>::ValidatorBalanceAccumulatorInnerLevel(targets),
                data,
            )
        } else {
            let (targets, data) = build_inner_level_circuit::<8, 1>(&prev_circuit_data);

            (
                ValidatorBalanceTargets::ValidatorBalanceInnerLevel(targets),
                data,
            )
        };

        if level == Some(i) || level == None {
            let circuit_bytes = data
                .to_bytes(&gate_serializer, &generator_serializer)
                .unwrap();

            write_to_file(
                &format!("{}/{}_{}.plonky2_circuit", CIRCUIT_DIR, circuit_name, i),
                &circuit_bytes,
            )
            .unwrap();

            let inner_level_targets = match targets {
                ValidatorBalanceTargets::ValidatorBalanceInnerLevel(targets) => {
                    targets.write_targets().unwrap()
                }
                ValidatorBalanceTargets::ValidatorBalanceAccumulatorInnerLevel(targets) => {
                    targets.write_targets().unwrap()
                }
                _ => unreachable!(),
            };

            write_to_file(
                &format!("{}/{}_{}.plonky2_targets", CIRCUIT_DIR, circuit_name, i),
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

fn write_first_level_circuit<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    first_level_data: &plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    gate_serializer: &DendrETHGateSerializer,
    generator_serializer: &DendrETHGeneratorSerializer<PoseidonGoldilocksConfig, 2>,
    validators_balance_verification_targets: ValidatorBalanceTargets<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >,
    circuit_name: &str,
) where
    [(); VALIDATORS_COUNT / 4]:,
{
    let circuit_bytes = first_level_data
        .to_bytes(gate_serializer, generator_serializer)
        .unwrap();

    write_to_file(
        &format!("{}/{}_0.plonky2_circuit", CIRCUIT_DIR, circuit_name),
        &circuit_bytes,
    )
    .unwrap();

    let validator_balance_verification_targets_bytes = match validators_balance_verification_targets
    {
        ValidatorBalanceTargets::ValidatorBalanceFirstLevel(targets) => {
            targets.write_targets().unwrap()
        }
        ValidatorBalanceTargets::ValidatorBalanceAccumulatorFirstLevel(targets) => {
            targets.write_targets().unwrap()
        }
        _ => unreachable!(),
    };

    write_to_file(
        &format!("{}/{}_0.plonky2_targets", CIRCUIT_DIR, circuit_name),
        &validator_balance_verification_targets_bytes,
    )
    .unwrap();
}
