use crate::{
    deposits_accumulator_balance_aggregator::validator_balance_circuit_accumulator::{
        DepositDataTarget, ValidatorBalanceVerificationAccumulatorTargets,
    },
    final_layer::build_final_circuit::FinalCircuitTargets,
    serializers::{
        biguint_to_str, bool_vec_as_int_vec, bool_vec_as_int_vec_nested, parse_biguint,
        ValidatorShaInput,
    },
    utils::{
        biguint::WitnessBigUint,
        hashing::{
            validator_hash_tree_root::ValidatorShaTargets,
            validator_hash_tree_root_poseidon::ValidatorTarget,
        },
        utils::SetBytesArray,
    },
    withdrawal_credentials_balance_aggregator::first_level::circuit::ValidatorBalanceVerificationTargets,
};
use itertools::Itertools;
use num::BigUint;
use plonky2::{
    hash::hash_types::{HashOut, RichField},
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub poseidon_hash: Vec<String>,
    pub sha256_hash: Vec<u64>,
    pub proof_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceProof {
    pub needs_change: bool,
    pub range_total_value: u64,
    pub validators_commitment: Vec<String>,
    pub balances_hash: String,
    pub withdrawal_credentials: Vec<String>,
    pub current_epoch: u64,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
    pub proof_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceAccumulatorProof {
    pub needs_change: bool,
    pub proof: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FinalCircuitInput {
    #[serde(with = "bool_vec_as_int_vec")]
    pub state_root: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub state_root_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub block_root: Vec<bool>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub slot: BigUint,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub slot_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub withdrawal_credentials: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub balance_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub validators_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub validators_size_bits: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FinalProof {
    pub needs_change: bool,
    pub block_root: String,
    pub withdrawal_credentials: Vec<String>,
    pub balance_sum: u64,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
    pub proof: Vec<u8>,
}

pub fn set_boolean_pw_values<F: RichField>(
    pw: &mut PartialWitness<F>,
    target: &[BoolTarget],
    source: &Vec<bool>,
) {
    for i in 0..target.len() {
        pw.set_bool_target(target[i], source[i]);
    }
}

pub trait SetPWValues<F: RichField, T> {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &T);
}

impl<F: RichField> SetPWValues<F, ValidatorInput> for ValidatorTarget {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &ValidatorInput) {
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

impl<F: RichField, const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize>
    SetPWValues<F, ValidatorBalancesInput>
    for ValidatorBalanceVerificationTargets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &ValidatorBalancesInput) {
        for i in 0..VALIDATORS_COUNT / 4 {
            set_boolean_pw_values(pw, &self.balances[i], &source.balances[i]);
        }

        for i in 0..VALIDATORS_COUNT {
            self.validators[i].set_pw_values(pw, &source.validators[i]);
        }

        for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
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

impl<F: RichField> SetPWValues<F, DepositDataInput> for DepositDataTarget {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &DepositDataInput) {
        pw.set_bytes_array(&self.pubkey, &hex::decode(&source.pubkey).unwrap());
        pw.set_bytes_array(
            &self.withdrawal_credentials,
            &hex::decode(&source.withdrawal_credentials).unwrap(),
        );
        pw.set_biguint_target(&self.amount, &BigUint::from(source.amount));
        pw.set_bytes_array(&self.signature, &hex::decode(&source.signature).unwrap());
    }
}

impl<F: RichField> SetPWValues<F, ValidatorBalanceAccumulatorInput>
    for ValidatorBalanceVerificationAccumulatorTargets
{
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &ValidatorBalanceAccumulatorInput) {
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
                .map(|&number| F::from_canonical_u64(number))
                .collect_vec(),
        );
        pw.set_hash_target(
            self.validators_poseidon_root,
            validators_poseidon_root_targets,
        );
    }
}

impl<F: RichField> SetPWValues<F, ValidatorShaInput> for ValidatorShaTargets {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &ValidatorShaInput) {
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

impl<F: RichField, const N: usize> SetPWValues<F, FinalCircuitInput> for FinalCircuitTargets<N> {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &FinalCircuitInput) {
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

pub fn from_str<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Vec::deserialize(deserializer)?;
    s.into_iter()
        .map(|str_val| str_val.parse::<u64>().map_err(serde::de::Error::custom))
        .collect()
}

pub fn to_string<S>(x: &Vec<u64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string_vec: Vec<String> = x.iter().map(|&num| num.to_string()).collect();
    string_vec.serialize(s)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInput {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub effective_balance: BigUint,
    pub slashed: u64,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub activation_eligibility_epoch: BigUint,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub activation_epoch: BigUint,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub exit_epoch: BigUint,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub withdrawable_epoch: BigUint,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorBalancesInput {
    pub validators: Vec<ValidatorInput>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub balances: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub withdrawal_credentials: Vec<Vec<bool>>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUint,
    #[serde(with = "bool_vec_as_int_vec")]
    pub validator_is_zero: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorBalanceAccumulatorInput {
    pub balances_root: String,
    pub balances_leaves: Vec<String>,
    pub balances_proofs: Vec<Vec<String>>,
    // pub validator_deposit_indexes: Vec<u64>,
    pub validator_indices: Vec<u64>,
    // pub validator_commitment_proofs: Vec<Vec<Vec<String>>>,
    pub validators: Vec<ValidatorInput>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub validator_is_not_zero: Vec<bool>,
    // pub validator_commitment_root: Vec<String>,
    pub current_epoch: u64,
    // pub current_eth1_deposit_index: u64,
    pub deposits_data: Vec<DepositDataInput>,
    pub validators_poseidon_root: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositDataInput {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub amount: u64,
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_deserialize() {
        let input = ValidatorBalancesInput {
            validators: vec![ValidatorInput {
                pubkey: "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
                withdrawal_credentials: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                effective_balance: BigUint::from(3u64),
                slashed: 0,
                activation_eligibility_epoch: BigUint::from(4u64),
                activation_epoch: BigUint::from(5u64),
                exit_epoch: BigUint::from(6u64),
                withdrawable_epoch: BigUint::from(7u64),
            }],
            balances: vec![vec![true, false, true], vec![false, true, false]],
            withdrawal_credentials: vec![[false; 256].to_vec()],
            current_epoch: BigUint::from(40u64),
            validator_is_zero: vec![false, false, false],
        };

        // Serialize
        let serialized = serde_json::to_string(&input).unwrap();

        // Deserialize
        let deserialized: ValidatorBalancesInput = serde_json::from_str(&serialized).unwrap();

        // Check that the original and deserialized structs are equal
        assert_eq!(input, deserialized);
    }
}
