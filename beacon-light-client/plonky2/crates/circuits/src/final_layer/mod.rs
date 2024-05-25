use crate::serializers::biguint_to_str;
use crate::serializers::parse_biguint;
use crate::serializers::serde_bool_array_to_hex_string;
use crate::serializers::serde_bool_array_to_hex_string_nested;
use crate::utils::hashing::is_valid_merkle_branch::assert_merkle_proof_is_valid_const;
use crate::{
    common_targets::{BasicProofTarget, Sha256MerkleBranchTarget, Sha256Target},
    utils::{
        hashing::sha256::sha256,
        utils::{biguint_to_bits_target, ssz_num_to_bits, target_to_le_bits},
    },
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::CircuitInputTarget;
use circuit::{Circuit, CircuitOutputTarget};
use circuit_derive::CircuitTarget;
use itertools::Itertools;
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    hash::hash_types::RichField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

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

    pub balance_verification_proof_target: BasicProofTarget,
    pub commitment_mapper_proof_target: BasicProofTarget,
}

const D: usize = 2;

pub struct BalanceVerificationFinalCircuit<const WITHDRAWAL_CREDENTIALS_COUNT: usize> {}

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

        // TODO: get rid of this call
        let (
            balance_verification_proof_target,
            balance_verifier_circuit_target,
            balance_verification_pi,
        ) = setup_balance_targets::<Self::F, Self::C, D, 8, WITHDRAWAL_CREDENTIALS_COUNT>(
            builder,
            balance_verification_circuit_data,
        );

        let (
            validators_commitment_mapper_proof_target,
            validators_commitment_mapper_verifier_circuit_target,
            validators_commitment_mapper_pi,
        ) = setup_commitment_mapper_targets(builder, validators_commitment_mapper_circuit_data);

        // Assert that the two proofs are made for the same validator set
        builder.connect_hashes(
            validators_commitment_mapper_pi.poseidon_hash_tree_root,
            balance_verification_pi.range_validator_commitment,
        );

        validate_input_against_block_root::<Self::F, Self::C, D, WITHDRAWAL_CREDENTIALS_COUNT>(
            builder,
            &input,
            &balance_verification_pi.range_balances_root,
            &validators_commitment_mapper_pi.sha256_hash_tree_root,
        );

        verify_slot_is_in_range::<Self::F, Self::C, D>(
            builder,
            &input.slot,
            &balance_verification_pi.current_epoch,
        );

        let accumulated_balance_bits = biguint_to_bits_target::<Self::F, D, 2>(
            builder,
            &balance_verification_pi.range_total_value,
        );

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
            balance_verification_proof_target: BasicProofTarget {
                proof: balance_verification_proof_target.clone(),
                verifier_circuit_target: balance_verifier_circuit_target,
            },
            commitment_mapper_proof_target: BasicProofTarget {
                proof: validators_commitment_mapper_proof_target,
                verifier_circuit_target: validators_commitment_mapper_verifier_circuit_target,
            },
        }
    }
}

fn validate_input_against_block_root<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    input: &CircuitInputTarget<BalanceVerificationFinalCircuit<WITHDRAWAL_CREDENTIALS_COUNT>>,
    balances_root_left: &Sha256Target,
    validators_root_left: &Sha256Target,
) {
    assert_merkle_proof_is_valid_const(
        builder,
        &input.state_root,
        &input.block_root,
        &input.state_root_branch,
        11,
    );

    assert_merkle_proof_is_valid_const(
        builder,
        validators_root_left,
        &input.state_root,
        &input.validators_branch,
        86,
    );

    assert_merkle_proof_is_valid_const(
        builder,
        balances_root_left,
        &input.state_root,
        &input.balances_branch,
        88,
    );

    let slot_ssz = ssz_num_to_bits(builder, &input.slot, 64);

    assert_merkle_proof_is_valid_const(
        builder,
        &slot_ssz,
        &input.state_root,
        &input.slot_branch,
        34,
    );
}

fn setup_balance_targets<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    CircuitOutputTarget<
        WithdrawalCredentialsBalanceAggregatorFirstLevel<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >,
    >,
)
where
    [(); VALIDATORS_COUNT / 4]:,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);

    let output = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::read_public_inputs_target(&proof_targets.public_inputs);

    (proof_targets, verifier_circuit_target, output)
}

fn setup_commitment_mapper_targets<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    CircuitOutputTarget<ValidatorsCommitmentMapperFirstLevel>,
)
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);

    let output = ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(
        &proof_targets.public_inputs,
    );

    (proof_targets, verifier_circuit_target, output)
}

fn verify_slot_is_in_range<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    slot: &BigUintTarget,
    current_epoch: &BigUintTarget,
) -> () {
    let slots_per_epoch = builder.constant_biguint(&BigUint::from_u32(32).unwrap());
    let slot_epoch = builder.div_biguint(slot, &slots_per_epoch);
    builder.connect_biguint(&slot_epoch, current_epoch);
}

fn setup_proof_targets<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    circuit_data: &CircuitData<F, C, D>,
    builder: &mut CircuitBuilder<F, D>,
) -> (ProofWithPublicInputsTarget<D>, VerifierCircuitTarget)
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: builder
            .add_virtual_cap(circuit_data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };

    let proof_targets = builder.add_virtual_proof_with_pis(&circuit_data.common);

    builder.verify_proof::<C>(
        &proof_targets,
        &verifier_circuit_target,
        &circuit_data.common,
    );

    (proof_targets, verifier_circuit_target)
}

#[allow(dead_code)]
fn create_final_config() -> CircuitConfig {
    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    CircuitConfig {
        num_routed_wires: 37,
        fri_config: FriConfig {
            rate_bits: 8,
            cap_height: 0,
            proof_of_work_bits: 20,
            reduction_strategy: FriReductionStrategy::MinSize(None),
            num_query_rounds: 10,
        },
        ..standard_recursion_config
    }
}

#[cfg(test)]
mod test_verify_slot_is_in_range {
    use num::{BigUint, FromPrimitive};
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };
    use plonky2_crypto::biguint::{CircuitBuilderBiguint, WitnessBigUint};

    use crate::final_layer::verify_slot_is_in_range;

    #[test]
    fn test_verify_slot_is_in_range() -> std::result::Result<(), anyhow::Error> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let slot_target = builder.add_virtual_biguint_target(2);
        let current_epoch = builder.add_virtual_biguint_target(2);

        verify_slot_is_in_range::<F, C, D>(&mut builder, &slot_target, &current_epoch);

        pw.set_biguint_target(&slot_target, &BigUint::from_u64(6953401).unwrap());
        pw.set_biguint_target(&current_epoch, &BigUint::from_u64(217293).unwrap());

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        data.verify(proof)
    }

    #[test]
    fn test_verify_slot_is_in_range_first_slot_in_epoch() -> std::result::Result<(), anyhow::Error>
    {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let slot_target = builder.add_virtual_biguint_target(2);
        let current_epoch = builder.add_virtual_biguint_target(2);

        verify_slot_is_in_range::<F, C, D>(&mut builder, &slot_target, &current_epoch);

        pw.set_biguint_target(&slot_target, &BigUint::from_u64(7314752).unwrap());
        pw.set_biguint_target(&current_epoch, &BigUint::from_u64(228586).unwrap());

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        data.verify(proof)
    }

    #[test]
    fn test_verify_slot_is_in_range_last_slot_in_epoch() -> std::result::Result<(), anyhow::Error> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let slot_target = builder.add_virtual_biguint_target(2);
        let current_epoch = builder.add_virtual_biguint_target(2);

        verify_slot_is_in_range::<F, C, D>(&mut builder, &slot_target, &current_epoch);

        pw.set_biguint_target(&slot_target, &BigUint::from_u64(7314751).unwrap());
        pw.set_biguint_target(&current_epoch, &BigUint::from_u64(228585).unwrap());

        let data = builder.build::<C>();
        let proof = data.prove(pw)?;

        data.verify(proof)
    }

    #[test]
    #[should_panic]
    fn test_verify_slot_is_not_in_range() -> () {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let slot_target = builder.add_virtual_biguint_target(2);
        let current_epoch = builder.add_virtual_biguint_target(2);

        verify_slot_is_in_range::<F, C, D>(&mut builder, &slot_target, &current_epoch);

        pw.set_biguint_target(&slot_target, &BigUint::from_u64(7314751).unwrap());
        pw.set_biguint_target(&current_epoch, &BigUint::from_u64(228586).unwrap());

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof).unwrap();
    }
}
