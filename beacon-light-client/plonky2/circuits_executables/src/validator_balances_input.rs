use crate::validator::{bool_vec_as_int_vec_nested, bool_vec_as_int_vec};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn from_str<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Vec::deserialize(deserializer)?;
    s.into_iter()
        .map(|str_val| str_val.parse::<u64>().map_err(serde::de::Error::custom))
        .collect()
}

fn to_string<S>(x: &Vec<u64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string_vec: Vec<String> = x.iter().map(|&num| num.to_string()).collect();
    string_vec.serialize(s)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorPoseidon {
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub pubkey: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub withdrawal_credentials: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub effective_balance: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub slashed: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub activation_eligibility_epoch: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub activation_epoch: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub exit_epoch: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub withdrawable_epoch: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorBalancesInput {
    pub validators: Vec<ValidatorPoseidon>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub balances: Vec<Vec<bool>>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub withdrawal_credentials: Vec<u64>,
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub current_epoch: Vec<u64>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub validator_is_zero: Vec<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_deserialize() {
        let input = ValidatorBalancesInput {
            validators: vec![ValidatorPoseidon {
                pubkey: vec![1, 2, 3],
                withdrawal_credentials: vec![4, 5, 6],
                effective_balance: vec![7, 8, 9],
                slashed: vec![10, 11, 12],
                activation_eligibility_epoch: vec![13, 14, 15],
                activation_epoch: vec![16, 17, 18],
                exit_epoch: vec![19, 20, 21],
                withdrawable_epoch: vec![22, 23, 24],
            }],
            balances: vec![vec![true, false, true], vec![false, true, false]],
            withdrawal_credentials: vec![28, 29, 30],
            current_epoch: vec![31, 32],
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
