use ethers::abi::parse_abi;
use ethers::types::U256;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::backend::circuit::{PublicInput, CircuitBuild};
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

use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    frontend::{
        eth::{
            // beacon::vars::BeaconValidatorVariable,
            vars::BLSPubkeyVariable
        },
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
        uint::uint64::U64Variable,
        vars::Bytes32Variable,
    },
    prelude::{
        bytes, bytes32, ArrayVariable, BoolVariable, CircuitBuilder, CircuitVariable, Field,
        PlonkParameters, U256Variable, Variable,
    },
};
use plonky2::field::goldilocks_field::GoldilocksField;

use crate::constants::{
    STATE_ROOT_PROOF_LEN, VALIDATORS_HASH_TREE_DEPTH, VALIDATORS_PER_COMMITTEE,
    VALIDATORS_ROOT_PROOF_LEN, VERSION_OBJ_BYTES, ZERO_HASHES, TEST_VALIDATORS_IN_COMMITMENT_SIZE
};
use crate::weigh_justification_and_finalization::checkpoint::{CheckpointValue, CheckpointVariable};

fn deserialize_checkpoint<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let checkpoint: CheckpointInput = Deserialize::deserialize(deserializer)?;
    Ok(checkpoint.root)
}

fn deserialize_validator_list_proof<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let list_proof: Vec<String> = Deserialize::deserialize(deserializer)?;

    let padded_result: Vec<String> = list_proof.into_iter()
        .enumerate()
        .map(|(index, item)|
            if item.is_empty() {
                 ZERO_HASHES[index].to_string()
                } 
            else { item.to_string() })
        .collect();

    Ok(padded_result)
}
    
#[derive(Debug, Clone, Deserialize)]
pub struct BeaconValidatorInput {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub exit_epoch: u64,
    pub withdrawable_epoch: u64,
}

impl BeaconValidatorInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>){
        input.write::<BLSPubkeyVariable>(bytes!(self.pubkey.clone())); 
        input.write::<Bytes32Variable>(bytes32!(self.withdrawal_credentials)); 
        input.write::<U64Variable>(self.effective_balance);
        input.write::<BoolVariable>(self.slashed);
        input.write::<U64Variable>(self.activation_eligibility_epoch); 
        input.write::<U64Variable>(self.activation_epoch); 
        input.write::<U64Variable>(self.exit_epoch); 
        input.write::<U64Variable>(self.withdrawable_epoch); 
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorDataInput {
    trusted: bool,
    validator_index: u64,

    #[serde(flatten)]
    beacon_validator_variable: BeaconValidatorInput,

    #[serde(deserialize_with="deserialize_validator_list_proof")]
    validator_list_proof: Vec<String>,
}

impl ValidatorDataInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        input.write::<BoolVariable>(self.trusted); 
        input.write::<U64Variable>(self.validator_index); 
        self.beacon_validator_variable.write(&mut input);

        input.write::<ArrayVariable<Bytes32Variable, VALIDATORS_HASH_TREE_DEPTH>>(
            self.validator_list_proof
            .iter()
            .map(|element| bytes32!(element))
            .collect()
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInput {
    root: String,
    epoch: u64,
}

impl CheckpointInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, input: &mut PublicInput<L, D>) {
        input.write::<Bytes32Variable>(bytes32!(self.root));
        input.write::<U64Variable>(self.epoch);
    }
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

#[derive(Debug, Clone, Deserialize)]
pub struct AttestationDataInput {
    slot: u64,
    index: u64,

    beacon_block_root: String,

    // #[serde(deserialize_with="deserialize_checkpoint")]
    source: CheckpointInput,

    // #[serde(deserialize_with="deserialize_checkpoint")]
    target: CheckpointInput,
}


impl AttestationDataInput {
    pub fn write<L: PlonkParameters<D>, const D: usize>(&self, mut input: &mut PublicInput<L, D>) {
        input.write::<U64Variable>(self.slot); 
        input.write::<U64Variable>(self.index);
        input.write::<Bytes32Variable>(bytes32!(self.beacon_block_root));
        self.source.write(&mut input);
        self.target.write(&mut input);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttestationInput {
    data: AttestationDataInput,
    // signature: String,
    fork: ForkInput,
    genesis_validators_root: String,
    state_root: String,
    state_root_proof: Vec<String>,
    validators_root: String,
    validators_root_proof: Vec<String>,
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
    circuit: &CircuitBuild<L,D>,
    attestation: &Value
) -> ProofWithPublicInputs<<L as PlonkParameters<D>>::Field, <L as PlonkParameters<D>>::Config, D>
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    // Parse Data and register as inputs for circuit
    // let attestation_input = parse_attestation_json(attestation);
    let attestation_input: AttestationInput = serde_json::from_value(attestation.clone()).unwrap();

    let validators: Vec<ValidatorDataInput>= attestation.get("validators").clone()
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .take(TEST_VALIDATORS_IN_COMMITMENT_SIZE)
        .map(|validator|serde_json::from_value(validator.clone()).unwrap())
        .collect();
    
    let mut input = circuit.input();

    //TODO: prev_block_root should be part of attestation_input and not hardcoded
    let prev_block_root: String =
        "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    input.write::<Bytes32Variable>(bytes32!(prev_block_root));

    attestation_input.write(&mut input);

    for validator in validators {
        validator.write(&mut input);
    }

    let (proof, _output) = circuit.prove(&input);
    println!("Attestation Proof: {:?}", _output);

    proof
}
