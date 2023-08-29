use itertools::Itertools;
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    build_commitment_mapper_first_level_circuit::{POSEIDON_HASH_PUB_INDEX, SHA256_HASH_PUB_INDEX},
    build_validator_balance_circuit::{
        CURRENT_EPOCH_PUB_INDEX, RANGE_BALANCES_ROOT_PUB_INDEX, RANGE_TOTAL_VALUE_PUB_INDEX,
        RANGE_VALIDATOR_COMMITMENT_PUB_INDEX, WITHDRAWAL_CREDENTIALS_PUB_INDEX,
        WITHDRAWAL_CREDENTIALS_SIZE,
    },
    is_valid_merkle_branch::{is_valid_merkle_branch, IsValidMerkleBranchTargets},
    sha256::make_circuits,
    utils::{
        biguint_to_bits_target, create_bool_target_array, reverse_endianness, ETH_SHA256_BIT_SIZE,
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

pub struct FinalCircuitTargets {
    pub balance_circuit_targets: BalanceFinalLayerTargets,
    pub commitment_mapper_circuit_targets: CommitmentMapperFinalLayerTargets,
    pub slot: BigUintTarget,
    pub slot_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 5],
    pub state_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub validators_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 5],
    pub balance_branch: [[BoolTarget; ETH_SHA256_BIT_SIZE]; 5],
    pub balance_sum: BigUintTarget,
    pub withdrawal_credentials: BigUintTarget,
    pub validator_size_bits: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub fn build_final_circuit(
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
    FinalCircuitTargets,
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
    ) = setup_balance_targets(&mut builder, balance_data);

    let (
        commitment_mapper_proof_targets,
        commitment_mapper_verifier_circuit_target,
        commitment_mapper_sha256_root,
    ) = setup_commitment_mapper_targets(&mut builder, commitment_data);

    // TODO: maybe return just the poseidon hash instead of the whole proof
    for i in 0..4 {
        builder.connect(
            commitment_mapper_proof_targets.public_inputs[POSEIDON_HASH_PUB_INDEX + i],
            balance_proof_targets.public_inputs[RANGE_VALIDATOR_COMMITMENT_PUB_INDEX + i],
        );
    }

    let state_root = create_bool_target_array(&mut builder);

    let validator_size_bits = create_bool_target_array(&mut builder);

    let validators_hasher = make_circuits(&mut builder, (2 * ETH_SHA256_BIT_SIZE) as u64);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(
            validators_hasher.message[i].target,
            commitment_mapper_sha256_root[i],
        );
        builder.connect(
            validators_hasher.message[i + ETH_SHA256_BIT_SIZE].target,
            validator_size_bits[i].target,
        );
    }

    let validators_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 43, &validators_hasher.digest, &state_root);

    let balances_hasher = make_circuits(&mut builder, (2 * ETH_SHA256_BIT_SIZE) as u64);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(balances_hasher.message[i].target, balance_root_hash[i]);
        builder.connect(
            balances_hasher.message[i + ETH_SHA256_BIT_SIZE].target,
            validator_size_bits[i].target,
        );
    }

    let balance_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 44, &balances_hasher.digest, &state_root);

    // TODO: instead of getting it from the proof just return the current_epoch
    let current_epoch = BigUintTarget {
        limbs: balance_proof_targets.public_inputs
            [CURRENT_EPOCH_PUB_INDEX..CURRENT_EPOCH_PUB_INDEX + 2]
            .iter()
            .cloned()
            .map(|x| U32Target(x))
            .collect_vec(),
    };

    let slot = builder.add_virtual_biguint_target(2);

    verify_slot_is_in_range(&mut builder, &slot, &current_epoch);

    let mut slot_bits = reverse_endianness(&biguint_to_bits_target::<_, 2, 2>(&mut builder, &slot));

    slot_bits.extend((64..ETH_SHA256_BIT_SIZE).map(|_| builder._false()));

    let slot_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 34, &slot_bits, &state_root);

    builder.register_public_inputs(&state_root.iter().map(|x| x.target).collect::<Vec<Target>>());

    builder.register_public_inputs(
        &withdrawal_credentials
            .limbs
            .iter()
            .map(|x| x.0)
            .collect_vec(),
    );

    builder.register_public_inputs(&balance_sum.limbs.iter().map(|x| x.0).collect_vec());

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
            state_root,
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

fn setup_balance_targets(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    Vec<Target>,
    BigUintTarget,
    BigUintTarget,
) {
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);

    let root_hash = proof_targets.public_inputs
        [RANGE_BALANCES_ROOT_PUB_INDEX..RANGE_BALANCES_ROOT_PUB_INDEX + ETH_SHA256_BIT_SIZE]
        .to_vec();

    let sum = BigUintTarget {
        limbs: proof_targets.public_inputs
            [RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
            .iter()
            .cloned()
            .map(|x| U32Target(x))
            .collect_vec(),
    };

    let withdrawal_credentials = BigUintTarget {
        limbs: proof_targets.public_inputs[WITHDRAWAL_CREDENTIALS_PUB_INDEX
            ..WITHDRAWAL_CREDENTIALS_PUB_INDEX + WITHDRAWAL_CREDENTIALS_SIZE]
            .iter()
            .cloned()
            .map(|x| U32Target(x))
            .collect_vec(),
    };
    // WHY we don't get the current epoch from the proof?

    (
        proof_targets,
        verifier_circuit_target,
        root_hash,
        sum,
        withdrawal_credentials,
    )
}

fn setup_commitment_mapper_targets(
    builder: &mut CircuitBuilder<F, D>,
    data: &CircuitData<F, C, D>,
) -> (
    ProofWithPublicInputsTarget<D>,
    VerifierCircuitTarget,
    Vec<Target>,
) {
    let (proof_targets, verifier_circuit_target) = setup_proof_targets(data, builder);
    let sha256_root = proof_targets.public_inputs
        [SHA256_HASH_PUB_INDEX..SHA256_HASH_PUB_INDEX + ETH_SHA256_BIT_SIZE]
        .to_vec();

    (proof_targets, verifier_circuit_target, sha256_root)
}

fn verify_slot_is_in_range(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    slot: &BigUintTarget,
    current_epoch: &BigUintTarget,
) -> () {
    let slots_per_epoch = builder.constant_biguint(&BigUint::from_u32(32).unwrap());

    let current_slot_min = builder.mul_biguint(&current_epoch, &slots_per_epoch);

    let slots_to_add = builder.constant_biguint(&BigUint::from_u32(31).unwrap());

    let current_slot_max = builder.add_biguint(&current_slot_min, &slots_to_add);

    let cmp1 = builder.cmp_biguint(&current_slot_min, &slot);

    let cmp2 = builder.cmp_biguint(&slot, &current_slot_max);

    let slot_is_in_range = builder.and(cmp1, cmp2);

    let _true = builder._true();

    builder.connect(slot_is_in_range.target, _true.target);
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
) -> IsValidMerkleBranchTargets {
    let merkle_branch = is_valid_merkle_branch(builder, 5);
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
