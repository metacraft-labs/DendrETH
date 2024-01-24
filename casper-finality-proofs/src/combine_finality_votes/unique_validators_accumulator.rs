use plonky2x::{
    backend::circuit::CircuitBuild,
    prelude::{CircuitBuilder, PlonkParameters}, frontend::{uint::uint64::U64Variable},
};

use crate::utils::plonky2x_extensions::assert_is_true;

use super::circuit::ProofWithPublicInputsTargetReader;

#[derive(Debug, Clone)]
pub struct UniqueValidatorsAccumulatorInner;

impl UniqueValidatorsAccumulatorInner {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        child_circuit: &CircuitBuild<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let one: U64Variable = builder.one();
        let zero: U64Variable = builder.zero();

        let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
        let left_proof = builder.proof_read(&child_circuit.data.common).into();
        let right_proof = builder.proof_read(&child_circuit.data.common).into();

        builder.verify_proof::<L>(&left_proof, &verifier_data, &child_circuit.data.common);
        builder.verify_proof::<L>(&right_proof, &verifier_data, &child_circuit.data.common);

        let mut left_proof_reader = ProofWithPublicInputsTargetReader::from(left_proof);
        let mut right_proof_reader = ProofWithPublicInputsTargetReader::from(right_proof);

        let l_rightmost = left_proof_reader.read::<U64Variable>();
        let l_leftmost = left_proof_reader.read::<U64Variable>();
        let l_commitment_accumulator = left_proof_reader.read::<U64Variable>();
        let l_total_unique =  left_proof_reader.read::<U64Variable>();

        let r_rightmost = right_proof_reader.read::<U64Variable>();
        let r_leftmost = right_proof_reader.read::<U64Variable>();
        let r_commitment_accumulator = right_proof_reader.read::<U64Variable>();
        let r_total_unique =  right_proof_reader.read::<U64Variable>();

        let commitment_aggregated = builder.add(l_commitment_accumulator, r_commitment_accumulator);

        let mut unique_count = builder.add(r_total_unique, l_total_unique);
        
        let is_repeated_border = builder.is_equal(l_rightmost, r_leftmost);
        let value_to_sub  = builder.select(is_repeated_border, one, zero);
        unique_count = builder.sub(unique_count, value_to_sub);

        let chunks_aligned_pred = builder.lte(l_leftmost, r_rightmost);
        assert_is_true(builder,chunks_aligned_pred);

        // TODO: Can we make the reading and writing more intuitive cause this reversal is hard
        builder.proof_write(unique_count);
        builder.proof_write(commitment_aggregated);
        builder.proof_write(l_leftmost);
        builder.proof_write(r_rightmost);
    }
}
