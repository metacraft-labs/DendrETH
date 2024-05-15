use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::proof::ProofWithPublicInputsTarget,
    util::serialization::{Buffer, IoResult, Read, Write},
};

pub trait SerdeCircuitTarget {
    fn serialize(&self) -> IoResult<Vec<u8>>;

    fn deserialize(buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized;
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
        panic!("Recursive proofs are not supported by SerdeCircuitTarget")
    }

    fn deserialize(_buffer: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        panic!("Recursive proofs are not supported by SerdeCircuitTarget")
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
