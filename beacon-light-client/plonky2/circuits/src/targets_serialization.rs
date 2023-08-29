use itertools::Itertools;
use plonky2::util::serialization::{Buffer, IoResult, Read, Write};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::biguint::BigUintTarget;

pub trait ReadTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized;
}

pub trait WriteTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>>;
}

impl ReadTargets for BigUintTarget {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(BigUintTarget {
            limbs: data
                .read_target_vec()?
                .iter()
                .map(|x| U32Target(*x))
                .collect_vec(),
        })
    }
}

impl WriteTargets for BigUintTarget {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_vec(&self.limbs.iter().map(|x| x.0).collect_vec())?;

        Ok(data)
    }
}
