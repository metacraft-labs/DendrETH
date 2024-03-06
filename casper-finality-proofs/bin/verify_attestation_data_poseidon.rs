use std::{fs::File, io::Read};
use itertools::Itertools;
use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::HashOut};
use serde_json::Value;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters}, frontend::{hash::poseidon::poseidon256::PoseidonHashOutVariable, vars::{ArrayVariable, Bytes32Variable, Variable}}, prelude::{bytes32, CircuitBuilder}
};
use casper_finality_proofs::{constants::TEST_VALIDATORS_IN_COMMITMENT_SIZE, utils::{eth_objects::{AttestationInput, HashOutPoseidonInput, ValidatorDataInput, ValidatorDataPoseidonInput}, json::read_json_from_file}, verify_attestation_data::{verify_attestation_data::VerifyAttestationData, verify_attestation_data_poseidon::VerifyAttestationDataPoseidon}};
fn main() {
    plonky2x::utils::setup_logger();
    
        type L = DefaultParameters;
        const D: usize = 2;

    let file_path_attestation 
        // Using this because of memory overflow
        = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/merged_234400_poseidon_short.json";
    let attestations_json = read_json_from_file(file_path_attestation).unwrap();

    let file_path_validators_poseidon 
        = "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400_poseidon.json";
    let validators_poseidon_json = read_json_from_file(file_path_validators_poseidon).unwrap();

    let mut builder = CircuitBuilder::<L, D>::new();

    VerifyAttestationDataPoseidon::define(&mut builder);

    let circuit = builder.build();
    let mut input = circuit.input();

    //TODO: prev_block_root should be part of attestation_input and not hardcoded
    // let prev_block_root: String = "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    // input.write::<Bytes32Variable>(bytes32!(prev_block_root));

    let poseidon_hash_tree_root = validators_poseidon_json
        .get("data").clone()
        .unwrap()
        .get("poseidon_hash_tree_root")
        .unwrap();

    let poseidon_hash_tree_root_input: HashOutPoseidonInput = serde_json::from_value(poseidon_hash_tree_root.clone()).unwrap();
    poseidon_hash_tree_root_input.write(&mut input);

    // Parse and Write Attestation
    let attestation = attestations_json.get("attestations")
        .and_then(Value::as_array)
        .and_then(|array| array.get(0))
        .unwrap();

    let attestation_input: AttestationInput = serde_json::from_value(attestation.clone()).unwrap();
    attestation_input.write(&mut input);

    // Parse and Write Validator Objects
    let validators: Vec<ValidatorDataPoseidonInput>= validators_poseidon_json
        .get("data").clone()
        .unwrap()
        .get("validators").clone()
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .take(TEST_VALIDATORS_IN_COMMITMENT_SIZE)
        .map(|validator|serde_json::from_value(validator.clone()).unwrap())
        .collect();

    for validator in validators {
        validator.write(&mut input);
    }

    let (_proof, output) = circuit.prove(&input);
    println!("{:?}",output);

}
