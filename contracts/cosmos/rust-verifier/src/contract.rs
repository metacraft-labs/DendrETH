#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::StdError;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{read_to_string, read_input_from_json, CircuitProof, CircuitVerifyingKey};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rust-verifier";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {



    deps.storage.set("key_in_bytes".as_bytes(), "init value".as_bytes());

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError>
{
    match msg {
        ExecuteMsg::Update {
            proof_path,
            input_path,
            vkey_path,
        } => execute_update(
            deps,
            _env,
            _info,
            proof_path,
            input_path,
            vkey_path,
        ),
    }
}
pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proof_path: String,
    input_path: String,
    vkey_path: String,

) -> Result<Response, ContractError> {

    // let stored_key = deps.storage.get("key_in_bytes".as_bytes()).unwrap();
    let input_str = read_to_string(vkey_path).unwrap();
    let vkey = CircuitVerifyingKey::read_input_from_json(&input_str);
    let out = ark_groth16::VerifyingKey::from(vkey);
    let pvk = ark_groth16::prepare_verifying_key(&out);

    let proof_str = read_to_string(proof_path).unwrap();
    let proof = ark_groth16::Proof::from(CircuitProof::read_input_from_json(&proof_str));

    let pub_input_str = read_to_string(input_path).unwrap();
    let pub_input = read_input_from_json(&pub_input_str);

    if ark_groth16::verify_proof(&pvk, &proof, &pub_input).unwrap() {

        deps.storage.set("optimistic_header_hash_array".as_bytes(), "testing value".as_bytes());


    } else {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "{:?}",
            "Incorrect update",
        ))));
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::LastHeaderHash {} => get_last_header(deps, _env),
    }
}

fn get_last_header(deps: Deps, _env: Env) -> StdResult<Binary> {
    let optimistic_arr = deps.storage.get("optimistic_header_hash_array".as_bytes()).unwrap();
    to_binary(&optimistic_arr)
}

#[cfg(test)]
mod tests {}

