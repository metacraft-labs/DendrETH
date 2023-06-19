use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use std::fs::File;
use std::io::BufReader;
mod is_valid_merkle_branch;
use is_valid_merkle_branch::is_valid_merkle_branch;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawMerkleProof {
    root: Vec<String>,
    leaf: Vec<String>,
    branch: Vec<Vec<String>>,
    index: u64,
}

#[derive(Debug)]
struct MerkleProof {
    root: Vec<bool>,
    leaf: Vec<bool>,
    branch: Vec<Vec<bool>>,
    index: u64,
}

fn main() -> Result<()> {
    let input_file = File::open("input.json")?;
    let reader = BufReader::new(input_file);
    let raw_merkle_proof: RawMerkleProof = serde_json::from_reader(reader)?;

    let merkle_proof = MerkleProof {
        root: raw_merkle_proof
            .root
            .into_iter()
            .map(|s| s == "1")
            .collect(),
        leaf: raw_merkle_proof
            .leaf
            .into_iter()
            .map(|s| s == "1")
            .collect(),
        branch: raw_merkle_proof
            .branch
            .into_iter()
            .map(|v| v.into_iter().map(|s| s == "1").collect())
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
    let mut pw = PartialWitness::new();

    let hasher = is_valid_merkle_branch(&mut builder, merkle_proof.branch.len());

    pw.set_target(hasher.index, F::from_canonical_u64(merkle_proof.index));

    for i in 0..256 {
        pw.set_bool_target(hasher.leaf[i], merkle_proof.leaf[i]);
        pw.set_bool_target(hasher.root[i], merkle_proof.root[i]);
    }

    for i in 0..merkle_proof.branch.len() {
        for j in 0..256 {
            pw.set_bool_target(hasher.branch[i][j], merkle_proof.branch[i][j]);
        }
    }

    let data = builder.build::<C>();
    let proof = data.prove(pw).unwrap();

    let res = data.verify(proof);

    res
}
