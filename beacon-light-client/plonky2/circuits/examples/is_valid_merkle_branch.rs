use anyhow::Result;
use circuits::is_valid_merkle_branch::is_valid_merkle_branch_sha256_result;
use circuits::sha256::make_circuits;
use circuits::utils::ETH_SHA256_BIT_SIZE;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::println;
use std::time::Instant;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Debug, Deserialize)]
struct RawMerkleProof {
    root: Vec<u64>,
    leaf: Vec<u64>,
    branch: Vec<Vec<u64>>,
    size_bits: Vec<u64>,
    index: u64,
}

#[derive(Debug)]
struct MerkleProof {
    root: Vec<bool>,
    leaf: Vec<bool>,
    branch: Vec<Vec<bool>>,
    size_bits: Vec<bool>,
    index: u64,
}

fn main() -> Result<()> {
    let input_file = File::open("is_valid_merkle_branch_input.json")?;
    let reader = BufReader::new(input_file);
    println!("Read");
    let raw_merkle_proof: RawMerkleProof = serde_json::from_reader(reader)?;
    println!("Parsed");

    let merkle_proof = MerkleProof {
        root: raw_merkle_proof.root.into_iter().map(|s| s == 1).collect(),
        leaf: raw_merkle_proof.leaf.into_iter().map(|s| s == 1).collect(),
        branch: raw_merkle_proof
            .branch
            .into_iter()
            .map(|v| v.into_iter().map(|s| s == 1).collect())
            .collect(),
        size_bits: raw_merkle_proof
            .size_bits
            .into_iter()
            .map(|s| s == 1)
            .collect(),
        index: raw_merkle_proof.index,
    };

    create_proof(merkle_proof)?;

    Ok(())
}

fn create_proof(merkle_proof: MerkleProof) -> std::result::Result<(), anyhow::Error> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = GoldilocksField;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    let hasher = is_valid_merkle_branch_sha256_result(&mut builder, merkle_proof.branch.len());
    let hasher_sha = make_circuits(&mut builder, 512);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(hasher.leaf[i].target, hasher_sha.digest[i].target);
    }

    builder.register_public_inputs(&hasher.root.iter().map(|d| d.target).collect::<Vec<_>>());

    println!("Building circuit");
    let start = Instant::now();
    let data = builder.build::<C>();
    let duration = start.elapsed();

    println!("Duration {:?}", duration);

    println!("Building proof");

    let mut pw = PartialWitness::new();
    pw.set_target(hasher.index, F::from_canonical_u64(merkle_proof.index));

    for i in 0..ETH_SHA256_BIT_SIZE {
        pw.set_bool_target(hasher.root[i], merkle_proof.root[i]);
        pw.set_bool_target(hasher_sha.message[i], merkle_proof.leaf[i]);
        pw.set_bool_target(hasher_sha.message[i + 256], merkle_proof.size_bits[i]);
    }

    for i in 0..merkle_proof.branch.len() {
        for j in 0..ETH_SHA256_BIT_SIZE {
            pw.set_bool_target(hasher.branch[i][j], merkle_proof.branch[i][j]);
        }
    }

    let proof = data.prove(pw)?;

    println!("public inputs {:?}", proof.public_inputs);

    data.verify(proof)?;

    Ok(())
}
