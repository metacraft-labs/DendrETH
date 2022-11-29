#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cosmwasm_std::StdError;
use thiserror::Error;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StoreResponse};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
extern "C" {
    fn makePairsAndVerify(vk: *const u8, prf: *const u8, currentHeader: *const u8, newHeader: *const u8) -> bool;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    deps.storage.set("key_in_bytes".as_bytes(), &_msg.vkey);
    deps.storage.set("current_header_in_bytes".as_bytes(), &_msg.currentHeader);

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
     match msg {
        ExecuteMsg::Update { proof, newHeader } =>
          execute_update(deps, _env, info, proof, newHeader),
    }

}
pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proof: Vec<u8>,
    newHeader: Vec<u8>,
  ) -> Result<Response, ContractError> {
    let storedKey = deps.storage.get("key_in_bytes".as_bytes()).unwrap();
    let storedCurrentHeader = deps.storage.get("current_header_in_bytes".as_bytes()).unwrap();

    if unsafe{ makePairsAndVerify(storedKey.as_ptr(), proof.as_ptr(), storedCurrentHeader.as_ptr(), newHeader.as_ptr()) } {
        deps.storage.set("current_header_in_bytes".as_bytes(), &newHeader);
    } else { return Err(ContractError::Std(StdError::generic_err(format!("{:?} ", "Incorrect update")))); }

    Ok(Response::default())
  }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
  match _msg {
    QueryMsg::Header {} => query_resolver(_deps, _env),
}
}
fn query_resolver(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let header = _deps.storage.get("current_header_in_bytes".as_bytes()).unwrap();
    to_binary(&header)
}

#[cfg(test)]
mod tests {}
