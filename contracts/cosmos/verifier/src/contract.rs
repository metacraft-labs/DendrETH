#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
// use cw2::set_contract_version;
use cosmwasm_std::StdError;
use thiserror::Error;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StoreResponse};
use crate::state::{inter};
use crate::state::{header};


/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
extern "C" {
    fn testproc(a:i32, b:i32) -> i32;
    fn testproc2(a:i32, b:i32) -> bool;
    fn testVerify(vkey:i32, proof:i32, input:i32) -> i32;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    inter.save(_deps.storage, &_msg.vkey);
    header.save(_deps.storage, &_msg.currentHeader);

    //    let result = unsafe { testproc(_msg.data1, _msg.data2) };
    //return Err(ContractError::Std(StdError::generic_err(format!("{:?}", result))));
    //_deps.storage.set("tets".as_bytes(), &result);
//    inter.save(_deps.storage, &result)?;
//    lcs.save(_deps.storage, &result)?;
    //let light_client_store = unsafe {Vec::from_raw_parts("wewe", 4, 0)};
    //let lcs = "2";

    //_deps.storage.set("lcs".as_bytes(), &lcs);
    //let mut prev_header_hash1: Item<Uint256> = Item::new("uint256");
    //pub prev_header_hash2: Item<Uint256> = Item::new("uint256");
    //pub const Vkey: Item<VerificationKey> =  Item::new("VerificationKey")
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
        ExecuteMsg::Update { proofInput, newHeader } =>
          execute_update(deps, _env, info, proofInput, newHeader),
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
    proofInput: i32,
    newHeader: i32
  ) -> Result<Response, ContractError> {
    //get the verificationKey that we have saved on the contract
    let verificationKey = inter.load(deps.storage)?;
    //received proof
    let proof = proofInput;
    //preparing inputs
    let input = unsafe{ testproc(newHeader, header.load(deps.storage)?) };
    //verify
    let result = unsafe { testVerify(verificationKey,proof,input)};
    //if correct new header then save it
    if (result == 0){
        header.save(deps.storage, &newHeader);
    }



    //let result = unsafe { testproc(sth1, update_data) };
    //let newInt = update_data;
    // if (result == 84){
    //     inter.save(deps.storage, &result)?;
    // }
    // else {
    //     inter.save(deps.storage, &sth1)?;
    // }

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
    QueryMsg::Store {} => to_binary::<StoreResponse>(&inter.load(_deps.storage)?.into()),
    QueryMsg::Header {} => to_binary::<StoreResponse>(&header.load(_deps.storage)?.into()),
  }
}




// fn query_resolver(_deps: Deps, _env: Env) -> StdResult<Binary> {
//   let store = _deps.storage.get("test".as_bytes()).unwrap();
//   to_binary(&store)
// }

#[cfg(test)]
mod tests {}
