use anyhow::Result;
use circuit::{serde_circuit_target::deserialize_circuit_target, CircuitTargetType};
use circuits::validators_commitment_mapper::{
    first_level::ValidatorsCommitmentMapperFirstLevel,
    inner_level::ValidatorsCommitmentMapperInnerLevel,
};
use itertools::Itertools;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis::aio::Connection;
use redis_work_queue::{KeyPrefix, WorkQueue};

use crate::{
    cached_circuit_build::SERIALIZED_CIRCUITS_DIR,
    crud::{
        common::{load_circuit_data, read_from_file},
        proof_storage::proof_storage::ProofStorage,
    },
    db_constants::DB_CONSTANTS,
};

const CIRCUIT_NAME: &str = "commitment_mapper";

pub struct CommitmentMapperContext {
    pub redis_con: Connection,
    pub work_queues: Vec<WorkQueue>,
    pub work_queue_cfg: WorkQueueConfig,
    pub proof_storage: Box<dyn ProofStorage>,
    pub first_level_circuit: FirstLevelCircuit,
    pub inner_level_circuits: Vec<InnerLevelCircuit>,
}

impl CommitmentMapperContext {
    pub async fn new(
        redis_uri: &str,
        work_queue_cfg: WorkQueueConfig,
        proof_storage: Box<dyn ProofStorage>,
        serialized_circuits_dir: &str,
    ) -> Result<Self> {
        let client = redis::Client::open(redis_uri)?;
        let redis_con = client.get_async_connection().await?;

        let work_queues = (0..=40)
            .map(|depth| {
                let key_prefix_str = format!("{}:{}", DB_CONSTANTS.validator_proofs_queue, depth);
                WorkQueue::new(KeyPrefix::new(key_prefix_str))
            })
            .collect_vec();

        let first_level_circuit = FirstLevelCircuit {
            targets: get_first_level_targets(serialized_circuits_dir)?,
            data: load_circuit_data(&get_circuit_data_path(serialized_circuits_dir, 0))?,
        };

        let mut inner_level_circuits: Vec<InnerLevelCircuit> = Vec::new();
        for i in 1..41 {
            let inner_level_circuit = InnerLevelCircuit {
                targets: get_inner_targets(serialized_circuits_dir, i)?,
                data: load_circuit_data(&get_circuit_data_path(serialized_circuits_dir, i))?,
            };
            inner_level_circuits.push(inner_level_circuit);
        }

        let ctx = Self {
            redis_con,
            work_queues,
            work_queue_cfg,
            proof_storage,
            first_level_circuit,
            inner_level_circuits,
        };

        Ok(ctx)
    }
}

pub struct WorkQueueConfig {
    pub stop_after: u64,
    pub lease_for: u64,
}

pub struct FirstLevelCircuit {
    pub targets: CircuitTargetType<ValidatorsCommitmentMapperFirstLevel>,
    pub data: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
}

pub struct InnerLevelCircuit {
    pub targets: CircuitTargetType<ValidatorsCommitmentMapperInnerLevel>,
    pub data: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
}

fn get_circuit_data_path(exec_dir: &str, level: usize) -> String {
    return format!("{exec_dir}/{SERIALIZED_CIRCUITS_DIR}/{CIRCUIT_NAME}_{level}");
}

fn get_circuit_target_path(exec_dir: &str, level: usize) -> String {
    return format!("{exec_dir}/{SERIALIZED_CIRCUITS_DIR}/{CIRCUIT_NAME}_{level}.plonky2_targets");
}

fn get_inner_targets(
    serialized_circuits_dir: &str,
    i: usize,
) -> Result<CircuitTargetType<ValidatorsCommitmentMapperInnerLevel>> {
    let target_bytes = read_from_file(&get_circuit_target_path(serialized_circuits_dir, i))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(
        deserialize_circuit_target::<ValidatorsCommitmentMapperInnerLevel>(&mut target_buffer)
            .unwrap(),
    )
}

fn get_first_level_targets(
    serialized_circuits_dir: &str,
) -> Result<CircuitTargetType<ValidatorsCommitmentMapperFirstLevel>> {
    let target_bytes = read_from_file(&get_circuit_target_path(serialized_circuits_dir, 0))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(
        deserialize_circuit_target::<ValidatorsCommitmentMapperFirstLevel>(&mut target_buffer)
            .unwrap(),
    )
}
