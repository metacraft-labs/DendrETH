use plonky2::plonk::proof::ProofWithPublicInputsTarget;
use plonky2x::prelude::CircuitVariable;

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

impl<const D: usize> ProofWithPublicInputsTargetReader<D> {
    pub fn read<V: CircuitVariable>(&mut self) -> V {
        let public_inputs_len = self.inner.public_inputs.len();
        let result = V::from_targets(
            &self.inner.public_inputs
                [public_inputs_len - V::nb_elements()..public_inputs_len - self.elements_advanced],
        );
        self.elements_advanced += V::nb_elements();
        result
    }
}
