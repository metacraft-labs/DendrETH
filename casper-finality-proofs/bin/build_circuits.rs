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
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, EnumString, Display, EnumIter)]
enum Circuits {
    ComputeShuffledIndexMainnet,
    ComputeShuffledIndexMinimal,
    WeighJustificationAndFinalization,
    All,
}

enum TypeResult {
    OneCircuit((Box<dyn Fn() -> () + Send + Sync>, String)),
    AllCircuits(Vec<(Box<dyn Fn() -> () + Send + Sync>, String)>),
}

#[derive(Parser)]
struct CircuitSerializer {
    /// Enter name of circuit
    #[arg(long)]
    name: Circuits,
}

fn build_specific_circuit(circuit: Circuits) -> (Box<dyn Fn() -> () + Send + Sync>, String) {
    match circuit {
        Circuits::ComputeShuffledIndexMainnet => (
            Box::new(|| {
                Lazy::force(&circuit_mainnet);
            }),
            "compute_shuffled_index_mainnet".to_string(),
        ),
        Circuits::ComputeShuffledIndexMinimal => (
            Box::new(|| {
                Lazy::force(&circuit_minimal);
            }),
            "compute_shuffled_index_minimal".to_string(),
        ),
        Circuits::WeighJustificationAndFinalization => (
            Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }),
            "weigh_justification_and_finalization".to_string(),
        ),
        _ => (
            Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }),
            "weigh_justification_and_finalization".to_string(),
        ),
    }
}

// No need of returning strings as we will be building and executing every circuit anyway
fn build_all_circuits() -> Vec<(Box<dyn Fn() -> () + Send + Sync>, String)> {
    vec![
        (
            Box::new(|| {
                Lazy::force(&circuit_mainnet);
            }),
            "compute_shuffled_index_mainnet".to_string(),
        ),
        (
            Box::new(|| {
                Lazy::force(&circuit_minimal);
            }),
            "compute_shuffled_index_minimal".to_string(),
        ),
        (
            Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }),
            "weigh_justification_and_finalization".to_string(),
        ),
    ]
}

fn build_circuit(circuits: Circuits) -> TypeResult {
    match circuits {
        Circuits::ComputeShuffledIndexMainnet => TypeResult::OneCircuit((
            Box::new(|| {
                Lazy::force(&circuit_mainnet);
            }),
            "compute_shuffled_index_mainnet".to_string(),
        )),
        Circuits::ComputeShuffledIndexMinimal => TypeResult::OneCircuit((
            Box::new(|| {
                Lazy::force(&circuit_minimal);
            }),
            "compute_shuffled_index_minimal".to_string(),
        )),
        Circuits::WeighJustificationAndFinalization => TypeResult::OneCircuit((
            Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }),
            "weigh_justification_and_finalization".to_string(),
        )),
        Circuits::All => TypeResult::AllCircuits(vec![
            (
                Box::new(|| {
                    Lazy::force(&circuit_mainnet);
                }),
                "compute_shuffled_index_mainnet".to_string(),
            ),
            (
                Box::new(|| {
                    Lazy::force(&circuit_minimal);
                }),
                "compute_shuffled_index_minimal".to_string(),
            ),
            (
                Box::new(|| {
                    Lazy::force(&circuit_weigh_justification_and_finalization);
                }),
                "weigh_justification_and_finalization".to_string(),
            ),
        ]),
    }
}

fn main() {
    type L = DefaultParameters;
    const D: usize = 2;
    let builder = CircuitBuilder::<DefaultParameters, D>::new();

    let circuit = builder.build();
    let hint_serializer = HintRegistry::<L, D>::new();
    let gate_serializer = GateRegistry::<L, D>::new();
    let circuit_serializer: CircuitSerializer = CircuitSerializer::parse();

    if circuit_serializer.name != Circuits::All {
        // let my_variable: Box<dyn Fn() -> () + Send + Sync> = Box::new(|| {
        //     Lazy::force(&circuit_minimal);
        // });
        // my_variable = build_circuit(circuit_serializer.name);
        // let mut test: TypeResult = TypeResult::OneCircuit((_, "asd".to_string()));
        // [()] or ()
        let (_, circuit_name) = build_specific_circuit(circuit_serializer.name);
        let path = format!("build/${}", circuit_name);
        circuit.save(&path, &gate_serializer, &hint_serializer);
    } else {
        build_all_circuits();
        for i in 0..Circuits::iter().count() {
            let path = format!("build/${}", circuit_name);
            circuit.save(&path, &gate_serializer, &hint_serializer);
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
