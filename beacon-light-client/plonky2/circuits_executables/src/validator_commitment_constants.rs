use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorCommitmentConstants {
    pub validator_key: String,
    pub validator_proof_key: String,
    pub validator_proofs_queue: String,
    pub validator_balance_input_key: String,
    pub balance_verification_queue: String,
    pub balance_verification_proof_key: String,
    pub final_proof_input_key: String,
    pub final_layer_proof_key: String,
}

pub fn get_validator_commitment_constants() -> ValidatorCommitmentConstants {
    serde_json::from_str(include_str!(
        "../../constants/validator_commitment_constants.json"
    ))
    .unwrap()
}
