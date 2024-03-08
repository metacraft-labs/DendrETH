use lazy_static::lazy_static;
use num_bigint::BigUint;
use plonky2::field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Field};
use plonky2::hash::hash_types::{HashOut, RichField};
use plonky2::hash::hashing::hash_n_to_hash_no_pad;
use plonky2::hash::poseidon::PoseidonPermutation;
use plonky2::plonk::config::GenericHashOut;

use crate::objects::Validator;

lazy_static! {
    static ref SYNTHETIC_ZERO_NODES: Vec<MerkleTree> = {
        const MAX_TREE_DEPTH: usize = 32;
        (0..=MAX_TREE_DEPTH).map(MerkleTree::Zero).collect()
    };
}

#[derive(Debug, PartialEq)]
pub enum MerkleTree {
    Finalized(HashOut<GoldilocksField>),
    Leaf(HashOut<GoldilocksField>),
    Node(HashOut<GoldilocksField>, Box<Self>, Box<Self>),
    Zero(usize),
}

impl MerkleTree {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        leaves: &[HashOut<GoldilocksField>],
        depth: usize,
    ) -> Self {
        if leaves.is_empty() {
            return MerkleTree::Zero(depth);
        }

        match depth {
            0 => {
                debug_assert_eq!(leaves.len(), 1);
                MerkleTree::Leaf(leaves[0])
            }
            _ => {
                const EMPTY_SLICE: &[HashOut<GoldilocksField>] = &[];
                let subtree_capacity = 2usize.pow(depth as u32 - 1);
                let (left_leaves, right_leaves) = if leaves.len() <= subtree_capacity {
                    (leaves, EMPTY_SLICE)
                } else {
                    leaves.split_at(subtree_capacity)
                };

                let left_subtree = MerkleTree::new::<F, D>(left_leaves, depth - 1);
                let right_subtree = MerkleTree::new::<F, D>(right_leaves, depth - 1);
                let hash = combine_two_hash_n_to_hash_no_pad::<F, D>(
                    left_subtree.hash::<F, D>(),
                    right_subtree.hash::<F, D>(),
                );

                MerkleTree::Node(hash, Box::new(left_subtree), Box::new(right_subtree))
            }
        }
    }

    pub fn generate_proof<F: RichField + Extendable<D>, const D: usize>(
        &self,
        index: usize,
        depth: usize,
    ) -> Result<(HashOut<GoldilocksField>, Vec<HashOut<GoldilocksField>>), MerkleTreeError> {
        let mut proof = vec![];
        let mut current_node = self;
        let mut current_depth = depth;
        while current_depth > 0 {
            let ith_bit = (index >> (current_depth - 1)) & 0x01;
            if let &MerkleTree::Finalized(_) = current_node {
                return Err(MerkleTreeError::ProofEncounteredFinalizedNode);
            }
            let (left, right) = current_node.left_and_right_branches().unwrap();

            if ith_bit == 1 {
                proof.push(left.hash::<F, D>());
                current_node = right;
            } else {
                proof.push(right.hash::<F, D>());
                current_node = left;
            }
            current_depth -= 1;
        }

        debug_assert_eq!(proof.len(), depth);
        debug_assert!(current_node.is_leaf());

        proof.reverse();

        Ok((current_node.hash::<F, D>(), proof))
    }

    fn left_and_right_branches(&self) -> Option<(&Self, &Self)> {
        match *self {
            MerkleTree::Finalized(_) | MerkleTree::Leaf(_) | MerkleTree::Zero(0) => None,
            MerkleTree::Node(_, ref l, ref r) => Some((l, r)),
            MerkleTree::Zero(depth) => Some((
                &SYNTHETIC_ZERO_NODES[depth - 1],
                &SYNTHETIC_ZERO_NODES[depth - 1],
            )),
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, MerkleTree::Leaf(_))
    }

    fn zero_hashes<F: RichField + Extendable<D>, const D: usize>() -> Vec<HashOut<GoldilocksField>>
    {
        pub const ZERO_HASHES_MAX_INDEX: usize = 48;
        let mut hashes = vec![
            hash_n_to_hash_no_pad::<
                GoldilocksField,
                PoseidonPermutation<GoldilocksField>,
            >(&[GoldilocksField::from_canonical_u8(0); 32]);
            ZERO_HASHES_MAX_INDEX + 1
        ];

        for i in 0..ZERO_HASHES_MAX_INDEX {
            hashes[i + 1] = combine_two_hash_n_to_hash_no_pad::<F, D>(hashes[i], hashes[i]);
        }

        hashes
    }

    fn hash<F: RichField + Extendable<D>, const D: usize>(&self) -> HashOut<GoldilocksField> {
        match *self {
            MerkleTree::Finalized(h) => h,
            MerkleTree::Leaf(h) => h,
            MerkleTree::Node(h, _, _) => h,
            MerkleTree::Zero(depth) => Self::zero_hashes::<F, D>()[depth],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MerkleTreeError {
    ProofEncounteredFinalizedNode,
}

pub fn compute_validators_merkle_proof<F: RichField + Extendable<D>, const D: usize>(
    validators: Vec<Validator>,
    index: usize,
    depth: usize,
) -> Result<Vec<HashOut<GoldilocksField>>, MerkleTreeError> {
    let validators_hashes = return_every_validator_hash::<F, D>(validators);
    let merkle_tree = MerkleTree::new::<F, D>(&validators_hashes, depth);

    let proof = merkle_tree.generate_proof::<F, D>(index, depth);
    let proof = match proof {
        Ok((_, path)) => path,
        Err(err) => {
            eprintln!("Error: {:?}", err);

            return Err(MerkleTreeError::ProofEncounteredFinalizedNode);
        }
    };

    Ok(proof)
}

fn combine_two_hash_n_to_hash_no_pad<F: RichField + Extendable<D>, const D: usize>(
    left: HashOut<GoldilocksField>,
    right: HashOut<GoldilocksField>,
) -> HashOut<GoldilocksField> {
    let left_node_in_goldilocks: Vec<GoldilocksField> = left
        .to_bytes()
        .iter()
        .map(|&x| GoldilocksField::from_canonical_u8(x))
        .collect();

    let right_node_in_goldilocks: Vec<GoldilocksField> = right
        .to_bytes()
        .iter()
        .map(|&x| GoldilocksField::from_canonical_u8(x))
        .collect();

    let combined_nodes: Vec<GoldilocksField> = left_node_in_goldilocks
        .into_iter()
        .chain(right_node_in_goldilocks.into_iter())
        .collect();

    hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&combined_nodes)
}

fn hash_bits_arr_in_goldilocks_to_hash_no_pad<F: RichField + Extendable<D>, const D: usize>(
    validator_data: &[bool],
) -> HashOut<GoldilocksField> {
    let validator_data_in_goldilocks: Vec<GoldilocksField> = validator_data
        .iter()
        .map(|x| GoldilocksField::from_bool(*x))
        .collect();

    hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(
        validator_data_in_goldilocks.as_slice(),
    )
}

fn hash_biguint_in_goldilocks_to_hash_no_pad<F: RichField + Extendable<D>, const D: usize>(
    validator_data: BigUint,
) -> HashOut<GoldilocksField> {
    let mut validator_data_in_goldilocks = validator_data.to_u32_digits();
    assert!(validator_data_in_goldilocks.len() <= 2);
    validator_data_in_goldilocks.resize(2, 0);
    hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
        GoldilocksField::from_canonical_u32(validator_data_in_goldilocks[0]),
        GoldilocksField::from_canonical_u32(validator_data_in_goldilocks[1]),
    ])
}

pub fn compute_poseidon_hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
    leaves_len: usize,
    leaves: Vec<HashOut<GoldilocksField>>,
) -> HashOut<GoldilocksField> {
    let mut hashers: Vec<HashOut<GoldilocksField>> = Vec::new();
    for i in 0..(leaves_len / 2) {
        let goldilocks_leaves = leaves[i * 2]
            .elements
            .iter()
            .copied()
            .chain(leaves[i * 2 + 1].elements.iter().copied())
            .into_iter();
        let goldilocks_leaves_collected: Vec<GoldilocksField> = goldilocks_leaves.collect();
        hashers.push(hash_n_to_hash_no_pad::<
            GoldilocksField,
            PoseidonPermutation<GoldilocksField>,
        >(&goldilocks_leaves_collected));
    }

    let mut k = 0;
    for _ in leaves_len / 2..leaves_len - 1 {
        let goldilocks_leaves = hashers[k * 2]
            .elements
            .iter()
            .copied()
            .chain(hashers[k * 2 + 1].elements.iter().copied());
        let goldilocks_leaves_collected: Vec<GoldilocksField> = goldilocks_leaves.collect();
        hashers.push(hash_n_to_hash_no_pad::<
            GoldilocksField,
            PoseidonPermutation<GoldilocksField>,
        >(&goldilocks_leaves_collected));

        k += 1;
    }

    hashers[leaves_len - 2]
}

pub fn compute_validator_poseidon_hash<F: RichField + Extendable<D>, const D: usize>(
    validator: Validator,
) -> HashOut<GoldilocksField> {
    let leaves = vec![
        hash_bits_arr_in_goldilocks_to_hash_no_pad::<F, D>(&validator.pubkey),
        hash_bits_arr_in_goldilocks_to_hash_no_pad::<F, D>(&validator.withdrawal_credentials),
        hash_biguint_in_goldilocks_to_hash_no_pad::<F, D>(validator.effective_balance.clone()),
        hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
            GoldilocksField::from_bool(validator.slashed),
        ]),
        hash_biguint_in_goldilocks_to_hash_no_pad::<F, D>(
            validator.activation_eligibility_epoch.clone(),
        ),
        hash_biguint_in_goldilocks_to_hash_no_pad::<F, D>(validator.activation_epoch.clone()),
        hash_biguint_in_goldilocks_to_hash_no_pad::<F, D>(validator.exit_epoch.clone()),
        hash_biguint_in_goldilocks_to_hash_no_pad::<F, D>(validator.withdrawable_epoch.clone()),
    ];
    let poseidon_hash_tree_root =
        compute_poseidon_hash_tree_root::<F, D>(leaves.len(), leaves.clone());

    poseidon_hash_tree_root
}

pub fn return_every_validator_hash<F: RichField + Extendable<D>, const D: usize>(
    validators: Vec<Validator>,
) -> Vec<HashOut<GoldilocksField>> {
    let mut validators_root_hashes = Vec::new();
    for i in 0..validators.len() {
        validators_root_hashes.push(compute_validator_poseidon_hash::<F, D>(
            validators[i].clone(),
        ))
    }

    validators_root_hashes
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use plonky2::field::goldilocks_field::GoldilocksField;

    use crate::tree::{compute_validators_merkle_proof, Validator};

    #[test]
    fn batch_test() {
        fn main() {
            const D: usize = 2;
            type F = GoldilocksField;
        
            let validator = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000000 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator1 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000001 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator2 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000002 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator3 = Validator {
                trusted: true,
                validator_index: 1,
                
                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000003 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator4 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000004 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator5 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000005 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator6 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000006 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator7 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000007 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator8 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000008 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let validator9 = Validator {
                trusted: true,
                validator_index: 1,

                pubkey: [
                    true, false, true, true, true, false, false, false, false, false, true, false, false,
                    true, false, false, true, false, true, true, false, true, false, true, true, true,
                    true, false, true, true, false, true, true, true, true, false, false, false, true,
                    true, false, false, true, true, true, false, true, false, false, true, true, true,
                    true, false, true, true, false, false, false, false, false, true, false, true, true,
                    false, true, false, false, false, true, true, false, true, true, true, true, false,
                    false, false, true, false, true, false, true, false, false, false, false, true, false,
                    false, true, false, true, true, false, false, false, true, true, false, false, false,
                    false, false, true, true, true, false, true, true, false, true, false, false, true,
                    false, true, true, true, true, false, false, false, true, true, true, true, true, true,
                    false, false, true, true, true, true, true, false, true, true, false, true, true, true,
                    false, false, false, true, false, false, true, false, true, false, false, true, true,
                    false, false, true, true, true, false, false, true, false, false, true, false, false,
                    false, true, false, true, true, false, true, true, false, false, true, false, true,
                    true, false, false, true, true, true, true, true, false, false, false, true, false,
                    true, false, true, false, false, false, false, true, true, false, false, true, false,
                    false, true, false, true, true, true, true, true, false, true, false, false, true,
                    true, false, true, false, true, true, false, false, true, true, true, true, false,
                    true, true, false, true, true, true, false, false, true, false, true, false, false,
                    false, true, false, true, true, true, true, false, true, false, true, false, false,
                    false, false, false, false, true, true, false, false, false, false, true, false, false,
                    false, false, false, false, true, true, true, false, true, false, true, true, false,
                    false, true, true, false, false, true, true, true, true, false, false, false, true,
                    true, false, true, false, false, false, true, false, false, true, false, false, false,
                    false, false, false, false, false, false, true, true, true, true, false, true, false,
                    true, true, false, false, false, false, false, true, false, true, true, true, false,
                    true, true, false, false, false, true, true, true, true, true, false, false, true,
                    false, true, false, true, true, true, true, true, false, true, false, true, false,
                    false, true, true, true, false, false, true, false, true, false, true, true, true,
                    true, false, false, true, false,
                ],
                withdrawal_credentials: [
                    false, false, false, false, false, false, false, false, false, true, false, true,
                    false, false, true, false, true, true, true, true, true, true, true, false, true, true,
                    false, true, false, false, true, false, false, true, true, true, true, false, true,
                    true, true, false, true, false, true, true, false, true, true, false, true, true, true,
                    true, false, true, true, false, true, true, false, true, true, false, false, true,
                    true, true, true, true, false, true, false, true, false, true, true, true, false,
                    false, true, false, true, false, true, true, false, true, false, true, true, false,
                    true, false, true, false, false, true, false, true, true, false, false, true, false,
                    false, true, false, true, false, false, true, true, false, true, false, true, true,
                    false, false, false, true, false, false, false, true, false, false, true, false, true,
                    false, true, false, true, true, true, true, false, true, false, false, false, false,
                    false, false, true, true, false, true, false, false, false, true, true, false, true,
                    true, true, true, false, true, true, true, false, true, true, false, true, true, true,
                    true, false, false, false, false, false, false, false, false, false, true, true, false,
                    false, false, false, true, false, false, false, false, false, false, false, false,
                    false, false, true, false, true, false, true, false, true, false, true, true, false,
                    true, true, true, true, true, false, true, true, true, true, true, false, true, true,
                    true, false, true, false, true, false, true, false, true, false, true, false, false,
                    true, false, false, false, true, true, false, false, true, true, true, true, false,
                    false, true, true, false, false, true, true,
                ],
                effective_balance: BigUint::from(32000000009 as u64),
                slashed: false,
                activation_eligibility_epoch: BigUint::from(0 as u8),
                activation_epoch: BigUint::from(0 as u8),
                exit_epoch: BigUint::from(18446744073709551615 as u64),
                withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
            };
        
            let _even_validators = vec![
                validator.clone(),
                validator1.clone(),
                validator2.clone(),
                validator3.clone(),
            ];
        
            let _odd_validators = vec![
                validator.clone(),
                validator1.clone(),
                validator2.clone(),
                validator3.clone(),
                validator4.clone(),
            ];
        
            let _even_validators_not_power_of_2 = vec![
                validator.clone(),
                validator1.clone(),
                validator2.clone(),
                validator3.clone(),
                validator4.clone(),
                validator5.clone(),
                validator6.clone(),
                validator7.clone(),
                validator8.clone(),
                validator9.clone(),
            ];
        
            let even_zero_index_depth_two_path =
                compute_validators_merkle_proof::<F, D>(_even_validators, 0, 2);
            println!(
                "zero_index_depth_two_path is: {:?}",
                even_zero_index_depth_two_path.unwrap()
            );
        
            println!("||||||||||||||||||||||||||||||||||||||||");
        
            let odd_zero_index_depth_three_path =
                compute_validators_merkle_proof::<F, D>(_odd_validators, 0, 3);
            println!(
                "odd_zero_index_depth_three_path is: {:?}",
                odd_zero_index_depth_three_path.unwrap()
            );
        
            println!("||||||||||||||||||||||||||||||||||||||||");
        
            let even_not_power_2_zero_index_depth_four_path =
                compute_validators_merkle_proof::<F, D>(_even_validators_not_power_of_2, 0, 4);
            println!(
                "even_not_power_2_zero_index_depth_four_path is: {:?}",
                even_not_power_2_zero_index_depth_four_path.unwrap()
            );
        }
        
    }
}
