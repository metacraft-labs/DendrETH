use std::{str::FromStr, time::Instant};

use ark_bls12_381::{G1Affine, G2Affine};
use circuits::{
    common_targets::BasicRecursiveInnerCircuitTarget,
};
use num_bigint::BigUint;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_data::CircuitData,
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputs,
    },
};

use anyhow::Result;

pub fn prove_inner_level(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_target: &BasicRecursiveInnerCircuitTarget,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    let inner_proof1 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof1_bytes,
            &circuit_data.common,
        )?;

    let inner_proof2 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof2_bytes,
            &circuit_data.common,
        )?;

    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&inner_circuit_target.proof1, &inner_proof1);
    pw.set_proof_with_pis_target(&inner_circuit_target.proof2, &inner_proof2);

    Ok(inner_circuit_data.prove(pw)?)
}

// TODO: Don't hard code the D
pub fn prove_inner_level2<F: RichField + Extendable<2>, C: GenericConfig<2, F = F>>(
    proof1: &ProofWithPublicInputs<F, C, 2>,
    proof2: &ProofWithPublicInputs<F, C, 2>,
    inner_circuit_target: &BasicRecursiveInnerCircuitTarget,
    inner_circuit_data: &CircuitData<F, C, 2>,
) -> Result<ProofWithPublicInputs<F, C, 2>>
where
    <C as GenericConfig<2>>::Hasher: AlgebraicHasher<F>,
{
    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&inner_circuit_target.proof1, proof1);
    pw.set_proof_with_pis_target(&inner_circuit_target.proof2, proof2);

    Ok(inner_circuit_data.prove(pw)?)
}
