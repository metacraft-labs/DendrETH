use crate::{
    common_targets::Sha256MerkleBranchTarget,
    deposits_accumulator_balance_aggregator::first_level::DepositAccumulatorBalanceAggregatorFirstLevel,
    deposits_accumulator_commitment_mapper::first_level::DepositsCommitmentMapperFirstLevel,
    final_layer::verify_slot_is_in_range,
    serializers::{
        biguint_to_str, parse_biguint, serde_bool_array_to_hex_string,
        serde_bool_array_to_hex_string_nested,
    },
    utils::circuit::{
        biguint_to_bits_target,
        hashing::{
            merkle::{sha256::assert_merkle_proof_is_valid_const_sha256, ssz::ssz_num_to_bits},
            sha256::sha256,
        },
        target_to_le_bits, verify_proof,
    },
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
};
use circuit::{Circuit, CircuitInputTarget};
use circuit_derive::CircuitTarget;
use itertools::Itertools;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::biguint::BigUintTarget;

use crate::common_targets::Sha256Target;

#[derive(CircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccumulatorBalanceAggregatorFinalLayerTargets {
    pub balance_aggregation_proof: ProofWithPublicInputsTarget<2>,
    pub validators_commitment_mapper_root_proof: ProofWithPublicInputsTarget<2>,
    pub deposits_commitment_mapper_root_proof: ProofWithPublicInputsTarget<2>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub block_root: Sha256Target,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub state_root: Sha256Target,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub state_root_branch: Sha256MerkleBranchTarget<3>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub validators_branch: Sha256MerkleBranchTarget<5>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balance_branch: Sha256MerkleBranchTarget<22>,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub execution_block_number: BigUintTarget,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub execution_block_number_branch: Sha256MerkleBranchTarget<10>,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub slot: BigUintTarget,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub slot_branch: Sha256MerkleBranchTarget<5>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub eth1_deposit_index_branch: Sha256MerkleBranchTarget<5>,
}

const D: usize = 2;

pub struct DepositAccumulatorBalanceAggregatorFinalLayer;

impl Circuit for DepositAccumulatorBalanceAggregatorFinalLayer {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = DepositAccumulatorBalanceAggregatorFinalLayerTargets;

    type Params = (
        CircuitData<Self::F, Self::C, { Self::D }>,
        CircuitData<Self::F, Self::C, { Self::D }>,
        CircuitData<Self::F, Self::C, { Self::D }>,
    );

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        (
            deposit_accumulator_balance_aggregator_circuit_data,
            validators_commitment_mapper_circuit_data,
            deposit_commitment_mapper_circuit_data,
        ): &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let balance_aggregation_proof = verify_proof(
            builder,
            &deposit_accumulator_balance_aggregator_circuit_data,
        );
        let validators_commitment_mapper_root_proof =
            verify_proof(builder, &validators_commitment_mapper_circuit_data);
        let deposits_commitment_mapper_root_proof =
            verify_proof(builder, &deposit_commitment_mapper_circuit_data);

        let balances_pi = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(
            &balance_aggregation_proof.public_inputs,
        );

        let validators_commitment_mapper_pi =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(
                &validators_commitment_mapper_root_proof.public_inputs,
            );
        let deposit_commitment_mapper_pi =
            DepositsCommitmentMapperFirstLevel::read_public_inputs_target(
                &deposits_commitment_mapper_root_proof.public_inputs,
            );

        builder.connect_hashes(
            validators_commitment_mapper_pi.poseidon_hash_tree_root,
            balances_pi.commitment_mapper_root,
        );
        builder.connect_hashes(
            deposit_commitment_mapper_pi.poseidon_hash_tree_root,
            balances_pi.deposits_commitment_mapper_root,
        );

        validate_data_against_block_root(
            builder,
            &input,
            &balances_pi.balances_root,
            &validators_commitment_mapper_pi.sha256_hash_tree_root,
            &balances_pi.eth1_deposit_index,
        );

        verify_slot_is_in_range(builder, &input.slot, &balances_pi.current_epoch);

        let final_sum_bits =
            biguint_to_bits_target::<Self::F, D, 2>(builder, &balances_pi.accumulated_data.balance);

        let block_number_bits =
            biguint_to_bits_target::<Self::F, D, 2>(builder, &input.execution_block_number);

        let deposit_count_bits =
            target_to_le_bits(builder, balances_pi.accumulated_data.deposits_count);

        let number_of_non_activated_validators_bits = target_to_le_bits(
            builder,
            balances_pi
                .accumulated_data
                .validator_status_stats
                .non_activated_count,
        );
        let number_of_active_validators_bits = target_to_le_bits(
            builder,
            balances_pi
                .accumulated_data
                .validator_status_stats
                .active_count,
        );
        let number_of_exited_validators_bits = target_to_le_bits(
            builder,
            balances_pi
                .accumulated_data
                .validator_status_stats
                .exited_count,
        );
        let number_of_slashed_validators_bits = target_to_le_bits(
            builder,
            balances_pi
                .accumulated_data
                .validator_status_stats
                .slashed_count,
        );

        let mut public_inputs_hash = sha256(
            builder,
            &[
                balances_pi.genesis_fork_version.as_slice(),
                input.block_root.as_slice(),
                block_number_bits.as_slice(),
                deposit_commitment_mapper_pi
                    .sha256_hash_tree_root
                    .as_slice(),
                deposit_count_bits.as_slice(),
                final_sum_bits.as_slice(),
                number_of_non_activated_validators_bits.as_slice(),
                number_of_active_validators_bits.as_slice(),
                number_of_exited_validators_bits.as_slice(),
                number_of_slashed_validators_bits.as_slice(),
            ]
            .concat(),
        );

        // Mask the last 3 bits in big endian as zero
        public_inputs_hash[0] = builder._false();
        public_inputs_hash[1] = builder._false();
        public_inputs_hash[2] = builder._false();

        let public_inputs_hash_bytes = public_inputs_hash
            .chunks(8)
            .map(|x| builder.le_sum(x.iter().rev()))
            .collect_vec();

        builder.register_public_inputs(&public_inputs_hash_bytes);

        Self::Target {
            balance_aggregation_proof,
            validators_commitment_mapper_root_proof,
            deposits_commitment_mapper_root_proof,
            block_root: input.block_root,
            state_root: input.state_root,
            state_root_branch: input.state_root_branch,
            validators_branch: input.validators_branch,
            balance_branch: input.balance_branch,
            execution_block_number: input.execution_block_number,
            execution_block_number_branch: input.execution_block_number_branch,
            slot: input.slot,
            slot_branch: input.slot_branch,
            eth1_deposit_index_branch: input.eth1_deposit_index_branch,
        }
    }
}

fn validate_data_against_block_root<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    input: &CircuitInputTarget<DepositAccumulatorBalanceAggregatorFinalLayer>,
    balances_root_level_22: &Sha256Target,
    validators_root_left: &Sha256Target,
    eth1_deposit_index: &BigUintTarget,
) {
    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &input.state_root,
        &input.block_root,
        &input.state_root_branch,
        11,
    );

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &validators_root_left,
        &input.state_root,
        &input.validators_branch,
        86,
    );

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &balances_root_level_22,
        &input.state_root,
        &input.balance_branch,
        5767168,
    );

    let slot_ssz = ssz_num_to_bits(builder, &input.slot, 64);

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &slot_ssz,
        &input.state_root,
        &input.slot_branch,
        34,
    );

    let block_number_ssz = ssz_num_to_bits(builder, &input.execution_block_number, 64);

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &block_number_ssz,
        &input.state_root,
        &input.execution_block_number_branch,
        1798,
    );

    let eth1_deposit_index_ssz = ssz_num_to_bits(builder, eth1_deposit_index, 64);

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &eth1_deposit_index_ssz,
        &input.state_root,
        &input.eth1_deposit_index_branch,
        42,
    );
}
