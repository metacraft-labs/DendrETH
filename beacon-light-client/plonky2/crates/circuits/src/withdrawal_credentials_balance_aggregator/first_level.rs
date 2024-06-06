use crate::{
    common_targets::ValidatorTarget,
    serializers::{serde_bool_array_to_hex_string, serde_bool_array_to_hex_string_nested},
    utils::circuit::{
        bool_arrays_are_equal,
        hashing::merkle::{
            poseidon::{hash_tree_root_poseidon, hash_validator_poseidon_or_zeroes},
            sha256::hash_tree_root_sha256,
            ssz::ssz_num_from_bits,
        },
        select_biguint,
        validator_status::get_validator_status,
    },
};
use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions,
    targets::uint::{
        ops::arithmetic::{Add, Zero},
        Uint64Target,
    },
    Circuit,
};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use itertools::Itertools;

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

use crate::{
    common_targets::Sha256Target,
    serializers::{biguint_to_str, parse_biguint},
};

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorBalanceVerificationTargets<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:,
{
    #[target(in)]
    pub validators: [ValidatorTarget; VALIDATORS_COUNT],

    #[target(in)]
    pub non_zero_validator_leaves_mask: [BoolTarget; VALIDATORS_COUNT],

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balances_leaves: [Sha256Target; VALIDATORS_COUNT / 4],

    #[target(in, out)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub withdrawal_credentials: [Sha256Target; WITHDRAWAL_CREDENTIALS_COUNT],

    #[target(in, out)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUintTarget,

    #[target(out)]
    // #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub range_total_value: Uint64Target,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub range_balances_root: Sha256Target,

    #[target(out)]
    pub range_validator_commitment: HashOutTarget,

    #[target(out)]
    pub number_of_non_activated_validators: Target,

    #[target(out)]
    pub number_of_active_validators: Target,

    #[target(out)]
    pub number_of_exited_validators: Target,

    #[target(out)]
    pub number_of_slashed_validators: Target,
}

pub struct WithdrawalCredentialsBalanceAggregatorFirstLevel<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:;

const D: usize = 2;

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> Circuit
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target =
        ValidatorBalanceVerificationTargets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>;

    fn define(builder: &mut CircuitBuilder<Self::F, D>, _: &Self::Params) -> Self::Target {
        if !VALIDATORS_COUNT.is_power_of_two() {
            panic!("VALIDATORS_COUNT must be a power of two");
        }

        let input = Self::read_circuit_input_target(builder);

        let range_balances_root = hash_tree_root_sha256(builder, &input.balances_leaves);

        let validators_leaves = input
            .validators
            .iter()
            .zip(input.non_zero_validator_leaves_mask)
            .map(|(validator, is_not_zero)| {
                hash_validator_poseidon_or_zeroes(builder, &validator, is_not_zero)
            })
            .collect_vec();

        let validators_hash_tree_root_poseidon =
            hash_tree_root_poseidon(builder, &validators_leaves);

        let mut range_total_value = Uint64Target::zero(builder);
        let mut number_of_non_activated_validators = builder.zero();
        let mut number_of_active_validators = builder.zero();
        let mut number_of_exited_validators = builder.zero();
        let mut number_of_slashed_validators = builder.zero();

        for i in 0..VALIDATORS_COUNT {
            let mut validator_is_considered = builder._false();

            for j in 0..WITHDRAWAL_CREDENTIALS_COUNT {
                let is_equal_inner = bool_arrays_are_equal(
                    builder,
                    &input.validators[i].withdrawal_credentials,
                    &input.withdrawal_credentials[j],
                );

                validator_is_considered = builder.or(is_equal_inner, validator_is_considered);
            }

            let balance = ssz_num_from_bits(
                builder,
                &input.balances_leaves[i / 4][((i % 4) * 64)..(((i % 4) * 64) + 64)],
            );

            let zero = builder.zero_biguint();

            let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
                get_validator_status(
                    builder,
                    &input.validators[i].activation_epoch,
                    &input.current_epoch,
                    &input.validators[i].exit_epoch,
                );

            let will_be_counted = builder.and(validator_is_considered, is_valid_validator);

            let zero_u64 = Uint64Target::zero(builder);
            let current = builder.select_target(will_be_counted, &balance, &zero_u64);

            // range_total_value = builder.add_biguint(&range_total_value, &current);
            range_total_value = range_total_value.add(current, builder);

            number_of_active_validators =
                builder.add(number_of_active_validators, will_be_counted.target);

            let will_be_counted = builder.and(validator_is_considered, is_non_activated_validator);

            number_of_non_activated_validators =
                builder.add(number_of_non_activated_validators, will_be_counted.target);

            let will_be_counted = builder.and(validator_is_considered, is_exited_validator);

            number_of_exited_validators =
                builder.add(number_of_exited_validators, will_be_counted.target);

            let validator_is_considered_and_is_slashed =
                builder.and(validator_is_considered, input.validators[i].slashed);
            number_of_slashed_validators = builder.add(
                number_of_slashed_validators,
                validator_is_considered_and_is_slashed.target,
            );

            range_total_value.limbs.pop();
        }

        Self::Target {
            non_zero_validator_leaves_mask: input.non_zero_validator_leaves_mask,
            range_total_value,
            range_balances_root,
            range_validator_commitment: validators_hash_tree_root_poseidon,
            validators: input.validators,
            balances_leaves: input.balances_leaves,
            withdrawal_credentials: input.withdrawal_credentials,
            current_epoch: input.current_epoch,
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exited_validators,
            number_of_slashed_validators,
        }
    }
}
