use plonky2::field::goldilocks_field::GoldilocksField;
use tree::compute_validator_poseidon_hash_tree_root;
pub mod tree;

fn main() {
    const D: usize = 2;
    type F = GoldilocksField;

    compute_validator_poseidon_hash_tree_root::<F, D>();
}
