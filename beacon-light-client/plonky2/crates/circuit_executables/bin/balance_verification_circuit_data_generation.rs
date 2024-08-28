#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use anyhow::{bail, Result};
use circuit::Circuit;
use circuit_executables::{
    cached_circuit_build::serialize_recursive_circuit_single_level,
    utils::CommandLineOptionsBuilder,
};
use circuits::withdrawal_credentials_balance_aggregator::{
    first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
    inner_level::WithdrawalCredentialsBalanceAggregatorInnerLevel,
};
use num::clamp;

use clap::Arg;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "balance_verification";

// NOTE: config for the withdrawal credentials circuit
const VALIDATORS_COUNT: usize = 8;
const WITHDRAWAL_CREDENTIALS_COUNT: usize = 1;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("balance_verification_circuit_data_generation")
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

    if level != None && level.unwrap() > 37 {
        bail!(
            "Supplied level {} is larger than the maximum allowed level 37",
            level.unwrap()
        );
    }

    let (first_level_target, first_level_data) = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::build(&());

    if level == None || level == Some(0) {
        serialize_recursive_circuit_single_level(
            &first_level_target,
            &first_level_data,
            serialized_circuits_dir,
            CIRCUIT_NAME,
            0,
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
        if level == Some(i) || level == None {
            let (target, data) = WithdrawalCredentialsBalanceAggregatorInnerLevel::<
                VALIDATORS_COUNT,
                WITHDRAWAL_CREDENTIALS_COUNT,
            >::build(&prev_circuit_data);

            serialize_recursive_circuit_single_level(
                &target,
                &data,
                serialized_circuits_dir,
                CIRCUIT_NAME,
                i,
            );
            prev_circuit_data = data;
        }

        if level == Some(i) {
            return Ok(());
        }
    }

    Ok(())
}
