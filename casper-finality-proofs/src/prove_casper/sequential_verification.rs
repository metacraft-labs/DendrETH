use ethers::abi::parse_abi;
use ethers::core::k256::U256;
// use ethers::types::U256;
use plonky2x::backend::circuit::PublicInput;
use plonky2x::frontend::uint::uint64;
use plonky2x::frontend::vars::BytesVariable;
use serde_json::{Value, Error};
use serde::Deserialize;
use serde_with::serde_as;
use snap::write;
use std::any;
use std::fs::File;
use std::io::{Read, Error as IOError};

use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes,bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable, U256Variable}, 
    frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable},
        vars::{Bytes32Variable}, uint::uint64::U64Variable, hash::poseidon::poseidon256::PoseidonHashOutVariable},
};
use crate::verify_attestation_data::verify_attestation_data::VerifyAttestationData;

use super::super::utils::eth_objects::{Fork};

const VALIDATORS_PER_COMMITTEE: usize = 412; // 2048
const PLACEHOLDER: usize = 11;

struct ForkInput {
    previous_version: String,
    current_version: String,
    epoch: u64
}

impl ForkInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<BytesVariable<4>>(bytes!(&self.previous_version));
        input.write::<BytesVariable<4>>(bytes!(&self.current_version));
        input.write::<U64Variable>(self.epoch); //TODO: U64 should be U256 by spec definition
    }
}

struct AttestationDataInput {
    slot: u64, // Plonky2X parses it as U256
    index: u64,

    beacon_block_root: String,

    source: String,
    target: String
}

impl AttestationDataInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<U64Variable>(self.slot); //TODO: U64 should be U256 by spec definition
        input.write::<U64Variable>(self.index); //TODO: U64 should be U256 by spec definition
        input.write::<Bytes32Variable>(bytes32!(self.beacon_block_root));
        input.write::<Bytes32Variable>(bytes32!(self.source));
        input.write::<Bytes32Variable>(bytes32!(self.target));
    } 
}

struct AttestationInput {
    data: AttestationDataInput,
    signature: String,
    fork: ForkInput,
    genesis_validators_root: String,
    state_root: String,
    state_root_proof: Vec<String>,
    validators_root: String,
    validators_root_proof: Vec<String>,
    // validators - array of BeaconValidatorVariables not passed
    // validator_list_proof - not passed
}

impl AttestationInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self,mut input: &mut PublicInput<L, D>) {
        self.data.write(&mut input);

        // input.write::<BLSPubkeyVariable>(); //TODO:
        self.fork.write(&mut input);
        input.write::<Bytes32Variable>(bytes32!(self.genesis_validators_root));

        input.write::<Bytes32Variable>(bytes32!(self.state_root));
        input.write::<ArrayVariable<Bytes32Variable, 3>>(self.state_root_proof
            .iter()
            .map(|element| bytes32!(element))
            .collect()
        );

        input.write::<Bytes32Variable>(bytes32!(self.validators_root));
        input.write::<ArrayVariable<Bytes32Variable, 5>>(self.validators_root_proof
            .iter()
            .map(|element| bytes32!(element))
            .collect()
        );
    }
}

fn parse_fork_json(fork: &Value) -> ForkInput {
    ForkInput {
        previous_version: fork.get("previous_version")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        current_version: fork.get("current_version")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        epoch: fork.get("epoch")
            .unwrap()
            .as_u64()
            .unwrap()
    }
}


pub fn parse_attestation_data_json(data: &Value) -> AttestationDataInput {
    AttestationDataInput {
        slot: data.get("slot")
            .unwrap()
            .as_u64()
            .unwrap(),
        index: data.get("index")
            .unwrap()
            .as_u64()
            .unwrap(),
        beacon_block_root: data.get("beacon_block_root")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        source: data.get("source")
            .unwrap()
            .get("root")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        target: data.get("target")
            .unwrap()
            .get("root")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string()
    }
}

pub fn parse_attestation(attestation: &Value) -> AttestationInput {
    let attestation_data = parse_attestation_data_json(attestation.get("data").unwrap());

    let signature = attestation.get("signature")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_string();

    let fork = parse_fork_json(attestation.get("fork").unwrap());

    let genesis_validators_root = attestation.get("genesis_validators_root")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_string();

    let state_root = attestation.get("state_root")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_string();

    let state_root_proof: Vec<String> = attestation
                            .get("state_root_proof")
                            .and_then(Value::as_array)
                            .unwrap()
                            .iter()
                            .filter_map(|v| Some(v.to_string()
                                .trim_matches('"')
                                .to_string()
                            ))
                            .collect();

    let validators_root = attestation.get("validators_root")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_string();

    let validators_root_proof: Vec<String> = attestation
                            .get("validators_root_proof")
                            .and_then(Value::as_array)
                            .unwrap()
                            .iter()
                            .filter_map(|v| Some(v.to_string()
                                .trim_matches('"')
                                .to_string()
                            ))
                            .collect();

    AttestationInput {
        data: attestation_data,
        signature: signature,
        fork: fork,
        genesis_validators_root: genesis_validators_root,
        state_root: state_root,
        state_root_proof: state_root_proof,
        validators_root: validators_root,
        validators_root_proof: validators_root_proof
    }

}

pub fn prove_verify_attestation_data<L: PlonkParameters<D>, const D: usize>(attestations: &Vec<Value>) 
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{

    let mut counter = 0;
    // For each attestation run VerifyAttestationData (TODO: Missing validator_list_proof from original object)
    for attestation in attestations.iter().take(10) { //TODO: Only 1st 10
        counter = counter+1;
        println!("====Attestation {}====",counter);
        // Parse Data and register as inputs for circuit
        let attestation_input = parse_attestation(attestation);
        
        let mut builder = CircuitBuilder::<L, D>::new();
        VerifyAttestationData::define(&mut builder);

        let circuit = builder.build();
        let mut input = circuit.input();
        attestation_input.write(&mut input);
        //WORKS
        let (proof, mut output) = circuit.prove(&input);

        println!("OUTPUT: {:?}", output);

        }
}

fn print_json_value(value: &Value, indent: usize) {
    match value {
        Value::Null => println!("null"),
        Value::Bool(b) => println!("{}", b),
        Value::Number(num) => println!("{}", num),
        Value::String(s) => println!("\"{}\"", s),
        Value::Array(arr) => {
            println!("[");
            for item in arr {
                print!("{}  ", " ".repeat(indent + 2));
                print_json_value(item, indent + 2);
            }
            println!("{}]", " ".repeat(indent));
        }
        Value::Object(obj) => {
            println!("{}", "{");
            for (key, value) in obj {
                print!("{}\"{}\": ", " ".repeat(indent + 2), key);
                print_json_value(value, indent + 2);
            }
            println!("{}}}", " ".repeat(indent));
        }
    }
}
