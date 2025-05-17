#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit::{set_witness::SetWitness, Circuit, CircuitDataType, CircuitTargetType};
use circuit_executables::{
    cached_circuit_build::build_recursive_circuit_single_level_cached,
    constants::VALIDATOR_REGISTRY_LIMIT,
    crud::{
        common::{
            delete_balance_verification_diva_proof_dependencies, fetch_proofs_balances,
            fetch_validator_balance_aggregator_input, save_balance_aggregator_proof,
        },
        proof_storage::{get_storage_key_from_matches, MetadataBlobStorage, ProofStorage},
    },
    db_constants::DB_CONSTANTS,
    provers::prove_inner_level,
    utils::{parse_balance_verification_command_line_options, CommandLineOptionsBuilder},
};
use circuits::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    deposit_accumulator_balance_aggregator_diva::{
        first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
        inner_level::DepositAccumulatorBalanceAggregatorDivaInnerLevel,
    },
    redis_storage_types::DepositAccumulatorBalanceAggregatorDivaProofData,
};
use colored::Colorize;
use std::{
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
};

use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "deposit_accumulator_balance_aggregator_diva";

#[allow(clippy::large_enum_variant)]
enum Targets {
    FirstLevel(Option<CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>>),
    InnerLevel(Option<BasicRecursiveInnerCircuitTarget>),
}

//#[allow(clippy::large_enum_variant)]
//enum CircuitState {
//    None,
//    FirstLevel(
//        CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
//        CircuitDataType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
//    ),
//    InnerLevel(
//        CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
//        CircuitDataType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
//    ),
//}

struct FirstLevelCircuitState {
    target: CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    data: CircuitDataType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
}

struct InnerLevelCircuitState {
    target: CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
    data: CircuitDataType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
    verifier_data: CircuitDataType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
}

enum CircuitState {
    FirstLevel(FirstLevelCircuitState),
    InnerLevel(InnerLevelCircuitState),
}

impl CircuitState {
    pub fn into_inner_level_state(self) -> InnerLevelCircuitState {
        match self {
            Self::FirstLevel(_) => unreachable!(),
            Self::InnerLevel(state) => state,
        }
    }

    pub fn get_data_owned(
        self,
    ) -> CircuitDataType<DepositAccumulatorBalanceAggregatorDivaFirstLevel> {
        match self {
            CircuitState::FirstLevel(FirstLevelCircuitState { data, .. }) => data,
            CircuitState::InnerLevel(InnerLevelCircuitState { data, .. }) => data,
        }
    }
}

#[derive(Default)]
struct State {
    current_level: usize,
    circuit: Option<Box<CircuitState>>,
    is_outdated: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            current_level: 0,
            circuit: None,
            is_outdated: true,
        }
    }

    fn get_circuit(&self) -> &CircuitState {
        self.circuit.as_ref().unwrap()
    }

    pub fn load_circuit_cached(&mut self, serialized_circuits_dir: &str) {
        if !self.is_outdated {
            return;
        }

        if self.current_level == 0 {
            let (target, data) = build_recursive_circuit_single_level_cached(
                serialized_circuits_dir,
                CIRCUIT_NAME,
                self.current_level,
                &|| DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&()),
            );

            self.circuit = Some(Box::new(CircuitState::FirstLevel(FirstLevelCircuitState {
                target,
                data,
            })));
        } else {
            let previous_circuit_box = self
                .circuit
                .take()
                .expect("Circuit state should exist when current_level > 0");

            let child_circuit_data = previous_circuit_box.get_data_owned();

            let (target, data) = build_recursive_circuit_single_level_cached(
                serialized_circuits_dir,
                CIRCUIT_NAME,
                self.current_level,
                &|| DepositAccumulatorBalanceAggregatorDivaInnerLevel::build(&child_circuit_data),
            );

            self.circuit = Some(Box::new(CircuitState::InnerLevel(InnerLevelCircuitState {
                target,
                data,
                verifier_data: child_circuit_data,
            })));
        }

        self.is_outdated = false;
    }

    pub fn transition_to_next_level(&mut self) {
        self.current_level += 1;
        self.is_outdated = true;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("balance_verification")
        .with_balance_verification_options()
        .with_work_queue_options()
        .with_serialized_circuits_dir()
        .with_proof_storage_config()
        .add_proof_storage(None)
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let config = parse_balance_verification_command_line_options(&matches);

    let storage_config_filepath = matches.get_one::<String>("proof_storage_cfg").unwrap();
    let storage_key = get_storage_key_from_matches(&matches, None)?;
    let mut storage = MetadataBlobStorage::from_file(storage_config_filepath, storage_key).await?;

    let protocol = matches.value_of("protocol").unwrap();

    println!(
        "{}",
        &format!(
            "{}:{}:{}",
            protocol, DB_CONSTANTS.deposit_balance_verification_queue, config.circuit_level
        )
    );

    execute_tasks_by_level_ascending(&mut storage, protocol, serialized_circuits_dir, false).await;
    Ok(())

    //let start: Instant = Instant::now();
    //process_queue(
    //    &mut storage,
    //    &queue,
    //    &circuit_data,
    //    &inner_circuit_data,
    //    &targets,
    //    config.circuit_level,
    //    start,
    //    config.time_to_run,
    //    config.stop_after,
    //    config.lease_for,
    //    config.preserve_intermediary_proofs,
    //    protocol,
    //)
    //.await
}

async fn execute_tasks_by_level_ascending(
    storage: &mut MetadataBlobStorage,
    protocol: &str,
    serialized_circuits_dir: &str,
    preserve_intermediary_proofs: bool,
) {
    let mut state = State::new();

    while state.current_level < 37 {
        let first_level_prefix = format!(
            "{}:{}:{}",
            protocol, DB_CONSTANTS.deposit_balance_verification_queue, state.current_level,
        );

        let work_queue = WorkQueue::new(KeyPrefix::new(first_level_prefix));

        let queue_len = work_queue.queue_len(&mut storage.metadata).await.unwrap();

        if queue_len > 0 {
            // Process tasks on level `current_level`
            state.load_circuit_cached(serialized_circuits_dir);

            let Some(item) = work_queue
                .lease(&mut storage.metadata, None, Duration::from_secs(5))
                .await
                .unwrap()
            else {
                continue;
            };

            match state.get_circuit() {
                CircuitState::FirstLevel(first_level_state) => {
                    handle_first_level_task(
                        storage,
                        &work_queue,
                        item,
                        protocol,
                        &first_level_state.data,
                        &first_level_state.target,
                    )
                    .await
                }
                CircuitState::InnerLevel(inner_level_state) => {
                    handle_inner_level_task(
                        storage,
                        &work_queue,
                        item,
                        &inner_level_state.data,
                        &inner_level_state.target,
                        &inner_level_state.verifier_data,
                        state.current_level,
                        preserve_intermediary_proofs,
                        protocol,
                    )
                    .await
                }
            }
        } else {
            // Move to the next level
            state.transition_to_next_level();
        }
    }
}

async fn handle_first_level_task(
    storage: &mut MetadataBlobStorage,
    queue: &WorkQueue,
    queue_item: Item,
    protocol: &str,
    circuit_data: &CircuitDataType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    circuit_target: &CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
) {
    let res = process_first_level_task(
        &mut storage.metadata,
        storage.blob.as_mut(),
        queue,
        queue_item,
        circuit_data,
        circuit_target,
        protocol,
    )
    .await;

    if let Err(err) = res {
        println!(
            "{}",
            format!("Error processing first level task {:?}", err)
                .red()
                .bold()
        );
    };
}

async fn handle_inner_level_task(
    storage: &mut MetadataBlobStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitDataType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
    circuit_target: &CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
    verifier_data: &CircuitDataType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    level: usize,
    preserve_intermediary_proofs: bool,
    protocol: &str,
) {
    let res = process_inner_level_job(
        &mut storage.metadata,
        storage.blob.as_mut(),
        queue,
        queue_item,
        circuit_data,
        verifier_data,
        circuit_target,
        level as u64,
        preserve_intermediary_proofs,
        protocol,
    )
    .await;

    if let Err(err) = res {
        println!(
            "{}",
            format!("Error processing inner level task {:?}", err)
                .red()
                .bold()
        );
    };
}

//async fn handle_work_queue_item(ctx: &mut CommitmentMapperContext, item: &VCMWorkQueueItem) {
//    match CommitmentMapperTask::deserialize(&item.item.data) {
//        Ok(task) => {
//            task.log();
//
//            match handle_task(ctx, task).await {
//                Ok(_) => complete_task(ctx, item).await,
//                Err(err) => log_error_and_wait(err),
//            }
//        }
//        Err(err) => {
//            complete_task(ctx, item).await;
//            println!("{}", format!("Error parsing task: {err}").bold().red());
//            println!("{}", format!("Got bytes: {:?}", item.item.data).red());
//        }
//    }
//}

#[allow(clippy::too_many_arguments)]
async fn process_queue(
    storage: &mut MetadataBlobStorage,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &Option<CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets,
    level: u64,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    preserve_intermediary_proofs: bool,
    protocol: &str,
) -> Result<()> {
    while time_to_run.is_none() || start.elapsed() < time_to_run.unwrap() {
        let queue_item = match queue
            .lease(
                &mut storage.metadata,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        {
            Some(item) => item,
            None => {
                println!("{}", "No tasks left in queue".bright_green().bold());

                return Ok(());
            }
        };

        if queue_item.data.is_empty() {
            println!("{}", "Skipping empty data task".yellow());
            queue.complete(&mut storage.metadata, &queue_item).await?;

            continue;
        }

        match targets {
            Targets::FirstLevel(targets) => {
                let res = process_first_level_task(
                    &mut storage.metadata,
                    storage.blob.as_mut(),
                    queue,
                    queue_item,
                    circuit_data,
                    targets.as_ref().unwrap(),
                    protocol,
                )
                .await;

                if let Err(err) = res {
                    println!(
                        "{}",
                        format!("Error processing first level task {:?}", err)
                            .red()
                            .bold()
                    );
                    continue;
                };
            }
            Targets::InnerLevel(inner_circuit_targets) => {
                let res = process_inner_level_job(
                    &mut storage.metadata,
                    storage.blob.as_mut(),
                    queue,
                    queue_item,
                    circuit_data,
                    inner_circuit_data.as_ref().unwrap(),
                    inner_circuit_targets.as_ref().unwrap(),
                    level,
                    preserve_intermediary_proofs,
                    protocol,
                )
                .await;

                if let Err(err) = res {
                    println!(
                        "{}",
                        format!("Error processing inner level task {:?}", err)
                            .red()
                            .bold()
                    );
                    continue;
                };
            }
        }
    }

    Ok(())
}

async fn process_first_level_task(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: &CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    protocol: &str,
) -> Result<()> {
    let balance_input_index = u64::from_be_bytes(queue_item.data[0..8].try_into().unwrap());

    if balance_input_index as usize != VALIDATOR_REGISTRY_LIMIT {
        println!(
            "{}",
            format!(
                "Processing task for index {}...",
                balance_input_index.to_string().magenta()
            )
            .blue()
            .bold()
        );
    } else {
        println!("{}", "Processing task for zero proof...".blue().bold());
    }

    let start = Instant::now();
    let validator_balance_input =
        fetch_validator_balance_aggregator_input(con, protocol.to_owned(), balance_input_index)
            .await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let mut pw = PartialWitness::new();
    targets.set_witness(&mut pw, &validator_balance_input);
    let proof = circuit_data.prove(pw)?;

    match save_balance_aggregator_proof(
        con,
        proof_storage,
        protocol.to_owned(),
        proof,
        0,
        balance_input_index,
    )
    .await
    {
        Err(err) => {
            println!(
                "{}",
                format!("Error while saving balance proof: {}", err)
                    .red()
                    .bold()
            );
            thread::sleep(Duration::from_secs(5));
            return Err(err);
        }
        Ok(_) => {
            queue.complete(con, &queue_item).await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_inner_level_job(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_target: &BasicRecursiveInnerCircuitTarget,
    level: u64,
    preserve_intermediary_proofs: bool,
    protocol: &str,
) -> Result<()> {
    let index = u64::from_be_bytes(queue_item.data[0..8].try_into().unwrap());

    if index as usize != VALIDATOR_REGISTRY_LIMIT {
        println!(
            "{}",
            format!(
                "Processing task for index {}...",
                index.to_string().magenta()
            )
            .blue()
            .bold()
        );
    } else {
        println!("{}", "Processing task for zero proof...".blue().bold());
    }

    match fetch_proofs_balances::<DepositAccumulatorBalanceAggregatorDivaProofData>(
        con,
        proof_storage,
        protocol.to_owned(),
        level,
        index,
    )
    .await
    {
        Err(err) => {
            println!(
                "{}",
                format!("Error while fetching balance proofs: {}", err)
                    .red()
                    .bold()
            );
            Err(err)
        }
        Ok(proofs) => {
            let proof = prove_inner_level(
                proofs.0,
                proofs.1,
                inner_circuit_data,
                inner_circuit_target,
                circuit_data,
            )?;

            match save_balance_aggregator_proof(
                con,
                proof_storage,
                protocol.to_owned(),
                proof,
                level,
                index,
            )
            .await
            {
                Err(err) => {
                    println!(
                        "{}",
                        format!("Error while saving balance proof: {}", err)
                            .red()
                            .bold()
                    );
                    thread::sleep(Duration::from_secs(5));
                    return Err(err);
                }
                Ok(_) => {
                    queue.complete(con, &queue_item).await?;
                    if !preserve_intermediary_proofs {
                        // delete child nodes
                        delete_balance_verification_diva_proof_dependencies(
                            con,
                            proof_storage,
                            protocol,
                            level,
                            index,
                        )
                        .await?;
                    }
                }
            }
            Ok(())
        }
    }
}
