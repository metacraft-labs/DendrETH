use anyhow::{Error, Result};

use circuit_executables::{
    commitment_mapper_context::{CommitmentMapperContext, WorkQueueConfig},
    commitment_mapper_task::{
        handle_task, pick_work_queue_item_prioritize_lower_levels, CommitmentMapperTask,
        VCMWorkQueueItem,
    },
    crud::proof_storage::proof_storage::load_storage_config,
    utils::CommandLineOptionsBuilder,
};
use colored::Colorize;
use jemallocator::Jemalloc;

use std::{format, println, thread::sleep, time::Duration};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("commitment_mapper")
        .with_work_queue_options()
        .with_serialized_circuits_dir()
        .with_proof_storage_config()
        .get_matches();

    let work_queue_cfg = WorkQueueConfig {
        stop_after: matches
            .value_of("stop_after")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        lease_for: matches
            .value_of("lease_for")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
    };

    let storage_cfg_filepath = matches.value_of("proof_storage_cfg").unwrap();
    let storage_cfg = load_storage_config(&storage_cfg_filepath)?;

    println!(
        "storage config: {}",
        serde_json::to_string_pretty(&storage_cfg).unwrap()
    );

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let mut ctx = CommitmentMapperContext::new(
        work_queue_cfg,
        &storage_cfg_filepath,
        serialized_circuits_dir,
    )
    .await?;

    loop {
        match pick_work_queue_item_prioritize_lower_levels(&mut ctx).await {
            Ok(maybe_item) => match maybe_item {
                Some(item) => handle_work_queue_item(&mut ctx, &item).await,
                None => {
                    println!("{}", "Waiting for task...".yellow());
                    sleep(Duration::from_secs(ctx.work_queue_cfg.stop_after));
                }
            },
            Err(err) => log_error_and_wait(err),
        }
    }
}

async fn handle_work_queue_item(ctx: &mut CommitmentMapperContext, item: &VCMWorkQueueItem) {
    match CommitmentMapperTask::deserialize(&item.item.data) {
        Ok(task) => {
            task.log();

            match handle_task(ctx, task).await {
                Ok(_) => complete_task(ctx, item).await,
                Err(err) => log_error_and_wait(err),
            }
        }
        Err(err) => {
            complete_task(ctx, item).await;
            println!("{}", format!("Error parsing task: {err}").bold().red());
            println!("{}", format!("Got bytes: {:?}", item.item.data).red());
        }
    }
}

async fn complete_task(ctx: &mut CommitmentMapperContext, item: &VCMWorkQueueItem) {
    if ctx.work_queues[item.depth]
        .complete(&mut ctx.storage.metadata, &item.item)
        .await
        .is_err()
    {
        println!("{}", format!("Cannot complete task").bold().red());
    }
}

fn log_error_and_wait(err: Error) {
    let error_message = format!("Error: {}", err).red().bold();
    println!("{}", error_message);
    sleep(Duration::from_secs(10));
}
