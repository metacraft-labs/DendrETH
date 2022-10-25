use schemars::JsonSchema;
use schemars::gen::SchemaGenerator;
use schemars::schema::*;
use hex::{ FromHex, ToHex, };
use serde::ser::{Serialize, SerializeStruct, Serializer, SerializeTuple};

#[macro_use]

use serde;
extern crate serde_json;

use std::fmt;
use std::marker::PhantomData;
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, Error};


use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg};
use crate::{msg::ExecuteMsg, types::{Hash256, PublicKeyBytes, PubKey, HashArray}, ContractError};

fn rem_first_two(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next();
    chars.as_str()
}

pub fn hash256_to_hex_string(bytes: Hash256) -> String {
   "0x".to_string() + &(bytes.encode_hex::<String>())
}

pub fn addr_to_hash256(addr: &Addr) -> Result<Hash256, ContractError>{
    let mut addrAsString = addr.as_str();
    if addrAsString[0..2].eq("0x") {
        addrAsString = rem_first_two(addrAsString);
    }
    let decoded = <[u8; 32]>::from_hex(addrAsString);

    return decoded.map_err(|e| ContractError::InvalidHex(e));
}

pub fn addr_to_public_key_bytes(addr: &Addr) -> Result<PublicKeyBytes, ContractError>{
    let mut addrAsString = addr.as_str();
    if addrAsString[0..2].eq("0x") {
        addrAsString = rem_first_two(addrAsString);
    }
    let decoded = <[u8; 48]>::from_hex(addrAsString);

    return decoded.map_err(|e| ContractError::InvalidHex(e));
}

impl JsonSchema for PubKey {
    fn is_referenceable() -> bool {
        true
    }

    fn schema_name() -> String {
        format!("Array_size_{}_of_{}", 48, u8::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(gen.subschema_for::<u8>().into()),
                max_items: Some(48),
                min_items: Some(48),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }

    fn _schemars_private_is_option() -> bool {
        false
    }
}

impl JsonSchema for HashArray {
    fn is_referenceable() -> bool {
        true
    }

    fn schema_name() -> String {
        format!("Array_size_{}_of_{}", 512, PubKey::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(gen.subschema_for::<PubKey>().into()),
                max_items: Some(512),
                min_items: Some(512),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }

    fn _schemars_private_is_option() -> bool {
        false
    }
}


pub trait BigArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>;
}

macro_rules! big_array {
    ($($len:expr,)+) => {
        $(
            impl<'de, T> BigArray<'de> for [T; $len]
                where T: Default + Copy + Serialize + Deserialize<'de>
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer
                {
                    let mut seq = serializer.serialize_tuple(self.len())?;
                    for elem in &self[..] {
                        seq.serialize_element(elem)?;
                    }
                    seq.end()
                }

                fn deserialize<D>(deserializer: D) -> Result<[T; $len], D::Error>
                    where D: Deserializer<'de>
                {
                    struct ArrayVisitor<T> {
                        element: PhantomData<T>,
                    }

                    impl<'de, T> Visitor<'de> for ArrayVisitor<T>
                        where T: Default + Copy + Deserialize<'de>
                    {
                        type Value = [T; $len];

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("an array of length ", $len))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<[T; $len], A::Error>
                            where A: SeqAccess<'de>
                        {
                            let mut arr = [T::default(); $len];
                            for i in 0..$len {
                                arr[i] = seq.next_element()?
                                    .ok_or_else(|| Error::invalid_length(i, &self))?;
                            }
                            Ok(arr)
                        }
                    }

                    let visitor = ArrayVisitor { element: PhantomData };
                    deserializer.deserialize_tuple($len, visitor)
                }
            }
        )+
    }
}

big_array! {
    40, 48, 50, 56, 64, 72, 96, 100, 128, 160, 192, 200, 224, 256, 384, 512,
    768, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
}
