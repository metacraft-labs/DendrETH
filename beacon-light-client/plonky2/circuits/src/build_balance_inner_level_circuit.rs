use plonky2::{
    hash::{poseidon::PoseidonHash},
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::CircuitBuilderBiguint,
    build_validator_balance_circuit::{
        set_public_variables, ValidatorBalanceProofTargetsExt,
    },
    sha256::make_circuits,
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

pub fn build_inner_level_circuit<const N: usize>(
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

    let poseidon_hash = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_range_validator_commitment(&pt1);

    let sha256_hash = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_range_balances_root(&pt1);

    let poseidon_hash2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_range_validator_commitment(&pt2);

    let sha256_hash2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_range_balances_root(&pt2);

    let hasher = make_circuits(&mut builder, (2 * ETH_SHA256_BIT_SIZE) as u64);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(hasher.message[i].target, sha256_hash[i].target);
        builder.connect(
            hasher.message[i + ETH_SHA256_BIT_SIZE].target,
            sha256_hash2[i].target,
        );
    }

    let hash = builder.hash_n_to_hash_no_pad::<PoseidonHash>(
        poseidon_hash
            .elements
            .iter()
            .chain(poseidon_hash2.elements.iter())
            .cloned()
            .collect(),
    );

    let number_of_non_activated_validators1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_number_of_non_activated_validators(&pt1);
    let number_of_non_activated_validators2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_number_of_non_activated_validators(&pt2);

    let number_of_non_activated_validators = builder.add(
        number_of_non_activated_validators1,
        number_of_non_activated_validators2,
    );

    let number_of_active_validators1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_number_of_active_validators(&pt1);
    let number_of_active_validators2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_number_of_active_validators(&pt2);

    let number_of_active_validators =
        builder.add(number_of_active_validators1, number_of_active_validators2);

    let number_of_exited_validators1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_number_of_exited_validators(&pt1);
    let number_of_exited_validators2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_number_of_exited_validators(&pt2);

    let number_of_exited_validators =
        builder.add(number_of_exited_validators1, number_of_exited_validators2);

    let sum1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_range_total_value(&pt1);

    let sum2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_range_total_value(&pt2);

    let mut sum = builder.add_biguint(&sum1, &sum2);

    // pop carry
    sum.limbs.pop();

    let withdrawal_credentials1: [[BoolTarget; 256]; N] = pt1.get_withdrawal_credentials();
    let withdrawal_credentials2: [[BoolTarget; 256]; N] = pt2.get_withdrawal_credentials();

    for i in 0..N {
        for j in 0..ETH_SHA256_BIT_SIZE {
            builder.connect(
                withdrawal_credentials1[i][j].target,
                withdrawal_credentials2[i][j].target,
            );
        }
    }

    let current_epoch1 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_current_epoch(&pt1);
    let current_epoch2 = <ProofWithPublicInputsTarget<2> as ValidatorBalanceProofTargetsExt<N>>::get_current_epoch(&pt2);

    builder.connect_biguint(&current_epoch1, &current_epoch2);

    set_public_variables(
        &mut builder,
        &sum,
        hasher.digest.try_into().unwrap(),
        &withdrawal_credentials1,
        hash,
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
