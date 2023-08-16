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

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    is_valid_merkle_branch::{is_valid_merkle_branch, IsValidMerkleBranchTargets},
    utils::{
        biguint_to_bits_target, bits_to_biguint_target, create_bool_target_array,
        epoch_to_mixed_endian, ETH_SHA256_BIT_SIZE,
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
    pub balance_sum: Target,
    pub withdrawal_credentials: [Target; 5],
}

pub fn build_final_circuit(
    balance_final_layer_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    commitment_mapper_final_layer_circuit_data: &CircuitData<
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
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let final_config = CircuitConfig {
        num_routed_wires: 37,
        fri_config: FriConfig {
            rate_bits: 8,
            cap_height: 0,
            proof_of_work_bits: 20,
            reduction_strategy: FriReductionStrategy::MinSize(None),
            num_query_rounds: 10,
        },
        ..standard_recursion_config
    };

    let mut builder = CircuitBuilder::<F, D>::new(final_config);

    let balance_verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(
            balance_final_layer_circuit_data
                .common
                .config
                .fri_config
                .cap_height,
        ),
        circuit_digest: builder.add_virtual_hash(),
    };

    let balance_proof_targets: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&balance_final_layer_circuit_data.common);

    builder.verify_proof::<C>(
        &balance_proof_targets,
        &balance_verifier_circuit_target,
        &balance_final_layer_circuit_data.common,
    );

    let balance_validators_poseidon_hash: &[Target] =
        &balance_proof_targets.public_inputs[262..266];

    let balance_root_hash = &balance_proof_targets.public_inputs[1..257];

    let balance_sum = balance_proof_targets.public_inputs[0];

    let withdrawal_credentials = &balance_proof_targets.public_inputs[257..262];

    builder.register_public_input(balance_sum);

    let commitment_mapper_verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(
            commitment_mapper_final_layer_circuit_data
                .common
                .config
                .fri_config
                .cap_height,
        ),
        circuit_digest: builder.add_virtual_hash(),
    };

    let commitment_mapper_proof_targets: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&commitment_mapper_final_layer_circuit_data.common);

    builder.verify_proof::<C>(
        &commitment_mapper_proof_targets,
        &commitment_mapper_verifier_circuit_target,
        &commitment_mapper_final_layer_circuit_data.common,
    );

    let commitment_mapper_poseidon_hash = &commitment_mapper_proof_targets.public_inputs[0..4];

    for i in 0..4 {
        builder.connect(
            commitment_mapper_poseidon_hash[i],
            balance_validators_poseidon_hash[i],
        );
    }

    let commitment_mapper_sha256_root = &commitment_mapper_proof_targets.public_inputs[4..260];

    let state_root = create_bool_target_array(&mut builder);

    let validators_merkle_branch = create_and_connect_merkle_branch(
        &mut builder,
        43,
        commitment_mapper_sha256_root,
        &state_root,
    );

    let balance_merkle_branch =
        create_and_connect_merkle_branch(&mut builder, 44, balance_root_hash, &state_root);

    let balance_circuit_epoch = &balance_proof_targets.public_inputs[266..268];

    let balance_circuit_epoch_bits =
        epoch_to_mixed_endian(&mut builder, balance_circuit_epoch.try_into().unwrap());

    let current_epoch = bits_to_biguint_target(&mut builder, balance_circuit_epoch_bits);

    let slots_per_epoch = builder.constant_biguint(&BigUint::from_u32(32).unwrap());

    let current_slot_min = builder.mul_biguint(&current_epoch, &slots_per_epoch);

    let slots_to_add = builder.constant_biguint(&BigUint::from_u32(31).unwrap());

    let current_slot_max = builder.add_biguint(&current_slot_min, &slots_to_add);

    let slot = builder.add_virtual_biguint_target(2);

    let cmp1 = builder.cmp_biguint(&current_slot_min, &slot);

    let cmp2 = builder.cmp_biguint(&slot, &current_slot_max);

    let slot_is_in_range = builder.and(cmp1, cmp2);

    let _true = builder._true();

    builder.connect(slot_is_in_range.target, _true.target);

    let bits = biguint_to_bits_target::<_, 2, 2>(&mut builder, &slot);

    let slot_merkle_branch = create_and_connect_merkle_branch(
        &mut builder,
        34,
        &bits.iter().map(|x| x.target).collect::<Vec<Target>>(),
        &state_root,
    );

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
            validators_branch: balance_merkle_branch.branch.try_into().unwrap(),
            state_root,
            balance_branch: validators_merkle_branch.branch.try_into().unwrap(),
            balance_sum,
            slot,
            slot_branch: slot_merkle_branch.branch.try_into().unwrap(),
            withdrawal_credentials: withdrawal_credentials.try_into().unwrap(),
        },
        data,
    )
}

fn create_and_connect_merkle_branch(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    index: u32,
    leaf_targets: &[Target],
    root_targets: &[BoolTarget; ETH_SHA256_BIT_SIZE],
) -> IsValidMerkleBranchTargets {
    let merkle_branch = is_valid_merkle_branch(builder, 5);
    let index = builder.constant(GoldilocksField::from_canonical_u32(index));

    builder.connect(merkle_branch.index, index);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(merkle_branch.leaf[i].target, leaf_targets[i]);
    }

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(merkle_branch.root[i].target, root_targets[i].target);
    }

    merkle_branch
}
