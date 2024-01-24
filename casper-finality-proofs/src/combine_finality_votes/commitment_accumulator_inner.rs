use plonky2x::{
    backend::circuit::CircuitBuild,
    //frontend::eth::vars::BLSPubkeyVariable,
    prelude::{CircuitBuilder, PlonkParameters}, frontend::{uint::uint64::U64Variable, vars::{Bytes32Variable, ArrayVariable}},
};

use super::circuit::ProofWithPublicInputsTargetReader;
use crate::{utils::eth_objects::CheckpointVariable, constants::{STATE_ROOT_PROOF_LEN, VALIDATORS_ROOT_PROOF_LEN}};

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

        let l_sigma =  left_proof_reader.read::<U64Variable>();
        let l_commitment = left_proof_reader.read::<U64Variable>();
        let l_target = left_proof_reader.read::<CheckpointVariable>();
        let l_source = left_proof_reader.read::<CheckpointVariable>();

        let l_state_root = left_proof_reader.read::<Bytes32Variable>();
        let l_state_root_proof = left_proof_reader.read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>();
        let l_validators_root = left_proof_reader.read::<Bytes32Variable>();
        let l_validators_root_proof = left_proof_reader.read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>();
        let l_slot = left_proof_reader.read::<U64Variable>();

        let r_sigma =  right_proof_reader.read::<U64Variable>();
        let r_commitment = right_proof_reader.read::<U64Variable>();
        let r_target = right_proof_reader.read::<CheckpointVariable>();
        let r_source = right_proof_reader.read::<CheckpointVariable>();

        let r_state_root = right_proof_reader.read::<Bytes32Variable>();
        let _r_state_root_proof = right_proof_reader.read::<ArrayVariable<Bytes32Variable, STATE_ROOT_PROOF_LEN>>();
        let r_validators_root = right_proof_reader.read::<Bytes32Variable>();
        let _r_validators_root_proof = right_proof_reader.read::<ArrayVariable<Bytes32Variable, VALIDATORS_ROOT_PROOF_LEN>>();
        let r_slot = right_proof_reader.read::<U64Variable>();

        builder.assert_is_equal(l_sigma, r_sigma);
        builder.assert_is_equal(l_source, r_source);
        builder.assert_is_equal(l_target, r_target);

        builder.assert_is_equal(l_state_root, r_state_root);
        // builder.assert_is_equal(&l_state_root_proof, &r_state_root_proof); //TODO: Don't need to verify state_root_proofs & validator_state_root_proofs?
        builder.assert_is_equal(l_validators_root, r_validators_root);
        // builder.assert_is_equal(&l_validators_root_proof, &r_validators_root_proof);
        builder.assert_is_equal(l_slot, r_slot);

        let commitment = builder.add(l_commitment, r_commitment);

        builder.proof_write(l_slot);
        builder.proof_write(l_validators_root_proof);
        builder.proof_write(l_validators_root);
        builder.proof_write(l_state_root_proof);
        builder.proof_write(l_state_root);
      
        builder.proof_write(l_source);
        builder.proof_write(l_target);
        builder.proof_write(commitment);
        builder.proof_write(l_sigma);


    }
}
