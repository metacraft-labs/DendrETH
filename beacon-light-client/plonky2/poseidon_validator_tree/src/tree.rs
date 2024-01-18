use num_bigint::BigUint;
use plonky2::field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Field};
use plonky2::hash::hash_types::{HashOut, RichField};
use plonky2::hash::hashing::hash_n_to_hash_no_pad;
use plonky2::hash::poseidon::PoseidonPermutation;
use plonky2::plonk::config::GenericHashOut;

#[derive(Clone)]
pub struct Validator {
    pub pubkey: [bool; 384],
    pub withdrawal_credentials: [bool; 256],
    pub effective_balance: BigUint,
    pub slashed: bool,
    pub activation_eligibility_epoch: BigUint,
    pub activation_epoch: BigUint,
    pub exit_epoch: BigUint,
    pub withdrawable_epoch: BigUint,
}

#[derive(Clone, Debug, PartialEq, Eq)] // -> maybe remove
pub enum Positioned<T> {
    /// The value was found in the left branch
    Left(T),

    /// The value was found in the right branch
    Right(T),
}

#[derive(Clone, Debug, PartialEq, Eq)] // -> maybe remove
pub struct Lemma {
    pub node_hash: HashOut<GoldilocksField>,
    pub sibling_hash: Option<Positioned<HashOut<GoldilocksField>>>,
    pub sub_lemma: Option<Box<Lemma>>,
}

pub enum Tree {
    Empty {
        hash: HashOut<GoldilocksField>,
    },

    Leaf {
        hash: HashOut<GoldilocksField>,
        value: Validator,
    },

    Node {
        hash: HashOut<GoldilocksField>,
        left: Box<Tree>,
        right: Box<Tree>,
    },
}

impl Tree {
    /// Create a new tree
    pub fn new_tree<F: RichField + Extendable<D>, const D: usize>(validator: Validator) -> Self {
        Tree::Leaf {
            hash: compute_validator_poseidon_hash_tree_root::<F, D>(validator.clone()),
            value: validator,
        }
    }

    /// Create a new leaf
    pub fn new_leaf<F: RichField + Extendable<D>, const D: usize>(validator: Validator) -> Tree {
        let hash = compute_validator_poseidon_hash_tree_root::<F, D>(validator.clone());
        Tree::new_tree::<F, D>(validator)
    }

    /// Returns a hash from the tree.
    pub fn hash(&self) -> HashOut<GoldilocksField> {
        match *self {
            Tree::Empty { hash } => hash,
            Tree::Leaf { hash, .. } => hash,
            Tree::Node { hash, .. } => hash,
        }
    }
}

impl Validator {
    pub fn new<F: RichField + Extendable<D>, const D: usize>() -> Validator {
        Validator {
            pubkey: [false; 384],
            withdrawal_credentials: [false; 256],
            effective_balance: BigUint::default(),
            slashed: false,
            activation_eligibility_epoch: BigUint::default(),
            activation_epoch: BigUint::default(),
            exit_epoch: BigUint::default(),
            withdrawable_epoch: BigUint::default(),
        }
    }
}

pub struct MerkleTree {
    root: HashOut<GoldilocksField>,
    tree: Tree,
    validator_leaves: Vec<Validator>,
    nodes: Vec<HashOut<GoldilocksField>>,
}

impl MerkleTree {
    /// Constructs a Merkle Tree from a vector of data blocks.
    /// Returns `None` if `values` is empty.
    pub fn from_vec<F: RichField + Extendable<D>, const D: usize>(
        validators: Vec<Validator>,
    ) -> Self {
        if validators.is_empty() {
            return MerkleTree {
                root: hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(
                    &[GoldilocksField::from_bool(false)],
                ),
                tree: Tree::Empty {
                    hash: hash_n_to_hash_no_pad::<
                        GoldilocksField,
                        PoseidonPermutation<GoldilocksField>,
                    >(&[GoldilocksField::from_bool(false)]),
                },
                validator_leaves: validators,
                nodes: vec![hash_n_to_hash_no_pad::<
                    GoldilocksField,
                    PoseidonPermutation<GoldilocksField>,
                >(&[GoldilocksField::from_bool(false)])],
            };
        }

        let mut merkle_tree_height = 0;
        let mut current_merkle_validators_tree = Vec::with_capacity(validators.len());
        for validator in validators.clone() {
            let new_leaf = Tree::new_leaf::<F, D>(validator);
            current_merkle_validators_tree.push(new_leaf);
        }

        while current_merkle_validators_tree.len() > 1 {
            let mut next_level = Vec::new();
            while !current_merkle_validators_tree.is_empty() {
                if current_merkle_validators_tree.len() == 1 {
                    next_level.push(current_merkle_validators_tree.remove(0));
                } else {
                    let left = current_merkle_validators_tree.remove(0);
                    let right = current_merkle_validators_tree.remove(0);

                    let left_node_in_goldilocks: Vec<GoldilocksField> = left
                        .hash()
                        .to_bytes()
                        .iter()
                        .map(|&x| GoldilocksField::from_canonical_u8(x))
                        .collect();

                    let right_node_in_goldilocks: Vec<GoldilocksField> = right
                        .hash()
                        .to_bytes()
                        .iter()
                        .map(|&x| GoldilocksField::from_canonical_u8(x))
                        .collect();

                    let combined_nodes: Vec<GoldilocksField> = left_node_in_goldilocks
                        .into_iter()
                        .chain(right_node_in_goldilocks.into_iter())
                        .collect();

                    let combined_nodes_hash = hash_n_to_hash_no_pad::<
                        GoldilocksField,
                        PoseidonPermutation<GoldilocksField>,
                    >(&combined_nodes);

                    let node = Tree::Node {
                        hash: combined_nodes_hash,
                        left: Box::new(left),
                        right: Box::new(right),
                    };

                    next_level.push(node);
                }
            }

            merkle_tree_height += 1;

            current_merkle_validators_tree = next_level;
        }

        assert!(current_merkle_validators_tree.len() == 1);

        let root = current_merkle_validators_tree.remove(0);

        MerkleTree {
            root: compute_validator_poseidon_hash_tree_root::<F, D>(validators[0].clone()),
            tree: root,
            validator_leaves: validators.clone(),
            nodes: return_every_validator_hash_tree_root::<F, D>(validators),
        }
    }

    pub fn root_hash(&self) -> HashOut<GoldilocksField> {
        self.root
    }

    pub fn generate_validator_proof<F: RichField + Extendable<D>, const D: usize>(
        &self,
        validator: Validator,
    ) {
        let root_hash = self.root_hash();
        let validator_hash: HashOut<GoldilocksField> =
            compute_validator_poseidon_hash_tree_root::<F, D>(validator);

        Self::new::<F, D>(&self.tree, validator_hash);
    }

    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        tree: &Tree,
        validator_hash: HashOut<GoldilocksField>,
    ) -> Option<Lemma> {
        match *tree {
            Tree::Empty { .. } => None,

            Tree::Leaf { hash, .. } => Self::new_leaf_proof(hash, validator_hash),

            Tree::Node {
                hash,
                ref left,
                ref right,
            } => Self::new_tree_proof::<F, D>(hash, validator_hash, left, right),
        }
    }

    fn new_tree_proof<F: RichField + Extendable<D>, const D: usize>(
        hash: HashOut<GoldilocksField>,
        validator_hash: HashOut<GoldilocksField>,
        left: &Tree,
        right: &Tree,
    ) -> Option<Lemma> {
        Self::new::<F, D>(left, validator_hash)
            .map(|lemma| {
                let right_hash = right.hash().clone();
                let sub_lemma = Some(Positioned::Right(right_hash));
                (lemma, sub_lemma)
            })
            .or_else(|| {
                let sub_lemma = Self::new::<F, D>(right, validator_hash);
                sub_lemma.map(|lemma| {
                    let left_hash = left.hash().clone();
                    let sub_lemma = Some(Positioned::Left(left_hash));
                    (lemma, sub_lemma)
                })
            })
            .map(|(sub_lemma, sibling_hash)| Lemma {
                node_hash: hash,
                sibling_hash,
                sub_lemma: Some(Box::new(sub_lemma)),
            })
    }

    fn new_leaf_proof(
        leaf_hash: HashOut<GoldilocksField>,
        validator_hash: HashOut<GoldilocksField>,
    ) -> Option<Lemma> {
        if leaf_hash.eq(&validator_hash) {
            Some(Lemma {
                node_hash: leaf_hash,
                sibling_hash: None,
                sub_lemma: None,
            })
        } else {
            None
        }
    }

    pub fn generate_validator_proof_by_index() {}
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

fn compute_poseidon_hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
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

pub fn compute_validator_poseidon_hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
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
    let hash_tree_root_poseidon =
        compute_poseidon_hash_tree_root::<F, D>(leaves.len(), leaves.clone());

    println!("hash_tree_root is: {:?}", hash_tree_root_poseidon);

    hash_tree_root_poseidon
}

pub fn return_every_validator_hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
    validators: Vec<Validator>,
) -> Vec<HashOut<GoldilocksField>> {
    let mut validators_root_hashes = Vec::new();
    for i in 0..validators.len() {
        validators_root_hashes.push(compute_validator_poseidon_hash_tree_root::<F, D>(
            validators[i].clone(),
        ))
    }

    validators_root_hashes
}

// function verify(

// bytes32[] memory proof, -> the array of all the hashes that are needed to compute the merkle root
// bytes32 root -> the merkle root itself
// bytes32 leaf -> the hash of the element in the array that was used to construct the merkle tree
// uint index -> the index in the array where the element is stored
// )

// MerkleTree
// pub fn return_specific_validator_hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
//     validators: Vec<Validator>,
//     index: usize,
// ) {
//     let validators_hashes_hashes =
//         return_every_validator_hash_tree_root::<F, D>(validators.clone());
//     let validators_merkle_tree = MerkleTree::from_vec::<F, D>(validators);
//     // let specific_validdator =
// }

// TODO: create compute_hash_tree_root_validators -> accepting Vec<Validator> -> DONE = return_every_validator_hash_tree_root

// MerkleTree
// compute_hash_tree_root -> of validators
// get_merkle_proof(0) -> merke proof

//      R
//    /  \
//    h1  h2
//   / \ / \
//  0  1 2  3
//  |  |
// v1 v2
