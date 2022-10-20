use std::{process::exit, io::stdout, str::from_utf8};
use std::convert::TryFrom;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128, Uint64};
use std::{fmt::Write, num::ParseIntError};
use crate::ContractError;

use hex::FromHex;


pub type Hash256 = ([u8; 32]);

fn slice_to_arr64<T>(slice: &[T]) -> Option<&[T; 64]> {
    if slice.len() == 64 {
        Some(unsafe { &*(slice as *const [T] as *const [T; 64]) })
    } else {
        None
    }
}

fn rem_first_two(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next();
    chars.as_str()
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn vector_as_u8_32_array(vector: Vec<u8>) -> [u8;32] {
    let mut arr = [0u8;32];
    for i in (0..32) {
        arr[i] = vector[i];
    }
    arr
}

#[cw_serde]
pub struct BeaconBlockHeader {
    pub slot: Uint64,
    pub proposer_index: Uint64,
    pub parent_root: Hash256,
    pub state_root: Hash256,
    pub body_root: Hash256,
}

#[no_mangle]
pub fn addrToHash256(addr: &Addr) -> Result<Hash256, ContractError>{
    // TODO FIX ME
    let bytes = addr.as_str();
    let hash = &bytes[2..bytes.len()];
    let decoded = <[u8; 32]>::from_hex(rem_first_two(addr.as_str()));
    if hash.len() == 64 {
        return Ok(decoded.unwrap());
    }else {
        return Err(ContractError::CavemanError("PROBLEM".to_string()));
    }
}
