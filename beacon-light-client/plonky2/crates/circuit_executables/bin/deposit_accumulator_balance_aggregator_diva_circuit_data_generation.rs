#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit::Circuit;
use circuit_executables::cached_circuit_build::build_recursive_circuit_single_level_cached;
use circuits::deposit_accumulator_balance_aggregator_diva::{
    first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
    inner_level::DepositAccumulatorBalanceAggregatorDivaInnerLevel,
};
use std::println;


use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "deposit_accumulator_balance_aggregator_diva";

fn main() {
    let mut circuit_data = Vec::new();

    println!("Building 0 level circuit data...");

    let first_level = build_recursive_circuit_single_level_cached(
        CIRCUIT_NAME,
        0,
        &|| DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&()),
    );

    circuit_data.push(first_level.1);

    for i in 1..32 {
        println!("Building {} level circuit data...", i);

        let circuit = build_recursive_circuit_single_level_cached(
            CIRCUIT_NAME,
            i,
            &|| DepositAccumulatorBalanceAggregatorDivaInnerLevel::build(&circuit_data[i - 1]),
        );

        circuit_data.push(circuit.1);
    }
}
