#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use crate::msg::{BeaconBlockResponse, SlotResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SyncCommitteeResponse, NumberResponse,};

// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{CONFIG, SLOT, SYNCCOMMITTEE, RES};
use crate::types::{BeaconBlockHeader, Hash256, SyncCommitteeDumb, PublicKeyBytes, FixedVector, SyncCommitteeSize, SyncCommittee, PubKey, HashArray};
use crate::helpers::{addr_to_hash256, addr_to_public_key_bytes,};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:light-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
extern "C" {
    fn printSlotOfHeader(a: &BeaconBlockHeader) -> Hash256;
    fn changeParetRootSlotOfHeader(a: &BeaconBlockHeader) -> ();
    fn testSyncCommittee(a: &SyncCommitteeDumb) -> PubKey;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut pubkeysVec: HashArray = HashArray::default();
    let mut index = 0;
    for el in _msg.pubkeys {
        pubkeysVec.data[index] = (PubKey{
            blob: addr_to_public_key_bytes(&el).unwrap(),
        });
        index = index + 1;
    }
    // let pubkey: Result<FixedVector<PubKey, SyncCommitteeSize>, _> = FixedVector::new(pubkeysVec);

    let sync_committee: SyncCommitteeDumb = SyncCommitteeDumb{
        pubkeys: pubkeysVec,
        aggregate_pubkey: PubKey{
            blob: addr_to_public_key_bytes(&_msg.aggregate_pubkey).unwrap(),
        },
    };
    // CONFIG.save(_deps.storage, &header)?;
    SYNCCOMMITTEE.save(_deps.storage, &sync_committee)?;
    let res = unsafe { testSyncCommittee(&sync_committee) };
    RES.save(_deps.storage, &res)?;
    // let res = unsafe { printSlotOfHeader(&header) };
    // SLOT.save(_deps.storage, &res)?;
    // unsafe { changeParetRootSlotOfHeader(&header) };
    // CONFIG.save(_deps.storage, &header)?;

    Ok(Response::default())

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::BeaconBlockHeader {} => to_binary::<BeaconBlockResponse>(&CONFIG.load(_deps.storage)?.into()),
        QueryMsg::SlotResponse {} => to_binary::<SlotResponse>(&SLOT.load(_deps.storage)?.into()),
        QueryMsg::SyncCommittee {} => to_binary::<SyncCommitteeResponse>(&SYNCCOMMITTEE.load(_deps.storage)?.into()),
        QueryMsg::Res {} => query_resolver(_deps, _env),

    }
}

fn query_resolver(_deps: Deps, _env: Env) -> StdResult<Binary> {

    let pubkeys: SyncCommitteeDumb = SYNCCOMMITTEE.load(_deps.storage)?.into();
    let resp = NumberResponse {len: pubkeys,};

    to_binary(&resp)
}
#[cfg(test)]
mod tests {}
