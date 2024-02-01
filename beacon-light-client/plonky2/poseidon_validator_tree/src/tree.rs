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

pub const ZERO_HASHES: [&str; 64] = [
    "0000000000000000000000000000000000000000000000000000000000000000",
    "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
    "db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
    "c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
    "536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c",
    "9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30",
    "d88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1",
    "87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c",
    "26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193",
    "506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1",
    "ffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b",
    "6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220",
    "b7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f",
    "df6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e",
    "b58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784",
    "d49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb",
    "8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb",
    "8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab",
    "95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4",
    "f893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f",
    "cddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa",
    "8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c",
    "feb3c337d7a51a6fbf00b9e34c52e1c9195c969bd4e7a0bfd51d5c5bed9c1167",
    "e71f0aa83cc32edfbefa9f4d3e0174ca85182eec9f3a09f6a6c0df6377a510d7",
    "31206fa80a50bb6abe29085058f16212212a60eec8f049fecb92d8c8e0a84bc0",
    "21352bfecbeddde993839f614c3dac0a3ee37543f9b412b16199dc158e23b544",
    "619e312724bb6d7c3153ed9de791d764a366b389af13c58bf8a8d90481a46765",
    "7cdd2986268250628d0c10e385c58c6191e6fbe05191bcc04f133f2cea72c1c4",
    "848930bd7ba8cac54661072113fb278869e07bb8587f91392933374d017bcbe1",
    "8869ff2c22b28cc10510d9853292803328be4fb0e80495e8bb8d271f5b889636",
    "b5fe28e79f1b850f8658246ce9b6a1e7b49fc06db7143e8fe0b4f2b0c5523a5c",
    "985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7",
    "c6f67e02e6e4e1bdefb994c6098953f34636ba2b6ca20a4721d2b26a886722ff",
    "1c9a7e5ff1cf48b4ad1582d3f4e4a1004f3b20d8c5a2b71387a4254ad933ebc5",
    "2f075ae229646b6f6aed19a5e372cf295081401eb893ff599b3f9acc0c0d3e7d",
    "328921deb59612076801e8cd61592107b5c67c79b846595cc6320c395b46362c",
    "bfb909fdb236ad2411b4e4883810a074b840464689986c3f8a8091827e17c327",
    "55d8fb3687ba3ba49f342c77f5a1f89bec83d811446e1a467139213d640b6a74",
    "f7210d4f8e7e1039790e7bf4efa207555a10a6db1dd4b95da313aaa88b88fe76",
    "ad21b516cbc645ffe34ab5de1c8aef8cd4e7f8d2b51e8e1456adc7563cda206f",
    "6bfe8d2bcc4237b74a5047058ef455339ecd7360cb63bfbb8ee5448e6430ba04",
    "a7f23ce9181740dc220c814782654fee6aceb9f1ec9222c4e2467d0ab1680837",
    "aef9476c89590a2c8cc9b3b74f4967c757c49d9866a44bacf21fa2ed675ddfa2",
    "9a42bcad82f6a9e41284d808ead319f29f3b08209d680f0e2ce71510d071e205",
    "d1a66d354a67b9cf179571d8e5f97792716e8dd4ec44196839a3f7c6b74f8bac",
    "fafa3025f2f89509c2c71c74fba0cd92858ef49b0780fb5479746c8a9bfcb346",
    "3334a7c1e7f6705aa6011a6a949645016db4acde0ca9abd66dc79d8266423056",
    "0796fd75664faef744ee4e52d7271e2bbb769f91ed6f9b74d8b694f56606852c",
    "7ba3ae4a417fe8545b142bc89f4adcd7ae13941cbab7750b83e9f0a66d16be64",
    "788fafcc4aa520399adbaed195f8b12c4eb31ec10168e50aabc659a6aea516dc",
    "e833d7a67160e68bf4c9044a53077df2727ad00cf36f4949c7b681a912140cbb",
    "309eabf095dc6714f9f4d864bba5affae0b35ae2f5e3565bcc3a47b212767701",
    "226a8ebefa288665a644a50273335efbb610510f241b5b720c8a368d59a69a5d",
    "41abfd995425827625938131af0c4f33fe0bd4688c222c21fa9da8e89caa03f8",
    "442c642ef50fa1a667a6e6d105c77c5cc3fec8d7aa2570cf1a3077b503c38069",
    "a0a08dfc9b42d96c2de19b6d127b8ae136ddcf3e5ad0dce422c45a56f61f6a74",
    "7d348382af096dbe0bf086c7bb39b2a2c0bc36b621ab0c738e9885d731d81740",
    "3ab134751d191269026c86994eaa8b43a83b4ad1f6d0e77381c4e2974afbc8f6",
    "9a7452611db2d23eae26f9bdbb88958ef44c64d0fe987be9f726adf938f50f6c",
    "725c7f816037bfe452cd1e7ba35ac47edcb49a9a2b27aeca70dce483cb7ded1f",
    "2cea1af51fb28b62887c39998ac9fef4dfdeda1f07e071ba558a173afd06cbc3",
    "ff1d59f98b6c551d95089357057d5c8be26402279e9df0b1df1a10b72bf3927f",
    "2f8a181f7c99dd215a7529bfe296a9603a1446737186d21aeb8bc7ae59e1fd21",
    "ecc502c9b1145f3950cb7d3e3842446f81a4f0df1df537cee139ef64ea984bd9",
];


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
