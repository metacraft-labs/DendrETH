use std::{print};

use anyhow::Result;
use circuits::build_first_level_circuit::build_first_level_circuit;
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        config::{PoseidonGoldilocksConfig},
        proof::{ProofWithPublicInputs},
    },
};
use redis::{aio::Connection, AsyncCommands};



async fn fetch_proof(con: &mut Connection, depth: usize, index: usize) -> Result<Vec<u8>> {
    let proof: Vec<u8> = con
        .get(format!("validator_proof:{}:{}", depth, index))
        .await?;

    return Ok(proof);
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let mut con = redis::Client::open("redis://127.0.0.1:6379")?
        .get_async_connection()
        .await?;

    let (_, circuit) = build_first_level_circuit();

    let proof_bytes = fetch_proof(&mut con, 0, 0).await?;

    let proof = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        proof_bytes,
        &circuit.common,
    )?;

    for i in 4..260 {
        print!("{}, ", proof.public_inputs[i]);
    }

    Ok(())
}
