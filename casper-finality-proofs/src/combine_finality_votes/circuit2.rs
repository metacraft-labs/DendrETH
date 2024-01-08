use crate::utils::plonky2x_extensions::assert_is_true;
use crate::weigh_justification_and_finalization::checkpoint::CheckpointVariable;

use plonky2x::backend::circuit::Circuit;
use plonky2x::prelude::{
    ArrayVariable, BoolVariable, CircuitBuilder, PlonkParameters, U256Variable, Variable,
};
use plonky2x::prelude::{CircuitVariable, Field};
use crate:: constants::VALIDATORS_PER_COMMITTEE;

pub type PublicKey = U256Variable;
pub type Commitment = PublicKey;

fn u256_variable_from_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    variable: Variable,
) -> U256Variable {
    let zero = builder.zero();
    let variables = vec![variable, zero, zero, zero, zero, zero, zero, zero];
    U256Variable::from_variables(builder, &variables)
}

fn commit_pubkey<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    pubkey: PublicKey,
    random_value: Variable,
) -> Commitment {
    let random_value_256 = u256_variable_from_variable(builder, random_value);
    builder.mul(pubkey, random_value_256)
}

pub fn compute_commitment<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    pubkeys: &[PublicKey],
    trusted_validators_bitmask: &[BoolVariable],
    random_value: Variable,
) -> Commitment {
    /*
    let one = builder.one();
    let at_least_one_validator_pred = builder.gte(count, one);
    assert_is_true(builder, at_least_one_validator_pred);
    */

    // let mut commitment = commit_pubkey(builder, pubkeys.first().unwrap().clone());
    let mut commitment = builder.zero();

    for i in 0..pubkeys.len() {
        // let key_out_of_range_pred = builder.gte(idx, count);

        /*
        if enforce_ordering {
            let ordering_pred = builder.lt(pubkeys[i - 1], pubkeys[i]);
            let valid_key_pred = builder.or(ordering_pred, key_out_of_range_pred);
            assert_is_true(builder, valid_key_pred);
        }
        */

        // let key_in_range_pred = builder.lt(idx, count);
        let mut key_commitment = commit_pubkey(builder, pubkeys[i], random_value);
        let is_trusted_validator_256 =
            u256_variable_from_variable(builder, trusted_validators_bitmask[i].variable);
        key_commitment = builder.mul(key_commitment, is_trusted_validator_256);
        commitment = builder.add(commitment, key_commitment);
    }

    commitment
}

// fn validate_pubkeys_input<L: PlonkParameters<D>, const D: usize>(
//     builder: &mut CircuitBuilder<L, D>,
//     pubkeys: &[PublicKey],

// ) {
//     let mut input_is_valid_pred = builder._true();

//     for i in 0..pubkeys.len() {
//         let idx = builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(i));
//         let in_range_pred = builder.lt(idx, count);
//         let not_in_range_pred = builder.not(in_range_pred);

//         let zero = builder.zero();
//         let is_zero_pred = builder.is_equal(pubkeys[i], zero);

//         let not_in_range_and_zero_pred = builder.and(not_in_range_pred, is_zero_pred);
//         let is_valid_pred = builder.or(in_range_pred, not_in_range_and_zero_pred);
//         input_is_valid_pred = builder.and(input_is_valid_pred, is_valid_pred);
//     }
//     assert_is_true(builder, input_is_valid_pred);
// }

#[derive(Debug, Clone)]
pub struct CommitTrustedValidatorPubkeys;

impl Circuit for CommitTrustedValidatorPubkeys {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        // padded with zeroes until the end for validator keys that haven't signed
        let random_value = builder.read::<Variable>();
        let source = builder.read::<CheckpointVariable>();
        let target = builder.read::<CheckpointVariable>();

        let pubkeys = builder.read::<ArrayVariable<PublicKey, VALIDATORS_PER_COMMITTEE>>();
        let trusted_validators_bitmask =
            builder.read::<ArrayVariable<BoolVariable, VALIDATORS_PER_COMMITTEE>>();

        // validate_pubkeys_input(builder, pubkeys.as_slice()); 

        let commitment = compute_commitment(
            builder,
            pubkeys.as_slice(),
            trusted_validators_bitmask.as_slice(),
            random_value,
        );

        builder.watch(&commitment, "commitment in circuit");

        builder.write(commitment);
        builder.write(target);
        builder.write(source);
    }
}
