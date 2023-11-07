use casper_finality_proofs::prove_finality::circuit::ProveFinality;
use casper_finality_proofs::weigh_justification_and_finalization::checkpoint::{
    CheckpointValue, CheckpointVariable,
};
use casper_finality_proofs::weigh_justification_and_finalization::justification_bits::{
    JustificationBitsValue, JustificationBitsVariable,
};
use plonky2x::backend::circuit::Circuit;
use plonky2x::backend::circuit::DefaultParameters;
use plonky2x::{
    prelude::{CircuitBuilder, PlonkParameters, U64Variable},
    utils::bytes32,
};

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder: CircuitBuilder<DefaultParameters, 2> =
        CircuitBuilder::<DefaultParameters, 2>::new();
    ProveFinality::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();

    let total_number_of_validators = 65536;
    let previous_epoch_attested_validators = 52430;
    let current_epoch_attested_validators = 52430;

    let justification_bits = JustificationBitsValue::<<L as PlonkParameters<D>>::Field> {
        bits: vec![true, true, false, true],
    };

    let source = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 123456788,
        root: bytes32!("0x0000000000000000000000000000000000000000000000000000000000000000"),
    };

    let target = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: 123456790,
        root: bytes32!("0x0000000000000000000000000000000000000000000000000000000000000000"),
    };

    let slot = 3950617280;

    input.write::<CheckpointVariable>(source);
    input.write::<CheckpointVariable>(target);
    input.write::<U64Variable>(slot);
    input.write::<U64Variable>(total_number_of_validators);
    input.write::<JustificationBitsVariable>(justification_bits);
    input.write::<U64Variable>(previous_epoch_attested_validators);
    input.write::<U64Variable>(current_epoch_attested_validators);

    let (_witness, mut _output) = circuit.prove(&input);
    println!("Successfully passed!");
}
