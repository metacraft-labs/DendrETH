use circuits::{validator_commitment::ValidatorCommitment, build_inner_level_circuit::InnerCircuitTargets};
use plonky2::{iop::{witness::{PartialWitness, WitnessWrite}, target::BoolTarget}, field::goldilocks_field::GoldilocksField, plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs, circuit_data::CircuitData}};

use crate::validator::Validator;

use anyhow::Result;

pub fn handle_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &InnerCircuitTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    is_zero: bool,
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

    Ok(circuit_data.prove(pw)?)
}

pub fn handle_first_level_proof(
    validator: Validator,
    validator_commitment: &ValidatorCommitment,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
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

    Ok(circuit_data.prove(pw)?)
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
