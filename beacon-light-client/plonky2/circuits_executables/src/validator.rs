use serde::{Deserialize, Serialize};

pub mod bool_vec_as_int_vec {
    use std::fmt;

    use serde::{
        de::{self, SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserializer, Serializer,
    };

    pub fn serialize<S>(x: &Vec<bool>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(x.len()))?;
        for element in x {
            seq.serialize_element(&(*element as i32))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BoolVecVisitor;

        impl<'de> Visitor<'de> for BoolVecVisitor {
            type Value = Vec<bool>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of 0s or 1s")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut bool_vec = Vec::new();
                while let Some(value) = seq.next_element::<i32>()? {
                    match value {
                        0 => bool_vec.push(false),
                        1 => bool_vec.push(true),
                        _ => return Err(de::Error::custom("expected 0 or 1")),
                    }
                }
                Ok(bool_vec)
            }
        }

        deserializer.deserialize_seq(BoolVecVisitor)
    }

    // New functions for Vec<Vec<bool>>
    pub fn serialize_nested<S>(x: &Vec<Vec<bool>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(x.len()))?;
        for vec in x {
            let bool_as_int_vec: Vec<i32> = vec.iter().map(|&x| x as i32).collect();
            seq.serialize_element(&bool_as_int_vec)?;
        }
        seq.end()
    }

    pub fn deserialize_nested<'de, D>(deserializer: D) -> Result<Vec<Vec<bool>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BoolVecVecVisitor;

        impl<'de> Visitor<'de> for BoolVecVecVisitor {
            type Value = Vec<Vec<bool>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of sequences of 0s or 1s")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut bool_vec_vec = Vec::new();
                while let Some(inner_vec) = seq.next_element::<Vec<i32>>()? {
                    let mut bool_vec = Vec::new();
                    for value in inner_vec {
                        match value {
                            0 => bool_vec.push(false),
                            1 => bool_vec.push(true),
                            _ => return Err(de::Error::custom("expected 0 or 1")),
                        }
                    }
                    bool_vec_vec.push(bool_vec);
                }
                Ok(bool_vec_vec)
            }
        }

        deserializer.deserialize_seq(BoolVecVecVisitor)
    }
}

pub mod bool_vec_as_int_vec_nested {
    use serde::{Deserializer, Serializer};

    use super::bool_vec_as_int_vec; // Import the parent module

    // Nested versions of the functions
    pub fn serialize<S>(x: &Vec<Vec<bool>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bool_vec_as_int_vec::serialize_nested(x, s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<bool>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        bool_vec_as_int_vec::deserialize_nested(deserializer)
    }
}

pub const VALIDATOR_REGISTRY_LIMIT: usize = 1099511627776;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorShaInput {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: String,
    pub slashed: String,
    pub activation_eligibility_epoch: String,
    pub activation_epoch: String,
    pub exit_epoch: String,
    pub withdrawable_epoch: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        let validator = ValidatorShaInput {
            pubkey: "933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95".to_string(),
            withdrawal_credentials: "0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50".to_string(),
            effective_balance: "0040597307000000000000000000000000000000000000000000000000000000".to_string(),
            slashed: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            activation_eligibility_epoch: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            activation_epoch: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            exit_epoch: "ffffffffffffffff000000000000000000000000000000000000000000000000".to_string(),
            withdrawable_epoch: "ffffffffffffffff000000000000000000000000000000000000000000000000".to_string(),
        };

        let serialized = serde_json::to_string(&validator).unwrap();
        assert_eq!(serialized, "{\"pubkey\":\"933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95\",\"withdrawalCredentials\":\"0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50\",\"effectiveBalance\":\"0040597307000000000000000000000000000000000000000000000000000000\",\"slashed\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"activationEligibilityEpoch\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"activationEpoch\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"exitEpoch\":\"ffffffffffffffff000000000000000000000000000000000000000000000000\",\"withdrawableEpoch\":\"ffffffffffffffff000000000000000000000000000000000000000000000000\"}");
    }

    #[test]
    fn test_deserialize() {
        let data = "{\"pubkey\":\"933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95\",\"withdrawalCredentials\":\"0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50\",\"effectiveBalance\":\"0040597307000000000000000000000000000000000000000000000000000000\",\"slashed\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"activationEligibilityEpoch\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"activationEpoch\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"exitEpoch\":\"ffffffffffffffff000000000000000000000000000000000000000000000000\",\"withdrawableEpoch\":\"ffffffffffffffff000000000000000000000000000000000000000000000000\"}";
        let deserialized: ValidatorShaInput = serde_json::from_str(data).unwrap();

        assert_eq!(deserialized.pubkey, "933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95");
        assert_eq!(
            deserialized.withdrawal_credentials,
            "0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50"
        );
        assert_eq!(deserialized.effective_balance, "0040597307000000000000000000000000000000000000000000000000000000");
        assert_eq!(deserialized.slashed, "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(
            deserialized.activation_eligibility_epoch,
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(deserialized.activation_epoch, "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(deserialized.exit_epoch, "ffffffffffffffff000000000000000000000000000000000000000000000000");
        assert_eq!(deserialized.withdrawable_epoch, "ffffffffffffffff000000000000000000000000000000000000000000000000");
    }
}
