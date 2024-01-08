use plonky2x::{
    backend::circuit::CircuitBuild,
    //frontend::eth::vars::BLSPubkeyVariable,
    prelude::{CircuitBuilder, PlonkParameters, U256Variable}, frontend::uint::uint64::U64Variable,
};

use crate::weigh_justification_and_finalization::checkpoint::CheckpointVariable;

use super::circuit::ProofWithPublicInputsTargetReader;

#[derive(Debug, Clone)]
pub struct CommitmentAccumulatorInner;

impl CommitmentAccumulatorInner {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        child_circuit: &CircuitBuild<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
        let left_proof = builder.proof_read(&child_circuit.data.common).into();
        let right_proof = builder.proof_read(&child_circuit.data.common).into();

        builder.verify_proof::<L>(&left_proof, &verifier_data, &child_circuit.data.common);
        builder.verify_proof::<L>(&right_proof, &verifier_data, &child_circuit.data.common);

        let mut left_proof_reader = ProofWithPublicInputsTargetReader::from(left_proof);
        let mut right_proof_reader = ProofWithPublicInputsTargetReader::from(right_proof);

        let l_source = left_proof_reader.read::<CheckpointVariable>();
        let l_target = left_proof_reader.read::<CheckpointVariable>();
        let l_commitment = left_proof_reader.read::<U64Variable>();
        let l_sigma =  left_proof_reader.read::<U64Variable>();

        let r_source = right_proof_reader.read::<CheckpointVariable>();
        let r_target = right_proof_reader.read::<CheckpointVariable>();
        let r_commitment = right_proof_reader.read::<U64Variable>();
        let r_sigma =  left_proof_reader.read::<U64Variable>();

        builder.assert_is_equal(l_sigma, r_sigma);

        builder.assert_is_equal(l_source.epoch, r_source.epoch);
        builder.assert_is_equal(l_source.root, r_source.root);

        builder.assert_is_equal(l_target.epoch, r_target.epoch);
        builder.assert_is_equal(l_target.root, r_target.root);

        let commitment = builder.add(l_commitment, r_commitment);

        builder.proof_write(commitment);
        builder.proof_write(l_sigma);
        builder.proof_write(l_target);
        builder.proof_write(l_source);
    }
}
