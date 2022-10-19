use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128, Uint64};
use cw_storage_plus::{Item, Map};


#[cw_serde]
pub struct BeaconBlockHeader {
    pub slot: Uint64,
    pub proposer_index: Uint64,
    pub parent_root: Addr,
    pub state_root: Addr,
    pub body_root: Addr,
}
// #[cw_serde]
// pub struct LightClientBootstrap {
//     // The requested beacon block header
//     pub header: BeaconBlockHeader,

//     // Current sync committee corresponding to `header`
//     pub current_sync_committee: SyncCommittee,

//     pub current_sync_committee_branch: CurrentSyncCommitteeBranch,
// }

pub const CONFIG: Item<BeaconBlockHeader> = Item::new("config");
