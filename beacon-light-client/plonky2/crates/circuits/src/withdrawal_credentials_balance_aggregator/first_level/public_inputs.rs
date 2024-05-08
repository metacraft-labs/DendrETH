use crate::{
    common_targets::Sha256Target,
    utils::{
        biguint::BigUintTarget,
        utils::{
            biguint_from_field_elements, hex_string_from_field_element_bits, ETH_SHA256_BIT_SIZE,
            POSEIDON_HASH_SIZE,
        },
    },
    withdrawal_credentials_balance_aggregator::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::{
    public_inputs::{
        field_reader::PublicInputsFieldReader, target_reader::PublicInputsTargetReader,
    },
    CircuitWithPublicInputs,
};
use itertools::Itertools;
use num::ToPrimitive;
use plonky2::{
    field::types::PrimeField64,
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
};

pub struct PublicInputsTarget<const WITHDRAWAL_CREDENTIALS_COUNT: usize> {
    pub range_total_value: BigUintTarget,
    pub range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub withdrawal_credentials: [[BoolTarget; ETH_SHA256_BIT_SIZE]; WITHDRAWAL_CREDENTIALS_COUNT],
    pub range_validator_commitment: HashOutTarget,
    pub current_epoch: BigUintTarget,
    pub number_of_non_activated_validators: Target,
    pub number_of_active_validators: Target,
    pub number_of_exitted_validators: Target,
}

pub struct PublicInputs<const WITHDRAWAL_CREDENTIALS_COUNT: usize> {
    pub range_total_value: u64,
    pub range_balances_root: String,
    pub withdrawal_credentials: [String; WITHDRAWAL_CREDENTIALS_COUNT],
    pub range_validator_commitment: [u64; POSEIDON_HASH_SIZE],
    pub current_epoch: u64,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exitted_validators: u64,
}

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize>
    CircuitWithPublicInputs
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    type PublicInputs = PublicInputs<WITHDRAWAL_CREDENTIALS_COUNT>;
    type PublicInputsTarget = PublicInputsTarget<WITHDRAWAL_CREDENTIALS_COUNT>;

    fn read_public_inputs(public_inputs: &[Self::F]) -> Self::PublicInputs {
        let mut reader = PublicInputsFieldReader::new(public_inputs);

        let range_total_value = reader.read_n(2);
        let range_balances_root = reader.read_n(256);

        let withdrawal_credentials = [(); WITHDRAWAL_CREDENTIALS_COUNT].map(|_| reader.read_n(256));

        let range_validator_commitment = reader.read_n(POSEIDON_HASH_SIZE);
        let current_epoch = reader.read_n(2);
        let number_of_non_activated_validators = reader.read();
        let number_of_active_validators = reader.read();
        let number_of_exitted_validators = reader.read();

        Self::PublicInputs {
            range_total_value: biguint_from_field_elements(range_total_value)
                .to_u64()
                .unwrap(),
            range_balances_root: hex_string_from_field_element_bits(range_balances_root),
            withdrawal_credentials: withdrawal_credentials.map(hex_string_from_field_element_bits),
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
            number_of_exitted_validators: number_of_exitted_validators.to_canonical_u64(),
        }
    }

    fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget {
        let mut reader = PublicInputsTargetReader::new(public_inputs);

        let range_total_value = reader.read_object::<BigUintTarget>();
        let range_balances_root = reader.read_object::<Sha256Target>();
        let withdrawal_credentials =
            reader.read_object::<[Sha256Target; WITHDRAWAL_CREDENTIALS_COUNT]>();
        let range_validator_commitment = reader.read_object::<HashOutTarget>();
        let current_epoch = reader.read_object::<BigUintTarget>();
        let number_of_non_activated_validators = reader.read_object::<Target>();
        let number_of_active_validators = reader.read_object::<Target>();
        let number_of_exitted_validators = reader.read_object::<Target>();

        Self::PublicInputsTarget {
            range_total_value,
            range_balances_root,
            withdrawal_credentials,
            range_validator_commitment,
            current_epoch,
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exitted_validators,
        }
    }
}
