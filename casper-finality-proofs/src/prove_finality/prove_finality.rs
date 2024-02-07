use plonky2::{field::goldilocks_field::GoldilocksField, plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs}};
use plonky2x::{backend::circuit::{Circuit, CircuitBuild}, frontend::{builder::CircuitBuilder, uint::uint64::U64Variable, vars::Bytes32Variable}};
use serde_json::Value;
use plonky2x::{
    backend::circuit::DefaultParameters,
    prelude::{bytes32, PlonkParameters},
};

use crate::{constants::{TEST_ATTESTATIONS_READ, TEST_VALIDATORS_IN_COMMITMENT_SIZE}, prove_finality::circuit_final::ProveFinality, utils::{eth_objects::{BeaconStateInput, ValidatorDataInput}, json::read_json_from_file}, verify_attestation_data::verify_attestation_data::VerifyAttestationData};

pub fn prove_finality<L: PlonkParameters<D>, const D: usize>(
    file_path_beacon_state: &str,
    vad_recurssive_circuit: CircuitBuild<L, D>,
    vad_recurssive_proof_final: ProofWithPublicInputs<L::Field, L::Config, D>,
    // // Proof when source was target
    // prev_vad_recurssive_proof_final: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>

    cuv_recurssive_circuit: CircuitBuild<L, D>,
    cuv_recurssive_proof_final: ProofWithPublicInputs<L::Field, L::Config, D>,
) -> (
    ProofWithPublicInputs<<L as PlonkParameters<D>>::Field, <L as PlonkParameters<D>>::Config, D>,
    CircuitBuild<L, D>
)
where
    <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
        plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
{

    let beacon_state_json = read_json_from_file(file_path_beacon_state).unwrap();

    let beacon_state_value = beacon_state_json.get("beacon_state").unwrap();
    // let beacon_state: BeaconStateInput = serde_json::from_value(beacon_state_value.clone()).unwrap();

    let justification_bits = beacon_state_value["justification_bits"].as_u64().unwrap();
    let previous_justified_checkpoint = &beacon_state_value["previous_justified_checkpoint"];
    let current_justified_checkpoint = &beacon_state_value["current_justified_checkpoint"];

    let beacon_state = BeaconStateInput {
        justification_bits: justification_bits,
        previous_justified_checkpoint: serde_json::from_value(previous_justified_checkpoint.clone()).unwrap(),
        current_justified_checkpoint: serde_json::from_value(current_justified_checkpoint.clone()).unwrap()
    };

    let mut finality_builder = CircuitBuilder::<L, D>::new();

    //TODO: Test with recurssive circuits from preivous epoch
    // ProveFinality::define::<L,D>(&mut finality_builder, &vad_recurssive_circuit, &cuv_recurssive_circuit);
    let finality_circuit = finality_builder.build();

    let mut input = finality_circuit.input();

    input.proof_write(vad_recurssive_proof_final);
    // input.proof_write(prev_vad_recurssive_proof_final);
    input.proof_write(cuv_recurssive_proof_final);

    beacon_state.write(&mut input);

    let prev_block_root: String =
        "d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7".to_string();
    input.write::<Bytes32Variable>(bytes32!(prev_block_root));

    let total_num_validators: u64 = 849681;
    input.write::<U64Variable>(total_num_validators);

    let (proof, output) = finality_circuit.prove(&input);

    println!("\n Finality Proof Output: \n {:?}", output);

    (proof, finality_circuit)
}
