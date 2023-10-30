use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorCommitmentConstants<'a> {
    pub validator_key: &'a str,
    pub validator_proof_key: &'a str,
    pub validator_proofs_queue: &'a str,
    pub validator_accumulator_key: &'a str,
    pub validator_accumulator_proof_key: &'a str,
    pub validator_accumulator_proof_queue: &'a str,
    pub validator_balance_input_key: &'a str,
    pub balance_verification_queue: &'a str,
    pub balance_verification_accumulator_proof_queue: &'a str,
    pub balance_verification_proof_key: &'a str,
    pub balance_verification_accumulator_key: &'a str,
    pub balance_verification_accumulator_proof_key: &'a str,
    pub final_proof_input_key: &'a str,
    pub final_layer_proof_key: &'a str,
    pub epoch_lookup_key: &'a str,
    pub validator_proof_storage: &'a str,
    pub balance_verification_proof_storage: &'a str,
    pub validators_length_key: &'a str,
    pub validators_root_key: &'a str,
}

pub fn get_validator_commitment_constants() -> ValidatorCommitmentConstants<'static> {
    serde_json::from_str(include_str!(
        "../../constants/validator_commitment_constants.json"
    ))
    .unwrap()
}

pub static VALIDATOR_COMMITMENT_CONSTANTS: Lazy<ValidatorCommitmentConstants> =
    Lazy::new(|| get_validator_commitment_constants());
