use crate::{
    common_targets::{Sha256MerkleBranchTarget, Sha256Target},
    serializers::{
        biguint_to_str, parse_biguint, serde_bool_array_to_hex_string,
        serde_bool_array_to_hex_string_nested,
    },
    utils::circuit::{
        assert_slot_is_in_epoch::assert_slot_is_in_epoch,
        biguint_to_bits_target,
        hashing::{
            merkle::{sha256::assert_merkle_proof_is_valid_const_sha256, ssz::ssz_num_to_bits},
            sha256::sha256,
        },
        target_to_le_bits, verify_proof,
    },
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
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

const D: usize = 2;

#[derive(CircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct FinalCircuitTargets<const WITHDRAWAL_CREDENTIALS_COUNT: usize> {
    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub slot: BigUintTarget,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub slot_branch: Sha256MerkleBranchTarget<5>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub state_root: Sha256Target,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub block_root: Sha256Target,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub state_root_branch: Sha256MerkleBranchTarget<3>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub validators_branch: Sha256MerkleBranchTarget<6>,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balances_branch: Sha256MerkleBranchTarget<6>,

    pub balance_verification_proof: ProofWithPublicInputsTarget<D>,
    pub validators_commitment_mapper_proof: ProofWithPublicInputsTarget<D>,
}

pub struct BalanceVerificationFinalCircuit<const WITHDRAWAL_CREDENTIALS_COUNT: usize>;

impl<const WITHDRAWAL_CREDENTIALS_COUNT: usize> Circuit
    for BalanceVerificationFinalCircuit<WITHDRAWAL_CREDENTIALS_COUNT>
{
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = FinalCircuitTargets<WITHDRAWAL_CREDENTIALS_COUNT>;

    type Params = (
        CircuitData<Self::F, Self::C, D>,
        CircuitData<Self::F, Self::C, D>,
    );

    fn define(
        builder: &mut CircuitBuilder<Self::F, D>,
        (balance_verification_circuit_data, validators_commitment_mapper_circuit_data): &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let balance_verification_proof = verify_proof(builder, &balance_verification_circuit_data);
        let validators_commitment_mapper_proof =
            verify_proof(builder, &validators_commitment_mapper_circuit_data);

        let balance_verification_pi =
            WithdrawalCredentialsBalanceAggregatorFirstLevel::<
                8, // placeholder value
                WITHDRAWAL_CREDENTIALS_COUNT,
            >::read_public_inputs_target(&balance_verification_proof.public_inputs);

        let validators_commitment_mapper_pi =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(
                &validators_commitment_mapper_proof.public_inputs,
            );

        // Assert that the two proofs are made for the same validator set
        builder.connect_hashes(
            validators_commitment_mapper_pi.poseidon_hash_tree_root,
            balance_verification_pi.range_validator_commitment,
        );

        validate_input_against_block_root(
            builder,
            &input,
            &balance_verification_pi.range_balances_root,
            &validators_commitment_mapper_pi.sha256_hash_tree_root,
        );

        assert_slot_is_in_epoch(builder, &input.slot, &balance_verification_pi.current_epoch);

        let accumulated_balance_bits =
            biguint_to_bits_target(builder, &balance_verification_pi.range_total_value);

        let flattened_withdrawal_credentials = balance_verification_pi
            .withdrawal_credentials
            .iter()
            .flat_map(|array| array.iter())
            .cloned()
            .collect_vec();

        let number_of_non_activated_validators_bits = target_to_le_bits(
            builder,
            balance_verification_pi.number_of_non_activated_validators,
        );
        let number_of_active_validators_bits =
            target_to_le_bits(builder, balance_verification_pi.number_of_active_validators);
        let number_of_exited_validators_bits =
            target_to_le_bits(builder, balance_verification_pi.number_of_exited_validators);
        let number_of_slashed_validators_bits = target_to_le_bits(
            builder,
            balance_verification_pi.number_of_slashed_validators,
        );

        let mut public_inputs_hash = sha256(
            builder,
            &[
                input.block_root.as_slice(),
                flattened_withdrawal_credentials.as_slice(),
                accumulated_balance_bits.as_slice(),
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
            block_root: input.block_root,
            state_root: input.state_root,
            state_root_branch: input.state_root_branch,
            balances_branch: input.balances_branch,
            slot: input.slot,
            slot_branch: input.slot_branch,
            validators_branch: input.validators_branch,
            balance_verification_proof,
            validators_commitment_mapper_proof,
        }
    }
}

fn validate_input_against_block_root<
    F: RichField + Extendable<D>,
    const D: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    input: &CircuitInputTarget<BalanceVerificationFinalCircuit<WITHDRAWAL_CREDENTIALS_COUNT>>,
    balances_root_left: &Sha256Target,
    validators_root_left: &Sha256Target,
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
        validators_root_left,
        &input.state_root,
        &input.validators_branch,
        86,
    );

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        balances_root_left,
        &input.state_root,
        &input.balances_branch,
        88,
    );

    let slot_ssz = ssz_num_to_bits(builder, &input.slot, 64);

    assert_merkle_proof_is_valid_const_sha256(
        builder,
        &slot_ssz,
        &input.state_root,
        &input.slot_branch,
        34,
    );
}
