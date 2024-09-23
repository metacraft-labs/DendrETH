use std::time::Duration;

use anyhow::{bail, ensure, Context, Result};
use circuit::SetWitness;
use circuits::redis_storage_types::ValidatorsCommitmentMapperProofData;
use colored::Colorize;

use num::FromPrimitive;
use num_derive::FromPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis_work_queue::Item;

use crate::{
    commitment_mapper_context::CommitmentMapperContext,
    constants::VALIDATOR_REGISTRY_LIMIT,
    crud::common::{
        fetch_proofs, fetch_validator, fetch_zero_proof, save_validator_proof,
        save_zero_validator_proof, ProofProvider,
    },
    provers::prove_inner_level,
    utils::{get_depth_for_gindex, gindex_from_validator_index},
};

#[derive(FromPrimitive)]
#[repr(u8)]
enum CommitmentMapperTaskType {
    UpdateProofNode,
    ProveZeroForDepth,
    UpdateValidatorProof,
    ZeroOutValidator,
}

type Gindex = u64;
type Slot = u64;
type ValidatorIndex = u64;
type Depth = u64;

#[derive(Debug)]
pub enum CommitmentMapperTask {
    UpdateProofNode(Gindex, Slot),
    ProveZeroForDepth(Depth),
    UpdateValidatorProof(ValidatorIndex, Slot),
    ZeroOutValidator(ValidatorIndex, Slot),
}

impl CommitmentMapperTask {
    pub fn log(&self) {
        match *self {
            CommitmentMapperTask::UpdateProofNode(gindex, slot) => println!(
                "{}",
                format!(
                    "Updating proof node at gindex {} for slot {}...",
                    gindex.to_string().magenta(),
                    slot.to_string().cyan()
                )
                .blue()
                .bold()
            ),
            CommitmentMapperTask::ProveZeroForDepth(depth) => {
                println!(
                    "{}",
                    format!("Proving zero for depth {}...", depth.to_string().magenta())
                        .blue()
                        .bold(),
                )
            }
            CommitmentMapperTask::UpdateValidatorProof(validator_index, slot) => {
                if validator_index != VALIDATOR_REGISTRY_LIMIT as u64 {
                    println!(
                        "{}",
                        format!(
                            "Updating validator proof at index {} for slot {}...",
                            validator_index.to_string().magenta(),
                            slot.to_string().cyan()
                        )
                        .blue()
                        .bold()
                    );
                } else {
                    println!("{}", "Proving zero validator...".blue().bold());
                }
            }
            CommitmentMapperTask::ZeroOutValidator(validator_index, slot) => println!(
                "{}",
                format!(
                    "Zeroing out validator {} for slot {}...",
                    validator_index.to_string().magenta(),
                    slot.to_string().cyan()
                )
                .blue()
                .bold()
            ),
        };
    }
}

impl CommitmentMapperTask {
    pub fn deserialize(bytes: &[u8]) -> Result<CommitmentMapperTask> {
        ensure!(bytes.len() > 0, "Task buffer is empty");

        let task_tag = FromPrimitive::from_u8(u8::from_be_bytes(bytes[0..1].try_into()?))
            .context("Invalid task tag")?;

        match task_tag {
            CommitmentMapperTaskType::UpdateProofNode => {
                let gindex = u64::from_be_bytes(bytes[1..9].try_into()?);
                let slot = u64::from_be_bytes(bytes[9..17].try_into()?);
                Ok(CommitmentMapperTask::UpdateProofNode(gindex, slot))
            }
            CommitmentMapperTaskType::ProveZeroForDepth => {
                let depth = u64::from_be_bytes(bytes[1..9].try_into()?);
                Ok(CommitmentMapperTask::ProveZeroForDepth(depth))
            }
            CommitmentMapperTaskType::UpdateValidatorProof => {
                let validator_index = u64::from_be_bytes(bytes[1..9].try_into()?);
                let slot = u64::from_be_bytes(bytes[9..17].try_into()?);
                Ok(CommitmentMapperTask::UpdateValidatorProof(
                    validator_index,
                    slot,
                ))
            }
            CommitmentMapperTaskType::ZeroOutValidator => {
                let validator_index = u64::from_be_bytes(bytes[1..9].try_into()?);
                let slot = u64::from_be_bytes(bytes[9..17].try_into()?);
                Ok(CommitmentMapperTask::ZeroOutValidator(
                    validator_index,
                    slot,
                ))
            }
        }
    }
}

pub async fn handle_task(
    ctx: &mut CommitmentMapperContext,
    task: CommitmentMapperTask,
) -> Result<()> {
    match task {
        CommitmentMapperTask::UpdateValidatorProof(validator_index, slot) => {
            handle_update_validator_proof_task(ctx, validator_index, slot).await
        }
        CommitmentMapperTask::UpdateProofNode(gindex, slot) => {
            handle_update_proof_node_task(ctx, gindex, slot).await
        }
        CommitmentMapperTask::ProveZeroForDepth(depth) => {
            handle_prove_zero_for_depth_task(ctx, depth).await
        }
        CommitmentMapperTask::ZeroOutValidator(validator_index, slot) => {
            handle_zero_out_validator_task(ctx, validator_index, slot).await
        }
    }
}

async fn handle_update_validator_proof_task(
    ctx: &mut CommitmentMapperContext,
    validator_index: u64,
    slot: u64,
) -> Result<()> {
    match fetch_validator(&mut ctx.storage.metadata, validator_index, slot).await {
        Ok(input) => {
            let mut pw = PartialWitness::new();
            ctx.first_level_circuit.target.set_witness(&mut pw, &input);
            let proof = prove_until_verifiable(&ctx.first_level_circuit.data, || {
                ctx.first_level_circuit.data.prove(pw.clone())
            })?;

            if input.is_real {
                let save_result = save_validator_proof(
                    &mut ctx.storage.metadata,
                    ctx.storage.blob.as_mut(),
                    proof,
                    gindex_from_validator_index(validator_index, 40),
                    slot,
                )
                .await;

                if let Err(err) = save_result {
                    bail!("Error while proving zero validator: {}", err);
                };
            } else {
                let save_result = save_zero_validator_proof(
                    &mut ctx.storage.metadata,
                    ctx.storage.blob.as_mut(),
                    proof,
                    40,
                )
                .await;

                if let Err(err) = save_result {
                    bail!("Error while proving validator: {}", err);
                }
            }
        }
        Err(err) => bail!("Error while fetching validator: {}", err),
    };
    Ok(())
}

async fn handle_update_proof_node_task(
    ctx: &mut CommitmentMapperContext,
    gindex: u64,
    slot: u64,
) -> Result<()> {
    let level = 39 - get_depth_for_gindex(gindex) as usize;

    let fetch_result = fetch_proofs::<ValidatorsCommitmentMapperProofData>(
        &mut ctx.storage.metadata,
        ctx.storage.blob.as_mut(),
        gindex,
        slot,
    )
    .await;

    match fetch_result {
        Ok((left_proof, right_proof)) => {
            let inner_circuit_data = if level > 0 {
                &ctx.inner_level_circuits[level - 1].data
            } else {
                &ctx.first_level_circuit.data
            };

            let proof = prove_until_verifiable(&ctx.inner_level_circuits[level].data, || {
                prove_inner_level(
                    left_proof.clone(),
                    right_proof.clone(),
                    inner_circuit_data,
                    &ctx.inner_level_circuits[level].target,
                    &ctx.inner_level_circuits[level].data,
                )
            })?;

            let save_result = save_validator_proof(
                &mut ctx.storage.metadata,
                ctx.storage.blob.as_mut(),
                proof,
                gindex,
                slot,
            )
            .await;

            if let Err(err) = save_result {
                bail!("Error while saving validator proof: {}", err);
            };
        }
        Err(err) => bail!("Error while fetching validator proof: {}", err),
    };
    Ok(())
}

async fn handle_prove_zero_for_depth_task(
    ctx: &mut CommitmentMapperContext,
    depth: u64,
) -> Result<()> {
    // the level in the inner proofs tree
    let level = 39 - depth as usize;

    match fetch_zero_proof::<ValidatorsCommitmentMapperProofData>(
        &mut ctx.storage.metadata,
        depth + 1,
    )
    .await
    {
        Ok(lower_proof) => {
            let inner_circuit_data = if level > 0 {
                &ctx.inner_level_circuits[level - 1].data
            } else {
                &ctx.first_level_circuit.data
            };

            let lower_proof_bytes = lower_proof.get_proof(ctx.storage.blob.as_mut()).await;

            let proof = prove_until_verifiable(&ctx.inner_level_circuits[level].data, || {
                prove_inner_level(
                    lower_proof_bytes.clone(),
                    lower_proof_bytes.clone(),
                    inner_circuit_data,
                    &ctx.inner_level_circuits[level].target,
                    &ctx.inner_level_circuits[level].data,
                )
            })?;

            let save_result = save_zero_validator_proof(
                &mut ctx.storage.metadata,
                ctx.storage.blob.as_mut(),
                proof,
                depth,
            )
            .await;

            if let Err(err) = save_result {
                bail!("Error while saving zero validator proof: {}", err);
            }
        }
        Err(err) => {
            bail!("Error while proving zero for depth {}: {}", depth, err);
        }
    };
    Ok(())
}

async fn handle_zero_out_validator_task(
    ctx: &mut CommitmentMapperContext,
    validator_index: u64,
    slot: u64,
) -> Result<()> {
    match fetch_validator(
        &mut ctx.storage.metadata,
        VALIDATOR_REGISTRY_LIMIT as u64,
        slot,
    )
    .await
    {
        Ok(input) => {
            let mut pw = PartialWitness::new();
            ctx.first_level_circuit.target.set_witness(&mut pw, &input);
            let proof = ctx.first_level_circuit.data.prove(pw)?;

            let save_result = save_validator_proof(
                &mut ctx.storage.metadata,
                ctx.storage.blob.as_mut(),
                proof,
                gindex_from_validator_index(validator_index, 40),
                slot,
            )
            .await;

            if let Err(err) = save_result {
                bail!(
                    "Error while zeroing out validator {}: {}",
                    validator_index,
                    err
                );
            }
        }
        Err(_) => bail!("Could not fetch zero validator"),
    }
    Ok(())
}

#[derive(Debug)]
pub struct VCMWorkQueueItem {
    pub depth: usize,
    pub item: Item,
}

pub async fn pick_work_queue_item_prioritize_lower_levels(
    ctx: &mut CommitmentMapperContext,
) -> Result<Option<VCMWorkQueueItem>> {
    for depth in (0..=40).rev() {
        let work_queue = &mut ctx.work_queues[depth];

        if work_queue.queue_len(&mut ctx.storage.metadata).await? > 0 {
            let item_opt = work_queue
                .lease(
                    &mut ctx.storage.metadata,
                    Some(Duration::from_secs(ctx.work_queue_cfg.stop_after)),
                    Duration::from_secs(ctx.work_queue_cfg.lease_for),
                )
                .await?;

            if let Some(item) = item_opt {
                return Ok(Some(VCMWorkQueueItem { depth, item }));
            }
        }
    }

    Ok(None)
}

fn prove_until_verifiable<F, E>(
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    prove: F,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>, E>
where
    F: Fn() -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>, E>,
{
    let mut proof = prove()?;
    while circuit_data.verify(proof.clone()).is_err() {
        println!("{}", "Proof does not verify".bold().red());
        proof = prove()?;
    }
    Ok(proof)
}
