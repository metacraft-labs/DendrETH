mod is_active_validator;

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
    },
};
use circuit::Circuit;
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use itertools::Itertools;

use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
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

use self::is_active_validator::get_validator_status;

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
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub range_total_value: BigUintTarget,

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

        let mut range_total_value = builder.zero_biguint();
        let mut number_of_non_activated_validators = builder.zero();
        let mut number_of_active_validators = builder.zero();
        let mut number_of_exited_validators = builder.zero();
        let mut number_of_slashed_validators = builder.zero();

        for i in 0..VALIDATORS_COUNT {
            let mut is_equal = builder._false();

            for j in 0..WITHDRAWAL_CREDENTIALS_COUNT {
                let is_equal_inner = bool_arrays_are_equal(
                    builder,
                    &input.validators[i].withdrawal_credentials,
                    &input.withdrawal_credentials[j],
                );

                is_equal = builder.or(is_equal_inner, is_equal);
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

            let will_be_counted = builder.and(is_equal, is_valid_validator);

            let current = select_biguint(builder, will_be_counted, &balance, &zero);

            range_total_value = builder.add_biguint(&range_total_value, &current);

            number_of_active_validators =
                builder.add(number_of_active_validators, will_be_counted.target);

            let will_be_counted = builder.and(is_equal, is_non_activated_validator);

            number_of_non_activated_validators =
                builder.add(number_of_non_activated_validators, will_be_counted.target);

            let will_be_counted = builder.and(is_equal, is_exited_validator);

            number_of_exited_validators =
                builder.add(number_of_exited_validators, will_be_counted.target);

            let validator_is_considered_and_is_slashed =
                builder.and(is_equal, input.validators[i].slashed);
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

pub(crate) fn reconcile_validator_field<const N: usize>(
    builder: &mut CircuitBuilder<GoldilocksField, D>,
    field: &str,
) -> [BoolTarget; N] {
    field
        .as_bytes()
        .iter()
        .map(|b| BoolTarget::new_unsafe(builder.constant(GoldilocksField::from_canonical_u8(*b))))
        .collect_vec()
        .try_into()
        .unwrap()
}

#[cfg(test)]
mod test_withdrawal_credentials_balance_aggregator_first_level {
    use crate::{
        common_targets::ValidatorTarget,
        utils::circuit::hashing::merkle::{
            poseidon::hash_tree_root_poseidon, sha256::hash_tree_root_sha256,
        },
        withdrawal_credentials_balance_aggregator::first_level::{
            reconcile_validator_field, ValidatorBalanceVerificationTargets,
        },
    };
    use circuit::Circuit;
    use itertools::Itertools;
    use num::{BigUint, FromPrimitive};
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::{target::BoolTarget, witness::PartialWitness},
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };
    use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint, WitnessBigUint};

    use crate::withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel;

    #[test]
    fn test_withdrawal_credentials_balance_aggregator_first_level(
    ) -> std::result::Result<(), anyhow::Error> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        // one less than withdrawable epoch of the validator target
        let current_epoch = builder.constant_biguint(&BigUint::from(11088797506618031724u64));
        let number_of_non_activated_validators = builder.zero();
        let number_of_active_validators = builder.one();
        let number_of_exited_validators = builder.zero();
        let number_of_slashed_validators = builder.zero();
        let non_zero_validator_leaves_mask = [builder._true()];

        let pubkey = "0xdce8886981784ec14df0d772ec184989d730c3aa693067c18f69ae545e0a75bdce897cd1b2a80b752d81fa5273fe4ef9";
        let withdrawal_credentials =
            "0x9da1972f5f429488f41d31ce8e9284de3226ebf98dcc18016ddb91257ce999b6";
        let validator_target = ValidatorTarget {
            pubkey: reconcile_validator_field::<384>(&mut builder, pubkey),
            withdrawal_credentials: reconcile_validator_field::<256>(
                &mut builder,
                withdrawal_credentials,
            ),
            effective_balance: builder.constant_biguint(&BigUint::from(14214326743871124494u64)),
            slashed: builder._false(),
            activation_eligibility_epoch: builder
                .constant_biguint(&BigUint::from(17597259681949569805u64)),
            activation_epoch: builder.constant_biguint(&BigUint::from(6353383518537632738u64)),
            exit_epoch: builder.constant_biguint(&BigUint::from(9536164400595174516u64)),
            withdrawable_epoch: builder.constant_biguint(&BigUint::from(11088797506618031725u64)),
        };

        let withdrawal_credentials = [validator_target.withdrawal_credentials];
        let balances_leaves = [validator_target.effective_balance.clone()]; // perhaps it's not the effective balance of the validator?
                                                                            // let range_balances_root = hash_tree_root_sha256(&mut builder, &balances_leaves);
        let mut range_total_value = validator_target.effective_balance.limbs;
        range_total_value.pop();
        let range_total_value = range_total_value.to_owned();
        // let range_validator_commitment = hash_tree_root_poseidon(&mut builder, &validators_leaves);

        // WithdrawalCredentialsBalanceAggregatorFirstLevel::define(&mut builder, params);

        // let slot_target = builder.add_virtual_biguint_target(2);
        // let current_epoch = builder.add_virtual_biguint_target(2);

        // pw.set_biguint_target(&slot_target, &BigUint::from_u64(6953401).unwrap());
        // pw.set_biguint_target(&current_epoch, &BigUint::from_u64(217293).unwrap());

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        data.verify(proof)
    }
}
