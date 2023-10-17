use std::println;

use casper_finality_proofs::{
    commitment_mapper_first_level::CommitmentMapperFirstLevel, proof_utils::ProofWithPublicInputsTargetReader,
};
use plonky2::gates::poseidon;
use plonky2x::{
    backend::circuit::{Circuit, CircuitBuild},
    frontend::{
        eth::beacon::vars::BeaconValidatorVariable,
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
    },
    prelude::{Bytes32Variable, CircuitBuilder, DefaultParameters, PlonkParameters},
    utils::eth::beacon::BeaconValidator,
};

fn main() {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    CommitmentMapperFirstLevel::define(&mut builder);
    let circuit = builder.build();

    let mut input = circuit.input();
    input.write::<BeaconValidatorVariable>( BeaconValidator {
        pubkey: "0x123000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        withdrawal_credentials: "0x1230000000000000000000000000000000000000000000000000000000000000".to_string(),
        activation_epoch: "6152".to_string(),
        activation_eligibility_epoch: "6152".to_string(),
        exit_epoch: "6152".to_string(),
        slashed: false,
        effective_balance: 32,
        withdrawable_epoch: "6152".to_string(),
    });

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    let sha256_result = output.read::<Bytes32Variable>();

    let poseidon_result = output.read::<PoseidonHashOutVariable>();

    println!("sha256_result {:?}", sha256_result);
    println!("poseidon_result {:?}", poseidon_result);

    println!("proof public inputs {:?}", proof.public_inputs);

    let mut inner_level_builder = CircuitBuilder::<DefaultParameters, 2>::new();

    define_inner_level(&mut inner_level_builder, circuit);

    let inner_circuit = inner_level_builder.build();
    let mut inner_input = inner_circuit.input();

    inner_input.proof_write(proof.clone());
    inner_input.proof_write(proof);

    let (inner_proof, mut inner_output) = inner_circuit.prove(&inner_input);


    println!("inner output {:?}", inner_output);
    inner_circuit.data.verify(inner_proof).unwrap();

    println!("sha256_result {:?}", inner_output.proof_read::<Bytes32Variable>());
    println!(
        "poseidon_result {:?}",
        inner_output.proof_read::<PoseidonHashOutVariable>()
    );
}

pub fn define_inner_level<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    child_circuit: CircuitBuild<L, D>,
) where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
    let proof1 = builder.proof_read(&child_circuit).into();
    let proof2 = builder.proof_read(&child_circuit).into();

    builder.verify_proof::<L>(&proof1, &verifier_data, &child_circuit.data.common);

    builder.verify_proof::<L>(&proof2, &verifier_data, &child_circuit.data.common);

    let mut proof1_reader = ProofWithPublicInputsTargetReader::from(proof1);
    let mut proof2_reader = ProofWithPublicInputsTargetReader::from(proof2);

    let poseidon_hash = proof1_reader.read::<PoseidonHashOutVariable>();
    let sha256_hash1 = proof1_reader.read::<Bytes32Variable>();

    // let poseidon_hash2 = proof2_reader.read::<PoseidonHashOutVariable>();
    // let sha256_hash2 = proof2_reader.read::<Bytes32Variable>();

    // let sha256 = builder.sha256_pair(sha256_hash1, sha256_hash2);
    // let poseidon = builder.poseidon_hash_pair(poseidon_hash, poseidon_hash2);

    builder.proof_write(sha256_hash1);
    builder.proof_write(poseidon_hash);
}
