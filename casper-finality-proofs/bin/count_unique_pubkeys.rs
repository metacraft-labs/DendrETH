pub mod count_unique_validators;

use casper_finality_proofs::combine_finality_votes::count_unique_pubkeys::CountUniquePubkeys;
use plonky2x::{
    frontend::eth::vars::BLSPubkeyVariable,
    prelude::{CircuitBuilder, DefaultParameters},
    utils::bytes,
};

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;

    plonky2x::utils::setup_logger();

    let mut builder = CircuitBuilder::<L, D>::new();
    CountUniquePubkeys::define(&mut builder);
    let leaf_circuit = builder.build();
    let mut input = leaf_circuit.input();
    input.write::<BLSPubkeyVariable>(bytes!("0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95"));

    let (proof, output) = leaf_circuit.prove(&input);
    leaf_circuit.verify(&proof, &input, &output);
}
