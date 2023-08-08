use circuits::{
    build_balance_inner_level_circuit::BalanceInnerCircuitTargets,
    build_inner_level_circuit::InnerCircuitTargets, validator_commitment::ValidatorCommitment,
};
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_data::{CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use crate::{validator::Validator, validator_balances_input};

use anyhow::Result;

pub fn handle_generic_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    proof1_target: &ProofWithPublicInputsTarget<2>,
    proof2_target: &ProofWithPublicInputsTarget<2>,
    verifier_circuit_target: &VerifierCircuitTarget,
    is_zero_target: Option<BoolTarget>,
    is_zero: Option<bool>,
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

    if let (Some(target), Some(value)) = (is_zero_target, is_zero) {
        pw.set_bool_target(target, value);
    }

    Ok(circuit_data.prove(pw)?)
}

pub fn handle_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &InnerCircuitTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    is_zero: bool,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    handle_generic_inner_level_proof(
        proof1_bytes,
        proof2_bytes,
        inner_circuit_data,
        &inner_circuit_targets.proof1,
        &inner_circuit_targets.proof2,
        &inner_circuit_targets.verifier_circuit_target,
        Some(inner_circuit_targets.is_zero),
        Some(is_zero),
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
        None,
        None,
        circuit_data,
    )
}

pub fn handle_first_level_proof(
    validator: Validator,
    validator_commitment: &ValidatorCommitment,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    let mut pw = PartialWitness::new();

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.pubkey,
        validator.pubkey,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.withdrawal_credentials,
        validator.withdrawal_credentials,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.effective_balance,
        validator.effective_balance,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.slashed,
        validator.slashed,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.activation_eligibility_epoch,
        validator.activation_eligibility_epoch,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.activation_epoch,
        validator.activation_epoch,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.exit_epoch,
        validator.exit_epoch,
    );

    set_boolean_pw_values(
        &mut pw,
        &validator_commitment.validator.withdrawable_epoch,
        validator.withdrawable_epoch,
    );

    Ok(circuit_data.prove(pw)?)
}

pub fn set_boolean_pw_values(
    pw: &mut PartialWitness<GoldilocksField>,
    target: &[BoolTarget],
    source: Vec<bool>,
) {
    for i in 0..target.len() {
        pw.set_bool_target(target[i], source[i]);
    }
}

pub fn set_pw_values(
    pw: &mut PartialWitness<GoldilocksField>,
    target: &[Target],
    source: Vec<u64>,
) {
    for i in 0..target.len() {
        pw.set_target(target[i], GoldilocksField::from_canonical_u64(source[i]));
    }
}

pub fn set_validator_pw_values(
    pw: &mut PartialWitness<GoldilocksField>,
    target: &circuits::validator_hash_tree_root_poseidon::ValidatorPoseidon,
    source: &validator_balances_input::ValidatorPoseidon,
) {
    set_pw_values(pw, &target.pubkey, source.pubkey.clone());

    set_pw_values(
        pw,
        &target.withdrawal_credentials,
        source.withdrawal_credentials.clone(),
    );

    set_pw_values(
        pw,
        &target.effective_balance,
        source.effective_balance.clone(),
    );

    set_pw_values(pw, &target.slashed, source.slashed.clone());

    set_pw_values(
        pw,
        &target.activation_eligibility_epoch,
        source.activation_eligibility_epoch.clone(),
    );

    set_pw_values(
        pw,
        &target.activation_epoch,
        source.activation_epoch.clone(),
    );

    set_pw_values(pw, &target.exit_epoch, source.exit_epoch.clone());

    set_pw_values(
        pw,
        &target.withdrawable_epoch,
        source.withdrawable_epoch.clone(),
    );
}
