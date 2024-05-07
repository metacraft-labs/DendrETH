use crate::utils::public_inputs::field_reader::PublicInputsFieldReader;
use crate::utils::public_inputs::to_targets::ToTargets;
use crate::utils::public_inputs::{
    field_reader::PublicInputsReadable, target_reader::PublicInputsTargetReader,
};
use circuit::Circuit;
use circuit::TargetsWithPublicInputs;
use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;

use circuit_with_public_inputs_derive::PublicInputs;
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
    common_targets::Sha256Target,
    utils::{
        biguint::{BigUintTarget, CircuitBuilderBiguint},
        hashing::{
            hash_tree_root::hash_tree_root,
            hash_tree_root_poseidon::hash_tree_root_poseidon,
            sha256::{bool_arrays_are_equal, connect_bool_arrays},
            validator_hash_tree_root_poseidon::{
                hash_tree_root_validator_poseidon, ValidatorPoseidonTargets,
            },
        },
        is_active_validator::get_validator_status,
        utils::{create_bool_target_array, select_biguint, ssz_num_from_bits},
    },
};

// TODO: mark which ones are public inputs and generate the
// CircuitWithPublicInputs trait with a procedural macro (trait funcions and
// associated types)

#[derive(PublicInputs)]
pub struct ValidatorBalanceVerificationTargets<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:,
{
    #[public_input]
    pub range_total_value: BigUintTarget,
    #[public_input]
    pub range_balances_root: Sha256Target,
    #[public_input]
    pub range_validator_commitment: HashOutTarget,
    pub validators: [ValidatorPoseidonTargets; VALIDATORS_COUNT],
    pub non_zero_validator_leaves_mask: [BoolTarget; VALIDATORS_COUNT],
    pub balances: [Sha256Target; VALIDATORS_COUNT / 4],
    #[public_input]
    pub withdrawal_credentials: [Sha256Target; WITHDRAWAL_CREDENTIALS_COUNT],
    #[public_input]
    pub current_epoch: BigUintTarget,
    #[public_input]
    pub number_of_non_activated_validators: Target,
    #[public_input]
    pub number_of_active_validators: Target,
    #[public_input]
    pub number_of_exited_validators: Target,
}

// maybe implement a SerializableCircuit trait
// maybe add a function to expose the targets (returns Targets)

type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

pub struct WithdrawalCredentialsBalanceAggregatorFirstLevel<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:,
{
    // maybe sneak the targets in here as well
    pub targets: <Self as Circuit>::Targets,
    pub data: CircuitData<F, C, D>,
}

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> Circuit
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    type F = F;
    type C = C;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Targets =
        ValidatorBalanceVerificationTargets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>;
    type Params = ();

    fn define(builder: &mut CircuitBuilder<F, D>, _params: ()) -> Self::Targets {
        if !VALIDATORS_COUNT.is_power_of_two() {
            panic!("validators_len must be a power of two");
        }

        let balances_len = VALIDATORS_COUNT / 4;

        let balances_leaves = [(); VALIDATORS_COUNT / 4].map(|_| create_bool_target_array(builder));

        let balances_hash_tree_root_targets = hash_tree_root(builder, balances_len);

        for i in 0..balances_len {
            connect_bool_arrays(
                builder,
                &balances_hash_tree_root_targets.leaves[i],
                &balances_leaves[i],
            );
        }

        let validators_leaves =
            [(); VALIDATORS_COUNT].map(|_| hash_tree_root_validator_poseidon(builder));

        let hash_tree_root_poseidon_targets = hash_tree_root_poseidon(builder, VALIDATORS_COUNT);

        let validator_is_zero =
            [(); VALIDATORS_COUNT].map(|_| builder.add_virtual_bool_target_safe());

        let zero_hash = builder.zero();

        for i in 0..VALIDATORS_COUNT {
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

        for i in 0..VALIDATORS_COUNT {
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

        Self::Targets {
            non_zero_validator_leaves_mask: validator_is_zero,
            range_total_value: sum,
            range_balances_root: balances_hash_tree_root_targets.hash_tree_root,
            range_validator_commitment: hash_tree_root_poseidon_targets.hash_tree_root,
            validators: validators_leaves.map(|v| v.validator.clone()),
            balances: balances_leaves.try_into().unwrap(),
            withdrawal_credentials,
            current_epoch,
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exited_validators,
        }
    }
}
