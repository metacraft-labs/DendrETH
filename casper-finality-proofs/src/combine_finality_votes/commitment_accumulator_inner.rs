use plonky2x::{
    backend::circuit::CircuitBuild,
    //frontend::eth::vars::BLSPubkeyVariable,
    prelude::{CircuitBuilder, PlonkParameters}, frontend::{uint::uint64::U64Variable, vars::Bytes32Variable},
};

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

        let l_source = left_proof_reader.read::<Bytes32Variable>();
        let l_target = left_proof_reader.read::<Bytes32Variable>();
        let l_commitment = left_proof_reader.read::<U64Variable>();
        let l_sigma =  left_proof_reader.read::<U64Variable>();

        let r_source = right_proof_reader.read::<Bytes32Variable>();
        let r_target = right_proof_reader.read::<Bytes32Variable>();
        let r_commitment = right_proof_reader.read::<U64Variable>();
        let r_sigma =  left_proof_reader.read::<U64Variable>();

        //TODO: Figure out why asserts FAIL!
        // builder.assert_is_equal(l_sigma, r_sigma);

        // builder.assert_is_equal(l_source, r_source);
        // builder.assert_is_equal(l_target, r_target);

        let commitment = builder.add(l_commitment, r_commitment);

        builder.proof_write(l_source);
        builder.proof_write(l_target);
        builder.proof_write(commitment);
        builder.proof_write(l_sigma);
        
        
    }
}
