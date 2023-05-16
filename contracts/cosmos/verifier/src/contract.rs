#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::StdError;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

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
        new_optimistic_header_root: *const u8,
        new_finalized_header_root: *const u8,
        new_execution_state_root: *const u8,
        current_slot: *const u8,
        domain: *const u8
    ) -> bool;
}


fn on_index(arr:&Vec<u8>, pos: usize) -> Vec<u8> {
    let begin: usize = (pos-1)*32;
    let end: usize = pos*32;
    arr[begin..end].to_vec()
}
fn get_current_index_asi32(counter:Vec<u8>) -> i32{
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
    deps.storage.set("current_index".as_bytes(), &1_i32.to_ne_bytes());
    deps.storage.set("current_slot".as_bytes(), &_msg.current_slot.to_ne_bytes());
    deps.storage.set("domain".as_bytes(), &_msg.domain);

    const SIZE_S:usize = 32*32;
    let mut array_optimistic_header_roots: [u8; SIZE_S] = [0 as u8; SIZE_S];
    let empty_array_finalized_header_roots: [u8; SIZE_S] = [0 as u8; SIZE_S];
    let empty_array_execution_state_root: [u8; SIZE_S] = [0 as u8; SIZE_S];

    for i in 0..32{
        array_optimistic_header_roots[i] = _msg.current_header_hash[i];
    }

    deps.storage.set("optimistic_header_hash_array".as_bytes(), &array_optimistic_header_roots);
    deps.storage.set("finalized_header_hash_array".as_bytes(), &empty_array_finalized_header_roots);
    deps.storage.set("execution_state_root_array".as_bytes(), &empty_array_execution_state_root);

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
            new_optimistic_header_root,
            new_finalized_header_root,
            new_execution_state_root,
            new_slot,
        } => execute_update(
            deps,
            _env,
            info,
            proof,
            new_optimistic_header_root,
            new_finalized_header_root,
            new_execution_state_root,
            new_slot,
        ),
    }
}
pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proof: Vec<u8>,
    new_optimistic_header_root: Vec<u8>,
    new_finalized_header_root: Vec<u8>,
    new_execution_state_root: Vec<u8>,
    new_slot: i64,
) -> Result<Response, ContractError> {
    let stored_key = deps.storage.get("key_in_bytes".as_bytes()).unwrap();
    let mut current_index = get_current_index_asi32(deps.storage.get("current_index".as_bytes()).unwrap());

    let mut optimistic_arr = deps.storage.get("optimistic_header_hash_array".as_bytes()).unwrap();
    let mut finalized_arr = deps.storage.get("finalized_header_hash_array".as_bytes()).unwrap();
    let mut execution_state_arr = deps.storage.get("execution_state_root_array".as_bytes()).unwrap();
    let domain = deps.storage.get("domain".as_bytes()).unwrap();

    let stored_current_header_root = on_index(&optimistic_arr,current_index as usize);

    if unsafe {
        makePairsAndVerify(
            stored_key.as_ptr(),
            proof.as_ptr(),
            stored_current_header_root.as_ptr() as *mut u8,
            new_optimistic_header_root.as_ptr(),
            new_finalized_header_root.as_ptr(),
            new_execution_state_root.as_ptr(),
            new_slot.to_ne_bytes().as_ptr(),
            domain.as_ptr())
    } {
        if current_index == 32
        {
            current_index = 1;
        }
        else{
            current_index += 1;
        }
        for i in 0..32 {
            let cur = (current_index-1)*32+i;
            optimistic_arr[cur as usize] = stored_current_header_root[i as usize];
            finalized_arr[cur as usize] = new_finalized_header_root[i as usize];
            execution_state_arr[cur as usize] = new_execution_state_root[i as usize];
        }

        deps.storage.set("optimistic_header_hash_array".as_bytes(), &optimistic_arr);
        deps.storage.set("finalized_header_hash_array".as_bytes(), &finalized_arr);
        deps.storage.set("execution_state_root_array".as_bytes(), &execution_state_arr);
        deps.storage.set("current_index".as_bytes(), &current_index.to_ne_bytes());
        deps.storage.set("current_slot".as_bytes(), &new_slot.to_ne_bytes());

    } else {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "{:?} \n{:?} \n{:?} \n{:?} \n{:?} \n{:?} \n{:?}",
            "Incorrect update",
            &stored_current_header_root,
            &new_optimistic_header_root,
            &new_finalized_header_root,
            &new_execution_state_root,
            &new_slot,
            &domain
        ))));
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::LastHeaderHash {} => get_last_header(_deps, _env),
        QueryMsg::LastFinalizedHeaderHash {} => get_last_finalized(_deps, _env),
        QueryMsg::LastExecStateRoot {} => get_last_exec_state(_deps, _env),
        QueryMsg::HeaderHashBeforeNum {num} => get_header_before_n_headers(_deps, _env, num),
        QueryMsg::AllHeaderHashes {} => get_all_headers(_deps, _env),
        QueryMsg::AllHeaderHashesOrdered {} => get_all_headers_ordered(_deps, _env),
        QueryMsg::AllFinalizedHeaderHashes {} => get_all_finalized(_deps, _env),
        QueryMsg::AllFinalizedHeaderHashesOrdered {} => get_all_finalized_ordered(_deps, _env),
        QueryMsg::AllExecStateRoots {} => get_all_exec_state(_deps, _env),
        QueryMsg::AllExecStateRootsOrdered {} => get_all_exec_state_ordered(_deps, _env),
        QueryMsg::CurrentSlot {} => get_current_slot(_deps, _env),
        QueryMsg::Domain {} => get_domain(_deps, _env),
    }
}

fn get_last_header(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("optimistic_header_hash_array".as_bytes()).unwrap();
    let current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());
    let header = on_index(&optimistic_arr,current_index as usize);
    to_binary(&header)
}
fn get_last_finalized(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let finalized_arr = _deps.storage.get("finalized_header_hash_array".as_bytes()).unwrap();
    let current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());
    let header = on_index(&finalized_arr,current_index as usize);
    to_binary(&header)
}
fn get_last_exec_state(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let exec_state_arr = _deps.storage.get("execution_state_root_array".as_bytes()).unwrap();
    let current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());
    let header = on_index(&exec_state_arr,current_index as usize);
    to_binary(&header)
}

fn get_header_before_n_headers(_deps: Deps, _env: Env, minus_pos: i32) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("optimistic_header_hash_array".as_bytes()).unwrap();
    let current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());
    let mut pos_to_get = current_index - minus_pos;
    if pos_to_get <= 0
    {
        pos_to_get+=32;
    }
    let header = on_index(&optimistic_arr,pos_to_get as usize);
    to_binary(&header)
}

fn get_all_headers(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("optimistic_header_hash_array".as_bytes()).unwrap();

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  on_index(&optimistic_arr, 1 as usize);

    for i in 2..32
    {
        all_headers_ordered.extend_from_slice(&[on_index(&optimistic_arr, i as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn get_all_headers_ordered(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = _deps.storage.get("optimistic_header_hash_array".as_bytes()).unwrap();
    let mut current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  on_index(&optimistic_arr, current_index as usize);

    for _i in 1..32
    {
        current_index-=1;
        if current_index == 0
        {
            current_index = 32;
        }
        all_headers_ordered.extend_from_slice(&[on_index(&optimistic_arr, current_index as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn get_all_finalized(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let finalized_arr = _deps.storage.get("finalized_header_hash_array".as_bytes()).unwrap();

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  on_index(&finalized_arr, 1 as usize);

    for i in 2..32
    {
        all_headers_ordered.extend_from_slice(&[on_index(&finalized_arr, i as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn get_all_finalized_ordered(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let finalized_arr = _deps.storage.get("finalized_header_hash_array".as_bytes()).unwrap();
    let mut current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  on_index(&finalized_arr, current_index as usize);

    for _i in 1..32
    {
        current_index-=1;
        if current_index == 0
        {
            current_index = 32;
        }
        all_headers_ordered.extend_from_slice(&[on_index(&finalized_arr, current_index as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn get_all_exec_state(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let exec_state_arr = _deps.storage.get("execution_state_root_array".as_bytes()).unwrap();

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  on_index(&exec_state_arr, 1 as usize);

    for i in 2..32
    {
        all_headers_ordered.extend_from_slice(&[on_index(&exec_state_arr, i as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn get_all_exec_state_ordered(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let exec_state_arr = _deps.storage.get("execution_state_root_array".as_bytes()).unwrap();
    let mut current_index = get_current_index_asi32(_deps.storage.get("current_index".as_bytes()).unwrap());

    let mut all_headers_ordered: Vec<Vec<u8>> = [[0].to_vec()].to_vec();
    all_headers_ordered[0] =  on_index(&exec_state_arr, current_index as usize);

    for _i in 1..32
    {
        current_index-=1;
        if current_index == 0
        {
            current_index = 32;
        }
        all_headers_ordered.extend_from_slice(&[on_index(&exec_state_arr, current_index as usize)]);
    }
    to_binary(&all_headers_ordered)
}

fn get_current_slot(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let current_slot = _deps.storage.get("current_slot".as_bytes()).unwrap();
    let mut a: [u8; 4] = [0,0,0,0];
    for n in 0..4 {
        a[n] = current_slot[n];
    }
    let slot =  i32::from_ne_bytes(a);

    to_binary(&slot)
}

fn get_domain(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let domain = _deps.storage.get("domain".as_bytes()).unwrap();
    to_binary(&domain)
}

#[cfg(test)]
mod tests {}
