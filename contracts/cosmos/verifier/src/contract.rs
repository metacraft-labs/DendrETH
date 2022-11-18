#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
// use cw2::set_contract_version;
use cosmwasm_std::StdError;
use thiserror::Error;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StoreResponse};
use crate::state::{INTER};
use crate::state::{HEADER};


/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
extern "C" {
    fn testVerify(vkey:i32, proof:i32, input:i32) -> i32;
    fn getVerificationKeySize() -> usize;
    fn testInitializeVKey() -> *mut u8;
    fn createVerificationKeyWithString() -> *mut u8;
    fn getProofSize() -> usize;
    fn createProofWithString() -> *mut u8;
    fn getHeaderSize() -> usize;
    fn createOldHeaderWithString() -> *mut u8;
    fn createNewHeaderWithString() -> *mut u8;
    // fn getInputSize() -> usize;
    // fn createInputWithString() -> *mut u8;
    fn makePairsAndVerifyWithPointers(vk: *const u8, prf: *const u8, oldh: *const u8, newh: *const u8) -> bool;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    INTER.save(_deps.storage, &_msg.vkey);
    HEADER.save(_deps.storage, &_msg.current_header);


    let keyPointer = unsafe { createVerificationKeyWithString() };
    let key_bytes = unsafe { Vec::from_raw_parts(keyPointer, getVerificationKeySize(), 0) };
    _deps.storage.set("key_in_bytes".as_bytes(), &key_bytes);
    let store = _deps.storage.get("key_in_bytes".as_bytes()).unwrap();

    let proofPointer = unsafe { createProofWithString() };
    let proof_bytes = unsafe { Vec::from_raw_parts(proofPointer, getProofSize(), 0) };
    _deps.storage.set("proof_in_bytes".as_bytes(), &proof_bytes);
    let storePrf = _deps.storage.get("proof_in_bytes".as_bytes()).unwrap();

    let oldHeaderPointer = unsafe { createOldHeaderWithString() };
    let old_header_bytes = unsafe { Vec::from_raw_parts(oldHeaderPointer, getHeaderSize(), 0) };
    _deps.storage.set("old_header_in_bytes".as_bytes(), &old_header_bytes);
    let storeOldHeader = _deps.storage.get("old_header_in_bytes".as_bytes()).unwrap();

    let newHeaderPointer = unsafe { createNewHeaderWithString() };
    let new_header_bytes = unsafe { Vec::from_raw_parts(newHeaderPointer, getHeaderSize(), 0) };
    _deps.storage.set("new_header_in_bytes".as_bytes(), &new_header_bytes);
    let storeNewHeader = _deps.storage.get("new_header_in_bytes".as_bytes()).unwrap();

    // let inputPointer = unsafe { createInputWithString() };
    // let input_bytes = unsafe { Vec::from_raw_parts(inputPointer, getInputSize(), 0) };
    // _deps.storage.set("input_in_bytes".as_bytes(), &input_bytes);
    // let storeInput = _deps.storage.get("input_in_bytes".as_bytes()).unwrap();



    let mut tok = 2;
    if unsafe{ makePairsAndVerifyWithPointers(store.as_ptr(), storePrf.as_ptr(), storeOldHeader.as_ptr(), storeNewHeader.as_ptr()) } {
        tok=1;
    }
    // var k = unsafe {makePairsAndVerifyWithPointers(store.as_ptr() ,storePrf.as_ptr() ,storeOldHeader.as_ptr(), storeNewHeader.as_ptr(), storeInput.as_ptr())}

    return Err(ContractError::Std(StdError::generic_err(format!("{:?} ",tok))));
    //    let result = unsafe { testproc(_msg.data1, _msg.data2) };
    //_deps.storage.set("tets".as_bytes(), &result);
//    INTER.save(_deps.storage, &result)?;
//    lcs.save(_deps.storage, &result)?;
    //let light_client_store = unsafe {Vec::from_raw_parts("wewe", 4, 0)};
    //let lcs = "2";

    //_deps.storage.set("lcs".as_bytes(), &lcs);
    //let mut prev_header_hash1: Item<Uint256> = Item::new("uint256");
    //pub prev_header_hash2: Item<Uint256> = Item::new("uint256");
    //pub const Vkey: Item<verification_key> =  Item::new("verification_key")
    //_deps.storage.set(prev_header_hash1.as_???, &prev_header_hash1)
    //_deps.storage.set(prev_header_hash2.as_???, &prev_header_hash2)
    //_deps.storage.set(Vkey.as_???, &Vkey)


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
        ExecuteMsg::Update { proof_input, new_header } =>
          execute_update(deps, _env, info, proof_input, new_header),
    }

    //match _msg {
    //    ExecuteMsg::UpdateAndValidation { new_header_hash1, new_header_hash2} =>
    //        execute_update_and_validation()
    //}
    // unimplemented!()
}
pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proof_input: i32,
    new_header: i32
  ) -> Result<Response, ContractError> {



    Ok(Response::default())
  }

// pub fn execute_update_and_validation(
//     _deps: DepsMut,
//     _env: Env,
//     _info: MessageInfo,
//     _msg: ExecuteMsg,
// ) -> Result<Response, ContractError> {
//     //input = prev_header + new_header
//     //makePairsAndVerify(Vkey,proof??,input)
//     unimplemented!()
//     //OK(Response::default())
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
  match _msg {
    QueryMsg::Store {} => to_binary::<StoreResponse>(&INTER.load(_deps.storage)?.into()),
    QueryMsg::Header {} => to_binary::<StoreResponse>(&HEADER.load(_deps.storage)?.into()),
  }
}




// fn query_resolver(_deps: Deps, _env: Env) -> StdResult<Binary> {
//   let store = _deps.storage.get("test".as_bytes()).unwrap();
//   to_binary(&store)
// }

#[cfg(test)]
mod tests {}
