use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DBConstants<'a> {
    pub validator_key: &'a str,
    pub validator_proof_key: &'a str,
    pub validator_proofs_queue: &'a str,
    pub validator_balance_input_key: &'a str,
    pub balance_verification_queue: &'a str,
    pub balance_verification_proof_key: &'a str,
    pub final_proof_input_key: &'a str,
    pub final_layer_proof_key: &'a str,
    pub slot_lookup_key: &'a str,
    pub validator_proof_storage: &'a str,
    pub balance_verification_proof_storage: &'a str,
    pub validators_length_key: &'a str,
    pub validators_root_key: &'a str,
    pub validator_accumulator_key: &'a str,
    pub validator_accumulator_proof_key: &'a str,
    pub validator_accumulator_proof_queue: &'a str,
    pub balance_verification_accumulator_proof_queue: &'a str,
    pub balance_verification_accumulator_key: &'a str,
    pub balance_verification_accumulator_proof_key: &'a str,
    pub bls_verification_queue: &'a str,
    // pub bls_verification_proof_key: &'a str,
}

pub fn get_db_constants() -> DBConstants<'static> {
    serde_json::from_str(include_str!("../../../kv_db_constants.json")).unwrap()
}

pub static DB_CONSTANTS: Lazy<DBConstants> = Lazy::new(|| get_db_constants());
