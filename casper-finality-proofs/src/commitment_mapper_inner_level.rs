use plonky2x::{
    backend::circuit::CircuitBuild,
    frontend::hash::poseidon::poseidon256::PoseidonHashOutVariable,
    prelude::{Bytes32Variable, CircuitBuilder, PlonkParameters},
};

use crate::proof_utils::ProofWithPublicInputsTargetReader;

pub fn define_commitment_mapper_inner_level<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    child_circuit: &CircuitBuild<L, D>,
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

    let poseidon_hash2 = proof2_reader.read::<PoseidonHashOutVariable>();
    let sha256_hash2 = proof2_reader.read::<Bytes32Variable>();

    let sha256 = builder.sha256_pair(sha256_hash1, sha256_hash2);
    let poseidon = builder.poseidon_hash_pair(poseidon_hash, poseidon_hash2);

    builder.proof_write(sha256);
    builder.proof_write(poseidon);
}
