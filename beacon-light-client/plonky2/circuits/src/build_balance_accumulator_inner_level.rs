use std::vec;

use plonky2::{
    hash::poseidon::PoseidonHash,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2_u32::gadgets::multiple_comparison::list_le_circuit;

use crate::{
    biguint::CircuitBuilderBiguint,
    build_validator_balance_accumulator_circuit::{
        set_public_variables, ValidatorBalanceProofAccumulatorTargetsExt,
    },
    targets_serialization::{ReadTargets, WriteTargets},
    utils::ETH_SHA256_BIT_SIZE,
};

pub struct BalanceInnerCircuitTargets {
    pub proof1: ProofWithPublicInputsTarget<2>,
    pub proof2: ProofWithPublicInputsTarget<2>,
    pub verifier_circuit_target: VerifierCircuitTarget,
}

impl ReadTargets for BalanceInnerCircuitTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self> {
        Ok(BalanceInnerCircuitTargets {
            proof1: data.read_target_proof_with_public_inputs()?,
            proof2: data.read_target_proof_with_public_inputs()?,
            verifier_circuit_target: data.read_target_verifier_circuit()?,
        })
    }
}

impl WriteTargets for BalanceInnerCircuitTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_proof_with_public_inputs(&self.proof1)?;
        data.write_target_proof_with_public_inputs(&self.proof2)?;
        data.write_target_verifier_circuit(&self.verifier_circuit_target)?;

        Ok(data)
    }
}

pub fn build_inner_level_circuit(
    inner_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) -> (
    BalanceInnerCircuitTargets,
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

    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    let verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: builder
            .constant_merkle_cap(&inner_circuit_data.verifier_only.constants_sigmas_cap),
        circuit_digest: builder.constant_hash(inner_circuit_data.verifier_only.circuit_digest),
    };

    let pt1 = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);
    let pt2 = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);

    builder.verify_proof::<C>(&pt1, &verifier_circuit_target, &inner_circuit_data.common);

    builder.verify_proof::<C>(&pt2, &verifier_circuit_target, &inner_circuit_data.common);

    let poseidon_hash = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_validator_commitment(&pt1);

    let sha256_hash = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_balances_root(&pt1);

    let poseidon_hash2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_validator_commitment(&pt2);

    let sha256_hash2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_balances_root(&pt2);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(sha256_hash[i].target, sha256_hash2[i].target);
    }

    builder.connect_hashes(poseidon_hash, poseidon_hash2);

    let accumulator_hash = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_validator_accumulator(&pt1);

    let accumulator_hash2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_validator_accumulator(&pt2);

    let hash = builder.hash_n_to_hash_no_pad::<PoseidonHash>(
        accumulator_hash
            .elements
            .iter()
            .chain(accumulator_hash2.elements.iter())
            .cloned()
            .collect(),
    );

    let deposit_count1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_deposit_count(&pt1);

    let deposit_count2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_deposit_count(&pt2);

    let deposit_count = builder.add(deposit_count1, deposit_count2);

    let current_eth1_deposit_index1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_current_eth1_deposit_index(&pt1);

    let current_eth1_deposit_index2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_current_eth1_deposit_index(&pt2);

    builder.connect_biguint(&current_eth1_deposit_index1, &current_eth1_deposit_index2);

    let number_of_non_activated_validators1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_number_of_non_activated_validators(&pt1);
    let number_of_non_activated_validators2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_number_of_non_activated_validators(&pt2);

    let number_of_non_activated_validators = builder.add(
        number_of_non_activated_validators1,
        number_of_non_activated_validators2,
    );

    let number_of_active_validators1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_number_of_active_validators(&pt1);
    let number_of_active_validators2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_number_of_active_validators(&pt2);

    let number_of_active_validators =
        builder.add(number_of_active_validators1, number_of_active_validators2);

    let number_of_exited_validators1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_number_of_exited_validators(&pt1);
    let number_of_exited_validators2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_number_of_exited_validators(&pt2);

    let number_of_exited_validators =
        builder.add(number_of_exited_validators1, number_of_exited_validators2);

    let sum1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_total_value(&pt1);

    let sum2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_total_value(&pt2);

    let mut sum = builder.add_biguint(&sum1, &sum2);

    // pop carry
    sum.limbs.pop();

    let current_epoch1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_current_epoch(&pt1);
    let current_epoch2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_current_epoch(&pt2);

    builder.connect_biguint(&current_epoch1, &current_epoch2);

    let range_start1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_start(&pt1);
    let range_end1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_end(&pt1);

    let range_start2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_start(&pt2);
    let range_end2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofAccumulatorTargetsExt>::get_range_end(&pt2);

    let range_check1 = list_le_circuit(&mut builder, vec![range_start1], vec![range_end1], 32);
    let range_check2 = list_le_circuit(&mut builder, vec![range_end1], vec![range_start2], 32);
    let range_check3 = list_le_circuit(&mut builder, vec![range_start2], vec![range_end2], 32);

    let mut all = builder.and(range_check1, range_check2);
    all = builder.and(all, range_check3);

    let _true = builder._true();
    builder.connect(all.target, _true.target);

    set_public_variables(
        &mut builder,
        &sum,
        range_start1,
        range_end2,
        deposit_count,
        sha256_hash,
        hash,
        poseidon_hash,
        &current_eth1_deposit_index1,
        &current_epoch1,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
    );

    let data = builder.build::<C>();

    (
        BalanceInnerCircuitTargets {
            proof1: pt1,
            proof2: pt2,
            verifier_circuit_target: verifier_circuit_target,
        },
        data,
    )
}
