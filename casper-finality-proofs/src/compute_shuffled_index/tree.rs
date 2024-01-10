use plonky2::hash::{
    hash_types::RichField, hashing::hash_n_to_hash_no_pad, poseidon::PoseidonHash,
};
use plonky2x::backend::circuit::{DefaultParameters, PlonkParameters};
use plonky2x::frontend::builder::CircuitBuilder;
use plonky2x::frontend::uint::uint64::U64Variable;
use plonky2x::frontend::vars::{BoolVariable, Bytes32Variable, CircuitVariable};
use plonky2x::utils::bytes32;
use primitive_types::H256;

pub struct Validator {
    pub pubkey: [BoolVariable; 384],
    pub withdrawal_credentials: Bytes32Variable,
    pub effective_balance: U64Variable,
    pub slashed: BoolVariable,
    pub activation_eligibility_epoch: U64Variable,
    pub activation_epoch: U64Variable,
    pub exit_epoch: U64Variable,
    pub withdrawable_epoch: U64Variable,
}

impl Validator {
    pub fn new<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>,
    ) -> Validator {
        let empty_h256: H256 =
            bytes32!("0x0000000000000000000000000000000000000000000000000000000000000000");
        Validator {
            pubkey: [BoolVariable::constant(builder, false); 384],
            withdrawal_credentials: Bytes32Variable::constant(builder, empty_h256),
            effective_balance: U64Variable::constant(builder, 0),
            slashed: BoolVariable::constant(builder, false),
            activation_eligibility_epoch: U64Variable::constant(builder, 0),
            activation_epoch: U64Variable::constant(builder, 0),
            exit_epoch: U64Variable::constant(builder, 0),
            withdrawable_epoch: U64Variable::constant(builder, 0),
        }
    }
}

pub fn do_something() {
    // let r = hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
    //     GoldilocksField::from_canonical_u64(123),
    // ]);

    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();

    let validator = Validator::new(builder);

    // builder.hash_n_to_hash_no_pad::<PoseidonPermutation<GoldilocksField>>
    let leaves = vec![
        // SHOULD USE:
        // builder.api.hash_n_to_hash_no_pad(inputs);
        builder.api.hash_n_to_hash_no_pad::<PoseidonHash>(
            // validator.pubkey.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.api.hash_n_to_hash_no_pad::<PoseidonHash>(
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
