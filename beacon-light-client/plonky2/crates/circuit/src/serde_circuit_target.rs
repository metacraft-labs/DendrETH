use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{circuit_data::VerifierCircuitTarget, proof::ProofWithPublicInputsTarget},
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::Circuit;

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
