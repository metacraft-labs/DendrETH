use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use tree::do_something;
pub mod tree;

fn main() {
    const D: usize = 2;
    type F = GoldilocksField;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);
    do_something::<F, D>(&mut builder);
}
