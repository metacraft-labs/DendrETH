use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_sha256::circuit::make_circuits;

pub fn hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pw: &mut PartialWitness<F>,
    leaves: &Vec<[bool; 256]>,
) {
    for i in 0..leaves.len() - 1 {
        let hasher = make_circuits(builder, 512);

        for j in 0..256 {
            pw.set_bool_target(hasher.message[i], leaves[i][j]);
        }

        for j in 0..256 {
            pw.set_bool_target(hasher.message[i + 256], leaves[i + 1][j]);
        }
    }
}
