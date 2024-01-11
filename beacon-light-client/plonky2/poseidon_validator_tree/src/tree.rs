use num_bigint::BigUint;
use plonky2::field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Field};
use plonky2::hash::hash_types::{HashOut, RichField};
use plonky2::hash::hashing::hash_n_to_hash_no_pad;
use plonky2::hash::poseidon::PoseidonPermutation;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use circuits::hash_tree_root_poseidon::hash_tree_root_poseidon;

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

fn hash_validator_data_in_goldilocks_to_hash_no_pad<
    F: RichField + Extendable<D>,
    const D: usize,
>(
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

pub fn do_something<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) {
    let validator = Validator::new::<F, D>();

    // let limbs = validator.activation_eligibility_epoch.to_u32_digits();
    // limbs.resize(2, 0);

    let leaves = vec![
        hash_validator_data_in_goldilocks_to_hash_no_pad::<F, D>(&validator.pubkey),
        hash_validator_data_in_goldilocks_to_hash_no_pad::<F, D>(&validator.withdrawal_credentials),
        // hash_validator_data_in_goldilocks_to_hash_no_pad::<F, D>(&validator.effective_balance),
        // hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
        //     GoldilocksField::from_bool(validator.slashed),
        // ]),
        // hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(
        //     GoldilocksField::from_canonical_u32(limbs[0]),
        //     GoldilocksField::from_canonical_u32(limbs[1]),
        // ),
        // hash_validator_data_in_goldilocks_to_hash_no_pad::<F, D>(&validator.activation_epoch),
        // hash_validator_data_in_goldilocks_to_hash_no_pad::<F, D>(&validator.exit_epoch),
        // hash_validator_data_in_goldilocks_to_hash_no_pad::<F, D>(&validator.withdrawable_epoch),
    ];

    let hash_tree_root_poseidon = hash_tree_root_poseidon(builder, leaves.len());

    // for i in 0..leaves.len() {
    //     builder.connect_hashes(leaves[i], hash_tree_root_poseidon.leaves[i]);
    // }

    // ValidatorPoseidonHashTreeRootTargets {
    //     validator,
    //     hash_tree_root: hash_tree_root_poseidon.hash_tree_root,
    // }

    println!("leaves = {:?}", leaves);
}
