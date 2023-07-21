use anyhow::Result;
use circuits::{
    build_first_level_circuit::build_first_level_circuit,
    build_inner_level_circuit::{build_inner_circuit, InnerCircuitTargets},
};
use circuits_executables::crud::{ValidatorProof, fetch_proof};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use std::{print, println};

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

    for i in 1..4 {
        inner_circuits.push(build_inner_circuit(&inner_circuits[i - 1].1));
    }

    let proof = fetch_proof::<ValidatorProof>(&mut con, 3, 0).await?;

    println!("Up to here");
    let plonky2_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof.proof,
            &inner_circuits[2].1.common,
        )?;

    print!("public inputs, {:?}", plonky2_proof.public_inputs);

    inner_circuits[2].1.verify(plonky2_proof)?;

    println!("written poseidon hash {:?}", proof.poseidon_hash);

    println!("written sha256 hash {:?}", proof.sha256_hash);

    Ok(())
}
