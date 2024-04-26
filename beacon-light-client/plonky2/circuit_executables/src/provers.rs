use circuits::{
    deposits_accumulator_balance_aggregator::validator_balance_circuit_accumulator::{
        DepositDataTarget, ValidatorBalanceVerificationAccumulatorTargets,
    },
    final_layer::build_final_circuit::FinalCircuitTargets,
    utils::{
        biguint::WitnessBigUint,
        hashing::{
            validator_hash_tree_root::ValidatorShaTargets,
            validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
        },
        utils::SetBytesArray,
    },
    validators_commitment_mapper::build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    withdrawal_credentials_balance_aggregator::{
        build_balance_inner_level_circuit::BalanceInnerCircuitTargets,
        validator_balance_circuit::ValidatorBalanceVerificationTargets,
    },
};
use itertools::Itertools;
use num::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::hash_types::HashOut,
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
    crud::common::FinalCircuitInput,
    validator::ValidatorShaInput,
    validator_balances_input::{
        DepositDataInput, ValidatorBalanceAccumulatorInput, ValidatorBalancesInput,
        ValidatorPoseidonInput,
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
        pw.set_bytes_array(&self.pubkey, &hex::decode(&source.pubkey).unwrap());

        pw.set_bytes_array(
            &self.withdrawal_credentials,
            &hex::decode(&source.withdrawal_credentials).unwrap(),
        );

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

impl<const N: usize> SetPWValues<ValidatorBalancesInput>
    for ValidatorBalanceVerificationTargets<N>
{
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

        for i in 0..N {
            set_boolean_pw_values(
                pw,
                &self.withdrawal_credentials[i],
                &source.withdrawal_credentials[i],
            );
        }

        set_boolean_pw_values(
            pw,
            &self.non_zero_validator_leaves_mask,
            &source.validator_is_zero,
        );

        pw.set_biguint_target(&self.current_epoch, &source.current_epoch);
    }
}

impl SetPWValues<DepositDataInput> for DepositDataTarget {
    fn set_pw_values(&self, pw: &mut PartialWitness<GoldilocksField>, source: &DepositDataInput) {
        pw.set_bytes_array(&self.pubkey, &hex::decode(&source.pubkey).unwrap());
        pw.set_bytes_array(
            &self.withdrawal_credentials,
            &hex::decode(&source.withdrawal_credentials).unwrap(),
        );
        pw.set_biguint_target(&self.amount, &BigUint::from(source.amount));
        pw.set_bytes_array(&self.signature, &hex::decode(&source.signature).unwrap());
    }
}

impl SetPWValues<ValidatorBalanceAccumulatorInput>
    for ValidatorBalanceVerificationAccumulatorTargets
{
    fn set_pw_values(
        &self,
        pw: &mut PartialWitness<GoldilocksField>,
        source: &ValidatorBalanceAccumulatorInput,
    ) {
        for i in 0..source.balances_leaves.len() {
            pw.set_bytes_array(
                &self.balances_leaves[i],
                &hex::decode(&source.balances_leaves[i]).unwrap(),
            );
        }

        pw.set_bytes_array(
            &self.balances_root,
            &hex::decode(&source.balances_root).unwrap(),
        );

        for i in 0..source.validator_is_not_zero.len() {
            pw.set_bool_target(
                self.non_zero_validator_leaves_mask[i],
                source.validator_is_not_zero[i], // TODO: rename this
            );
        }

        for i in 0..source.balances_proofs.len() {
            for j in 0..source.balances_proofs[i].len() {
                pw.set_bytes_array(
                    &self.balances_proofs[i][j],
                    &hex::decode(&source.balances_proofs[i][j]).unwrap(),
                );
            }
        }

        for i in 0..source.validators.len() {
            self.validators[i].set_pw_values(pw, &source.validators[i]);
        }

        for i in 0..source.validator_indices.len() {
            pw.set_biguint_target(
                &self.validator_indices[i],
                &BigUint::from(source.validator_indices[i]),
            );
        }

        pw.set_biguint_target(&self.current_epoch, &BigUint::from(source.current_epoch));

        for i in 0..source.deposits_data.len() {
            self.deposits_data[i].set_pw_values(pw, &source.deposits_data[i]);
        }

        let validators_poseidon_root_targets = HashOut::from_vec(
            source
                .validators_poseidon_root
                .iter()
                .map(|&number| GoldilocksField::from_canonical_u64(number))
                .collect_vec(),
        );
        pw.set_hash_target(
            self.validators_poseidon_root,
            validators_poseidon_root_targets,
        );
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

impl<const N: usize> SetPWValues<FinalCircuitInput> for FinalCircuitTargets<N> {
    fn set_pw_values(&self, pw: &mut PartialWitness<GoldilocksField>, source: &FinalCircuitInput) {
        set_boolean_pw_values(pw, &self.state_root, &source.state_root);

        for i in 0..source.state_root_branch.len() {
            set_boolean_pw_values(pw, &self.state_root_branch[i], &source.state_root_branch[i]);
        }

        set_boolean_pw_values(pw, &self.block_root, &source.block_root);

        pw.set_biguint_target(&self.slot, &source.slot);

        for i in 0..source.slot_branch.len() {
            set_boolean_pw_values(pw, &self.slot_branch[i], &source.slot_branch[i]);
        }

        for i in 0..N {
            set_boolean_pw_values(
                pw,
                &self.withdrawal_credentials[i],
                &source.withdrawal_credentials[i],
            );
        }

        for i in 0..source.balance_branch.len() {
            set_boolean_pw_values(pw, &self.balance_branch[i], &source.balance_branch[i]);
        }

        for i in 0..source.validators_branch.len() {
            set_boolean_pw_values(pw, &self.validators_branch[i], &source.validators_branch[i]);
        }

        set_boolean_pw_values(pw, &self.validator_size_bits, &source.validators_size_bits);
    }
}
