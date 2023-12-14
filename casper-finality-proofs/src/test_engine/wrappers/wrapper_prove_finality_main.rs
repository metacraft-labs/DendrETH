use once_cell::sync::Lazy;
use plonky2x::{
    backend::circuit::{Circuit, CircuitBuild, DefaultParameters, PlonkParameters},
    frontend::{builder::CircuitBuilder, uint::uint64::U64Variable},
};

use crate::{
    prove_finality::prove_finality_main::ProveFinality,
    test_engine::{
        types::prove_finality_data::ProveFinalityData, utils::parsers::parse_file::read_fixture,
    },
    weigh_justification_and_finalization::{
        checkpoint::{CheckpointValue, CheckpointVariable},
        justification_bits::{JustificationBitsValue, JustificationBitsVariable},
    },
};

// Singleton-like pattern
pub static CIRCUIT: Lazy<CircuitBuild<DefaultParameters, 2>> = Lazy::new(|| {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    ProveFinality::define(&mut builder);
    builder.build()
});

pub fn wrapper(path: &str, _should_assert: bool) -> Result<String, anyhow::Error> {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder = CircuitBuilder::<L, D>::new();
    ProveFinality::define(&mut builder);
    let mut input = CIRCUIT.input();
    let json_data = read_fixture::<ProveFinalityData>(path);

    let total_number_of_validators = json_data.total_number_of_validators;
    let previous_epoch_attested_validators = json_data.previous_epoch_attested_validators;
    let current_epoch_attested_validators = json_data.current_epoch_attested_validators;
    let slot = json_data.slot;
    let source = CheckpointValue {
        epoch: json_data.source.epoch,
        root: json_data.source.root,
    };

    let target = CheckpointValue {
        epoch: json_data.target.epoch,
        root: json_data.target.root,
    };

    let justification_bits = JustificationBitsValue::<<L as PlonkParameters<D>>::Field> {
        bits: json_data.justification_bits,
    };

    let previous_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: json_data.previous_justified_checkpoint.epoch,
        root: json_data.previous_justified_checkpoint.root,
    };
    let current_justified_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: json_data.current_justified_checkpoint.epoch,
        root: json_data.current_justified_checkpoint.root,
    };
    let finalized_checkpoint = CheckpointValue::<<L as PlonkParameters<D>>::Field> {
        epoch: json_data.finalized_checkpoint.epoch,
        root: json_data.finalized_checkpoint.root,
    };

    input.write::<CheckpointVariable>(source.clone());
    input.write::<CheckpointVariable>(target);
    input.write::<U64Variable>(slot);
    input.write::<U64Variable>(total_number_of_validators);
    input.write::<JustificationBitsVariable>(justification_bits);
    input.write::<U64Variable>(previous_epoch_attested_validators);
    input.write::<U64Variable>(current_epoch_attested_validators);
    input.write::<CheckpointVariable>(previous_justified_checkpoint);
    input.write::<CheckpointVariable>(current_justified_checkpoint);
    input.write::<CheckpointVariable>(finalized_checkpoint);

    let (proof, output) = CIRCUIT.prove(&input);
    CIRCUIT.verify(&proof, &input, &output);

    Ok(String::new())
}
