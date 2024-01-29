use circuits::{
    biguint::WitnessBigUint, build_balance_inner_level_circuit::BalanceInnerCircuitTargets,
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    build_final_circuit::FinalCircuitTargets, utils::SetBytesArray,
    validator_balance_circuit::ValidatorBalanceVerificationTargets,
    validator_hash_tree_root::ValidatorShaTargets,
    validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
};

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_data::{CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use crate::{
    crud::FinalCircuitInput,
    validator::ValidatorShaInput,
    validator_balances_input::{ValidatorBalancesInput, ValidatorPoseidonInput},
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

fn set_boolean_pw_values(
    pw: &mut PartialWitness<GoldilocksField>,
    target: &[BoolTarget],
    source: &Vec<bool>,
) {
    for i in 0..target.len() {
        pw.set_bool_target(target[i], source[i]);
    }
}

pub trait SetPWValues<T> {
    fn set_pw_values(&self, pw: &mut PartialWitness<GoldilocksField>, source: &T);
}

impl SetPWValues<ValidatorPoseidonInput> for ValidatorPoseidonTargets {
    fn set_pw_values(
        &self,
        pw: &mut PartialWitness<GoldilocksField>,
        source: &ValidatorPoseidonInput,
    ) {
        pw.set_biguint_target(&self.pubkey, &source.pubkey);

        pw.set_biguint_target(&self.withdrawal_credentials, &source.withdrawal_credentials);

        pw.set_biguint_target(&self.effective_balance, &source.effective_balance);

        pw.set_bool_target(self.slashed, source.slashed == 1);

        pw.set_biguint_target(
            &self.activation_eligibility_epoch,
            &source.activation_eligibility_epoch,
        );

        pw.set_biguint_target(&self.activation_epoch, &source.activation_epoch);

        pw.set_biguint_target(&self.exit_epoch, &source.exit_epoch);

        pw.set_biguint_target(&self.withdrawable_epoch, &source.withdrawable_epoch);
    }
}

impl SetPWValues<ValidatorBalancesInput> for ValidatorBalanceVerificationTargets {
    fn set_pw_values(
        &self,
        pw: &mut PartialWitness<GoldilocksField>,
        source: &ValidatorBalancesInput,
    ) {
        for i in 0..self.balances.len() {
            set_boolean_pw_values(pw, &self.balances[i], &source.balances[i]);
        }

        for i in 0..self.validators.len() {
            self.validators[i].set_pw_values(pw, &source.validators[i]);
        }

        pw.set_biguint_target(&self.withdrawal_credentials, &source.withdrawal_credentials);

        set_boolean_pw_values(pw, &self.validator_is_zero, &source.validator_is_zero);

        pw.set_biguint_target(&self.current_epoch, &source.current_epoch);
    }
}

impl SetPWValues<ValidatorShaInput> for ValidatorShaTargets {
    fn set_pw_values(&self, pw: &mut PartialWitness<GoldilocksField>, source: &ValidatorShaInput) {
        pw.set_bytes_array(&self.pubkey, &hex::decode(&source.pubkey).unwrap());

        pw.set_bytes_array(
            &self.withdrawal_credentials,
            &hex::decode(&source.withdrawal_credentials).unwrap(),
        );

        pw.set_bytes_array(
            &self.effective_balance,
            &hex::decode(&source.effective_balance).unwrap(),
        );

        pw.set_bytes_array(&self.slashed, &hex::decode(&source.slashed).unwrap());

        pw.set_bytes_array(
            &self.activation_eligibility_epoch,
            &hex::decode(&source.activation_eligibility_epoch).unwrap(),
        );

        pw.set_bytes_array(
            &self.activation_epoch,
            &hex::decode(&source.activation_epoch).unwrap(),
        );

        pw.set_bytes_array(&self.exit_epoch, &hex::decode(&source.exit_epoch).unwrap());

        pw.set_bytes_array(
            &self.withdrawable_epoch,
            &hex::decode(&source.withdrawable_epoch).unwrap(),
        );
    }
}

impl SetPWValues<FinalCircuitInput> for FinalCircuitTargets {
    fn set_pw_values(&self, pw: &mut PartialWitness<GoldilocksField>, source: &FinalCircuitInput) {
        set_boolean_pw_values(pw, &self.state_root, &source.state_root);

        pw.set_biguint_target(&self.slot, &source.slot);

        for i in 0..source.slot_branch.len() {
            set_boolean_pw_values(pw, &self.slot_branch[i], &source.slot_branch[i]);
        }

        pw.set_biguint_target(&self.withdrawal_credentials, &source.withdrawal_credentials);

        for i in 0..source.balance_branch.len() {
            set_boolean_pw_values(pw, &self.balance_branch[i], &source.balance_branch[i]);
        }

        for i in 0..source.validators_branch.len() {
            set_boolean_pw_values(pw, &self.validators_branch[i], &source.validators_branch[i]);
        }

        set_boolean_pw_values(pw, &self.validator_size_bits, &source.validators_size_bits);
    }
}
