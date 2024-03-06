use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::backend::circuit::{CircuitBuild, PlonkParameters};
use plonky2x::frontend::uint::uint64::U64Variable;
use plonky2x::frontend::vars::{Bytes32Variable, ArrayVariable};

use plonky2x::{
    backend::circuit::PublicOutput,
    prelude::CircuitBuilder,
};

use crate::combine_finality_votes::commitment_accumulator_inner::CommitmentAccumulatorInner;
use crate::constants::{STATE_ROOT_PROOF_LEN, VALIDATORS_ROOT_PROOF_LEN};
use crate::utils::eth_objects::CheckpointVariable;

use super::sequential_verification::prove_attestations;

pub fn vad_recursive<L: PlonkParameters<D>, const D: usize>(file_path_attestations: &str, is_poseidon_hash: bool) 
    -> (ProofWithPublicInputs<L::Field, L::Config, D>, CircuitBuild<L,D>)
where
<<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
    plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let vad_recurssive_proof_final: ProofWithPublicInputs<L::Field, L::Config, D>;

    // VerifyAttestationData

    let (attestation_data_proofs, vad_circuit) = 
        prove_attestations(file_path_attestations, is_poseidon_hash);

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

    (vad_recurssive_proof_final, vad_child_circuit)
}
