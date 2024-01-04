// circuits_serializer --filter=all
// circuit_serializer --filter=compute_shuffled_index,weigh_justification...
use clap::{App, Arg};
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters, GateRegistry, HintRegistry},
    frontend::builder::CircuitBuilder,
};

use crate::weigh_justification_and_finalization::WeighJustificationAndFinalization;

use super::circuit::define;
fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder = CircuitBuilder::<DefaultParameters, D>::new();
    const SHUFFLE_ROUND_COUNT: u8 = 90;

    let circuit = builder.build();
    let hint_serializer = HintRegistry::<L, D>::new();
    let gate_serializer = GateRegistry::<L, D>::new();
    define(&mut builder, SHUFFLE_ROUND_COUNT);
    circuit.save(
        &"build/compute_shuffled_index".to_string(),
        &gate_serializer,
        &hint_serializer,
    );

    let circuit = builder.build();
    let hint_serializer = HintRegistry::<L, D>::new();
    let gate_serializer = GateRegistry::<L, D>::new();
    WeighJustificationAndFinalization::define(&mut builder);
    circuit.save(
        &"build/weigh_justification_and_finalization".to_string(),
        &gate_serializer,
        &hint_serializer,
    );

    let matches = App::new("")
        .arg(
            Arg::new("compute_shuffled_index")
                .long("compute shuffled index")
                .value_name("Redis Connection")
                .help("Sets a custom Redis connection")
                .default_value(""),
        )
        .arg(
            Arg::new("weigh_justification_and_finalization")
                .long("weigh justification and finalization")
                .value_name("Stop after")
                .help("Sets how much seconds to wait until the program stops if no new tasks are found in the queue")
                .default_value("20")
        )
        .get_matches();
}
