use plonky2x::{
    backend::circuit::CircuitBuild,
    prelude::{ArrayVariable, CircuitBuilder, PlonkParameters, Variable},
};

use super::{
    circuit::ProofWithPublicInputsTargetReader,
    verify_subcommittee_vote::VARIABLES_COUNT_LITTLE_BITMASK,
};

#[derive(Debug, Clone)]
pub struct ConcatBitmasks<const LEVEL: usize>;

impl<const LEVEL: usize> ConcatBitmasks<LEVEL> {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        child_circuit: &CircuitBuild<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
        [(); 2usize.pow(LEVEL as u32) * VARIABLES_COUNT_LITTLE_BITMASK]:,
        [(); 2usize.pow((LEVEL + 1) as u32) * VARIABLES_COUNT_LITTLE_BITMASK]:,
    {
        let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
        let proof1 = builder.proof_read(&child_circuit).into();
        let proof2 = builder.proof_read(&child_circuit).into();

        builder.verify_proof::<L>(&proof1, &verifier_data, &child_circuit.data.common);
        builder.verify_proof::<L>(&proof2, &verifier_data, &child_circuit.data.common);

        let mut proof1_reader = ProofWithPublicInputsTargetReader::from(proof1);
        let mut proof2_reader = ProofWithPublicInputsTargetReader::from(proof2);

        // read left proof
        let left_bitmask =
            proof1_reader.read::<ArrayVariable<
                Variable,
                { 2usize.pow(LEVEL as u32) * VARIABLES_COUNT_LITTLE_BITMASK },
            >>();
        let left_range_begin = proof1_reader.read::<Variable>();
        let left_voted_count = proof1_reader.read::<Variable>();
        let left_bls_signature = proof1_reader.read::<Variable>();
        let left_source = proof1_reader.read::<Variable>();
        let left_target = proof1_reader.read::<Variable>();

        // read right proof
        let right_bitmask =
            proof2_reader.read::<ArrayVariable<
                Variable,
                { 2usize.pow(LEVEL as u32) * VARIABLES_COUNT_LITTLE_BITMASK },
            >>();
        let _right_range_begin = proof2_reader.read::<Variable>();
        let right_voted_count = proof2_reader.read::<Variable>();
        let right_bls_signature = proof2_reader.read::<Variable>();
        let _right_source = proof2_reader.read::<Variable>();
        let _right_target = proof2_reader.read::<Variable>();

        let _voted_count = builder.one::<Variable>();

        let bitmask_data = [left_bitmask.as_slice(), right_bitmask.as_slice()].concat();

        let bitmask: ArrayVariable<
            Variable,
            { 2usize.pow((LEVEL + 1) as u32) * VARIABLES_COUNT_LITTLE_BITMASK },
        > = ArrayVariable::new(bitmask_data);

        let voted_count = builder.add(left_voted_count, right_voted_count);

        let bls_signature = builder.add(left_bls_signature, right_bls_signature);
        let range_begin = left_range_begin;

        builder.proof_write(left_target);
        builder.proof_write(left_source);

        builder.proof_write(bls_signature);
        builder.proof_write(voted_count);

        builder.proof_write(range_begin);
        builder.proof_write(bitmask);
    }
}
