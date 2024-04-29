use crate::{
    traits::CircuitWithPublicInputs,
    utils::{
        biguint::BigUintTarget,
        public_inputs_reader::{PublicInputsReader, PublicInputsTargetReader},
        utils::{
            biguint_from_field_elements, biguint_from_limbs_target,
            hex_string_from_field_element_bits, ETH_SHA256_BIT_SIZE, POSEIDON_HASH_SIZE,
        },
    },
    withdrawal_credentials_balance_aggregator::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use itertools::Itertools;
use num::ToPrimitive;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::{circuit_builder::CircuitBuilder, config::GenericConfig},
};

pub struct PublicInputsTarget<const WITHDRAWAL_CREDENTIALS_COUNT: usize> {
    pub range_total_value: BigUintTarget,
    pub range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub withdrawal_credentials: [[BoolTarget; ETH_SHA256_BIT_SIZE]; WITHDRAWAL_CREDENTIALS_COUNT],
    pub range_validator_commitment: HashOutTarget,
    pub current_epoch: BigUintTarget,
    pub number_of_non_activated_validators: Target,
    pub number_of_active_validators: Target,
    pub number_of_exited_validators: Target,
}

pub struct PublicInputs<const WITHDRAWAL_CREDENTIALS_COUNT: usize> {
    pub range_total_value: u64,
    pub range_balances_root: String,
    pub withdrawal_credentials: String,
    pub range_validator_commitment: [u64; POSEIDON_HASH_SIZE],
    pub current_epoch: u64,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
}

impl<
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
        const D: usize,
        const WITHDRAWAL_CREDENTIALS_COUNT: usize,
    > CircuitWithPublicInputs<F, C, D>
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<F, C, D, WITHDRAWAL_CREDENTIALS_COUNT>
{
    type PublicInputs = PublicInputs<WITHDRAWAL_CREDENTIALS_COUNT>;
    type PublicInputsTarget = PublicInputsTarget<WITHDRAWAL_CREDENTIALS_COUNT>;

    fn read_public_inputs(public_inputs: &[F]) -> Self::PublicInputs {
        let mut reader = PublicInputsReader::new(public_inputs);

        let range_total_value = reader.read_n(2);
        let range_balances_root = reader.read_n(256);
        let withdrawal_credentials = reader.read_n(256);
        let range_validator_commitment = reader.read_n(POSEIDON_HASH_SIZE);
        let current_epoch = reader.read_n(2);
        let number_of_non_activated_validators = reader.read();
        let number_of_active_validators = reader.read();
        let number_of_exited_validators = reader.read();

        Self::PublicInputs {
            range_total_value: biguint_from_field_elements(range_total_value)
                .to_u64()
                .unwrap(),
            range_balances_root: hex_string_from_field_element_bits(range_balances_root),
            withdrawal_credentials: hex_string_from_field_element_bits(withdrawal_credentials),
            range_validator_commitment: range_validator_commitment
                .iter()
                .map(|element| element.to_canonical_u64())
                .collect_vec()
                .try_into()
                .unwrap(),
            current_epoch: biguint_from_field_elements(current_epoch).to_u64().unwrap(),
            number_of_non_activated_validators: number_of_non_activated_validators
                .to_canonical_u64(),
            number_of_active_validators: number_of_active_validators.to_canonical_u64(),
            number_of_exited_validators: number_of_exited_validators.to_canonical_u64(),
        }
    }

    fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget {
        let mut reader = PublicInputsTargetReader::new(public_inputs);

        let range_total_value = reader.read_n(2);
        let range_balances_root = reader.read_n(256);
        let withdrawal_credentials = [(); WITHDRAWAL_CREDENTIALS_COUNT].map(|_| reader.read_n(256));
        let range_validator_commitment = reader.read_n(4);
        let current_epoch = reader.read_n(2);
        let number_of_non_activated_validators = reader.read();
        let number_of_active_validators = reader.read();
        let number_of_exited_validators = reader.read();

        Self::PublicInputsTarget {
            range_total_value: biguint_from_limbs_target(range_total_value),
            range_balances_root: range_balances_root
                .iter()
                .map(|&target| BoolTarget::new_unsafe(target))
                .collect_vec()
                .try_into()
                .unwrap(),
            withdrawal_credentials: {
                withdrawal_credentials
                    .map(|credentials_targets| -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
                        credentials_targets
                            .iter()
                            .map(|&target| BoolTarget::new_unsafe(target))
                            .collect_vec()
                            .try_into()
                            .unwrap()
                    })
                    .try_into()
                    .unwrap()
            },
            range_validator_commitment: HashOutTarget::from_vec(
                range_validator_commitment.to_owned(),
            ),
            current_epoch: biguint_from_limbs_target(current_epoch),
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exited_validators,
        }
    }
}

pub fn set_public_inputs<
    F: RichField + Extendable<D>,
    const D: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    range_total_value: &BigUintTarget,
    range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    withdrawal_credentials: &[[BoolTarget; ETH_SHA256_BIT_SIZE]; WITHDRAWAL_CREDENTIALS_COUNT],
    range_validator_commitment: HashOutTarget,
    current_epoch: &BigUintTarget,
    number_of_non_activated_validators: Target,
    number_of_active_validators: Target,
    number_of_exited_validators: Target,
) {
    builder.register_public_inputs(&range_total_value.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_inputs(&range_balances_root.map(|x| x.target));

    for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
        builder.register_public_inputs(
            &withdrawal_credentials[i]
                .iter()
                .map(|x| x.target)
                .collect_vec(),
        );
    }

    builder.register_public_inputs(&range_validator_commitment.elements);

    builder.register_public_inputs(&current_epoch.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_input(number_of_non_activated_validators);

    builder.register_public_input(number_of_active_validators);

    builder.register_public_input(number_of_exited_validators);
}
