use crate::{
    common_targets::{SSZTarget, ValidatorTarget},
    serializers::{serde_bool_array_to_hex_string, serde_bool_array_to_hex_string_nested},
    utils::circuit::{
        bool_arrays_are_equal,
        hashing::merkle::{
            poseidon::{hash_tree_root_poseidon, hash_validator_poseidon_or_zeroes},
            sha256::hash_tree_root_sha256,
            ssz::ssz_num_from_bits,
        },
        validator_status::get_validator_status,
    },
};
use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit};
use circuit_derive::{CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use itertools::{izip, Itertools};

use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::{HashOutTarget, RichField},
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

#[derive(PublicInputsReadable, TargetPrimitive, SerdeCircuitTarget)]
pub struct AccumulatedValidatorsData {
    pub balance: BigUintTarget,
    pub non_activated_count: Target,
    pub active_count: Target,
    pub exited_count: Target,
    pub slashed_count: Target,
}

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
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub range_balances_root: Sha256Target,

    #[target(out)]
    pub range_validator_commitment: HashOutTarget,

    #[target(out)]
    pub accumulated_data: AccumulatedValidatorsData,
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
            .zip_eq(input.non_zero_validator_leaves_mask)
            .map(|(validator, is_not_zero)| {
                hash_validator_poseidon_or_zeroes(builder, &validator, is_not_zero)
            })
            .collect_vec();

        let validators_hash_tree_root_poseidon =
            hash_tree_root_poseidon(builder, &validators_leaves);

        let accumulated_data = accumulate_data(
            builder,
            &input.validators,
            &input.balances_leaves,
            &input.withdrawal_credentials,
            &input.current_epoch,
        );

        Self::Target {
            validators: input.validators,
            non_zero_validator_leaves_mask: input.non_zero_validator_leaves_mask,
            withdrawal_credentials: input.withdrawal_credentials,
            balances_leaves: input.balances_leaves,
            current_epoch: input.current_epoch,
            range_balances_root,
            range_validator_commitment: validators_hash_tree_root_poseidon,
            accumulated_data,
        }
    }
}

fn accumulate_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators: &[ValidatorTarget],
    balance_leaves: &[SSZTarget],
    withdrawal_credentials: &[Sha256Target],
    current_epoch: &BigUintTarget,
) -> AccumulatedValidatorsData {
    let considered_validators_mask = validators
        .iter()
        .map(|validator| {
            withdrawal_credentials
                .iter()
                .fold(builder._false(), |acc, credentials| {
                    let credentials_match = bool_arrays_are_equal(
                        builder,
                        &validator.withdrawal_credentials,
                        credentials,
                    );
                    builder.or(acc, credentials_match)
                })
        })
        .collect_vec();

    let balances = balance_leaves
        .into_iter()
        .flatten()
        .copied()
        .collect_vec()
        .chunks(64)
        .into_iter()
        .map(|balance_bits| ssz_num_from_bits(builder, balance_bits))
        .collect_vec();

    let zero_accumulated_data: AccumulatedValidatorsData = builder.zero_init();

    izip!(validators, &balances, considered_validators_mask).fold(
        zero_accumulated_data,
        |acc, (validator, balance, is_considered)| {
            let (is_non_activated, is_active, is_exited) = get_validator_status(
                builder,
                &validator.activation_epoch,
                &current_epoch,
                &validator.exit_epoch,
            );

            let should_sum_balance = builder.and(is_considered, is_active);

            let mut summed_balance = builder.add_biguint(&acc.balance, balance);
            summed_balance.limbs.pop().unwrap();

            let new_balance =
                builder.select_target(should_sum_balance, &summed_balance, &acc.balance);

            let new_accumulated_data = AccumulatedValidatorsData {
                balance: new_balance,
                non_activated_count: builder.add(acc.non_activated_count, is_non_activated.target),
                active_count: builder.add(acc.active_count, is_active.target),
                exited_count: builder.add(acc.exited_count, is_exited.target),
                slashed_count: builder.add(acc.slashed_count, validator.slashed.target),
            };

            builder.select_target(is_considered, &new_accumulated_data, &acc)
        },
    )
}
