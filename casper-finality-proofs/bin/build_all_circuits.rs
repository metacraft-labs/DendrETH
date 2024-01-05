use casper_finality_proofs::{
    compute_shuffled_index::circuit::define,
    weigh_justification_and_finalization::WeighJustificationAndFinalization,
};
use clap::Parser;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters, GateRegistry, HintRegistry},
    frontend::builder::CircuitBuilder,
};

#[derive(Parser)]
struct CircuitSerializer {
    /// Enter name of circuit
    #[arg(long)]
    name: String,
}

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let mut builder = CircuitBuilder::<DefaultParameters, D>::new();
    const SHUFFLE_ROUND_COUNT: u8 = 90;

    let circuit = builder.build();
    let hint_serializer = HintRegistry::<L, D>::new();
    let gate_serializer = GateRegistry::<L, D>::new();
    let circuit_serializer: CircuitSerializer = CircuitSerializer::parse();

    let compute_shuffled_index = "compute_shuffled_index".to_string();
    let weigh_justification_and_finalization = "weigh_justification_and_finalization".to_string();

    match circuit_serializer.name.clone() {
        compute_shuffled_index => define(builder, SHUFFLE_ROUND_COUNT),
        weigh_justification_and_finalization => WeighJustificationAndFinalization::define(builder),
    }

    let path = format!("build/${}", circuit_serializer.name);
    circuit.save(&path, &gate_serializer, &hint_serializer);

    // let cmd = clap::Command::new("build")
    //     .bin_name("build")
    //     .subcommand_required(true)
    //     .subcommand(
    //         clap::command!("example").arg(
    //             clap::arg!(--"manifest-path" <PATH>)
    //                 .value_parser(clap::value_parser!(std::path::PathBuf)),
    //         ),
    //     );

    // let match_result = Command::new("filter")
    //     .arg(Arg::new("compute_shuffled_index"))
    //     .arg(Arg::new("weigh_justification_and_finalization"))
    //     .get_matches();

    //.conflicts_with_all()
}
