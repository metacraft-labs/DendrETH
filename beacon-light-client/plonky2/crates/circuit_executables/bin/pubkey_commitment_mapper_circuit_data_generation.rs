#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit_executables::{
    cached_circuit_build::serialize_recursive_circuit, utils::CommandLineOptionsBuilder,
};
use circuits::pubkey_commitment_mapper::{
    first_level::PubkeyCommitmentMapperFL, inner_level::PubkeyCommitmentMapperIL,
};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "pubkey_commitment_mapper";
const RECURSION_DEPTH: usize = 32;

fn main() {
    let matches =
        CommandLineOptionsBuilder::new("pubkey_commitment_mapper_circuit_data_generation")
            .with_serialized_circuits_dir()
            .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    serialize_recursive_circuit::<PubkeyCommitmentMapperFL, PubkeyCommitmentMapperIL>(
        CIRCUIT_NAME,
        serialized_circuits_dir,
        RECURSION_DEPTH,
        &(),
    );
}
