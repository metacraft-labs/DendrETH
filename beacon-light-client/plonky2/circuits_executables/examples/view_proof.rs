use anyhow::Result;
use circuits::{
    build_first_level_circuit::build_commitment_mapper_first_level_circuit,
    build_inner_level_circuit::{build_commitment_mapper_inner_circuit, InnerCircuitTargets}, build_balance_inner_level_circuit::{build_inner_level_circuit, BalanceInnerCircuitTargets}, build_validator_balance_circuit::build_validator_balance_circuit,
};
use circuits_executables::crud::{ValidatorProof, fetch_proof, BalanceProof};
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

    let (_, first_level_circuit_data) = build_validator_balance_circuit(8);

    // let mut inner_circuits: Vec<(
    //     BalanceInnerCircuitTargets,
    //     CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    // )> = Vec::new();

    // inner_circuits.push(build_balance_inner_circuit(&first_level_circuit_data));

    // for i in 1..7 {
    //     inner_circuits.push(build_balance_inner_circuit(&inner_circuits[i - 1].1));
    // }

    let proof = fetch_proof::<BalanceProof>(&mut con, 0, 992).await?;

    println!("Up to here");
    let plonky2_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof.proof,
            &first_level_circuit_data.common,
        )?;

    print!("public inputs, {:?}", plonky2_proof.public_inputs);

    first_level_circuit_data.verify(plonky2_proof)?;

    Ok(())
}
