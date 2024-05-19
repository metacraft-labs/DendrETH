use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use plonky2::{
    iop::target::BoolTarget,
    plonk::{circuit_data::VerifierCircuitTarget, proof::ProofWithPublicInputsTarget},
};

pub type Sha256Target = [BoolTarget; 256];
pub type SSZLeafTarget = [BoolTarget; 256];

#[derive(CircuitTarget, SerdeCircuitTarget)]
pub struct BasicRecursiveInnerCircuitTarget {
    pub proof1: ProofWithPublicInputsTarget<2>,
    pub proof2: ProofWithPublicInputsTarget<2>,
    pub verifier_circuit_target: VerifierCircuitTarget,
}
