use colored::Colorize;
use std::{thread::sleep, time::Duration};

use anyhow::Result;
use circuit_executables::{
    crud::proof_storage::proof_storage::create_proof_storage,
    pubkey_commitment_mapper::{
        append_pubkey_and_recalc_merkle_branch, complete_task, compute_merkle_root, finished_block,
        poll_processing_queue, save_branch, save_root_for_block_number,
        PubkeyCommitmentMapperContext,
    },
    utils::{parse_config_file, CommandLineOptionsBuilder},
};
use clap::Arg;
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_config_file("../../common_config.json".to_owned())?;

    let matches = CommandLineOptionsBuilder::new("commitment_mapper")
        .with_redis_options(&config.redis_host, config.redis_port)
        .with_proof_storage_options()
        .with_protocol_options()
        .arg(
            Arg::with_name("fast_sync_to")
                .long("fast-sync-to")
                .help("Only recomputes the merkle branch up until the given block number")
                .takes_value(true),
        )
        .get_matches();

    let fast_sync_block_number: u64 = if matches.contains_id("fast_sync_to") {
        matches.get_one::<String>("fast_sync_to").unwrap().parse()?
    } else {
        0
    };

    println!("Initializing context...");

    let mut ctx = {
        let redis_connection = matches.value_of("redis_connection").unwrap();
        let protocol = matches.value_of("protocol").unwrap();
        let proof_storage = create_proof_storage(&matches).await;
        PubkeyCommitmentMapperContext::new(redis_connection, proof_storage, protocol.to_owned())
            .await?
    };

    println!("Polling tasks...");

    loop {
        // poll the pubkey processing queue (the result is in the format "pubkey,block_number")
        match poll_processing_queue(&mut ctx.redis, &ctx.protocol).await {
            Ok((pubkey, block_number)) => {
                println!(
                    "{}",
                    format!("[{block_number}] Appending {pubkey}...").cyan()
                );

                let mut pipe = redis::pipe();
                pipe.atomic();

                append_pubkey_and_recalc_merkle_branch(&mut ctx, &mut pipe, &pubkey)?;
                save_branch(&mut pipe, &ctx.protocol, &ctx.branch);

                // Don't save the root if it's not the last deposit for the block
                let should_save_merkle_root = block_number >= fast_sync_block_number
                    && finished_block(&mut ctx.redis, &ctx.protocol, block_number).await?;

                if should_save_merkle_root {
                    println!(
                        "{}",
                        format!("[{block_number}] Computing merkle root...").yellow()
                    );
                    let merkle_root = &compute_merkle_root(&mut ctx)?;

                    save_root_for_block_number(
                        &mut pipe,
                        ctx.proof_storage.as_mut(),
                        &ctx.protocol,
                        &merkle_root,
                        block_number,
                    )
                    .await?;
                }

                complete_task(&mut pipe, &ctx.protocol);
                _ = pipe.query_async(&mut ctx.redis).await?;
            }
            Err(_) => sleep(Duration::from_secs(5)),
        }
    }
}
