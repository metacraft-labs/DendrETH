use std::{print, println};

use anyhow::Result;
use circuits::{
    build_first_level_circuit::build_first_level_circuit,
    build_inner_level_circuit::{build_inner_circuit, InnerCircuitTargets},
    validator_commitment::ValidatorCommitment,
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_data::CircuitData,
        config::PoseidonGoldilocksConfig,
        proof::{Proof, ProofWithPublicInputs},
    },
};
use redis::{aio::Connection, AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};

mod validator;

use validator::Validator;

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

    for i in 1..2 {
        inner_circuits.push(build_inner_circuit(&inner_circuits[i - 1].1));
    }

    let proof_bytes1 = fetch_proof(&mut con, 1, 7172).await?;

    let proof_bytes2 = fetch_proof(&mut con, 1, 7174).await?;

    // println!("Up to here ok");

    let result = handle_inner_level_proof(
        proof_bytes1,
        proof_bytes2,
        &inner_circuits[0].1,
        &inner_circuits[1].0,
        &inner_circuits[1].1,
        false,
    )?;

    let proof = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        result,
        &inner_circuits[1].1.common,
    )?;

    println!("proof public inputs {:?}", proof.public_inputs);

    let proof_bytes = fetch_proof(&mut con, 2, 7172).await?;

    let proof = ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        proof_bytes.clone(),
        &inner_circuits[1].1.common,
    )?;

    println!(
        "number of public inputs {}",
        inner_circuits[1].1.common.num_public_inputs
    );

    print!("public inputs, {:?}", proof.public_inputs);

    inner_circuits[1].1.verify(proof)?;

    Ok(())
}

fn handle_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &InnerCircuitTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    is_zero: bool,
) -> Result<Vec<u8>> {
    let inner_proof1 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof1_bytes,
            &inner_circuit_data.common,
        )?;

    let inner_proof2 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof2_bytes,
            &inner_circuit_data.common,
        )?;

    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&inner_circuit_targets.proof1, &inner_proof1);
    pw.set_proof_with_pis_target(&inner_circuit_targets.proof2, &inner_proof2);

    pw.set_cap_target(
        &inner_circuit_targets
            .verifier_circuit_target
            .constants_sigmas_cap,
        &inner_circuit_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        inner_circuit_targets.verifier_circuit_target.circuit_digest,
        inner_circuit_data.verifier_only.circuit_digest,
    );

    pw.set_bool_target(inner_circuit_targets.is_zero, is_zero);

    Ok(circuit_data.prove(pw)?.to_bytes())
}

fn handle_first_level_proof(
    validator: Validator,
    validator_commitment: &ValidatorCommitment,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<Vec<u8>> {
    let mut pw = PartialWitness::new();

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.pubkey,
        validator.pubkey,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.withdrawal_credentials,
        validator.withdrawal_credentials,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.effective_balance,
        validator.effective_balance,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.slashed,
        validator.slashed,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.activation_eligibility_epoch,
        validator.activation_eligibility_epoch,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.activation_epoch,
        validator.activation_epoch,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.exit_epoch,
        validator.exit_epoch,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.withdrawable_epoch,
        validator.withdrawable_epoch,
    );

    Ok(circuit_data.prove(pw)?.to_bytes())
}

fn set_pw_values(
    pw: &mut PartialWitness<GoldilocksField>,
    target: &[BoolTarget],
    source: Vec<bool>,
) {
    for i in 0..target.len() {
        pw.set_bool_target(target[i], source[i]);
    }
}
