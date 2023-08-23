use plonky2::util::serialization::{Buffer, IoResult};

pub trait ReadTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized;
}

pub trait WriteTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>>;
}
