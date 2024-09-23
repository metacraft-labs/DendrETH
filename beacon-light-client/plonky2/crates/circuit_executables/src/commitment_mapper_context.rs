use anyhow::Result;
use circuits::validators_commitment_mapper::{
    first_level::ValidatorsCommitmentMapperFirstLevel,
    inner_level::ValidatorsCommitmentMapperInnerLevel,
};
use itertools::Itertools;
use redis_work_queue::{KeyPrefix, WorkQueue};

use crate::{
    cached_circuit_build::CircuitTargetAndData,
    crud::proof_storage::proof_storage::RedisBlobStorage, db_constants::DB_CONSTANTS,
};

const CIRCUIT_NAME: &str = "commitment_mapper";

pub struct WorkQueueConfig {
    pub stop_after: u64,
    pub lease_for: u64,
}

pub struct CommitmentMapperContext {
    pub storage: RedisBlobStorage,
    pub work_queues: Vec<WorkQueue>,
    pub work_queue_cfg: WorkQueueConfig,
    pub first_level_circuit: CircuitTargetAndData<ValidatorsCommitmentMapperFirstLevel>,
    pub inner_level_circuits: Vec<CircuitTargetAndData<ValidatorsCommitmentMapperInnerLevel>>,
}

impl CommitmentMapperContext {
    pub async fn new(
        work_queue_cfg: WorkQueueConfig,
        storage_cfg_filepath: &str,
        serialized_circuits_dir: &str,
    ) -> Result<Self> {
        let work_queues = (0..=40)
            .map(|depth| {
                let key_prefix_str = format!("{}:{}", DB_CONSTANTS.validator_proofs_queue, depth);
                WorkQueue::new(KeyPrefix::new(key_prefix_str))
            })
            .collect_vec();

        let first_level_circuit =
            CircuitTargetAndData::load_recursive(serialized_circuits_dir, CIRCUIT_NAME, 0)?;

        let mut inner_level_circuits = Vec::new();

        for level in 1..=40 {
            inner_level_circuits.push(CircuitTargetAndData::load_recursive(
                serialized_circuits_dir,
                CIRCUIT_NAME,
                level,
            )?);
        }

        let storage =
            RedisBlobStorage::from_file(storage_cfg_filepath, "validators-commitment-mapper")
                .await?;

        let ctx = Self {
            storage,
            work_queues,
            work_queue_cfg,
            first_level_circuit,
            inner_level_circuits,
        };

        Ok(ctx)
    }
}
