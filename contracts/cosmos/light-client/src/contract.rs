#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use crate::msg::{ConfigResponse, SlotResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{CONFIG, SLOT};
use crate::types::{BeaconBlockHeader, addrToHash256, Hash256};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:light-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
extern "C" {
    fn printSlotOfHeader(a: &BeaconBlockHeader) -> Hash256;
    fn changeParetRootSlotOfHeader(a: &BeaconBlockHeader) -> ();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let header: BeaconBlockHeader = BeaconBlockHeader{
        slot: _msg.slot,
        proposer_index: _msg.proposer_index,
        parent_root: addrToHash256(&_msg.parent_root).unwrap(),
        state_root: addrToHash256(&_msg.state_root).unwrap(),
        body_root: addrToHash256(&_msg.body_root).unwrap(),
    };
    CONFIG.save(_deps.storage, &header)?;

    let res = unsafe { printSlotOfHeader(&header) };
    SLOT.save(_deps.storage, &res)?;
    unsafe { changeParetRootSlotOfHeader(&header) };
    CONFIG.save(_deps.storage, &header)?;

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
        QueryMsg::BeaconBlockHeader {} => to_binary::<BeaconBlockHeader>(&CONFIG.load(_deps.storage)?.into()),
        QueryMsg::SlotResponse {} => to_binary::<SlotResponse>(&SLOT.load(_deps.storage)?.into()),
    }
}

#[cfg(test)]
mod tests {}
