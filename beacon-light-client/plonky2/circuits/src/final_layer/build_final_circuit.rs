use itertools::Itertools;
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};

use crate::{
    utils::{
        biguint::{BigUintTarget, CircuitBuilderBiguint},
        hashing::{
            is_valid_merkle_branch::{is_valid_merkle_branch_sha256, IsValidMerkleBranchTargets},
            sha256::make_circuits,
        },
        utils::{
            biguint_to_bits_target, create_bool_target_array, ssz_num_to_bits, ETH_SHA256_BIT_SIZE,
        },
    },
    validators_commitment_mapper::build_commitment_mapper_first_level_circuit::CommitmentMapperProofTargetExt,
    withdrawal_credentials_balance_aggregator::build_validator_balance_circuit::ValidatorBalanceProofTargetsExt,
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

    let (
        balance_proof_targets,
        balance_verifier_circuit_target,
        balance_root_hash,
        balance_sum,
        withdrawal_credentials,
        current_epoch,
        balances_validator_poseidon_root,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
    ) = setup_balance_targets(&mut builder, balance_data);

    let (
        commitment_mapper_proof_targets,
        commitment_mapper_verifier_circuit_target,
        commitment_mapper_poseidon_root,
        commitment_mapper_sha256_root,
    ) = setup_commitment_mapper_targets(&mut builder, commitment_data);

    builder.connect_hashes(
        commitment_mapper_poseidon_root,
        balances_validator_poseidon_root,
    );

    let state_root = create_bool_target_array(&mut builder);

    let block_root = create_bool_target_array(&mut builder);

    let state_root_branch =
        create_and_connect_merkle_branch(&mut builder, 11, &state_root, &block_root, 3);

    let validator_size_bits = create_bool_target_array(&mut builder);

    let validators_hasher = make_circuits(&mut builder, (2 * ETH_SHA256_BIT_SIZE) as u64);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(
            validators_hasher.message[i].target,
            commitment_mapper_sha256_root[i].target,
        );
        builder.connect(
            validators_hasher.message[i + ETH_SHA256_BIT_SIZE].target,
            validator_size_bits[i].target,
        );
    }

    let validators_merkle_branch = create_and_connect_merkle_branch(
        &mut builder,
        43,
        &validators_hasher.digest,
        &state_root,
        5,
    );

    let balances_hasher = make_circuits(&mut builder, (2 * ETH_SHA256_BIT_SIZE) as u64);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(
            balances_hasher.message[i].target,
            balance_root_hash[i].target,
        );
        builder.connect(
            balances_hasher.message[i + ETH_SHA256_BIT_SIZE].target,
            validator_size_bits[i].target,
        );
    }

    let balance_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 44, &balances_hasher.digest, &state_root, 5);

    let slot = builder.add_virtual_biguint_target(2);

    verify_slot_is_in_range(&mut builder, &slot, &current_epoch);

    let slot_bits = ssz_num_to_bits(&mut builder, &slot, 64);

    let slot_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 34, &slot_bits, &state_root, 5);

    let public_inputs_hasher = make_circuits(&mut builder, ((N + 2) * ETH_SHA256_BIT_SIZE) as u64);

    let final_sum_bits = biguint_to_bits_target::<F, D, 2>(&mut builder, &balance_sum);

    let flattened_withdrawal_credentials = withdrawal_credentials
        .iter()
        .flat_map(|array| array.iter())
        .collect_vec();

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(public_inputs_hasher.message[i].target, block_root[i].target);
    }

    for i in 0..ETH_SHA256_BIT_SIZE * N {
        builder.connect(
            public_inputs_hasher.message[ETH_SHA256_BIT_SIZE + i].target,
            flattened_withdrawal_credentials[i].target,
        );
    }

    let number_of_non_activated_validators_bits = builder
        .split_le(number_of_non_activated_validators, 64)
        .into_iter()
        .rev()
        .collect_vec();

    let number_of_active_validators_bits = builder
        .split_le(number_of_active_validators, 64)
        .into_iter()
        .rev()
        .collect_vec();

    let number_of_exited_validators_bits = builder
        .split_le(number_of_exited_validators, 64)
        .into_iter()
        .rev()
        .collect_vec();

    for i in 0..64 {
        builder.connect(
            public_inputs_hasher.message[(N + 1) * ETH_SHA256_BIT_SIZE + i].target,
            final_sum_bits[i].target,
        );
        builder.connect(
            public_inputs_hasher.message[(N + 1) * ETH_SHA256_BIT_SIZE + 64 + i].target,
            number_of_non_activated_validators_bits[i].target,
        );
        builder.connect(
            public_inputs_hasher.message[(N + 1) * ETH_SHA256_BIT_SIZE + 128 + i].target,
            number_of_active_validators_bits[i].target,
        );
        builder.connect(
            public_inputs_hasher.message[(N + 1) * ETH_SHA256_BIT_SIZE + 192 + i].target,
            number_of_exited_validators_bits[i].target,
        );
    }

    let mut sha256_hash = public_inputs_hasher.digest;

    // Mask the last 3 bits in big endian as zero
    sha256_hash[0] = builder._false();
    sha256_hash[1] = builder._false();
    sha256_hash[2] = builder._false();

    let tokens = sha256_hash[0..256]
        .chunks(8)
        .map(|x| builder.le_sum(x.iter().rev()))
        .collect_vec();

    builder.register_public_inputs(&tokens);

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
            balance_sum,
            slot,
            slot_branch: slot_merkle_branch.branch.try_into().unwrap(),
            withdrawal_credentials: withdrawal_credentials.try_into().unwrap(),
            validator_size_bits,
        },
        data,
    )
}

fn setup_balance_targets<const N: usize>(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    [BoolTarget; ETH_SHA256_BIT_SIZE],
    BigUintTarget,
    [[BoolTarget; ETH_SHA256_BIT_SIZE]; N],
    BigUintTarget,
    HashOutTarget,
    Target,
    Target,
    Target,
) {
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);

    let root_hash = ValidatorBalanceProofTargetsExt::<N>::get_range_balances_root(&proof_targets);
    let sum = ValidatorBalanceProofTargetsExt::<N>::get_range_total_value(&proof_targets);
    let withdrawal_credentials: [[BoolTarget; 256]; N] = proof_targets.get_withdrawal_credentials();
    let current_epoch = ValidatorBalanceProofTargetsExt::<N>::get_current_epoch(&proof_targets);
    let poseidon_hash =
        ValidatorBalanceProofTargetsExt::<N>::get_range_validator_commitment(&proof_targets);
    let number_of_non_activated_validators =
        ValidatorBalanceProofTargetsExt::<N>::get_number_of_non_activated_validators(
            &proof_targets,
        );
    let number_of_active_validators =
        ValidatorBalanceProofTargetsExt::<N>::get_number_of_active_validators(&proof_targets);
    let number_of_exited_validators =
        ValidatorBalanceProofTargetsExt::<N>::get_number_of_exited_validators(&proof_targets);

    (
        proof_targets,
        verifier_circuit_target,
        root_hash,
        sum,
        withdrawal_credentials,
        current_epoch,
        poseidon_hash,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
    )
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
