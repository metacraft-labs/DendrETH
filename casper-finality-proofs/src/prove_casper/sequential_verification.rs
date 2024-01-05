use ethers::abi::parse_abi;
use ethers::types::U256;
use plonky2x::backend::circuit::PublicInput;
use plonky2x::frontend::uint::uint64;
use plonky2x::frontend::vars::BytesVariable;
use serde::{Deserialize, Deserializer};
use serde_derive::Serialize;
use serde_json::{Error, Value};
use serde_with::serde_as;
use snap::write;
use std::any;
use std::fs::File;
use std::io::{Error as IOError, Read};

use crate::verify_attestation_data::verify_attestation_data::VerifyAttestationData;
use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    frontend::{
        eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable},
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
        uint::uint64::U64Variable,
        vars::Bytes32Variable,
    },
    prelude::{
        bytes, bytes32, ArrayVariable, BoolVariable, CircuitBuilder, CircuitVariable, Field,
        PlonkParameters, U256Variable, Variable,
    },
};

use super::super::constants::{
    STATE_ROOT_PROOF_LEN, VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_PER_COMMITTEE,
    VALIDATORS_ROOT_PROOF_LEN, VERSION_OBJ_BYTES
};
use super::super::utils::eth_objects::Fork;

fn deserialize_checkpoint<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let checkpoint: CheckpointInput = Deserialize::deserialize(deserializer)?;
    Ok(checkpoint.root)
}

pub fn parse_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let str_value = U256::deserialize(deserializer)?;

    Ok(U256::from(str_value))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconValidatorVariableInput {
    pubkey: String,
    withdrawal_credentials: String,
    effective_balance: U256,
    slashed: bool,
    activation_eligibility_epoch: U256,
    activation_epoch: U256,
    exit_epoch: U256,
    withdrawable_epoch: U256,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorDataInput {
    is_trusted_validator: bool,
    validator_index: u64,
    beacon_validator_variable: BeaconValidatorVariableInput,

    validator_state_root: String,
    validator_leaf: String,
    validator_root_proof: Vec<String>,
    validator_gindex: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInput {
    root: String,
    epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInput {
    previous_version: String,
    current_version: String,
    epoch: u64,
}

impl ForkInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<BytesVariable<VERSION_OBJ_BYTES>>(bytes!(&self.previous_version));
        input.write::<BytesVariable<VERSION_OBJ_BYTES>>(bytes!(&self.current_version));
        input.write::<U64Variable>(self.epoch); 
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationDataInput {
    slot: u64,
    index: u64,

    beacon_block_root: String,

    #[serde(deserialize_with="deserialize_checkpoint")]
    source: String,

    #[serde(deserialize_with="deserialize_checkpoint")]
    target: String,
}

impl AttestationDataInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<U64Variable>(self.slot); 
        input.write::<U64Variable>(self.index);
        input.write::<Bytes32Variable>(bytes32!(self.beacon_block_root));
        input.write::<Bytes32Variable>(bytes32!(self.source));
        input.write::<Bytes32Variable>(bytes32!(self.target));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationInput {
    data: AttestationDataInput,
    // signature: String,
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
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        self.data.write(&mut input);

        // input.write::<BLSPubkeyVariable>(); //TODO:
        self.fork.write(&mut input);
        input.write::<Bytes32Variable>(bytes32!(self.genesis_validators_root));

        input.write::<Bytes32Variable>(bytes32!(self.state_root));
        input.write::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>(
            self.state_root_proof
                .iter()
                .map(|element| bytes32!(element))
                .collect(),
        );

        input.write::<Bytes32Variable>(bytes32!(self.validators_root));
        input.write::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>(
            self.validators_root_proof
                .iter()
                .map(|element| bytes32!(element))
                .collect(),
        );
    }
}

pub fn prove_verify_attestation_data<L: PlonkParameters<D>, const D: usize>(
    attestations: &Vec<Value>,
) where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let mut counter = 0;
    // For each attestation run VerifyAttestationData (TODO: Missing validator_list_proof from original object)
    for attestation in attestations.iter().take(10) {
        //TODO: Only 1st 8
        counter = counter + 1;
        println!("====Attestation {}====", counter);
        // Parse Data and register as inputs for circuit
        // let attestation_input = parse_attestation_json(attestation);
        let attestation_input: AttestationInput = serde_json::from_value(attestation.clone()).unwrap();

        let mut builder = CircuitBuilder::<L, D>::new();
        VerifyAttestationData::define(&mut builder);

        let circuit = builder.build();
        let mut input = circuit.input();

        //TODO: prev_block_root should be part of attestation_input and not hardcoded
        let prev_block_root: String =
            "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
        input.write::<Bytes32Variable>(bytes32!(prev_block_root));

        // attestation_input.write(&mut input);

        // let (proof, mut output) = circuit.prove(&input);

        // println!("OUTPUT: {:?}", output);
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
