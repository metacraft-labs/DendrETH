#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use anyhow::Result;
use circuit::Circuit;
use circuit_executables::cached_circuit_build::{
    serialize_recursive_circuit_single_level, SERIALIZED_CIRCUITS_DIR,
};
use circuits::deposit_accumulator_balance_aggregator_diva::{
    first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
    inner_level::DepositAccumulatorBalanceAggregatorDivaInnerLevel,
};
use clap::{App, Arg};
use itertools::Itertools;
use std::{fs, println};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "deposit_accumulator_balance_aggregator_diva";
const RECURSION_DEPTH: usize = 32;

fn main() -> Result<()> {
    let matches = App::new("")
        .arg(
            Arg::with_name("levels")
                .long("levels")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();

    let levels_arg: &String = matches.get_one("levels").unwrap();

    let levels = levels_arg
        .split_terminator(',')
        .map(|level_str| level_str.parse::<usize>().unwrap())
        .collect_vec();

    let max_level = *levels.iter().max().unwrap_or(&RECURSION_DEPTH);

    println!("Building level 0 circuit...");

    fs::create_dir_all(SERIALIZED_CIRCUITS_DIR).unwrap();

    let (first_level_target, first_level_data) =
        DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());

    if levels.is_empty() || levels.contains(&0) {
        println!("Serializing level 0 circuit...");
        serialize_recursive_circuit_single_level(
            &first_level_target,
            &first_level_data,
            CIRCUIT_NAME,
            0,
        );
    }

    let mut prev_circuit_data = first_level_data;

    for current_level in 1..=max_level {
        println!("Building level {current_level} circuit...");

        let (inner_target, inner_data) =
            DepositAccumulatorBalanceAggregatorDivaInnerLevel::build(&prev_circuit_data);

        if levels.is_empty() || levels.contains(&current_level) {
            println!("Serializing level {current_level} circuit...");
            serialize_recursive_circuit_single_level(
                &inner_target,
                &inner_data,
                CIRCUIT_NAME,
                current_level,
            );
        }

        prev_circuit_data = inner_data;
    }

    Ok(())
}
