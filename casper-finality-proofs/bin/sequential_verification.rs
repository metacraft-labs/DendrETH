use anyhow::Error;
use casper_finality_proofs::combine_finality_votes::commitment_accumulator_inner::CommitmentAccumulatorInner;
use casper_finality_proofs::combine_finality_votes::count_unique_validators::{CountUniqueValidators, self};
use casper_finality_proofs::combine_finality_votes::unique_validators_accumulator::UniqueValidatorsAccumulatorInner;
use casper_finality_proofs::constants::{TEST_ATTESTATIONS_READ, VALIDATOR_INDICES_IN_SPLIT, TEST_VALIDATORS_IN_COMMITMENT_SIZE};
use plonky2x::frontend::uint::uint64::U64Variable;
use plonky2x::frontend::vars::Bytes32Variable;
use serde_json::Value;
use std::fs::File;
use std::io::{Error as IOError, Read};

use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters, PublicOutput},
    prelude::CircuitBuilder,
};

use casper_finality_proofs::{
    prove_casper::sequential_verification::prove_verify_attestation_data,
    verify_attestation_data::verify_attestation_data::VerifyAttestationData,
    utils::json::read_json_from_file,
};

fn main() -> Result<(), IOError> {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let file_path_attestations = "./data/merged_234400.json";
    let attestations_json = read_json_from_file(file_path_attestations).unwrap();

    // VerifyAttestationData
    let mut attestation_data_proofs = vec![];

    let mut vad_builder = CircuitBuilder::<L, D>::new();
    VerifyAttestationData::define(&mut vad_builder);
    let vad_circuit = vad_builder.build();

    if let Some(attestations) = attestations_json.get("attestations").and_then(Value::as_array) {
        let mut counter = 1;
        for attestation in attestations.iter().take(TEST_ATTESTATIONS_READ) {
            println!("====Attestation {}====", counter);
            counter += 1;

            let proof = prove_verify_attestation_data::<L, D>(&vad_circuit, attestation);
            attestation_data_proofs.push(proof);
        }
    } else {
        panic!("No attestations found!");
    }

    //CombineAttestationData
    let mut proofs = attestation_data_proofs;
    let mut child_circuit = vad_circuit;
    let mut level = 0;

    loop {
        let mut inner_builder = CircuitBuilder::<L, D>::new();
        CommitmentAccumulatorInner::define(&mut inner_builder, &child_circuit);
        child_circuit = inner_builder.build();

        println!("Proving layer {}..", level + 1);
        let mut final_output: Option<PublicOutput<L, D>> = None;
        let mut new_proofs = vec![];
        for i in (0..proofs.len()).step_by(2) {
            println!("Pair [{}]", i / 2 + 1);
            let mut input = child_circuit.input();

            input.proof_write(proofs[i].clone());
            input.proof_write(proofs[i + 1].clone());

            let (proof, output) = child_circuit.prove(&input);

            final_output = Some(output);
            new_proofs.push(proof);
        }

        proofs = new_proofs;
        level += 1;

        if proofs.len() == 1 {
            let mut final_output = final_output.unwrap();
            let vad_aggregated_commitment = final_output.proof_read::<U64Variable>();
            let _sigma = final_output.proof_read::<U64Variable>();
            let _source = final_output.proof_read::<Bytes32Variable>();
            let _target = final_output.proof_read::<Bytes32Variable>();

            println!("\nFinal Commitment: {}\n", vad_aggregated_commitment);
            break;
        }
    }

    //CountUniquePubkeys
    let file_path_sorted_validators = "./data/sorted_validator_indices_test.json";
    let sorted_validators_json = read_json_from_file(file_path_sorted_validators).unwrap();

    let mut cuv_builder = CircuitBuilder::<L, D>::new();
    CountUniqueValidators::define(&mut cuv_builder);
    let cuv_circuit = cuv_builder.build();
    

    let sorted_validators: Vec<u64> = sorted_validators_json.as_array()
        .unwrap()
        .iter()
        .take(TEST_VALIDATORS_IN_COMMITMENT_SIZE * TEST_ATTESTATIONS_READ) //TODO: This is Test Size
        .map(|validator| serde_json::from_value(validator.clone()).unwrap())
        .collect();

    let chunk_size = VALIDATOR_INDICES_IN_SPLIT;
    let chunked_iter = sorted_validators.chunks_exact(chunk_size);
    //TODO: Use chunked_iter.into_remainder to parse final slice of validators

    let mut count_unique_validators_proofs = vec![];
    let mut counter = 0;
    let sigma: u64 = 1;
    for chunk in chunked_iter { 
        println!("===Proving Chunk {}====",counter);
        counter += 1;
        let mut input = cuv_circuit.input();
        input.write::<U64Variable>(sigma);
        for validator_index in chunk {
            input.write::<U64Variable>(validator_index.clone());
        }
        println!("CHUNK: {:?}", chunk);
        let (proof, mut _output) = cuv_circuit.prove(&input);
        count_unique_validators_proofs.push(proof);

        println!("Output: {:?}", _output);
    }
    println!("Sorted_validators: {:?}", sorted_validators);

    // Recurssive CountUniqueValidators
    let mut proofs = count_unique_validators_proofs;
    let mut child_circuit = cuv_circuit;
    let mut level = 0;
    
    loop {
        let mut inner_builder = CircuitBuilder::<L, D>::new();
        UniqueValidatorsAccumulatorInner::define(&mut inner_builder, &child_circuit);
        child_circuit = inner_builder.build();

        println!("Proving layer {}..", level + 1);

        let mut final_output: Option<PublicOutput<L, D>> = None;
        let mut new_proofs = vec![];
        for i in (0..proofs.len()).step_by(2) {
            println!("Pair [{}]", i / 2 + 1);
            let mut input = child_circuit.input();

            input.proof_write(proofs[i].clone());
            input.proof_write(proofs[i + 1].clone());

            let (proof, output) = child_circuit.prove(&input);
            println!("Current Unique: {:?}", output);

            final_output = Some(output);
            new_proofs.push(proof);
        }

        
        proofs = new_proofs;
        level += 1;

        if proofs.len() == 1 {
            let mut final_output = final_output.unwrap();
            let total_unique = final_output.proof_read::<U64Variable>();
            let final_commitment = final_output.proof_read::<U64Variable>();
            let validator_left = final_output.proof_read::<U64Variable>();
            let validator_right = final_output.proof_read::<U64Variable>();

            println!("\nFinal Commitment: {}\nTotal Unique: {}\nRight: {}\nLeft: {}\n", final_commitment, total_unique, validator_right,validator_left);
            break;
        }
    }

    Ok(())
}
