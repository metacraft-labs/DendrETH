use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::backend::circuit::{CircuitBuild, PlonkParameters};
use plonky2x::frontend::uint::uint64::U64Variable;

use plonky2x::{
    backend::circuit::PublicOutput,
    prelude::CircuitBuilder,
};

use crate::combine_finality_votes::unique_validators_accumulator::UniqueValidatorsAccumulatorInner;
use super::count_unique_pubkeys::count_unique_validators;

pub fn cuv_recursive<L: PlonkParameters<D>, const D: usize>(file_path_sorted_validators: &str) 
    -> (ProofWithPublicInputs<L::Field, L::Config, D>, CircuitBuild<L,D>)
where
<<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
    plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{

    let cuv_recurssive_proof_final: ProofWithPublicInputs<L::Field, L::Config, D>;

    let (mut proofs, cuv_circuit) = count_unique_validators::<L,D>(file_path_sorted_validators);

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
    (cuv_recurssive_proof_final, cuv_child_circuit)
}
