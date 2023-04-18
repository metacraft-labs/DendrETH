#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::StdError;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use thiserror::Error;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
extern "C" {
    fn makePairsAndVerify(
        vk: *const u8,
        prf: *const u8,
        current_header_root: *mut u8,
        new_optimistic_header: *const u8,
        new_finalized_header: *const u8,
        new_execution_state_root: *const u8
    ) -> bool;
}


fn OnIndex(arr:&Vec<u8>, pos: usize) -> Vec<u8> {
    let begin: usize = (pos-1)*32;
    let end: usize = pos*32;
    arr[begin..end].to_vec()
}
fn GetCurrentPosition(counter:Vec<u8>) -> i32{
    let mut a: [u8; 4] = [0,0,0,0];
    for n in 0..4 {
        a[n] = counter[n];
    }
    i32::from_ne_bytes(a)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.storage.set("key_in_bytes".as_bytes(), &_msg.vkey);
    deps.storage.set("current_position".as_bytes(), &1_i32.to_ne_bytes());

    const SIZE_S:usize = 32*32;
    let mut current_header_hash_array: [u8; SIZE_S] = [0 as u8; SIZE_S];
    let empty_array_finalized_header: [u8; SIZE_S] = [0 as u8; SIZE_S];
    let empty_array_execution_state_root: [u8; SIZE_S] = [0 as u8; SIZE_S];

    for i in 0..32{
        current_header_hash_array[i] = _msg.current_header_hash[i];
    }

    deps.storage.set("OptimisticHeader_array".as_bytes(), &current_header_hash_array);
    deps.storage.set("FinalizedHeader_array".as_bytes(), &empty_array_finalized_header);
    deps.storage.set("ExecutionStateRoot_array".as_bytes(), &empty_array_execution_state_root);

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
        ExecuteMsg::Update {
            proof,
            new_optimistic_header,
            new_finalized_header,
            new_execution_state_root,
        } => execute_update(
            deps,
            _env,
            info,
            proof,
            new_optimistic_header,
            new_finalized_header,
            new_execution_state_root,
        ),
    }
}
pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proof: Vec<u8>,
    new_optimistic_header: Vec<u8>,
    new_finalized_header: Vec<u8>,
    new_execution_state_root: Vec<u8>,
) -> Result<Response, ContractError> {
    let stored_key = deps.storage.get("key_in_bytes".as_bytes()).unwrap();
    let mut current_position = GetCurrentPosition(deps.storage.get("current_position".as_bytes()).unwrap());

    let mut optimistic_arr = deps.storage.get("OptimisticHeader_array".as_bytes()).unwrap();
    let mut finalized_arr = deps.storage.get("FinalizedHeader_array".as_bytes()).unwrap();
    let mut execution_state_arr = deps.storage.get("ExecutionStateRoot_array".as_bytes()).unwrap();

    let stored_current_header_root = OnIndex(&optimistic_arr,current_position as usize);

    if unsafe {
        makePairsAndVerify(
            stored_key.as_ptr(),
            proof.as_ptr(),
            stored_current_header_root.as_ptr() as *mut u8,
            new_optimistic_header.as_ptr(),
            new_finalized_header.as_ptr(),
            new_execution_state_root.as_ptr()
                )
    } {
        if current_position == 32
        {
            current_position = 1;
        }
        else{
            current_position += 1;
        }
        for i in 0..32 {
            let cur = (current_position-1)*32+i;
            optimistic_arr[cur as usize] = stored_current_header_root[i as usize];
            finalized_arr[cur as usize] = new_finalized_header[i as usize];
            execution_state_arr[cur as usize] = new_execution_state_root[i as usize];
        }

        deps.storage.set("OptimisticHeader_array".as_bytes(), &optimistic_arr);
        deps.storage.set("FinalizedHeader_array".as_bytes(), &finalized_arr);
        deps.storage.set("ExecutionStateRoot_array".as_bytes(), &execution_state_arr);
        deps.storage.set("current_position".as_bytes(), &current_position.to_ne_bytes());

    } else {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "{:?} \n{:?} \n{:?} \n{:?} \n{:?}",
            "Incorrect update",
            &stored_current_header_root,
            &new_optimistic_header,
            &new_finalized_header,
            &new_execution_state_root
        ))));
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::LastHeaderHash {} => return_last_header(_deps, _env),
        QueryMsg::LastFinalizedHeaderHash {} => return_last_finalized(_deps, _env),
        QueryMsg::LastExecStateRoot {} => return_last_exec_state(_deps, _env),
        QueryMsg::HeaderHashBeforeNum {num} => return_header_before_n_headers(_deps, _env, num),
        QueryMsg::AllHeaders {} => return_all_headers(_deps, _env),
        QueryMsg::AllHeadersOrdered {} => return_all_headers_ordered(_deps, _env),
        QueryMsg::AllFinalizedHeaders {} => return_all_finalized(_deps, _env),
        QueryMsg::AllFinalizedHeadersOrdered {} => return_all_finalized_ordered(_deps, _env),
        QueryMsg::AllExecStateRoots {} => return_all_exec_state(_deps, _env),
        QueryMsg::AllExecStateRootsOrdered {} => return_all_exec_state_ordered(_deps, _env),
    }
}

fn return_last_header(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("OptimisticHeader_array".as_bytes()).unwrap();
    let current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());
    let header = OnIndex(&optimistic_arr,current_position as usize);
    to_binary(&header)
}
fn return_last_finalized(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let finalized_arr = _deps.storage.get("FinalizedHeader_array".as_bytes()).unwrap();
    let current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());
    let header = OnIndex(&finalized_arr,current_position as usize);
    to_binary(&header)
}
fn return_last_exec_state(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let exec_state_arr = _deps.storage.get("ExecutionStateRoot_array".as_bytes()).unwrap();
    let current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());
    let header = OnIndex(&exec_state_arr,current_position as usize);
    to_binary(&header)
}

fn return_header_before_n_headers(_deps: Deps, _env: Env, minus_pos: i32) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("OptimisticHeader_array".as_bytes()).unwrap();
    let current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());
    let mut pos_to_return = current_position - minus_pos;
    if pos_to_return <= 0
    {
        pos_to_return+=32;
    }
    let header = OnIndex(&optimistic_arr,pos_to_return as usize);
    to_binary(&header)
}

fn return_all_headers(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("OptimisticHeader_array".as_bytes()).unwrap();

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  OnIndex(&optimistic_arr, 1 as usize);

    for i in 2..32
    {
        all_headers_ordered.extend_from_slice(&[OnIndex(&optimistic_arr, i as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn return_all_headers_ordered(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("OptimisticHeader_array".as_bytes()).unwrap();
    let mut current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  OnIndex(&optimistic_arr, current_position as usize);

    for _i in 1..32
    {
        current_position-=1;
        if current_position == 0
        {
            current_position = 32;
        }
        all_headers_ordered.extend_from_slice(&[OnIndex(&optimistic_arr, current_position as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn return_all_finalized(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let finalized_arr = _deps.storage.get("FinalizedHeader_array".as_bytes()).unwrap();

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  OnIndex(&finalized_arr, 1 as usize);

    for i in 2..32
    {
        all_headers_ordered.extend_from_slice(&[OnIndex(&finalized_arr, i as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn return_all_finalized_ordered(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let finalized_arr = _deps.storage.get("FinalizedHeader_array".as_bytes()).unwrap();
    let mut current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  OnIndex(&finalized_arr, current_position as usize);

    for _i in 1..32
    {
        current_position-=1;
        if current_position == 0
        {
            current_position = 32;
        }
        all_headers_ordered.extend_from_slice(&[OnIndex(&finalized_arr, current_position as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn return_all_exec_state(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let exec_state_arr = _deps.storage.get("ExecutionStateRoot_array".as_bytes()).unwrap();

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  OnIndex(&exec_state_arr, 1 as usize);

    for i in 2..32
    {
        all_headers_ordered.extend_from_slice(&[OnIndex(&exec_state_arr, i as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn return_all_exec_state_ordered(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let exec_state_arr = _deps.storage.get("ExecutionStateRoot_array".as_bytes()).unwrap();
    let mut current_position = GetCurrentPosition(_deps.storage.get("current_position".as_bytes()).unwrap());

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  OnIndex(&exec_state_arr, current_position as usize);

    for _i in 1..32
    {
        current_position-=1;
        if current_position == 0
        {
            current_position = 32;
        }
        all_headers_ordered.extend_from_slice(&[OnIndex(&exec_state_arr, current_position as usize)]);
    }
    to_binary(&all_headers_ordered)
}

#[cfg(test)]
mod tests {}
