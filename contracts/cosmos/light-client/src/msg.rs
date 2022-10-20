use std::ops::Add;
use core::str;
use hex;
use crate::state;
use crate::types::{BeaconBlockHeader, Hash256};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr,CanonicalAddr, DepsMut, StdResult, Uint128, Uint64};

// #[cw_serde]
#[cw_serde]
pub struct InstantiateMsg {
    pub slot: Uint64,
    pub proposer_index: Uint64,
    pub parent_root: Addr,
    pub state_root: Addr,
    pub body_root: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // #[returns(ConfigResponse)]
    // Config {},
    #[returns(BeaconBlockHeader)]
    BeaconBlockHeader {},
    #[returns(SlotResponse)]
    SlotResponse {},
}
#[cw_serde]
pub struct ConfigResponse {
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

impl From<BeaconBlockHeader> for ConfigResponse {
    fn from(header: BeaconBlockHeader) -> ConfigResponse {
        let parent_root = str::from_utf8(&header.parent_root);
        let state_root = str::from_utf8(&header.state_root);
        let body_root = str::from_utf8(&header.body_root);

        ConfigResponse {
            slot: header.slot,
            proposer_index: header.proposer_index,
            parent_root: parent_root.unwrap().to_string(),
            state_root: state_root.unwrap().to_string(),
            body_root: body_root.unwrap().to_string(),
            // proposer_index: header.proposer_index,
            // parent_root: CanonicalAddr::from(header.parent_root.as_bytes()),
            // state_root: CanonicalAddr::from(header.state_root.as_bytes()),
            // body_root: CanonicalAddr::from(header.body_root.as_bytes())
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
