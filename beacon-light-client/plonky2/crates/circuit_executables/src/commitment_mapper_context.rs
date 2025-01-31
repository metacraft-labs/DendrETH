use anyhow::Result;
use circuit::Circuit;
use circuits::validators_commitment_mapper::{
    first_level::ValidatorsCommitmentMapperFirstLevel,
    inner_level::ValidatorsCommitmentMapperInnerLevel,
};
use itertools::Itertools;
use redis_work_queue::{KeyPrefix, WorkQueue};

use crate::{
    cached_circuit_build::{build_recursive_circuit_cached, CircuitTargetAndData},
    crud::proof_storage::MetadataBlobStorage,
    db_constants::DB_CONSTANTS,
};

const CIRCUIT_NAME: &str = "commitment_mapper";
const DEPTH: usize = 40;

pub struct WorkQueueConfig {
    pub stop_after: u64,
    pub lease_for: u64,
}

pub struct CommitmentMapperContext {
    pub storage: MetadataBlobStorage,
    pub work_queues: Vec<WorkQueue>,
    pub work_queue_cfg: WorkQueueConfig,
    pub first_level_circuit: CircuitTargetAndData<ValidatorsCommitmentMapperFirstLevel>,
    pub inner_level_circuits: Vec<CircuitTargetAndData<ValidatorsCommitmentMapperInnerLevel>>,
}

impl CommitmentMapperContext {
    pub async fn new(
        work_queue_cfg: WorkQueueConfig,
        storage_cfg_filepath: &str,
        storage_name: &str,
        serialized_circuits_dir: &str,
    ) -> Result<Self> {
        let work_queues = (0..=DEPTH)
            .map(|depth| {
                let key_prefix_str = format!("{}:{}", DB_CONSTANTS.validator_proofs_queue, depth);
                WorkQueue::new(KeyPrefix::new(key_prefix_str))
            })
            .collect_vec();

        let (first_level_circuit, inner_level_circuits) = build_recursive_circuit_cached(
            serialized_circuits_dir,
            CIRCUIT_NAME,
            DEPTH,
            &|| ValidatorsCommitmentMapperFirstLevel::build(&()),
            &ValidatorsCommitmentMapperInnerLevel::build,
        );

        let storage = MetadataBlobStorage::from_file(storage_cfg_filepath, storage_name).await?;

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
