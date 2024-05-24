use circuit_derive::SerdeCircuitTarget;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
    },
};
use starky::{config::StarkConfig, proof::StarkProofWithPublicInputsTarget, stark::Stark};
use starky_bls12_381::aggregate_proof::define_recursive_proof;

const D: usize = 2;

#[derive(SerdeCircuitTarget)]
pub struct RecursiveStarkTargets {
    pub proof: StarkProofWithPublicInputsTarget<D>,
    pub zero: Target,
}

pub fn build_stark_proof_verifier<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    S: Stark<F, D> + Copy,
>(
    stark: S,
    inner_proof: &starky::proof::StarkProofWithPublicInputs<F, C, D>,
    inner_config: &StarkConfig,
) -> (RecursiveStarkTargets, CircuitData<F, C, D>)
where
    C::Hasher: AlgebraicHasher<F>,
{
    let circuit_config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(circuit_config);

    let proof = define_recursive_proof::<F, C, S, C, D>(
        stark,
        inner_proof,
        &inner_config,
        false,
        &mut builder,
    );

    let zero = builder.zero();

    let data = builder.build::<C>();

    (RecursiveStarkTargets { proof, zero }, data)
}
