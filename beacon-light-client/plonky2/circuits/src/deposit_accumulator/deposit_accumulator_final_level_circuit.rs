use ark_std::iterable::Iterable;
use itertools::Itertools;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::{hash_types::{HashOutTarget, RichField}, poseidon::PoseidonHash},
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};

use crate::{
    biguint::CircuitBuilderBiguint,
    deposit_accumulator::{
        traits::DepositAccumulatorNodeTargetExt, utils::{add_sha_target, set_final_layer_public_variables},
    },
    utils::{biguint_is_equal, bits_to_biguint_target, if_biguint, ETH_SHA256_BIT_SIZE},
};

/// TODO: Read circuit inputs from function for more succinctness
// pub fn get_data_from_recurssive_circuit<F: RichField + Extendable<D>, const D: usize>(
//     builder: &mut CircuitBuilder<F, D>,
//     circuit_data: &CircuitData<
//         plonky2::field::goldilocks_field::GoldilocksField,
//         PoseidonGoldilocksConfig,
//         2,
//     >,
// ) -> ProofWithPublicInputsTarget<2> {
//     let verifier_deposit_accumulator_inner = VerifierCircuitTarget {
//         constants_sigmas_cap: builder.constant_merkle_cap(
//             &circuit_data
//                 .verifier_only
//                 .constants_sigmas_cap,
//         ),
//         circuit_digest: builder.constant_hash(
//             deposit_accumulator_inner_circuit_data
//                 .verifier_only
//                 .circuit_digest,
//         ),
//     };

//     todo!()
// }

pub fn connect_deposit_root_proofs(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    // commitment_mapper_root_sha256: &[BoolTarget; ETH_SHA256_BIT_SIZE],
    commitment_mapper_root_poseidon: HashOutTarget,

    deposit_accumulator_inner_deposit: HashOutTarget,
    deposit_accumulator_inner_validator: HashOutTarget,

    // deposit_commitment_mapper_root_sha256: &[BoolTarget; ETH_SHA256_BIT_SIZE],
    deposit_commitment_mapper_root_poseidon: HashOutTarget,
) {

    builder.connect_hashes(commitment_mapper_root_poseidon, deposit_accumulator_inner_validator);
    builder.connect_hashes(deposit_commitment_mapper_root_poseidon, deposit_accumulator_inner_deposit);
}

pub fn connect_state_root_proofs(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
) {
    todo!()
}

pub fn build_commitment_mapper_final_circuit(
    deposit_accumulator_inner_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    commitment_mapper_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
    deposit_commitment_mapper_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    // Deposit Accumulator Inner Proof
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

    let deposit_accumulator_inner_proof =
        builder.add_virtual_proof_with_pis(&deposit_accumulator_inner_circuit_data.common);

    builder.verify_proof::<C>(
        &deposit_accumulator_inner_proof,
        &verifier_deposit_accumulator_inner,
        &deposit_accumulator_inner_circuit_data.common,
    );

    // Commitment Mapper Proof
    let verifier_commitment_mapper = VerifierCircuitTarget {
        constants_sigmas_cap: builder.constant_merkle_cap(
            &commitment_mapper_circuit_data
                .verifier_only
                .constants_sigmas_cap,
        ),
        circuit_digest: builder
            .constant_hash(commitment_mapper_circuit_data.verifier_only.circuit_digest),
    };
    let commitment_mapper_proof =
        builder.add_virtual_proof_with_pis(&commitment_mapper_circuit_data.common);

    builder.verify_proof::<C>(
        &commitment_mapper_proof,
        &verifier_commitment_mapper,
        &commitment_mapper_circuit_data.common,
    );

    // Deposit Commitment Mapper Proof
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

    let deposit_commitment_mapper_proof =
        builder.add_virtual_proof_with_pis(&deposit_commitment_mapper_circuit_data.common);

    builder.verify_proof::<C>(
        &deposit_commitment_mapper_proof,
        &verifier_deposit_commitment_mapper,
        &deposit_commitment_mapper_circuit_data.common,
    );
    
    let block_root_sha = add_sha_target(&mut builder);
    let state_root_sha = add_sha_target(&mut builder);
    let execution_block_number = builder.add_virtual_target();
    let execution_block_number_merkle_proof: Vec<[BoolTarget;ETH_SHA256_BIT_SIZE]>;
    let slot_merkle_proof: Vec<[BoolTarget;ETH_SHA256_BIT_SIZE]>;
    let eth1_deposit_index_proof: Vec<[BoolTarget;ETH_SHA256_BIT_SIZE]>;
    let slot: Target;


    let node = deposit_accumulator_inner_proof.get_node();

    // Inputs End

    let _true = builder._true();

    set_final_layer_public_variables(&mut builder);
}
