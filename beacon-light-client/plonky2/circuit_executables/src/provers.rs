use circuits::{
    validators_commitment_mapper::build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    withdrawal_credentials_balance_aggregator::inner_level_circuit::BalanceInnerCircuitTargets,
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_data::{CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use anyhow::Result;

pub fn handle_generic_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    proof1_target: &ProofWithPublicInputsTarget<2>,
    proof2_target: &ProofWithPublicInputsTarget<2>,
    verifier_circuit_target: &VerifierCircuitTarget,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
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

    pw.set_proof_with_pis_target(proof1_target, &inner_proof1);
    pw.set_proof_with_pis_target(proof2_target, &inner_proof2);

    pw.set_cap_target(
        &verifier_circuit_target.constants_sigmas_cap,
        &inner_circuit_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        verifier_circuit_target.circuit_digest,
        inner_circuit_data.verifier_only.circuit_digest,
    );

    Ok(circuit_data.prove(pw)?)
}

pub fn handle_commitment_mapper_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &CommitmentMapperInnerCircuitTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    handle_generic_inner_level_proof(
        proof1_bytes,
        proof2_bytes,
        inner_circuit_data,
        &inner_circuit_targets.proof1,
        &inner_circuit_targets.proof2,
        &inner_circuit_targets.verifier_circuit_target,
        circuit_data,
    )
}

pub fn handle_balance_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &BalanceInnerCircuitTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    handle_generic_inner_level_proof(
        proof1_bytes,
        proof2_bytes,
        inner_circuit_data,
        &inner_circuit_targets.proof1,
        &inner_circuit_targets.proof2,
        &inner_circuit_targets.verifier_circuit_target,
        circuit_data,
    )
}
