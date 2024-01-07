use std::fs::File;
use std::io::{Read, Error as IOError};
use std::any;
use curta::maybe_rayon::rayon::str::Bytes;
use itertools::Itertools;
use lighthouse_types::{Fork, Validator};
use serde_json::Value;
use curta::chip::field;
use ethers::types::U256;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable}, frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable}, vars::{Bytes32Variable, U256Variable}, uint::uint64::U64Variable},
};
use casper_finality_proofs::verify_attestation_data::verify_attestation_data::VerifyAttestationData;
use casper_finality_proofs::prove_casper::sequential_verification::{AttestationInput, ForkInput, AttestationDataInput, ValidatorDataInput};
use casper_finality_proofs::utils::eth_objects::{ValidatorData, AttestationData, Attestation};
fn main() {
    plonky2x::utils::setup_logger();

    let file_path = "./data/merged_234400.json";
    let file = File::open(file_path);
    let mut contents = String::new();

    file.unwrap().read_to_string(&mut contents).unwrap();
    let json_value: Value = serde_json::from_str(&contents).unwrap();

    let attestation = json_value.get("attestations")
        .and_then(Value::as_array)
        .and_then(|array| array.get(0))
        .unwrap();

    let attestation_input: AttestationInput = serde_json::from_value(attestation.clone()).unwrap();

    type L = DefaultParameters;
    const D: usize = 2;

    let mut builder = CircuitBuilder::<L, D>::new();
        
    VerifyAttestationData::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    //TODO: prev_block_root should be part of attestation_input and not hardcoded
    let prev_block_root: String = "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    input.write::<Bytes32Variable>(bytes32!(prev_block_root));


    attestation_input.write(&mut input);

    const N: usize = 10;
    let validator_samples: Vec<ValidatorDataInput> = attestation.get("validators").clone()
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .take(10)
        .map(|validator|serde_json::from_value(validator.clone()).unwrap())
        .collect_vec();


    println!("\nAttestation Read!\n");
    // let beacon_validator = validator_sample.get("") ;
    
    for i in 0..N {
        // let validator_sample_input: ValidatorDataInput = serde_json::from_value(validator_samples[i].clone()).unwrap();
        // validator_sample_input.write(&mut input);
    }

    // let samples = serde_json::from_value(validator_samples.clone()).unwrap();

    for validator in validator_samples {
        validator.write(&mut input);
    }

    let (_proof, output) = circuit.prove(&input);
    println!("OUTPUT: {:?}", output);

    // Json with

    // 

    // let mut output: Option<PublicOutput<L, D>> = None;
    // let (proof, mut output) = circuit.prove(&input);

    // let result = output.read::<Variable>();
    // println!("Bitmask: {:?}", result );

}
