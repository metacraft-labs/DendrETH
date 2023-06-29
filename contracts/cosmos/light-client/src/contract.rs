#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
  Binary,
  Deps,
  DepsMut,
  Env,
  MessageInfo,
  Response,
  StdResult,
  to_binary,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg };

extern crate base64;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:light-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

extern "C" {
  fn getLightClientStoreSize() -> usize;
  fn initializeLightClientStoreCosmos(offset: *const u8, len: usize) -> *mut u8;
  fn processLightClientUpdate(offset: *const u8, len: usize, store: *const u8);
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  _deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  _msg: InstantiateMsg
) -> Result<Response, ContractError> {
  let bootstrap_bytes = base64::decode(_msg.bootstrap_data).unwrap();
  let store = unsafe {
    initializeLightClientStoreCosmos(
      bootstrap_bytes.as_ptr(),
      bootstrap_bytes.len()
    )
  };
  let light_client_store = unsafe {
    Vec::from_raw_parts(store, getLightClientStoreSize(), 0)
  };

  _deps.storage.set("light_client_store".as_bytes(), &light_client_store);

  Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  msg: ExecuteMsg
) -> Result<Response, ContractError> {
  match msg {
    ExecuteMsg::Update { update_data } =>
      execute_update(deps, _env, info, update_data),
  }
}

pub fn execute_update(
  deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  update_data: String
) -> Result<Response, ContractError> {
  let update_bytes = base64::decode(update_data).unwrap();
  let light_client_store = deps.storage
    .get("light_client_store".as_bytes())
    .unwrap();

  unsafe {
    processLightClientUpdate(
      update_bytes.as_ptr(),
      update_bytes.len(),
      light_client_store.as_ptr()
    );
  }

  deps.storage.set("light_client_store".as_bytes(), &light_client_store);

  Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
  match _msg {
    QueryMsg::Store {} => query_resolver(_deps, _env),
  }
}
fn query_resolver(_deps: Deps, _env: Env) -> StdResult<Binary> {
  let store = _deps.storage.get("light_client_store".as_bytes()).unwrap();
  to_binary(&store)
}

#[cfg(test)]
mod tests {}
