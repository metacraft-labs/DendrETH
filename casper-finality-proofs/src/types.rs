use plonky2x::prelude::{ArrayVariable, Bytes32Variable, U64Variable};

pub type Epoch = U64Variable;
pub type Slot = U64Variable;
pub type Root = Bytes32Variable;
pub type Gwei = U64Variable;
pub type MerkleProof<const DEPTH: usize> = ArrayVariable<Bytes32Variable, DEPTH>;
pub type BeaconStateLeafProof = MerkleProof<5>;
