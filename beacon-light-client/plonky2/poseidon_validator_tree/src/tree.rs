use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field, extension::Extendable},
    hash::{
        hashing::hash_n_to_hash_no_pad,
        poseidon::{self, PoseidonHash, PoseidonPermutation}, hash_types::RichField,
    },
};
use plonky2::prelude::{CircuitBuilder, DefaultParameters};
use plonky2::iop::target::BoolTarget;
use plonky2x::frontend::uint::uint64::U64Variable;
use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},}

pub struct Validator {
    pub pubkey: [BoolTarget; 384],
    pub withdrawal_credentials: [BoolTarget; 256], // Change to Bytes32Variable?
    pub effective_balance: U64Variable,
    pub slashed: BoolTarget,
    pub activation_eligibility_epoch: U64Variable,
    pub activation_epoch: U64Variable,
    pub exit_epoch: U64Variable,
    pub withdrawable_epoch: U64Variable,
}

impl Validator {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Validator {
        Validator {
            pubkey: builder.add_virtual_biguint_target(12),
            withdrawal_credentials: builder.add_virtual_biguint_target(8),
            effective_balance: builder.add_virtual_biguint_target(2),
            slashed: builder.add_virtual_bool_target_safe(),
            activation_eligibility_epoch: builder.add_virtual_biguint_target(2),
            activation_epoch: builder.add_virtual_biguint_target(2),
            exit_epoch: builder.add_virtual_biguint_target(2),
            withdrawable_epoch: builder.add_virtual_biguint_target(2),
        }
    }
}

pub fn do_something() {
    // let r = hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
    //     GoldilocksField::from_canonical_u64(123),
    // ]);

    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();

    let validator = Validator::new(builder);

    let leaves = vec![
        // builder.hash_n_to_hash_no_pad::<PoseidonPermutation<GoldilocksField>>
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.pubkey.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawal_credentials
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .effective_balance
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![validator.slashed.target]),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .activation_eligibility_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .activation_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.exit_epoch.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawable_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
    ];

    // let hash_tree_root_poseidon = hash_tree_root_poseidon(builder, leaves.len());

    // for i in 0..leaves.len() {
    //     builder.connect_hashes(leaves[i], hash_tree_root_poseidon.leaves[i]);
    // }

    // ValidatorPoseidonHashTreeRootTargets {
    //     validator,
    //     hash_tree_root: hash_tree_root_poseidon.hash_tree_root,
    // }

    // println!("r = {:?}", r)
}
