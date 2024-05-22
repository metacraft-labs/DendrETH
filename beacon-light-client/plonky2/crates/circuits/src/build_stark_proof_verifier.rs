use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};
use starky::{
    config::StarkConfig,
    proof::{StarkOpeningSetTarget, StarkProofTarget, StarkProofWithPublicInputsTarget},
    stark::Stark,
};
use starky_bls12_381::aggregate_proof::define_recursive_proof;

use crate::serialization::targets_serialization::{ReadTargets, WriteTargets};

const D: usize = 2;

// Maybe handled by Marti refactoring
impl ReadTargets for StarkProofWithPublicInputsTarget<D> {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        let proof = StarkProofWithPublicInputsTarget {
            proof: StarkProofTarget {
                trace_cap: data.read_target_merkle_cap()?,
                auxiliary_polys_cap: if data.read_bool()? {
                    Some(data.read_target_merkle_cap()?)
                } else {
                    None
                },
                quotient_polys_cap: data.read_target_merkle_cap()?,
                openings: StarkOpeningSetTarget {
                    local_values: data.read_target_ext_vec()?,
                    next_values: data.read_target_ext_vec()?,
                    auxiliary_polys: if data.read_bool()? {
                        Some(data.read_target_ext_vec()?)
                    } else {
                        None
                    },
                    auxiliary_polys_next: if data.read_bool()? {
                        Some(data.read_target_ext_vec()?)
                    } else {
                        None
                    },
                    ctl_zs_first: if data.read_bool()? {
                        Some(data.read_target_vec()?)
                    } else {
                        None
                    },
                    quotient_polys: data.read_target_ext_vec()?,
                },
                opening_proof: data.read_target_fri_proof()?,
            },
            public_inputs: data.read_target_vec()?,
        };

        Ok(proof)
    }
}

impl WriteTargets for StarkProofWithPublicInputsTarget<D> {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut target_bytes: Vec<u8> = Vec::new();

        target_bytes.write_target_merkle_cap(&self.proof.trace_cap)?;

        target_bytes.write_bool(self.proof.auxiliary_polys_cap.is_some())?;

        if let Some(auxiliary_polys_cap) = &self.proof.auxiliary_polys_cap {
            target_bytes.write_target_merkle_cap(auxiliary_polys_cap)?;
        }

        target_bytes.write_target_merkle_cap(&self.proof.quotient_polys_cap)?;
        target_bytes.write_target_ext_vec(&self.proof.openings.local_values)?;
        target_bytes.write_target_ext_vec(&self.proof.openings.next_values)?;

        target_bytes.write_bool(self.proof.openings.auxiliary_polys.is_some())?;

        if let Some(auxiliary_polys) = &self.proof.openings.auxiliary_polys {
            target_bytes.write_target_ext_vec(auxiliary_polys)?;
        }

        target_bytes.write_bool(self.proof.openings.auxiliary_polys_next.is_some())?;

        if let Some(auxiliary_polys_next) = &self.proof.openings.auxiliary_polys_next {
            target_bytes.write_target_ext_vec(auxiliary_polys_next)?;
        }

        target_bytes.write_bool(self.proof.openings.ctl_zs_first.is_some())?;

        if let Some(ctl_zs_first) = &self.proof.openings.ctl_zs_first {
            target_bytes.write_target_vec(ctl_zs_first)?;
        }

        target_bytes.write_target_ext_vec(&self.proof.openings.quotient_polys)?;

        target_bytes.write_target_fri_proof(&self.proof.opening_proof)?;

        target_bytes.write_target_vec(&self.public_inputs)?;

        Ok(target_bytes)
    }
}

pub struct RecursiveStarkTargets {
    pub proof: StarkProofWithPublicInputsTarget<D>,
    pub zero: Target,
}

impl ReadTargets for RecursiveStarkTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        let proof = StarkProofWithPublicInputsTarget::read_targets(data)?;
        let zero = data.read_target()?;

        Ok(RecursiveStarkTargets { proof, zero })
    }
}

impl WriteTargets for RecursiveStarkTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut target_bytes = self.proof.write_targets()?;
        target_bytes.write_target(self.zero)?;

        Ok(target_bytes)
    }
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
