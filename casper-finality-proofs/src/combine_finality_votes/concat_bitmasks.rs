use plonky2::plonk::proof::ProofWithPublicInputsTarget;
use plonky2x::{
    backend::circuit::CircuitBuild,
    prelude::{ArrayVariable, CircuitBuilder, Field, PlonkParameters, Variable},
};

use super::{
    circuit::ProofWithPublicInputsTargetReader,
    verify_subcommittee_vote::{PACK_SIZE, VARIABLES_COUNT_LITTLE_BITMASK},
};

pub const fn get_input_bitmask_split_size<const LEVEL: usize>() -> usize {
    2usize.pow(LEVEL as u32) * VARIABLES_COUNT_LITTLE_BITMASK
}

pub const fn get_output_bitmask_split_size<const LEVEL: usize>() -> usize {
    2 * get_input_bitmask_split_size::<LEVEL>()
}

type InputBitmask<const LEVEL: usize> =
    ArrayVariable<Variable, { get_input_bitmask_split_size::<LEVEL>() }>;
type OutputBitmask<const LEVEL: usize> =
    ArrayVariable<Variable, { get_output_bitmask_split_size::<LEVEL>() }>;

fn extract_proof_outputs<const D: usize, const LEVEL: usize>(
    proof: ProofWithPublicInputsTarget<D>,
) -> (
    InputBitmask<LEVEL>,
    Variable,
    Variable,
    Variable,
    Variable,
    Variable,
)
where
    [(); get_input_bitmask_split_size::<LEVEL>()]:,
{
    let mut reader = ProofWithPublicInputsTargetReader::from(proof);
    (
        reader.read::<InputBitmask<LEVEL>>(),
        reader.read::<Variable>(),
        reader.read::<Variable>(),
        reader.read::<Variable>(),
        reader.read::<Variable>(),
        reader.read::<Variable>(),
    )
}

fn assert_bitmask_splits_are_incident<const LEVEL: usize, L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    left_begin: Variable,
    right_begin: Variable,
) {
    let expected_difference_between_begin_indices =
        builder.constant::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_usize(
            get_input_bitmask_split_size::<LEVEL>() * PACK_SIZE,
        ));
    let difference_between_begin_indices = builder.sub(right_begin, left_begin);
    builder.assert_is_equal(
        difference_between_begin_indices,
        expected_difference_between_begin_indices,
    );
}

#[derive(Debug, Clone)]
pub struct ConcatBitmasks<const LEVEL: usize>;

impl<const LEVEL: usize> ConcatBitmasks<LEVEL> {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        child_circuit: &CircuitBuild<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
        [(); get_input_bitmask_split_size::<LEVEL>()]:,
        [(); get_output_bitmask_split_size::<LEVEL>()]:,
    {
        let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
        let proof1 = builder.proof_read(&child_circuit).into();
        let proof2 = builder.proof_read(&child_circuit).into();

        builder.verify_proof::<L>(&proof1, &verifier_data, &child_circuit.data.common);
        builder.verify_proof::<L>(&proof2, &verifier_data, &child_circuit.data.common);

        let (l_bitmask, l_begin, l_voted_count, l_bls_signature, l_source, l_target) =
            extract_proof_outputs::<D, LEVEL>(proof1);

        let (r_bitmask, r_begin, r_voted_count, r_bls_signature, _, _) =
            extract_proof_outputs::<D, LEVEL>(proof2);

        assert_bitmask_splits_are_incident::<LEVEL, L, D>(builder, l_begin, r_begin);

        let voted_count = builder.add(l_voted_count, r_voted_count);
        let bls_signature = builder.add(l_bls_signature, r_bls_signature);
        let bitmask: OutputBitmask<LEVEL> =
            ArrayVariable::new([l_bitmask.as_slice(), r_bitmask.as_slice()].concat());

        builder.proof_write(l_target);
        builder.proof_write(l_source);

        builder.proof_write(bls_signature);
        builder.proof_write(voted_count);

        builder.proof_write(l_begin);
        builder.proof_write(bitmask);
    }
}
