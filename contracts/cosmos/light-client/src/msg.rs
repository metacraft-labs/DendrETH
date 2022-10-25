use std::ops::Add;
use hex;
use crate::state;
use crate::helpers::{hash256_to_hex_string};
use crate::types::{BeaconBlockHeader, Hash256, SyncCommitteeDumb, PubKey, FixedVector, SyncCommitteeSize, SyncCommittee, HashArray};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr,CanonicalAddr, DepsMut, StdResult, Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub pubkeys: Vec<Addr>,
    pub aggregate_pubkey: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BeaconBlockHeader)]
    BeaconBlockHeader {},
    #[returns(SyncCommitteeResponse)]
    SyncCommittee {},
    #[returns(SlotResponse)]
    SlotResponse {},
    #[returns(NumberResponse)]
    Res {},

}
#[cw_serde]
pub struct NumberResponse {
    pub len: SyncCommitteeDumb
}
#[cw_serde]
pub struct BeaconBlockResponse {
    pub slot: Uint64,
    pub proposer_index: Uint64,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

#[cw_serde]
pub struct SlotResponse {
    pub slot: Hash256,
}

#[cw_serde]
pub struct SyncCommitteeResponse {
    pub pubkeys: HashArray,
    pub aggregate_pubkey: PubKey,
}


impl From<BeaconBlockHeader> for BeaconBlockResponse {
    fn from(header: BeaconBlockHeader) -> BeaconBlockResponse {
        BeaconBlockResponse {
            slot: header.slot,
            proposer_index: header.proposer_index,
            parent_root: hash256_to_hex_string(header.parent_root),
            state_root: hash256_to_hex_string(header.state_root),
            body_root: hash256_to_hex_string(header.body_root),
        }
    }
}

impl From<Hash256> for SlotResponse {
    fn from(slot: Hash256) -> SlotResponse {
        SlotResponse {
            slot: slot,
        }
    }
}

// impl From<PubKey> for NumberResponse {
//     fn from(len: PubKey) -> NumberResponse {
//         NumberResponse {
//             len: "TEST".to_string(),
//         }
//     }
// }

impl From<SyncCommitteeDumb> for SyncCommitteeResponse {
    fn from(syncCommittee: SyncCommitteeDumb) -> SyncCommitteeResponse {
        SyncCommitteeResponse {
            pubkeys: syncCommittee.pubkeys,
            aggregate_pubkey: syncCommittee.aggregate_pubkey
        }
    }
}
