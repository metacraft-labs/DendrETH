use colored::Colorize;
use std::sync::Arc;

use anyhow::{bail, Result};
use circuit::{Array, Circuit, CircuitInput, SetWitness};
use circuits::{
    pubkey_commitment_mapper::{
        first_level::PubkeyCommitmentMapperFL, inner_level::PubkeyCommitmentMapperIL,
    },
    redis_storage_types::PubkeyCommitmentMapperRedisStorageData,
    utils::bits_to_bytes,
};
use futures::future::try_join_all;
use itertools::Itertools;
use plonky2::{
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, proof::ProofWithPublicInputs},
};
use redis::{aio::Connection, AsyncCommands, Pipeline};
use serde_json::json;

use crate::{
    cached_circuit_build::{build_recursive_circuit_cached, CircuitTargetAndData},
    crud::proof_storage::proof_storage::{ProofStorage, RedisBlobStorage},
    provers::prove_inner_level2,
};

pub const DEPTH: usize = 32;

type F = <PubkeyCommitmentMapperFL as Circuit>::F;
type C = <PubkeyCommitmentMapperFL as Circuit>::C;
const D: usize = <PubkeyCommitmentMapperFL as Circuit>::D;

pub type PubkeyCommitmentMapperProof = ProofWithPublicInputs<F, C, D>;
pub type PubkeyCommitmentMapperCircuitData = CircuitData<F, C, D>;

pub struct PubkeyCommitmentMapperContext {
    pub storage: RedisBlobStorage,
    pub protocol: String,
    pub deposit_count: u64,
    pub zero_hash_proofs: Vec<PubkeyCommitmentMapperProof>,
    pub branch: Vec<PubkeyCommitmentMapperProof>,
    pub first_level_circuit: CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    pub inner_level_circuits: Vec<CircuitTargetAndData<PubkeyCommitmentMapperIL>>,
}

impl PubkeyCommitmentMapperContext {
    pub async fn new(
        protocol: String,
        storage_config_filepath: &str,
        serialized_circuits_dir: &str,
    ) -> Result<Self> {
        let mut storage =
            RedisBlobStorage::from_file(storage_config_filepath, "pubkey-commitment-mapper")
                .await?;

        let (first_level_circuit, inner_level_circuits) = build_recursive_circuit_cached(
            serialized_circuits_dir,
            "pubkey_commitment_mapper",
            DEPTH,
            &|| PubkeyCommitmentMapperFL::build(&()),
            &|prev_circuit_data| PubkeyCommitmentMapperIL::build(prev_circuit_data),
        );

        let deposit_count: u64 = storage
            .metadata
            .get(format!(
                "{protocol}:pubkey_commitment_mapper:currently_computed_pubkey_mapping"
            ))
            .await?;

        let zero_hash_proofs = get_zero_hash_proofs(
            storage.blob.as_mut(),
            &first_level_circuit,
            &inner_level_circuits,
        )
        .await?;

        if deposit_count == 0 {
            let mut pipe = redis::pipe();
            pipe.atomic();
            save_branch(&mut pipe, &protocol, &zero_hash_proofs);
            pipe.query_async(&mut storage.metadata).await?;
        }

        let branch = load_branch(
            &mut storage.metadata,
            &protocol,
            &first_level_circuit,
            &inner_level_circuits,
        )
        .await?;

        Ok(Self {
            storage,
            protocol,
            deposit_count,
            zero_hash_proofs,
            branch,
            first_level_circuit,
            inner_level_circuits,
        })
    }
}

fn parse_processing_queue_item(item: &str) -> (String, u64) {
    let parts = item.split(",").collect_vec();
    let pubkey = parts[0];
    let block_number: u64 = parts[1].parse().unwrap();

    (pubkey.to_owned(), block_number)
}

pub async fn poll_processing_queue(
    redis: &mut Connection,
    protocol: &str,
) -> Result<(String, u64)> {
    let processing_queue_key = format!("{protocol}:pubkey_commitment_mapper:processing_queue");

    let head_opt: Option<String> = redis.lindex(&processing_queue_key, 0).await?;

    match head_opt {
        Some(head) => Ok(parse_processing_queue_item(&head)),
        None => bail!("No items in processing queue"),
    }
}

pub fn complete_task(redis_pipeline: &mut Pipeline, protocol: &str) {
    redis_pipeline.lpop(
        format!("{protocol}:pubkey_commitment_mapper:processing_queue"),
        None,
    );
}

fn generate_leaf_zero_hash_proof(
    first_level_circuit: &CircuitTargetAndData<PubkeyCommitmentMapperFL>,
) -> Result<PubkeyCommitmentMapperProof> {
    let input = CircuitInput::<PubkeyCommitmentMapperFL> {
        pubkey: Array([false; 384]),
        is_real: false,
    };

    let mut pw = PartialWitness::new();
    first_level_circuit.target.set_witness(&mut pw, &input);
    first_level_circuit.data.prove(pw)
}

pub fn generate_zero_hash_proofs(
    first_level_circuit: &CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    inner_level_circuits: &[CircuitTargetAndData<PubkeyCommitmentMapperIL>],
) -> Result<Vec<PubkeyCommitmentMapperProof>> {
    (1..DEPTH).try_fold(
        vec![generate_leaf_zero_hash_proof(first_level_circuit)?],
        |mut proofs, level| {
            proofs.push(prove_inner_level2(
                &proofs.last().unwrap(),
                &proofs.last().unwrap(),
                &inner_level_circuits[level - 1].target,
                &inner_level_circuits[level - 1].data,
            )?);
            Ok(proofs)
        },
    )
}

pub fn save_branch(
    redis_pipeline: &mut Pipeline,
    protocol: &str,
    branch: &[PubkeyCommitmentMapperProof],
) {
    for (level, proof) in branch.iter().enumerate() {
        redis_pipeline.set(
            format!("{protocol}:pubkey_commitment_mapper:branch:{level}"),
            &proof.to_bytes(),
        );
    }
}

async fn save_zero_hashes_branch(
    proof_storage: &mut dyn ProofStorage,
    branch: &[PubkeyCommitmentMapperProof],
) -> Result<()> {
    for (level, proof) in branch.iter().enumerate() {
        let proof_bytes = proof.to_bytes();

        proof_storage
            .set_proof(
                format!("pubkey_commitment_mapper:zero_hashes:{level}"),
                &proof_bytes,
            )
            .await?;
    }
    Ok(())
}

async fn zero_hashes_are_calculated(proof_storage: &mut dyn ProofStorage) -> bool {
    proof_storage
        .get_keys_count("pubkey_commitment_mapper:zero_hashes:*".to_owned())
        .await
        == DEPTH
}

fn pick_circuit_data_for_level<'a>(
    first_level_circuit: &'a CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    inner_level_circuits: &'a [CircuitTargetAndData<PubkeyCommitmentMapperIL>],
    level: usize,
) -> &'a PubkeyCommitmentMapperCircuitData {
    if level == 0 {
        &first_level_circuit.data
    } else {
        &inner_level_circuits[level - 1].data
    }
}

pub async fn fetch_zero_hash_proofs(
    proof_storage: &mut dyn ProofStorage,
    first_level_circuit: &CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    inner_level_circuits: &[CircuitTargetAndData<PubkeyCommitmentMapperIL>],
) -> Result<Vec<PubkeyCommitmentMapperProof>> {
    let proof_storage = Arc::new(tokio::sync::Mutex::new(proof_storage));

    try_join_all((0..DEPTH).map(|level| {
        let proof_storage = Arc::clone(&proof_storage);
        async move {
            let key = format!("pubkey_commitment_mapper:zero_hashes:{level}");
            let proof_bytes = proof_storage.lock().await.get_proof(key).await?;
            let circuit_data =
                pick_circuit_data_for_level(first_level_circuit, inner_level_circuits, level);
            PubkeyCommitmentMapperProof::from_bytes(proof_bytes, &circuit_data.common)
        }
    }))
    .await
}

pub async fn get_zero_hash_proofs(
    proof_storage: &mut dyn ProofStorage,
    first_level_circuit: &CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    inner_level_circuits: &[CircuitTargetAndData<PubkeyCommitmentMapperIL>],
) -> Result<Vec<PubkeyCommitmentMapperProof>> {
    Ok(if !zero_hashes_are_calculated(proof_storage).await {
        let proofs = generate_zero_hash_proofs(first_level_circuit, inner_level_circuits)?;
        save_zero_hashes_branch(proof_storage, &proofs).await?;
        proofs
    } else {
        fetch_zero_hash_proofs(proof_storage, first_level_circuit, inner_level_circuits).await?
    })
}

pub async fn load_branch(
    redis: &mut Connection,
    protocol: &str,
    first_level_circuit: &CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    inner_level_circuits: &[CircuitTargetAndData<PubkeyCommitmentMapperIL>],
) -> Result<Vec<PubkeyCommitmentMapperProof>> {
    let redis = Arc::new(tokio::sync::Mutex::new(redis));

    try_join_all((0..DEPTH).map(|level| {
        let redis = Arc::clone(&redis);
        async move {
            let key = format!("{protocol}:pubkey_commitment_mapper:branch:{level}");
            let proof_bytes = redis.lock().await.get(key).await?;
            let circuit_data =
                pick_circuit_data_for_level(first_level_circuit, inner_level_circuits, level);
            PubkeyCommitmentMapperProof::from_bytes(proof_bytes, &circuit_data.common)
        }
    }))
    .await
}

fn generate_pubkey_commitment_proof(
    first_level_circuit: &CircuitTargetAndData<PubkeyCommitmentMapperFL>,
    pubkey: &str,
) -> Result<PubkeyCommitmentMapperProof> {
    let mut pw = PartialWitness::new();

    first_level_circuit.target.set_witness(
        &mut pw,
        &serde_json::from_str(
            &json!({
                "pubkey": pubkey,
                "isReal": true
            })
            .to_string(),
        )?,
    );

    first_level_circuit.data.prove(pw)
}

pub fn append_pubkey_and_recalc_merkle_branch(
    ctx: &mut PubkeyCommitmentMapperContext,
    redis_pipeline: &mut Pipeline,
    pubkey: &str,
) -> Result<()> {
    ctx.deposit_count += 1;

    redis_pipeline.set(
        format!(
            "{}:pubkey_commitment_mapper:currently_computed_pubkey_mapping",
            ctx.protocol
        ),
        ctx.deposit_count,
    );

    let mut size = ctx.deposit_count;
    let mut node = generate_pubkey_commitment_proof(&ctx.first_level_circuit, pubkey)?;

    for height in 0..DEPTH {
        if size & 1 == 1 {
            ctx.branch[height] = node;
            break;
        }

        node = prove_inner_level2(
            &ctx.branch[height],
            &node,
            &ctx.inner_level_circuits[height].target,
            &ctx.inner_level_circuits[height].data,
        )?;

        size /= 2;
    }

    Ok(())
}

pub fn compute_merkle_root(
    ctx: &mut PubkeyCommitmentMapperContext,
) -> Result<PubkeyCommitmentMapperProof> {
    let mut size = ctx.deposit_count;
    let mut node = ctx.zero_hash_proofs[0].clone();

    for height in 0..DEPTH {
        node = if size & 1 == 1 {
            prove_inner_level2(
                &ctx.branch[height],
                &node,
                &ctx.inner_level_circuits[height].target,
                &ctx.inner_level_circuits[height].data,
            )?
        } else {
            prove_inner_level2(
                &node,
                &ctx.zero_hash_proofs[height],
                &ctx.inner_level_circuits[height].target,
                &ctx.inner_level_circuits[height].data,
            )?
        };

        size /= 2;
    }

    Ok(node)
}

pub async fn finished_block(
    redis: &mut Connection,
    protocol: &str,
    current_block_number: u64,
) -> Result<bool> {
    let processing_queue_key = format!("{protocol}:pubkey_commitment_mapper:processing_queue");

    let semi_head_opt: Option<String> = redis.lindex(&processing_queue_key, 1).await?;
    match semi_head_opt {
        Some(data) => {
            let (_, block_number) = parse_processing_queue_item(&data);
            Ok(current_block_number != block_number)
        }
        None => Ok(true),
    }
}

pub async fn save_root_for_block_number(
    redis_pipeline: &mut Pipeline,
    proof_storage: &mut dyn ProofStorage,
    protocol: &str,
    root: &PubkeyCommitmentMapperProof,
    block_number: u64,
) -> Result<()> {
    let proof_key = format!("{protocol}:pubkey_commitment_mapper:root_proofs:{block_number}");

    proof_storage
        .set_proof(proof_key.clone(), &root.to_bytes())
        .await?;

    let public_inputs = PubkeyCommitmentMapperFL::read_public_inputs(&root.public_inputs);
    let sha256_hex = hex::encode(&bits_to_bytes(public_inputs.sha256.as_slice()));

    println!(
        "{}",
        format!("[{block_number}] Merkle root proof saved: {sha256_hex}").green()
    );

    let redis_storage_data = PubkeyCommitmentMapperRedisStorageData {
        sha256: sha256_hex.clone(),
        poseidon: public_inputs.poseidon.into(),
        proof_key: proof_key.clone(),
    };

    redis_pipeline.set(proof_key, serde_json::to_string(&redis_storage_data)?);

    Ok(())
}
