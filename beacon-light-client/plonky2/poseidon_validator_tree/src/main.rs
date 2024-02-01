use num_bigint::BigUint;
use plonky2::{field::goldilocks_field::GoldilocksField};
use poseidon_validator_tree::{
    objects::Validator, parse_validators::read_validator_data, tree::{compute_validators_merkle_proof, return_every_validator_hash, MerkleTree}
};

pub fn main() {

    const D: usize = 2;
    type F = GoldilocksField;

    let file_path_attestations = 
                "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/full_validator_set.json";

    let validators_raw: Vec<Validator> = read_validator_data(file_path_attestations);

    let validators_hashed = return_every_validator_hash::<F,D>(validators_raw);
    let merkle_tree = MerkleTree::new::<F, D>(&validators_hashed, 40);

    const DEPTH: usize = 40;

    let mut proofs = vec![];

    for i in 0..validators_hashed.len() {
        proofs.push(merkle_tree.generate_proof::<F, D>(i, DEPTH).unwrap());

        if i%1000 == 0 {
            println!("{}",i);
        }
    }
    println!("{:?}", proofs.len());

}
