use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitData, VerifierCircuitTarget, VerifierOnlyCircuitData};
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig};
use plonky2::plonk::proof::ProofWithPublicInputsTarget;

pub fn create_verifier_circuit_target<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    verifier_only: &VerifierOnlyCircuitData<C, D>,
) -> VerifierCircuitTarget
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    VerifierCircuitTarget {
        constants_sigmas_cap: builder.constant_merkle_cap(&verifier_only.constants_sigmas_cap),
        circuit_digest: builder.constant_hash(verifier_only.circuit_digest),
    }
}

pub fn verify_proof<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    circuit_data: &CircuitData<F, C, D>,
) -> ProofWithPublicInputsTarget<D>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let proof = builder.add_virtual_proof_with_pis(&circuit_data.common);
    let verifier_circuit_data =
        create_verifier_circuit_target(builder, &circuit_data.verifier_only);
    builder.verify_proof::<C>(&proof, &verifier_circuit_data, &circuit_data.common);
    proof
}
