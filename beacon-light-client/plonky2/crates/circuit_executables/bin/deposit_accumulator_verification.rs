use anyhow::Result;

use circuit_executables::{
    commitment_mapper_context::{CommitmentMapperContext, WorkQueueConfig}, commitment_mapper_task::handle_commitment_mapper_task, crud::proof_storage::proof_storage::create_proof_storage, types::CommitmentMapperTask, utils::{parse_config_file, CommandLineOptionsBuilder}
};
use colored::Colorize;
use futures_lite::future;
use jemallocator::Jemalloc;

use std::{format, println, thread::sleep, time::Duration};

const CIRCUIT_NAME: &str = "deposit_accumulator";

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    future::block_on(async_main())
}



async fn async_main() -> Result<()> {
    let config = parse_config_file("../../common_config.json".to_owned())?;

    let matches = CommandLineOptionsBuilder::new("commitment_mapper")
        .with_redis_options(&config.redis_host, config.redis_port)
        .with_work_queue_options()
        .with_proof_storage_options()
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let stop_after = matches
        .value_of("stop_after")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let lease_for = matches
        .value_of("lease_for")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let work_queue_cfg = WorkQueueConfig {
        stop_after,
        lease_for,
    };

    let proof_storage = create_proof_storage(&matches).await;
    let mut ctx =
        CommitmentMapperContext::new(redis_connection, work_queue_cfg, proof_storage, CIRCUIT_NAME.to_string()).await?;

    loop {
        let Some(queue_item) = ctx
            .work_queue
            .lease(
                &mut ctx.redis_con,
                Some(Duration::from_secs(ctx.work_queue_cfg.stop_after)),
                Duration::from_secs(ctx.work_queue_cfg.lease_for),
            )
            .await?
        else {
            println!("{}", "Waiting for task...".yellow());
            continue;
        };

        let Some(task) = CommitmentMapperTask::deserialize(&queue_item.data) else {
            println!("{}", "Invalid task data".red().bold());
            println!("{}", format!("Got bytes: {:?}", queue_item.data).red());
            ctx.work_queue
                .complete(&mut ctx.redis_con, &queue_item)
                .await?;
            continue;
        };

        task.log();

        match handle_commitment_mapper_task(&mut ctx, task).await {
            Ok(_) => {
                ctx.work_queue
                    .complete(&mut ctx.redis_con, &queue_item)
                    .await?;
            }
            Err(err) => {
                let error_message = format!("Error: {}", err).red().bold();
                println!("{}", error_message);
                sleep(Duration::from_secs(10));
            }
        }
    }
}
