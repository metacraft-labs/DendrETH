use casper_finality_proofs::combine_finality_votes::count_unique_validators::CountUniqueValidators;
use plonky2x::{
    frontend::{uint::uint64::U64Variable, vars::Variable},
    prelude::{CircuitBuilder, DefaultParameters},
    utils::bytes, backend::circuit::PlonkParameters,
};

use casper_finality_proofs::{
    constants::VALIDATOR_INDICES_IN_SPLIT,
    utils::json::read_json_from_file,
};

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let file_path_sorted_validators = "./data/sorted_validator_indices.json";
    let sorted_validators_json = read_json_from_file(file_path_sorted_validators).unwrap();

    let mut builder = CircuitBuilder::<L, D>::new();
    CountUniqueValidators::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    let sorted_validators: Vec<u64> = sorted_validators_json.as_array()
        .unwrap()
        .iter()
        .take(VALIDATOR_INDICES_IN_SPLIT) 
        .map(|validator| serde_json::from_value(validator.clone()).unwrap())
        .collect();

    println!("Sorted_validators: {:?}", sorted_validators);


    let sigma: u64 = 1;
    for validator_index in sorted_validators {
        input.write::<U64Variable>(sigma);
        input.write::<U64Variable>(validator_index);
    }
    let (_proof, output) = circuit.prove(&input);
    print!("Output: {:?}", output);
}
