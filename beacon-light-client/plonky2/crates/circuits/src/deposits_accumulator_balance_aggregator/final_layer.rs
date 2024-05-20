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
        target_to_le_bits,
    },
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
};
use circuit::Circuit;
use circuit_derive::CircuitTarget;
use itertools::Itertools;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::biguint::BigUintTarget;

use crate::common_targets::Sha256Target;

#[derive(CircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccumulatorBalanceAggregatorFinalLayerTargets {
    pub deposit_accumulator_root_proof: ProofWithPublicInputsTarget<2>,
    pub commitment_mapper_root_proof: ProofWithPublicInputsTarget<2>,
    pub deposit_commitment_mapper_root_proof: ProofWithPublicInputsTarget<2>,

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
    pub balance_branch: Sha256MerkleBranchTarget<5>,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub execution_block_number: BigUintTarget,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub execution_block_number_branch: Sha256MerkleBranchTarget<5>,

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

pub struct DepositAccumulatorBalanceAggregatorFinalLayer {}

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

    fn define<'a>(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        (
            deposit_accumulator_inner_circuit_data,
            commitment_mapper_circuit_data,
            deposit_commitment_mapper_circuit_data,
        ): &Self::Params,
    ) -> Self::Target {
        let verifier_deposit_accumulator_inner = VerifierCircuitTarget {
            constants_sigmas_cap: builder.constant_merkle_cap(
                &deposit_accumulator_inner_circuit_data
                    .verifier_only
                    .constants_sigmas_cap,
            ),
            circuit_digest: builder.constant_hash(
                deposit_accumulator_inner_circuit_data
                    .verifier_only
                    .circuit_digest,
            ),
        };

        let deposit_accumulator_root_proof =
            builder.add_virtual_proof_with_pis(&deposit_accumulator_inner_circuit_data.common);

        builder.verify_proof::<Self::C>(
            &deposit_accumulator_root_proof,
            &verifier_deposit_accumulator_inner,
            &deposit_accumulator_inner_circuit_data.common,
        );

        let verifier_commitment_mapper = VerifierCircuitTarget {
            constants_sigmas_cap: builder.constant_merkle_cap(
                &commitment_mapper_circuit_data
                    .verifier_only
                    .constants_sigmas_cap,
            ),
            circuit_digest: builder
                .constant_hash(commitment_mapper_circuit_data.verifier_only.circuit_digest),
        };
        let commitment_mapper_root_proof =
            builder.add_virtual_proof_with_pis(&commitment_mapper_circuit_data.common);

        builder.verify_proof::<Self::C>(
            &commitment_mapper_root_proof,
            &verifier_commitment_mapper,
            &commitment_mapper_circuit_data.common,
        );

        let verifier_deposit_commitment_mapper = VerifierCircuitTarget {
            constants_sigmas_cap: builder.constant_merkle_cap(
                &deposit_commitment_mapper_circuit_data
                    .verifier_only
                    .constants_sigmas_cap,
            ),
            circuit_digest: builder.constant_hash(
                deposit_commitment_mapper_circuit_data
                    .verifier_only
                    .circuit_digest,
            ),
        };

        let deposit_commitment_mapper_root_proof =
            builder.add_virtual_proof_with_pis(&deposit_commitment_mapper_circuit_data.common);

        builder.verify_proof::<Self::C>(
            &deposit_commitment_mapper_root_proof,
            &verifier_deposit_commitment_mapper,
            &deposit_commitment_mapper_circuit_data.common,
        );

        let input = Self::read_circuit_input_target(builder);

        let node = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(
            &deposit_accumulator_root_proof.public_inputs,
        )
        .node;

        let commitment_mapper_root_public_inputs =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(
                &commitment_mapper_root_proof.public_inputs,
            );
        let deposit_commitment_mapper_root_public_inputs =
            DepositsCommitmentMapperFirstLevel::read_public_inputs_target(
                &deposit_commitment_mapper_root_proof.public_inputs,
            );

        builder.connect_hashes(
            commitment_mapper_root_public_inputs.poseidon_hash_tree_root,
            node.commitment_mapper_root,
        );

        builder.connect_hashes(
            deposit_commitment_mapper_root_public_inputs.poseidon_hash_tree_root,
            node.deposits_mapper_root,
        );

        assert_merkle_proof_is_valid_const_sha256(
            builder,
            &input.state_root,
            &input.block_root,
            &input.state_root_branch,
            11,
        );

        assert_merkle_proof_is_valid_const_sha256(
            builder,
            &commitment_mapper_root_public_inputs.sha256_hash_tree_root,
            &input.state_root,
            &input.validators_branch,
            86,
        );

        assert_merkle_proof_is_valid_const_sha256(
            builder,
            &node.balances_root,
            &input.state_root,
            &input.balance_branch,
            44,
        );

        let slot_ssz = ssz_num_to_bits(builder, &input.slot, 64);

        assert_merkle_proof_is_valid_const_sha256(
            builder,
            &slot_ssz,
            &input.state_root,
            &input.slot_branch,
            34,
        );

        verify_slot_is_in_range::<Self::F, Self::C, { Self::D }>(
            builder,
            &input.slot,
            &node.current_epoch,
        );

        let block_number_ssz = ssz_num_to_bits(builder, &input.execution_block_number, 64);

        assert_merkle_proof_is_valid_const_sha256(
            builder,
            &block_number_ssz,
            &input.state_root,
            &input.execution_block_number_branch,
            120,
        );

        let eth1_deposit_index_ssz = ssz_num_to_bits(builder, &node.eth1_deposit_index, 64);

        assert_merkle_proof_is_valid_const_sha256(
            builder,
            &eth1_deposit_index_ssz,
            &input.state_root,
            &input.eth1_deposit_index_branch,
            120,
        );

        let final_sum_bits =
            biguint_to_bits_target::<Self::F, D, 2>(builder, &node.accumulated.balance_sum);

        let block_number_bits =
            biguint_to_bits_target::<Self::F, D, 2>(builder, &input.execution_block_number);

        let deposit_count_bits = target_to_le_bits(builder, node.accumulated.deposits_count);

        let number_of_non_activated_validators_bits = target_to_le_bits(
            builder,
            node.accumulated
                .validator_stats
                .non_activated_validators_count,
        );
        let number_of_active_validators_bits = target_to_le_bits(
            builder,
            node.accumulated.validator_stats.active_validators_count,
        );
        let number_of_exited_validators_bits = target_to_le_bits(
            builder,
            node.accumulated.validator_stats.exited_validators_count,
        );
        let number_of_slashed_validators_bits = target_to_le_bits(
            builder,
            node.accumulated.validator_stats.slashed_validators_count,
        );

        let mut public_inputs_hash = sha256(
            builder,
            &[
                input.block_root.as_slice(),
                block_number_bits.as_slice(),
                deposit_commitment_mapper_root_public_inputs
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
            deposit_accumulator_root_proof,
            commitment_mapper_root_proof,
            deposit_commitment_mapper_root_proof,
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
