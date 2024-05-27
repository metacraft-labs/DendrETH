use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{circuit_data::VerifierCircuitTarget, proof::ProofWithPublicInputsTarget},
    util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2_crypto::biguint::BigUintTarget;
use starky::proof::{StarkOpeningSetTarget, StarkProofTarget, StarkProofWithPublicInputsTarget};

use crate::{Circuit, PublicInputsTargetReadable};

pub trait SerdeCircuitTarget {
    fn serialize(&self) -> IoResult<Vec<u8>>;

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized;
}

pub fn deserialize_circuit_target<T: Circuit>(
    buffer: &mut Buffer,
) -> IoResult<<T as Circuit>::Target>
where
    <T as Circuit>::Target: SerdeCircuitTarget,
{
    <<T as Circuit>::Target as SerdeCircuitTarget>::deserialize(buffer)
}

impl SerdeCircuitTarget for Target {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_target(*self)?;
        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(buffer.read_target()?)
    }
}

impl SerdeCircuitTarget for BoolTarget {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_target_bool(*self)?;
        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(buffer.read_target_bool()?)
    }
}

impl SerdeCircuitTarget for HashOutTarget {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_target_hash(self)?;
        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(buffer.read_target_hash()?)
    }
}

impl<const D: usize> SerdeCircuitTarget for ProofWithPublicInputsTarget<D> {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_target_proof_with_public_inputs(&self)?;
        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(buffer.read_target_proof_with_public_inputs()?)
    }
}

impl SerdeCircuitTarget for VerifierCircuitTarget {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_target_verifier_circuit(&self)?;
        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(buffer.read_target_verifier_circuit()?)
    }
}

impl<T: SerdeCircuitTarget + std::fmt::Debug, const N: usize> SerdeCircuitTarget for [T; N] {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        for item in self {
            buffer.extend(item.serialize()?);
        }
        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok([(); N].try_map(|_| T::deserialize(buffer))?)
    }
}

impl SerdeCircuitTarget for BigUintTarget {
    fn serialize(&self) -> plonky2::util::serialization::IoResult<Vec<u8>> {
        assert_eq!(self.num_limbs(), 2);

        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_target(self.limbs[0].0)?;
        buffer.write_target(self.limbs[1].0)?;
        Ok(buffer)
    }

    fn deserialize(
        buffer: &mut plonky2::util::serialization::Buffer,
    ) -> plonky2::util::serialization::IoResult<Self>
    where
        Self: Sized,
    {
        let first_limb = buffer.read_target()?;
        let second_limb = buffer.read_target()?;
        Ok(BigUintTarget::from_targets(&[first_limb, second_limb]))
    }
}

impl<const D: usize> SerdeCircuitTarget for StarkProofWithPublicInputsTarget<D> {
    fn serialize(&self) -> IoResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.write_target_merkle_cap(&self.proof.trace_cap)?;

        buffer.write_bool(self.proof.auxiliary_polys_cap.is_some())?;

        if let Some(auxiliary_polys_cap) = &self.proof.auxiliary_polys_cap {
            buffer.write_target_merkle_cap(auxiliary_polys_cap)?;
        }

        buffer.write_bool(self.proof.quotient_polys_cap.is_some())?;

        if let Some(quotient_polys_cap) = &self.proof.quotient_polys_cap {
            buffer.write_target_merkle_cap(quotient_polys_cap)?;
        }

        buffer.write_target_ext_vec(&self.proof.openings.local_values)?;
        buffer.write_target_ext_vec(&self.proof.openings.next_values)?;

        buffer.write_bool(self.proof.openings.auxiliary_polys.is_some())?;

        if let Some(auxiliary_polys) = &self.proof.openings.auxiliary_polys {
            buffer.write_target_ext_vec(auxiliary_polys)?;
        }

        buffer.write_bool(self.proof.openings.auxiliary_polys_next.is_some())?;

        if let Some(auxiliary_polys_next) = &self.proof.openings.auxiliary_polys_next {
            buffer.write_target_ext_vec(auxiliary_polys_next)?;
        }

        buffer.write_bool(self.proof.openings.ctl_zs_first.is_some())?;

        if let Some(ctl_zs_first) = &self.proof.openings.ctl_zs_first {
            buffer.write_target_vec(ctl_zs_first)?;
        }

        buffer.write_bool(self.proof.openings.quotient_polys.is_some())?;

        if let Some(quotient_polys) = &self.proof.openings.quotient_polys {
            buffer.write_target_ext_vec(quotient_polys)?;
        }

        buffer.write_target_fri_proof(&self.proof.opening_proof)?;

        buffer.write_target_vec(&self.public_inputs)?;

        Ok(buffer)
    }

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        let proof = StarkProofWithPublicInputsTarget {
            proof: StarkProofTarget {
                trace_cap: buffer.read_target_merkle_cap()?,
                auxiliary_polys_cap: if buffer.read_bool()? {
                    Some(buffer.read_target_merkle_cap()?)
                } else {
                    None
                },
                quotient_polys_cap: if buffer.read_bool()? {
                    Some(buffer.read_target_merkle_cap()?)
                } else {
                    None
                },
                openings: StarkOpeningSetTarget {
                    local_values: buffer.read_target_ext_vec()?,
                    next_values: buffer.read_target_ext_vec()?,
                    auxiliary_polys: if buffer.read_bool()? {
                        Some(buffer.read_target_ext_vec()?)
                    } else {
                        None
                    },
                    auxiliary_polys_next: if buffer.read_bool()? {
                        Some(buffer.read_target_ext_vec()?)
                    } else {
                        None
                    },
                    ctl_zs_first: if buffer.read_bool()? {
                        Some(buffer.read_target_vec()?)
                    } else {
                        None
                    },
                    quotient_polys: if buffer.read_bool()? {
                        Some(buffer.read_target_ext_vec()?)
                    } else {
                        None
                    },
                },
                opening_proof: buffer.read_target_fri_proof()?,
            },
            public_inputs: buffer.read_target_vec()?,
        };

        Ok(proof)
    }
}
