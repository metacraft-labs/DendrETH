use crate::{
    utils::{
        biguint::{BigUintTarget, CircuitBuilderBiguint},
        hashing::{
            is_valid_merkle_branch::{is_valid_merkle_branch_sha256, IsValidMerkleBranchTargets},
            sha256::{sha256, sha256_pair},
        },
        utils::{
            biguint_to_bits_target, create_bool_target_array, ssz_num_to_bits, target_to_le_bits,
            ETH_SHA256_BIT_SIZE,
        },
    },
    validators_commitment_mapper::build_commitment_mapper_first_level_circuit::CommitmentMapperProofTargetExt,
    withdrawal_credentials_balance_aggregator::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::{Circuit, TargetsWithPublicInputs};
use itertools::Itertools;
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    hash::hash_types::HashOutTarget,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};

pub struct BalanceFinalLayerTargets {
    pub proof: ProofWithPublicInputsTarget<2>,
    pub verifier_circuit_target: VerifierCircuitTarget,
}

pub struct CommitmentMapperFinalLayerTargets {
    pub proof: ProofWithPublicInputsTarget<2>,
    pub verifier_circuit_target: VerifierCircuitTarget,
}

pub struct FinalCircuitTargets<const N: usize> {
    pub balance_circuit_targets: BalanceFinalLayerTargets,
    pub commitment_mapper_circuit_targets: CommitmentMapperFinalLayerTargets,
    pub slot: BigUintTarget,
    pub slot_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 5],
    pub state_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub block_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub state_root_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 3],
    pub validators_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 5],
    pub balance_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 5],
    pub balance_sum: BigUintTarget,
    pub withdrawal_credentials: [[BoolTarget; ETH_SHA256_BIT_SIZE]; N],
    pub validator_size_bits: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub fn build_final_circuit<const N: usize>(
    balance_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    commitment_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) -> (
    FinalCircuitTargets<N>,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    let final_config = CircuitConfig::standard_recursion_config();

    let mut builder = CircuitBuilder::<F, D>::new(final_config);

    // TODO: get rid of this call
    let (balance_proof_targets, balance_verifier_circuit_target, balances_pi_target) =
        setup_balance_targets::<8, N>(&mut builder, balance_data);

    let (
        commitment_mapper_proof_targets,
        commitment_mapper_verifier_circuit_target,
        commitment_mapper_poseidon_root,
        commitment_mapper_sha256_root,
    ) = setup_commitment_mapper_targets(&mut builder, commitment_data);

    builder.connect_hashes(
        commitment_mapper_poseidon_root,
        balances_pi_target.range_validator_commitment,
    );

    let state_root = create_bool_target_array(&mut builder);

    let block_root = create_bool_target_array(&mut builder);

    let state_root_branch =
        create_and_connect_merkle_branch(&mut builder, 11, &state_root, &block_root, 3);

    let validator_size_bits = create_bool_target_array(&mut builder);

    let validators_root = sha256_pair(
        &mut builder,
        &commitment_mapper_sha256_root,
        &validator_size_bits,
    );

    let validators_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 43, &validators_root, &state_root, 5);

    let balances_root = sha256_pair(
        &mut builder,
        &balances_pi_target.range_balances_root,
        &validator_size_bits,
    );

    let balance_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 44, &balances_root, &state_root, 5);

    let slot = builder.add_virtual_biguint_target(2);

    verify_slot_is_in_range(&mut builder, &slot, &balances_pi_target.current_epoch);

    let slot_bits = ssz_num_to_bits(&mut builder, &slot, 64);

    let slot_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 34, &slot_bits, &state_root, 5);

    let final_sum_bits =
        biguint_to_bits_target::<F, D, 2>(&mut builder, &balances_pi_target.range_total_value);

    let flattened_withdrawal_credentials = balances_pi_target
        .withdrawal_credentials
        .iter()
        .flat_map(|array| array.iter())
        .cloned()
        .collect_vec();

    let number_of_non_activated_validators_bits = target_to_le_bits(
        &mut builder,
        balances_pi_target.number_of_non_activated_validators,
    );
    let number_of_active_validators_bits =
        target_to_le_bits(&mut builder, balances_pi_target.number_of_active_validators);
    let number_of_exitted_validators_bits = target_to_le_bits(
        &mut builder,
        balances_pi_target.number_of_exitted_validators,
    );

    let mut public_inputs_hash = sha256(
        &mut builder,
        &[
            block_root.as_slice(),
            flattened_withdrawal_credentials.as_slice(),
            final_sum_bits.as_slice(),
            number_of_non_activated_validators_bits.as_slice(),
            number_of_active_validators_bits.as_slice(),
            number_of_exitted_validators_bits.as_slice(),
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

    let data = builder.build::<C>();

    (
        FinalCircuitTargets {
            balance_circuit_targets: BalanceFinalLayerTargets {
                proof: balance_proof_targets.clone(),
                verifier_circuit_target: balance_verifier_circuit_target,
            },
            commitment_mapper_circuit_targets: CommitmentMapperFinalLayerTargets {
                proof: commitment_mapper_proof_targets,
                verifier_circuit_target: commitment_mapper_verifier_circuit_target,
            },
            validators_branch: validators_merkle_branch.branch.try_into().unwrap(),
            block_root,
            state_root,
            state_root_branch: state_root_branch.branch.try_into().unwrap(),
            balance_branch: balance_merkle_branch.branch.try_into().unwrap(),
            balance_sum: balances_pi_target.range_total_value,
            slot,
            slot_branch: slot_merkle_branch.branch.try_into().unwrap(),
            withdrawal_credentials: balances_pi_target.withdrawal_credentials,
            validator_size_bits,
        },
        data,
    )
}

fn setup_balance_targets<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize>(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    <<WithdrawalCredentialsBalanceAggregatorFirstLevel<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    > as Circuit>::Targets as TargetsWithPublicInputs>::PublicInputsTarget,
)
where
    [(); VALIDATORS_COUNT / 4]:,
{
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);

    let public_inputs_target = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::read_public_inputs_target_new(&proof_targets.public_inputs);

    (proof_targets, verifier_circuit_target, public_inputs_target)
}

fn setup_commitment_mapper_targets(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    HashOutTarget,
    [BoolTarget; ETH_SHA256_BIT_SIZE],
) {
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);
    let sha256_root = proof_targets.get_commitment_mapper_sha256_hash_tree_root();

    let poseidon_root = proof_targets.get_commitment_mapper_poseidon_hash_tree_root();

    (
        proof_targets,
        verifier_circuit_target,
        poseidon_root,
        sha256_root,
    )
}

// TODO: Rename this function
fn verify_slot_is_in_range(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    slot: &BigUintTarget,
    current_epoch: &BigUintTarget,
) -> () {
    let slots_per_epoch = builder.constant_biguint(&BigUint::from_u32(32).unwrap());

    let slot_epoch = builder.div_biguint(slot, &slots_per_epoch);

    builder.connect_biguint(&slot_epoch, current_epoch);
}

fn setup_proof_targets(
    circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
) -> (ProofWithPublicInputsTarget<2>, VerifierCircuitTarget) {
    let verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: builder
            .add_virtual_cap(circuit_data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };

    let proof_targets: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&circuit_data.common);

    builder.verify_proof::<PoseidonGoldilocksConfig>(
        &proof_targets,
        &verifier_circuit_target,
        &circuit_data.common,
    );

    (proof_targets, verifier_circuit_target)
}

fn create_and_connect_merkle_branch(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    index: u32,
    leaf_targets: &[BoolTarget],
    root_targets: &[BoolTarget; ETH_SHA256_BIT_SIZE],
    depth: usize,
) -> IsValidMerkleBranchTargets {
    let merkle_branch = is_valid_merkle_branch_sha256(builder, depth);
    let index = builder.constant(GoldilocksField::from_canonical_u32(index));

    builder.connect(merkle_branch.index, index);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(merkle_branch.leaf[i].target, leaf_targets[i].target);
    }

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(merkle_branch.root[i].target, root_targets[i].target);
    }

    merkle_branch
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

    use crate::{
        final_layer::build_final_circuit::verify_slot_is_in_range,
        utils::biguint::{CircuitBuilderBiguint, WitnessBigUint},
    };

    #[test]
    fn test_verify_slot_is_in_range() -> std::result::Result<(), anyhow::Error> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut pw = PartialWitness::new();

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let slot_target = builder.add_virtual_biguint_target(2);
        let current_epoch = builder.add_virtual_biguint_target(2);

        verify_slot_is_in_range(&mut builder, &slot_target, &current_epoch);

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

        verify_slot_is_in_range(&mut builder, &slot_target, &current_epoch);

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

        verify_slot_is_in_range(&mut builder, &slot_target, &current_epoch);

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

        verify_slot_is_in_range(&mut builder, &slot_target, &current_epoch);

        pw.set_biguint_target(&slot_target, &BigUint::from_u64(7314751).unwrap());
        pw.set_biguint_target(&current_epoch, &BigUint::from_u64(228586).unwrap());

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof).unwrap();
    }
}
