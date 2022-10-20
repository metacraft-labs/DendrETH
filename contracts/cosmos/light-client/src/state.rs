use crate::types::{BeaconBlockHeader, Hash256};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128, Uint64};
use cw_storage_plus::{Item, Map};


// #[cw_serde]
// pub struct LightClientBootstrap {
//     // The requested beacon block header
//     pub header: BeaconBlockHeader,

//     // Current sync committee corresponding to `header`
//     pub current_sync_committee: SyncCommittee,

//     pub current_sync_committee_branch: CurrentSyncCommitteeBranch,
// }
pub const CONFIG: Item<BeaconBlockHeader> = Item::new("config");
pub const SLOT: Item<Hash256> = Item::new("slot");


// const TestChecker: [u8; 4] = [0; std::mem::size_of::<BeaconBlockHeader>()];
