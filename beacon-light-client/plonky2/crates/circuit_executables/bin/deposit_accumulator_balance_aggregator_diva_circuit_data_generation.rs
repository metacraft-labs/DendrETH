#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use anyhow::Result;
use circuit::Circuit;
use circuit_executables::{
    cached_circuit_build::serialize_recursive_circuit_single_level,
    utils::CommandLineOptionsBuilder,
};
use circuits::deposit_accumulator_balance_aggregator_diva::{
    first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
    inner_level::DepositAccumulatorBalanceAggregatorDivaInnerLevel,
};
use std::println;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "deposit_accumulator_balance_aggregator_diva";
const RECURSION_DEPTH: usize = 32;

fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new(
        "deposit_accumulator_balance_aggregator_diva_circuit_data_generation",
    )
    .with_serialized_circuits_dir()
    .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    println!("Building level 0 circuit...");

    let (first_level_target, first_level_data) =
        DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&());

    println!("Serializing level 0 circuit...");
    serialize_recursive_circuit_single_level(
        &first_level_target,
        &first_level_data,
        serialized_circuits_dir,
        CIRCUIT_NAME,
        0,
    );

    let mut prev_circuit_data = first_level_data;

    for current_level in 1..=RECURSION_DEPTH {
        println!("Building level {current_level} circuit...");

        let (inner_target, inner_data) =
            DepositAccumulatorBalanceAggregatorDivaInnerLevel::build(&prev_circuit_data);

        println!("Serializing level {current_level} circuit...");
        serialize_recursive_circuit_single_level(
            &inner_target,
            &inner_data,
            serialized_circuits_dir,
            CIRCUIT_NAME,
            current_level,
        );

        prev_circuit_data = inner_data;
    }

    Ok(())
}
