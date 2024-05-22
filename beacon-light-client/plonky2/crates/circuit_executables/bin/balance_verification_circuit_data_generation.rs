#![feature(generic_const_exprs)]
use anyhow::{bail, Result};
use circuit::{Circuit, SerdeCircuitTarget};
use circuits::{
    serialization::generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    withdrawal_credentials_balance_aggregator::{
        first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
        inner_level::WithdrawalCredentialsBalanceAggregatorInnerLevel,
    },
};
use num::clamp;
use std::{fs, marker::PhantomData};

use clap::{App, Arg};
use futures_lite::future;

use jemallocator::Jemalloc;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        circuit_data::CircuitData,
        config::{AlgebraicHasher, GenericConfig},
    },
};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// TODO: this needs to be moved to some central place
const CIRCUIT_DIR: &str = "circuits";

const CIRCUIT_NAME: &str = "balance_verification";

// NOTE: config for the withdrawal credentials circuit
const VALIDATORS_COUNT: usize = 8;
const WITHDRAWAL_CREDENTIALS_COUNT: usize = 1;

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

    if level != None && level.unwrap() > 37 {
        bail!(
            "Supplied level {} is larger than the maximum allowed level 37",
            level.unwrap()
        );
    }

    fs::create_dir_all(CIRCUIT_DIR).unwrap();

    let (first_level_target, first_level_data) = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::build(&());

    if level == None || level == Some(0) {
        serialize_recursive_circuit_single_level(
            &first_level_target,
            &first_level_data,
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

            serialize_recursive_circuit_single_level(&target, &data, CIRCUIT_NAME, i);
            prev_circuit_data = data;
        }

        if level == Some(i) {
            return Ok(());
        }
    }

    Ok(())
}

fn serialize_recursive_circuit_single_level<
    T: SerdeCircuitTarget,
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    const D: usize,
>(
    target: &T,
    circuit_data: &CircuitData<F, C, D>,
    circuit_name: &str,
    level: usize,
) where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<C>,
    };

    let data_bytes = circuit_data
        .to_bytes(&gate_serializer, &generator_serializer)
        .unwrap();

    fs::write(
        &format!("{}/{}_{}.plonky2_circuit", CIRCUIT_DIR, circuit_name, level),
        &data_bytes,
    )
    .unwrap();

    let target_bytes = target.serialize().unwrap();

    fs::write(
        &format!("{}/{}_{}.plonky2_targets", CIRCUIT_DIR, circuit_name, level),
        &target_bytes,
    )
    .unwrap();
}
