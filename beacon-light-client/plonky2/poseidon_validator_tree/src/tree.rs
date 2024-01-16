use num_bigint::BigUint;
use plonky2::field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Field};
use plonky2::hash::hash_types::{HashOut, RichField};
use plonky2::hash::hashing::hash_n_to_hash_no_pad;
use plonky2::hash::poseidon::PoseidonPermutation;

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
