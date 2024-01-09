use plonky2::plonk::proof::ProofWithPublicInputsTarget;
use plonky2x::{
    backend::circuit::CircuitBuild,
    prelude::{
        ArrayVariable, BoolVariable, CircuitBuilder, CircuitVariable, PlonkParameters, Variable,
    },
};

use super::verify_subcommittee_vote::{PACKS_COUNT, VALIDATOR_SIZE_UPPER_BOUND};

pub struct ProofWithPublicInputsTargetReader<const D: usize> {
    inner: ProofWithPublicInputsTarget<D>,
    elements_advanced: usize,
}

impl<const D: usize> From<ProofWithPublicInputsTarget<D>> for ProofWithPublicInputsTargetReader<D> {
    fn from(proof: ProofWithPublicInputsTarget<D>) -> Self {
        ProofWithPublicInputsTargetReader {
            inner: proof,
            elements_advanced: 0,
        }
    }
}
/*
[NOTE] 
    Reader takes public inputs from last to first, when running recurssive proofs
    The order of writing public inputs should be inverse to the order of reading 
 */
impl<const D: usize> ProofWithPublicInputsTargetReader<D> {
    pub fn read<V: CircuitVariable>(&mut self) -> V {
        let public_inputs_len = self.inner.public_inputs.len();

        let result = V::from_targets(
            &self.inner.public_inputs[public_inputs_len - self.elements_advanced - V::nb_elements()
                ..public_inputs_len - self.elements_advanced],
        );
        self.elements_advanced += V::nb_elements();
        result
    }
}

fn unite_validators_bitmasks<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bitmask1: &ArrayVariable<BoolVariable, VALIDATOR_SIZE_UPPER_BOUND>,
    bitmask2: &ArrayVariable<BoolVariable, VALIDATOR_SIZE_UPPER_BOUND>,
) -> (
    ArrayVariable<BoolVariable, VALIDATOR_SIZE_UPPER_BOUND>,
    Variable,
) {
    let mut voted_count = builder.zero();
    let mut result_bitmask = vec![builder._false(); VALIDATOR_SIZE_UPPER_BOUND];

    for i in 0..VALIDATOR_SIZE_UPPER_BOUND {
        let either_bit_is_set = builder.or(bitmask1[i], bitmask2[i]);
        result_bitmask[i] = either_bit_is_set;
        voted_count = builder.add(voted_count, either_bit_is_set.variable);
    }

    (ArrayVariable::new(result_bitmask), voted_count)
}

#[derive(Debug, Clone)]
pub struct CombineFinalityVotes;

impl CombineFinalityVotes {
    pub fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
        child_circuit: &CircuitBuild<L, D>,
    ) where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let verifier_data = builder.constant_verifier_data::<L>(&child_circuit.data);
        let proof1 = builder.proof_read(&child_circuit.data.common).into();
        let proof2 = builder.proof_read(&child_circuit.data.common).into();

        builder.verify_proof::<L>(&proof1, &verifier_data, &child_circuit.data.common);
        builder.verify_proof::<L>(&proof2, &verifier_data, &child_circuit.data.common);

        let mut proof1_reader = ProofWithPublicInputsTargetReader::from(proof1);
        let mut proof2_reader = ProofWithPublicInputsTargetReader::from(proof2);

        let bitmask1 = proof1_reader.read::<ArrayVariable<Variable, PACKS_COUNT>>();
        let voted_count1 = proof1_reader.read::<Variable>();
        let target1 = proof1_reader.read::<Variable>();
        let source1 = proof1_reader.read::<Variable>();

        let bitmask2 = proof2_reader.read::<ArrayVariable<Variable, PACKS_COUNT>>();
        let voted_count2 = proof2_reader.read::<Variable>();
        let target2 = proof2_reader.read::<Variable>();
        let source2 = proof2_reader.read::<Variable>();

        // builder.assert_is_equal(source1, source2);
        // builder.assert_is_equal(target1, target2);

        // let (bitmask, voted_count) = unite_validators_bitmasks(builder, &bitmask1, &bitmask2);
        let voted_count = builder.one::<Variable>();
        let bitmask = bitmask1.clone();

        // NOTE: This doesn't need to be here
        // let voted_count_sum = builder.add(voted_count1, voted_count2);
        // let bitmask_sanity_check_pred = builder.gte(voted_count_sum, voted_count);
        // assert_is_true(builder, bitmask_sanity_check_pred);

        // builder.watch(&bitmask1, "bitmask1");
        // builder.watch(&bitmask2, "bitmask2");

        // builder.watch(&source1, "source");
        // builder.watch(&target1, "target");
        // builder.watch(&voted_count, "voted_count");
        // builder.watch(&bitmask, "bitmask");

        builder.proof_write(source1);
        builder.proof_write(target1);
        builder.proof_write(voted_count);
        builder.proof_write(bitmask);
    }
}
