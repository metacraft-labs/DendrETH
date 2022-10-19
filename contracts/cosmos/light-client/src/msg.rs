use crate::state::BeaconBlockHeader;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128, Uint64};

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
}
#[cw_serde]
pub struct ConfigResponse {
    pub slot: Uint64,
    pub proposer_index: Uint64,
    pub parent_root: Addr,
    pub state_root: Addr,
    pub body_root: Addr,
}

#[cw_serde]
pub struct SlotResponse {
    pub slot: u32,
}

impl From<BeaconBlockHeader> for ConfigResponse {
    fn from(header: BeaconBlockHeader) -> ConfigResponse {
        ConfigResponse {
            slot: header.slot,
            proposer_index: header.proposer_index,
            parent_root: header.parent_root,
            state_root: header.state_root,
            body_root: header.body_root,
        }
    }
}

impl From<u32> for SlotResponse {
    fn from(slot: u32) -> SlotResponse {
        SlotResponse {
            slot: slot,
        }
    }
}
