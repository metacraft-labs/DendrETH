use serde_json::{Value, Error};
use serde::Deserialize;
use serde_with::serde_as;
use std::any;
use std::fs::File;
use std::io::{Read, Error as IOError};

use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable, U256Variable}, 
    frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable},
    // frontend::{eth::vars::BLSPubkeyVariable,
        vars::{Bytes32Variable}, uint::uint64::U64Variable, hash::poseidon::poseidon256::PoseidonHashOutVariable},
};

use casper_finality_proofs::{
    prove_casper::sequential_verification::prove_verify_attestation_data,
    verify_attestation_data::verify_attestation_data::VerifyAttestationData,
};

const VALIDATORS_PER_COMMITTEE: usize = 412; // 2048
const PLACEHOLDER: usize = 11;


fn main() -> Result<(), IOError> {

    type L = DefaultParameters;
    const D: usize = 2;

    let file_path = "./data/merged_234400.json";

    let mut file = File::open(file_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // Parse JSON into a serde_json::Value
    let json_value: Value = serde_json::from_str(&contents)?;

    // VerifyAttestationData
    if let Some(attestations) = 
        json_value.get("attestations")
        .and_then(Value::as_array) {

            let mut builder = CircuitBuilder::<L, D>::new();
            VerifyAttestationData::define(&mut builder);

            prove_verify_attestation_data(attestations, builder)
        }
    else {
        panic!("No attestations found!");
    }
    // let result = serde_json::from_value(struct_definition);

    // Print the structure
    // print_json_value(&json_value, 0);



    Ok(())
}
