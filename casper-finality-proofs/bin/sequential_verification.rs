use casper_finality_proofs::constants::{STATE_ROOT_PROOF_LEN, VALIDATORS_ROOT_PROOF_LEN};
use casper_finality_proofs::prove_casper::{
    vad_recurssive::vad_recursive,
    cuv_recursive::cuv_recursive
};

use casper_finality_proofs::prove_finality::circuit_final::ProveFinality;
use casper_finality_proofs::prove_finality::prove_finality::prove_finality;
use casper_finality_proofs::utils::eth_objects::BeaconStateInput;
use casper_finality_proofs::utils::json::read_json_from_file;
use plonky2x::frontend::uint::uint64::U64Variable;
use plonky2x::frontend::vars::Bytes32Variable;
use plonky2x::utils::bytes32;
use std::io::Error as IOError;

use plonky2x::{
    backend::circuit::{DefaultParameters, PublicOutput},
    prelude::CircuitBuilder,
};

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let is_poseidon_hash = true;

    let file_path_attestations;
    if is_poseidon_hash {
        file_path_attestations = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/merged_234400_poseidon.json";
    }
    else {
        file_path_attestations = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/merged_234400.json";
    }

    println!("vad_proof_prev:");
    
    let (vad_proof_prev, vad_circuit_prev) 
        = vad_recursive::<L,D>(file_path_attestations, is_poseidon_hash);

    println!("cuv_proof_prev:");
    let file_path_sorted_validators = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/sorted_validator_indices_234400.json";
    let (cuv_proof_prev, cuv_circuit_prev) = 
        cuv_recursive::<L,D>(file_path_sorted_validators);

    println!("vad_proof_cur:");
    let file_path_attestations = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/merged_234401.json";
    let (vad_proof_cur, vad_circuit_cur) 
        = vad_recursive::<L,D>(file_path_attestations, is_poseidon_hash);

    println!("cuv_proof_cur:");
    let file_path_sorted_validators = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/sorted_validator_indices_234401.json";
    let (cuv_proof_cur, cuv_circuit_cur)
        = cuv_recursive::<L,D>(file_path_sorted_validators);

    let file_path_beacon_state 
        = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/justification_7500768.json";

    let (_finality_proof, _finality_circuit) 
        = prove_finality(
            file_path_beacon_state,

            vad_circuit_cur,
            vad_proof_cur,

            cuv_circuit_cur,
            cuv_proof_cur,

            vad_circuit_prev,
            vad_proof_prev,

            cuv_circuit_prev,
            cuv_proof_prev
    );
    println!("Finality Proof Done");

    // let file_path_beacon_state 
    //     = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/justification_7500768.json";

    // let beacon_state_json = read_json_from_file(file_path_beacon_state).unwrap();

    // let beacon_state_value = beacon_state_json.get("state").unwrap();
    // // let beacon_state: BeaconStateInput = serde_json::from_value(beacon_state_value.clone()).unwrap();

    // let justification_bits = beacon_state_value["justification_bits"].as_u64().unwrap();
    // let previous_justified_checkpoint = &beacon_state_value["previous_justified_checkpoint"];
    // let current_justified_checkpoint = &beacon_state_value["current_justified_checkpoint"];

    // let beacon_state = BeaconStateInput {
    //     justification_bits: justification_bits,
    //     previous_justified_checkpoint: serde_json::from_value(previous_justified_checkpoint.clone()).unwrap(),
    //     current_justified_checkpoint: serde_json::from_value(current_justified_checkpoint.clone()).unwrap()
    // };
    // beacon_state.write(&mut input);
    // println!("STATE: {:?}\n",beacon_state);

}
