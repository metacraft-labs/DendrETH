use casper_finality_proofs::test_engine::wrappers::{
    compute_shuffled_index::wrapper_mainnet::MAINNET_CIRCUIT as circuit_mainnet,
    compute_shuffled_index::wrapper_minimal::MINIMAL_CIRCUIT as circuit_minimal,
    wrapper_weigh_justification_and_finalization::CIRCUIT as circuit_weigh_justification_and_finalization,
};
use clap::Parser;
use once_cell::sync::Lazy;
use plonky2x::{
    backend::circuit::{DefaultParameters, GateRegistry, HintRegistry},
    frontend::builder::CircuitBuilder,
};
use std::env::args;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, EnumString, Display, EnumIter)]
#[allow(non_camel_case_types)]
enum Circuits {
    compute_shuffled_index_mainnet,
    compute_shuffled_index_minimal,
    weigh_justification_and_finalization,
    all,
}

// enum TypeResult {
//     OneCircuit((Box<dyn Fn() -> () + Send + Sync>, String)),
//     AllCircuits(Vec<(Box<dyn Fn() -> () + Send + Sync>, String)>),
// }

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CircuitSerializer {
    /// Enter name of circuit
    // #[arg(long)]
    #[clap(value_delimiter = ' ', num_args = 0.., use_value_delimiter = true)]
    name: Circuits,
}

fn build_specific_circuit(circuit: &str) -> Box<dyn Fn() -> () + Send + Sync> {
    match circuit {
        "compute_shuffled_index_mainnet" => Box::new(|| {
            Lazy::force(&circuit_mainnet);
        }),
        "compute_shuffled_index_minimal" => Box::new(|| {
            Lazy::force(&circuit_minimal);
        }),
        "weigh_justification_and_finalization" => Box::new(|| {
            Lazy::force(&circuit_weigh_justification_and_finalization);
        }),
        _ => Box::new(|| {
            Lazy::force(&circuit_weigh_justification_and_finalization);
        }),
    }
}

fn build_all_circuits() -> Vec<Box<dyn Fn() -> () + Send + Sync>> {
    vec![
        (Box::new(|| {
            Lazy::force(&circuit_mainnet);
        })),
        (Box::new(|| {
            Lazy::force(&circuit_minimal);
        })),
        (Box::new(|| {
            Lazy::force(&circuit_weigh_justification_and_finalization);
        })),
    ]
}

// fn build_circuit(circuits: Circuits) -> TypeResult {
//     match circuits {
//         Circuits::ComputeShuffledIndexMainnet => TypeResult::OneCircuit((
//             Box::new(|| {
//                 Lazy::force(&circuit_mainnet);
//             }),
//             "compute_shuffled_index_mainnet".to_string(),
//         )),
//         Circuits::ComputeShuffledIndexMinimal => TypeResult::OneCircuit((
//             Box::new(|| {
//                 Lazy::force(&circuit_minimal);
//             }),
//             "compute_shuffled_index_minimal".to_string(),
//         )),
//         Circuits::WeighJustificationAndFinalization => TypeResult::OneCircuit((
//             Box::new(|| {
//                 Lazy::force(&circuit_weigh_justification_and_finalization);
//             }),
//             "weigh_justification_and_finalization".to_string(),
//         )),
//         Circuits::All => TypeResult::AllCircuits(vec![
//             (
//                 Box::new(|| {
//                     Lazy::force(&circuit_mainnet);
//                 }),
//                 "compute_shuffled_index_mainnet".to_string(),
//             ),
//             (
//                 Box::new(|| {
//                     Lazy::force(&circuit_minimal);
//                 }),
//                 "compute_shuffled_index_minimal".to_string(),
//             ),
//             (
//                 Box::new(|| {
//                     Lazy::force(&circuit_weigh_justification_and_finalization);
//                 }),
//                 "weigh_justification_and_finalization".to_string(),
//             ),
//         ]),
//     }
// }

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let builder = CircuitBuilder::<DefaultParameters, D>::new();

    let circuit = builder.build();
    let hint_serializer = HintRegistry::<L, D>::new();
    let gate_serializer = GateRegistry::<L, D>::new();
    let circuit_serializer: CircuitSerializer = CircuitSerializer::parse();
    let command_line_arguments: Vec<String> = args().skip(1).collect();

    println!(
        "command_line_arguments are: {:?}",
        command_line_arguments.as_slice()
    );

    if circuit_serializer.name != Circuits::all {
        for (index, arg) in command_line_arguments.iter().enumerate() {
            println!(
                "command_line_arguments are: {:?}",
                &command_line_arguments[index]
            );
            for word in arg.split_whitespace() {
                println!("word: {:?}", word);
                build_specific_circuit(word);
                let path = format!("build/{}", word);
                circuit.save(&path, &gate_serializer, &hint_serializer);
            }
        }
    } else {
        build_all_circuits();
        for _circuit in Circuits::iter() {
            if _circuit != Circuits::all {
                let path = format!("build/{}", _circuit.to_string());
                circuit.save(&path, &gate_serializer, &hint_serializer);
            }
        }
    }
}
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
