use std::{print, println};

use anyhow::Result;
use circuits::{
    build_first_level_circuit::build_first_level_circuit,
    build_inner_level_circuit::{build_inner_circuit, InnerCircuitTargets},
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::{aio::Connection, AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub proof: Vec<u8>,
}

async fn fetch_proof(con: &mut Connection, depth: usize, index: usize) -> Result<Vec<u8>> {
    let proof_result: RedisResult<String> = con
        .get(format!("validator_proof:{}:{}", depth, index))
        .await;

    let proof: ValidatorProof = serde_json::from_str::<ValidatorProof>(&proof_result?)?;

    return Ok(proof.proof);
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let mut con = redis::Client::open("redis://127.0.0.1:6379")?
        .get_async_connection()
        .await?;

    let (_, first_level_circuit_data) = build_first_level_circuit();

    let mut inner_circuits: Vec<(
        InnerCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    )> = Vec::new();

    inner_circuits.push(build_inner_circuit(&first_level_circuit_data));

    for i in 1..40 {
        inner_circuits.push(build_inner_circuit(&inner_circuits[i - 1].1));
    }

    let proof_bytes = fetch_proof(&mut con, 40, 0).await?;

    let proof = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        proof_bytes,
        &inner_circuits[39].1.common,
    )?;

    println!(
        "number of public inputs {}",
        inner_circuits[39].1.common.num_public_inputs
    );

    print!("public inputs, {:?}", proof.public_inputs);

    inner_circuits[39].1.verify(proof)?;

    Ok(())
}
