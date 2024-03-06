use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::{backend::circuit::{Circuit, CircuitBuild, DefaultParameters}, frontend::{builder::CircuitBuilder, vars::Bytes32Variable}};
use serde_json::Value;
use plonky2x::prelude::{Field, bytes32, PlonkParameters};

use crate::{constants::{TEST_ATTESTATIONS_READ, TEST_VALIDATORS_IN_COMMITMENT_SIZE}, utils::{eth_objects::ValidatorDataInput, json::read_json_from_file}, verify_attestation_data::{verify_attestation_data::VerifyAttestationData, verify_attestation_data_poseidon::VerifyAttestationDataPoseidon}};
use crate::utils::eth_objects::AttestationInput;

pub fn prove_verify_attestation_data<L: PlonkParameters<D>, const D: usize>(
    circuit: &CircuitBuild<L,D>,
    attestation: &Value,
    is_poseidon_hash: bool
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
    // let prev_block_root: String =
    //     "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    // input.write::<Bytes32Variable>(bytes32!(prev_block_root));

    attestation_input.write(&mut input);

    for validator in validators {
        validator.write(&mut input);
    }

    let (proof, _output) = circuit.prove(&input);

    proof
}

pub fn prove_attestations<L: PlonkParameters<D>, const D: usize>(
    file_path_attestations: &str,
    is_poseidon_hash: bool
) -> (
    Vec<ProofWithPublicInputs<<L as PlonkParameters<D>>::Field, <L as PlonkParameters<D>>::Config, D>>,
    CircuitBuild<L, D>
)
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{
    let attestations_json = read_json_from_file(file_path_attestations).unwrap();
    
    let mut proofs = vec![];

    let mut vad_builder = CircuitBuilder::<L, D>::new();
    if is_poseidon_hash {
        VerifyAttestationDataPoseidon::define(&mut vad_builder);
    }
    else {
        VerifyAttestationData::define(&mut vad_builder);
    }
    let vad_circuit = vad_builder.build();

    let attestations = attestations_json.get("attestations").and_then(Value::as_array).unwrap();
    let mut counter = 1;
    for attestation in attestations.iter().take(TEST_ATTESTATIONS_READ) {
        println!("====Attestation {}====", counter);
        counter += 1;

        let proof = prove_verify_attestation_data::<L, D>(&vad_circuit, attestation,is_poseidon_hash);
        proofs.push(proof);
    }
    
    (proofs, vad_circuit)
}
