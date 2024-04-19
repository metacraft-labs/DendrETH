use std::fs;

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use redis::AsyncCommands;

use crate::poseidon_bn128_config::PoseidonBN128GoldilocksConfig;

pub async fn wrap_final_layer_in_poseidon_bn_128(
    mut con: redis::aio::Connection,
    compile_circuit: bool,
    final_layer_circuit: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    final_layer_proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    protocol: String,
) -> Result<(), anyhow::Error> {
    let (proof_target, circuit_data) = build_wrapper(final_layer_circuit);
    let proof = generate_wrapper_proof(final_layer_proof, &circuit_data, proof_target)?;

    let serialized_proof = serde_json::to_string(&proof)?;
    let verifier_only_circuit_data = serde_json::to_string(&circuit_data.verifier_only).unwrap();
    let common_circuit_data = serde_json::to_string(&circuit_data.common).unwrap();

    if compile_circuit {
        fs::write(
            "verifier_only_circuit_data.json",
            verifier_only_circuit_data.clone(),
        )
        .unwrap();

        fs::write("proof_with_public_inputs.json", serialized_proof).unwrap();

        fs::write("common_circuit_data.json", common_circuit_data).unwrap();

        con.set("balance_wrapper_verifier_only", verifier_only_circuit_data)
            .await?;
    } else {
        con.set(
            format!(
                "{}:{}",
                protocol, "balance_wrapper_proof_with_public_inputs"
            ),
            serialized_proof,
        )
        .await?;

        con.set("balance_wrapper_verifier_only", verifier_only_circuit_data)
            .await?;

        con.publish(format!("{}:{}", protocol, "gnark_proofs_channel"), "start")
            .await?;
    }

    Ok(())
}

pub fn generate_wrapper_proof(
    final_layer_proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    circuit_data: &CircuitData<GoldilocksField, PoseidonBN128GoldilocksConfig, 2>,
    proof_target: ProofWithPublicInputsTarget<2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonBN128GoldilocksConfig, 2>, anyhow::Error>
{
    let mut pw = PartialWitness::new();
    pw.set_proof_with_pis_target(&proof_target, &final_layer_proof);
    let proof = circuit_data.prove(pw)?;
    Ok(proof)
}

pub fn build_wrapper(
    final_layer_circuit: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> (
    ProofWithPublicInputsTarget<2>,
    CircuitData<GoldilocksField, PoseidonBN128GoldilocksConfig, 2>,
) {
    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let config = CircuitConfig {
        fri_config: FriConfig {
            rate_bits: 6,
            cap_height: 4,
            proof_of_work_bits: 16,
            reduction_strategy: FriReductionStrategy::ConstantArityBits(4, 5),
            num_query_rounds: 14,
        },
        ..standard_recursion_config
    };

    let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(config);
    let verifier_target = builder.constant_verifier_data(&final_layer_circuit.verifier_only);
    let proof_target = builder.add_virtual_proof_with_pis(&final_layer_circuit.common);
    builder.verify_proof::<PoseidonGoldilocksConfig>(
        &proof_target.clone(),
        &verifier_target,
        &final_layer_circuit.common,
    );
    builder.register_public_inputs(&proof_target.public_inputs);

    let circuit_data = builder.build::<PoseidonBN128GoldilocksConfig>();

    (proof_target, circuit_data)
}
