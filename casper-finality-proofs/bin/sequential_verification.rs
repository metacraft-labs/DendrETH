use anyhow::Error;
use casper_finality_proofs::combine_finality_votes::commitment_accumulator_inner::CommitmentAccumulatorInner;
use casper_finality_proofs::constants::TEST_ATTESTATIONS_READ;
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
    let mut leaf_builder = CircuitBuilder::<L, D>::new();
    CommitmentAccumulatorInner::define(&mut leaf_builder, &child_circuit);

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
            println!("\nFinal Commitment\n: {}", vad_aggregated_commitment);
            break;
        }
    }


    //CountUniquePubkeys
    let file_path_sorted_pubkeys = "./data/sorted_pubkeys.json";
    let sorted_pubkeys_json = read_json_from_file(file_path_sorted_pubkeys).unwrap();

    Ok(())
}
