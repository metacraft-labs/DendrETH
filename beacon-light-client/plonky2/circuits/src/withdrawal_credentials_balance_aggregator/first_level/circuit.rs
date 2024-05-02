use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};

use crate::{
    traits::{Circuit, CircuitConf},
    utils::{
        biguint::{BigUintTarget, CircuitBuilderBiguint},
        hashing::{
            hash_tree_root::hash_tree_root,
            hash_tree_root_poseidon::hash_tree_root_poseidon,
            sha256::{bool_arrays_are_equal, connect_bool_arrays},
            validator_hash_tree_root_poseidon::{
                hash_tree_root_validator_poseidon, ValidatorPoseidonHashTreeRootTargets,
                ValidatorPoseidonTargets,
            },
        },
        is_active_validator::get_validator_status,
        utils::{create_bool_target_array, select_biguint, ssz_num_from_bits, ETH_SHA256_BIT_SIZE},
    },
};

use super::public_inputs::set_public_inputs;

// TODO: mark which ones are public inputs and generate the
// CircuitWithPublicInputs trait with a procedural macro (trait funcions and
// associated types)
pub struct ValidatorBalanceVerificationTargets<const N: usize> {
    pub range_total_value: BigUintTarget,
    pub range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub range_validator_commitment: HashOutTarget,
    pub validators: Vec<ValidatorPoseidonTargets>,
    pub non_zero_validator_leaves_mask: Vec<BoolTarget>,
    pub balances: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub withdrawal_credentials: [[BoolTarget; ETH_SHA256_BIT_SIZE]; N],
    pub current_epoch: BigUintTarget,
    pub number_of_non_activated_validators: Target,
    pub number_of_active_validators: Target,
    pub number_of_exited_validators: Target,
}

// maybe implement a SerializableCircuit trait
// maybe add a function to expose the targets (returns Targets)

pub struct WithdrawalCredentialsBalanceAggregatorFirstLevel<
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> {
    // maybe sneak the targets in here as well
    pub targets: <Self as Circuit>::Targets,
    pub data: <Self as CircuitConf>::CircuitData,
}

// TODO: generate this trait with a derive macro
impl<const WITHDRAWAL_CREDENTIALS_COUNT: usize> CircuitConf
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<WITHDRAWAL_CREDENTIALS_COUNT>
{
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
}

impl<const WITHDRAWAL_CREDENTIALS_COUNT: usize> Circuit
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<WITHDRAWAL_CREDENTIALS_COUNT>
{
    type Targets = ValidatorBalanceVerificationTargets<WITHDRAWAL_CREDENTIALS_COUNT>;
    type Params = usize;

    fn build(validators_len: usize) -> (Self::Targets, Self::CircuitData) {
        let mut builder = Self::CircuitBuilder::new(CircuitConfig::standard_recursion_config());
        let targets = Self::define(&mut builder, validators_len);
        let circuit_data = builder.build::<Self::C>();
        (targets, circuit_data)
    }

    fn define(builder: &mut Self::CircuitBuilder, validators_len: usize) -> Self::Targets {
        if !validators_len.is_power_of_two() {
            panic!("validators_len must be a power of two");
        }

        let balances_len = validators_len / 4;

        let balances_leaves: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]> = (0..balances_len)
            .map(|_| create_bool_target_array(builder))
            .collect();

        let balances_hash_tree_root_targets = hash_tree_root(builder, balances_len);

        for i in 0..balances_len {
            connect_bool_arrays(
                builder,
                &balances_hash_tree_root_targets.leaves[i],
                &balances_leaves[i],
            );
        }

        let validators_leaves: Vec<ValidatorPoseidonHashTreeRootTargets> = (0..validators_len)
            .map(|_| hash_tree_root_validator_poseidon(builder))
            .collect();

        let hash_tree_root_poseidon_targets = hash_tree_root_poseidon(builder, validators_len);

        let validator_is_zero: Vec<BoolTarget> = (0..validators_len)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect();

        let zero_hash = builder.zero();

        for i in 0..validators_len {
            let mut elements = [zero_hash; 4];

            for (j, _) in validators_leaves[i]
                .hash_tree_root
                .elements
                .iter()
                .enumerate()
            {
                elements[j] = builder._if(
                    validator_is_zero[i],
                    zero_hash,
                    validators_leaves[i].hash_tree_root.elements[j],
                );
            }

            builder.connect_hashes(
                hash_tree_root_poseidon_targets.leaves[i],
                HashOutTarget { elements },
            );
        }

        let withdrawal_credentials =
            [(); WITHDRAWAL_CREDENTIALS_COUNT].map(|_| create_bool_target_array(builder));

        let current_epoch = builder.add_virtual_biguint_target(2);

        let mut sum = builder.zero_biguint();

        let mut number_of_non_activated_validators = builder.zero();

        let mut number_of_active_validators = builder.zero();

        let mut number_of_exited_validators = builder.zero();

        for i in 0..validators_len {
            let mut is_equal = builder._false();

            for j in 0..WITHDRAWAL_CREDENTIALS_COUNT {
                let is_equal_inner = bool_arrays_are_equal(
                    builder,
                    &validators_leaves[i].validator.withdrawal_credentials,
                    &withdrawal_credentials[j],
                );

                is_equal = builder.or(is_equal_inner, is_equal);
            }

            let balance = ssz_num_from_bits(
                builder,
                &balances_leaves[i / 4][((i % 4) * 64)..(((i % 4) * 64) + 64)],
            );

            let zero = builder.zero_biguint();

            let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
                get_validator_status(
                    builder,
                    &validators_leaves[i].validator.activation_epoch,
                    &current_epoch,
                    &validators_leaves[i].validator.exit_epoch,
                );

            let will_be_counted = builder.and(is_equal, is_valid_validator);

            let current = select_biguint(builder, will_be_counted, &balance, &zero);

            sum = builder.add_biguint(&sum, &current);

            number_of_active_validators =
                builder.add(number_of_active_validators, will_be_counted.target);

            let will_be_counted = builder.and(is_equal, is_non_activated_validator);

            number_of_non_activated_validators =
                builder.add(number_of_non_activated_validators, will_be_counted.target);

            let will_be_counted = builder.and(is_equal, is_exited_validator);

            number_of_exited_validators =
                builder.add(number_of_exited_validators, will_be_counted.target);

            sum.limbs.pop();
        }

        let targets = Self::Targets {
            non_zero_validator_leaves_mask: validator_is_zero,
            range_total_value: sum,
            range_balances_root: balances_hash_tree_root_targets.hash_tree_root,
            range_validator_commitment: hash_tree_root_poseidon_targets.hash_tree_root,
            validators: validators_leaves
                .iter()
                .map(|v| v.validator.clone())
                .collect(),
            balances: balances_leaves,
            withdrawal_credentials,
            current_epoch,
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exited_validators,
        };

        set_public_inputs(
            builder,
            &targets.range_total_value,
            targets.range_balances_root,
            &targets.withdrawal_credentials,
            targets.range_validator_commitment,
            &targets.current_epoch,
            targets.number_of_non_activated_validators,
            targets.number_of_active_validators,
            targets.number_of_exited_validators,
        );

        targets
    }
}
