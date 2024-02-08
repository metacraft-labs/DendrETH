use casper_finality_proofs::constants::{STATE_ROOT_PROOF_LEN, VALIDATORS_ROOT_PROOF_LEN};
use casper_finality_proofs::prove_casper::{
    vad_recurssive::vad_recursive,
    cuv_recursive::cuv_recursive
};

use casper_finality_proofs::prove_finality::circuit_final::ProveFinality;
use plonky2x::frontend::vars::Bytes32Variable;
use plonky2x::utils::bytes32;
use std::io::Error as IOError;

use plonky2x::{
    backend::circuit::{DefaultParameters, PublicOutput},
    prelude::CircuitBuilder,
};

fn main() -> Result<(), IOError> {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let file_path_attestations = "./data/merged_234400.json";
    let (vad_proof_prev, vad_circuit_prev) 
        = vad_recursive::<L,D>(file_path_attestations);

    let file_path_sorted_validators = "./data/sorted_validator_indices_234400.json";
    let (cuv_proof_prev, cuv_circuit_prev) = 
        cuv_recursive::<L,D>(file_path_sorted_validators);

    let file_path_attestations = "./data/merged_234401.json";
    let (vad_proof_cur, vad_circuit_cur) 
        = vad_recursive::<L,D>(file_path_attestations);

    let file_path_sorted_validators = "./data/sorted_validator_indices_234401.json";
    let (cuv_proof_cur, cuv_circuit_cur) = 
        cuv_recursive::<L,D>(file_path_sorted_validators);


    // Prove Finality
    let mut finality_builder = CircuitBuilder::<L, D>::new();
    

    //TODO: Test with recurssive circuits from preivous epoch
    ProveFinality::define(
        &mut finality_builder,
        &vad_circuit_cur,
        &cuv_circuit_cur,
        &vad_circuit_prev,
        &cuv_circuit_prev
        );
    let finality_circuit = finality_builder.build();
    let mut input = finality_circuit.input();

    input.proof_write(vad_proof_cur);
    input.proof_write(vad_proof_prev);
    input.proof_write(cuv_proof_cur);
    input.proof_write(cuv_proof_prev);
    
    let prev_block_root: String =
        "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    input.write::<Bytes32Variable>(bytes32!(prev_block_root));

    let (_proof, output) = finality_circuit.prove(&input);

    println!("\n Finality Proof Output: \n {:?}", output);

    Ok(())
}
