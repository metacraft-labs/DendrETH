use schemars::{schema_for, JsonSchema};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint64, Uint128, Uint512};
use serde::{Serialize, Deserialize};
use std::{fmt::Write, num::ParseIntError, marker::PhantomData};
use typenum::{U512};
use crate::{error::FixedVecError, helpers::BigArray};
// use bls::{PublicKeyBytes};

pub type SyncCommitteeSize = Uint512;

/// The byte-length of a BLS public key when serialized in compressed form.
pub const PUBLIC_KEY_BYTES_LEN: usize = 48;

pub type Hash256 = ([u8; 32]);
#[cw_serde]
pub struct BeaconBlockHeader {
    pub slot: Uint64,
    pub proposer_index: Uint64,
    pub parent_root: Hash256,
    pub state_root: Hash256,
    pub body_root: Hash256,
}

#[cw_serde]
pub struct FixedVector<T, N> {
    pub vec: Vec<T>,
    _phantom: PhantomData<N>,
}

impl<T, N> FixedVector<T, N> {
    /// Returns `Ok` if the given `vec` equals the fixed length of `Self`. Otherwise returns
    /// `Err`.
    pub fn new(vec: Vec<T>) -> Result<Self, FixedVecError> {
        if vec.len() == Self::capacity() {
            Ok(Self {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(FixedVecError::OutOfBounds {
                i: vec.len(),
                len: Self::capacity(),
            })
        }
    }

    /// Create a new vector filled with clones of `elem`.
    pub fn from_elem(elem: T) -> Self
    where
        T: Clone,
    {
        Self {
            // TODO: Find way to use `to_usize()`
            vec: vec![elem; std::mem::size_of::<N>()],
            _phantom: PhantomData,
        }
    }

    /// Identical to `self.capacity`, returns the type-level constant length.
    ///
    /// Exists for compatibility with `Vec`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// True if the type-level constant length of `self` is zero.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the type-level constant length.
    pub fn capacity() -> usize {
        // TODO: Find a way to use U512 (https://docs.rs/typenum/latest/typenum/type.U512.html)
        512
    }
}


// #[cw_serde]
// #[derive(JsonSchema)]
pub type PublicKeyBytes = ([u8; PUBLIC_KEY_BYTES_LEN]);

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Copy)]
pub struct PubKey{
    #[serde(with = "BigArray")]
    pub blob: [u8; PUBLIC_KEY_BYTES_LEN],
}
impl Default for PubKey {
    fn default() -> Self {
        PubKey{
            blob: [0; 48],
        }
    }
}

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Deserialize)]
#[derive(Serialize)]
pub struct HashArray {
    #[serde(with = "BigArray")]
    pub data: [PubKey; 512],
    #[serde(with = "BigArray")]
    pub hashes: [Hash256; 512],
}

impl Default for HashArray {
    fn default() -> Self {
        HashArray{
            data: [PubKey::default(); 512],
            hashes: [Hash256::default(); 512]
        }
    }
}
#[cw_serde]
pub struct SyncCommittee {
    pub pubkeys: FixedVector<PubKey, SyncCommitteeSize>,
    pub aggregate_pubkey: PubKey,
}


#[cw_serde]
pub struct SyncCommitteeDumb {
    pub pubkeys: HashArray,
    pub aggregate_pubkey: PubKey,
}

const _check_sizeof_block_header: [u8; 112] = [0; std::mem::size_of::<BeaconBlockHeader>()];
// const _check_sizeof_sync_committee: [u8; 112] = [0; std::mem::size_of::<SyncCommitteeDumb>()];
