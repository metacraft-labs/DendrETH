use casper_finality_proofs::combine_finality_votes::commitment_accumulator_inner::CommitmentAccumulatorInner;
use casper_finality_proofs::combine_finality_votes::unique_validators_accumulator::UniqueValidatorsAccumulatorInner;
use casper_finality_proofs::constants::{STATE_ROOT_PROOF_LEN, VALIDATORS_ROOT_PROOF_LEN};
use casper_finality_proofs::prove_casper::sequential_verification::prove_attestations;
use casper_finality_proofs::prove_finality::circuit_final::ProveFinality;
use casper_finality_proofs::utils::eth_objects::CheckpointVariable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::frontend::uint::uint64::U64Variable;
use plonky2x::frontend::vars::{Bytes32Variable, ArrayVariable};
use plonky2x::utils::bytes32;
use std::io::Error as IOError;

use plonky2x::{
    backend::circuit::{DefaultParameters, PublicOutput},
    prelude::CircuitBuilder,
};

use casper_finality_proofs::prove_casper::count_unique_pubkeys::count_unique_validators;

fn main() -> Result<(), IOError> {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let vad_recurssive_proof_final: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, D>;
    let cuv_recurssive_proof_final: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, D>;

    // VerifyAttestationData
    let file_path_attestations = "./data/merged_234400.json";
    let (attestation_data_proofs, vad_circuit) = prove_attestations(file_path_attestations);

    //CombineAttestationData
    let mut proofs = attestation_data_proofs;
    let mut vad_child_circuit = vad_circuit;
    let mut level = 0;

    loop {
        let mut inner_builder = CircuitBuilder::<L, D>::new();
        CommitmentAccumulatorInner::define(&mut inner_builder, &vad_child_circuit);
        vad_child_circuit = inner_builder.build();

        println!("Proving layer {}..", level + 1);
        let mut final_output: Option<PublicOutput<L, D>> = None;
        let mut new_proofs = vec![];
        for i in (0..proofs.len()).step_by(2) {
            println!("Pair [{}]", i / 2 + 1);
            let mut input = vad_child_circuit.input();

            input.proof_write(proofs[i].clone());
            input.proof_write(proofs[i + 1].clone());

            let (proof, output) = vad_child_circuit.prove(&input);

            println!("Output: {:?}",output);

            final_output = Some(output);
            new_proofs.push(proof);
        }

        proofs = new_proofs;
        level += 1;

        if proofs.len() == 1 {
            vad_recurssive_proof_final = proofs.pop().unwrap();

            let mut final_output = final_output.unwrap();
            let _l_state_root = final_output.proof_read::<Bytes32Variable>();
            let _l_state_root_proof = final_output.proof_read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>();
            let _l_validators_root = final_output.proof_read::<Bytes32Variable>();
            let _l_validators_root_proof = final_output.proof_read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>();

            let _source = final_output.proof_read::<CheckpointVariable>();
            let _target = final_output.proof_read::<CheckpointVariable>();
            let vad_aggregated_commitment = final_output.proof_read::<U64Variable>();
            let _sigma = final_output.proof_read::<U64Variable>();

            println!("\nFinal Commitment: {}\n", vad_aggregated_commitment);
            break;
        }
    }

    //CountUniquePubkeys
    let file_path_sorted_validators = "./data/sorted_validator_indices_test.json";
    let (mut proofs, cuv_circuit) = count_unique_validators(file_path_sorted_validators);

    // Recurssive CountUniqueValidators
    let mut cuv_child_circuit = cuv_circuit;
    let mut level = 0;
    
    loop {
        let mut inner_builder = CircuitBuilder::<L, D>::new();
        UniqueValidatorsAccumulatorInner::define(&mut inner_builder, &cuv_child_circuit);
        cuv_child_circuit = inner_builder.build();

        println!("Proving layer {}..", level + 1);

        let mut final_output: Option<PublicOutput<L, D>> = None;
        let mut new_proofs = vec![];
        for i in (0..proofs.len()).step_by(2) {
            println!("Pair [{}]", i / 2 + 1);
            let mut input = cuv_child_circuit.input();

            input.proof_write(proofs[i].clone());
            input.proof_write(proofs[i + 1].clone());

            let (proof, output) = cuv_child_circuit.prove(&input);
            println!("Current Unique: {:?}", output);

            final_output = Some(output);
            new_proofs.push(proof);
        }

        
        proofs = new_proofs;
        level += 1;

        if proofs.len() == 1 {
            cuv_recurssive_proof_final = proofs.pop().unwrap();

            let mut final_output = final_output.unwrap();
            let total_unique = final_output.proof_read::<U64Variable>();
            let final_commitment = final_output.proof_read::<U64Variable>();
            let validator_left = final_output.proof_read::<U64Variable>();
            let validator_right = final_output.proof_read::<U64Variable>();

            println!("\nFinal Commitment: {}\nTotal Unique: {}\nRight: {}\nLeft: {}\n", final_commitment, total_unique, validator_right,validator_left);
            break;
        }
    }

    // Prove Finality
    let mut finality_builder = CircuitBuilder::<L, D>::new();
    ProveFinality::define(&mut finality_builder, &vad_child_circuit, &cuv_child_circuit);
    let finality_circuit = finality_builder.build();
    let mut input = finality_circuit.input();

    input.proof_write(vad_recurssive_proof_final);
    input.proof_write(cuv_recurssive_proof_final);
    
    let prev_block_root: String =
        "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    input.write::<Bytes32Variable>(bytes32!(prev_block_root));

    let (_proof, output) = finality_circuit.prove(&input);

    println!("\n Finality Proof Output: \n {:?}", output);

    Ok(())
}
