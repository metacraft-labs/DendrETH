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

impl Circuits {
    fn from_str(circuit_as_str: &str) -> Option<Circuits> {
        match circuit_as_str {
            "compute_shuffled_index_mainnet" => Some(Circuits::compute_shuffled_index_mainnet),
            "compute_shuffled_index_minimal" => Some(Circuits::compute_shuffled_index_minimal),
            "weigh_justification_and_finalization" => {
                Some(Circuits::weigh_justification_and_finalization)
            }
            _ => None,
        }
    }
}

enum OneOrAllCircuits {
    OneCircuit(Box<dyn Fn() -> () + Send + Sync>),
    AllCircuits(Vec<Box<dyn Fn() -> () + Send + Sync>>),
}

#[derive(Parser, Debug)]
struct CommandLineCircuit {
    /// Enter name of circuit
    #[clap(value_delimiter = ' ', num_args = 0..)]
    name: Circuits,
}

fn build_circuit(circuit: Circuits) -> OneOrAllCircuits {
    match circuit {
        Circuits::compute_shuffled_index_mainnet => OneOrAllCircuits::OneCircuit(Box::new(|| {
            Lazy::force(&circuit_mainnet);
        })),
        Circuits::compute_shuffled_index_minimal => OneOrAllCircuits::OneCircuit(Box::new(|| {
            Lazy::force(&circuit_minimal);
        })),
        Circuits::weigh_justification_and_finalization => {
            OneOrAllCircuits::OneCircuit(Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }))
        }
        Circuits::all => OneOrAllCircuits::AllCircuits(vec![
            Box::new(|| {
                Lazy::force(&circuit_mainnet);
            }),
            Box::new(|| {
                Lazy::force(&circuit_minimal);
            }),
            Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }),
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
    let command_line_circuit: CommandLineCircuit = CommandLineCircuit::parse();
    let command_line_arguments: Vec<String> = args().skip(1).collect();

    if command_line_circuit.name != Circuits::all {
        for (_, arg) in command_line_arguments.iter().enumerate() {
            let arg_as_circuit = Circuits::from_str(&arg);
            build_circuit(arg_as_circuit.unwrap());
            let path = format!("build/{}", arg_as_circuit.unwrap().to_string());
            circuit.save(&path, &gate_serializer, &hint_serializer);
        }
    } else {
        for _circuit in Circuits::iter() {
            if _circuit != Circuits::all {
                build_circuit(_circuit);
                let path = format!("build/{}", _circuit.to_string());
                circuit.save(&path, &gate_serializer, &hint_serializer);
            }
        }
    }
}
