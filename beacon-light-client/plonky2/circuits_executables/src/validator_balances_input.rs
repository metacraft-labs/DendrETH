use crate::{
    crud::common::{biguint_to_str, parse_biguint},
    validator::{bool_vec_as_int_vec, bool_vec_as_int_vec_nested},
};
use num::BigUint;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
pub struct ValidatorPoseidonInput {
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
    pub validators: Vec<ValidatorPoseidonInput>,
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
    pub balances: Vec<String>,
    pub balances_proofs: Vec<Vec<String>>,
    pub validator_deposit_indexes: Vec<u64>,
    pub validators_gindices: Vec<u64>,
    pub validator_commitment_proofs: Vec<Vec<Vec<String>>>,
    pub validators: Vec<ValidatorPoseidonInput>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub validator_is_not_zero: Vec<bool>,
    pub validator_commitment_root: Vec<String>,
    pub current_epoch: u64,
    pub current_eth1_deposit_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_deserialize() {
        let input = ValidatorBalancesInput {
            validators: vec![ValidatorPoseidonInput {
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
