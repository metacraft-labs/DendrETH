use num::BigUint;
use serde::{Deserialize, Deserializer, Serializer};

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

pub mod serde_bool_array_to_hex_string {
    use core::fmt;

    use circuit::array::Array;
    use serde::{
        de::{self, Visitor},
        Deserializer, Serializer,
    };

    use crate::utils::utils::{bits_to_bytes, bytes_to_bits};

    pub fn serialize<S, const N: usize>(x: &Array<bool, N>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = hex::encode(bits_to_bytes(x.as_slice()));
        s.serialize_str(&hex_string)
    }

    pub struct HexStringVisitor<const N: usize>;

    impl<'de, const N: usize> Visitor<'de> for HexStringVisitor<N> {
        type Value = Array<bool, N>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a hex string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Array(
                bytes_to_bits(&hex::decode(v).unwrap()).try_into().unwrap(),
            ))
        }
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<Array<bool, N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexStringVisitor)
    }
}

pub mod serde_bool_array_to_hex_string_nested {
    use core::fmt;

    use circuit::array::Array;
    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserializer, Serializer,
    };

    use crate::utils::utils::{bits_to_bytes, bytes_to_bits};

    pub fn serialize<S, const N: usize, const M: usize>(
        x: &Array<Array<bool, M>, N>,
        s: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = s.serialize_tuple(N)?;
        for bits_array in x.iter() {
            let hex_string = hex::encode(bits_to_bytes(bits_array.as_slice()));
            tup.serialize_element(&hex_string)?;
        }
        tup.end()
    }

    pub fn deserialize<'de, D, const N: usize, const M: usize>(
        deserializer: D,
    ) -> Result<Array<Array<bool, M>, N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MultipleHexStringsVisitor<const N: usize, const M: usize>;

        impl<'de, const N: usize, const M: usize> Visitor<'de> for MultipleHexStringsVisitor<N, M> {
            type Value = Array<Array<bool, M>, N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of sequences of 0s or 1s")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(Array([(); N].map(|_| {
                    let Some(hex_string) = seq.next_element::<&str>().unwrap() else {
                        panic!("Could not deserialize hex string: not enough elements");
                    };
                    Array(
                        bytes_to_bits(&hex::decode(hex_string).unwrap())
                            .try_into()
                            .unwrap(),
                    )
                })))
            }
        }

        deserializer.deserialize_seq(MultipleHexStringsVisitor)
    }
}

pub fn biguint_to_str<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str_value = value.to_str_radix(10);
    serializer.serialize_str(&str_value)
}

pub fn parse_biguint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let str_value = String::deserialize(deserializer)?;

    str_value
        .parse::<BigUint>()
        .map_err(serde::de::Error::custom)
}
