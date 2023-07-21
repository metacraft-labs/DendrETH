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
pub struct Validator {
    #[serde(with = "bool_vec_as_int_vec")]
    pub pubkey: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub withdrawal_credentials: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub effective_balance: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub slashed: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub activation_eligibility_epoch: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub activation_epoch: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub exit_epoch: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub withdrawable_epoch: Vec<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        let validator = Validator {
            pubkey: vec![true, false, true],
            withdrawal_credentials: vec![false, false, true],
            effective_balance: vec![true, true, false],
            slashed: vec![false, true, false],
            activation_eligibility_epoch: vec![true, false, true],
            activation_epoch: vec![true, false, false],
            exit_epoch: vec![false, true, true],
            withdrawable_epoch: vec![true, true, true],
        };

        let serialized = serde_json::to_string(&validator).unwrap();
        assert_eq!(serialized, "{\"pubkey\":[1,0,1],\"withdrawalCredentials\":[0,0,1],\"effectiveBalance\":[1,1,0],\"slashed\":[0,1,0],\"activationEligibilityEpoch\":[1,0,1],\"activationEpoch\":[1,0,0],\"exitEpoch\":[0,1,1],\"withdrawableEpoch\":[1,1,1]}");
    }

    #[test]
    fn test_deserialize() {
        let data = "{\"pubkey\":[1,0,1],\"withdrawalCredentials\":[0,0,1],\"effectiveBalance\":[1,1,0],\"slashed\":[0,1,0],\"activationEligibilityEpoch\":[1,0,1],\"activationEpoch\":[1,0,0],\"exitEpoch\":[0,1,1],\"withdrawableEpoch\":[1,1,1]}";
        let deserialized: Validator = serde_json::from_str(data).unwrap();

        assert_eq!(deserialized.pubkey, vec![true, false, true]);
        assert_eq!(
            deserialized.withdrawal_credentials,
            vec![false, false, true]
        );
        assert_eq!(deserialized.effective_balance, vec![true, true, false]);
        assert_eq!(deserialized.slashed, vec![false, true, false]);
        assert_eq!(
            deserialized.activation_eligibility_epoch,
            vec![true, false, true]
        );
        assert_eq!(deserialized.activation_epoch, vec![true, false, false]);
        assert_eq!(deserialized.exit_epoch, vec![false, true, true]);
        assert_eq!(deserialized.withdrawable_epoch, vec![true, true, true]);
    }
}
