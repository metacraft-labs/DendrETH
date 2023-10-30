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

use crate::{
    biguint::CircuitBuilderBiguint,
    build_validator_balance_circuit::{set_public_variables, ValidatorBalanceProofTargetsExt},
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

    let pt1: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&inner_circuit_data.common);
    let pt2: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&inner_circuit_data.common);

    builder.verify_proof::<C>(&pt1, &verifier_circuit_target, &inner_circuit_data.common);

    builder.verify_proof::<C>(&pt2, &verifier_circuit_target, &inner_circuit_data.common);

    let poseidon_hash = pt1.get_range_validator_commitment();

    let sha256_hash = pt1.get_range_balances_root();

    let poseidon_hash2 = pt2.get_range_validator_commitment();

    let sha256_hash2 = pt2.get_range_balances_root();

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

    let sum1 = pt1.get_range_total_value();

    let sum2 = pt2.get_range_total_value();

    let mut sum = builder.add_biguint(&sum1, &sum2);

    // pop carry
    sum.limbs.pop();

    let withdrawal_credentials1 = pt1.get_withdrawal_credentials();
    let withdrawal_credentials2 = pt2.get_withdrawal_credentials();

    builder.connect_biguint(&withdrawal_credentials1, &withdrawal_credentials2);

    let current_epoch1 = pt1.get_current_epoch();
    let current_epoch2 = pt2.get_current_epoch();

    builder.connect_biguint(&current_epoch1, &current_epoch2);

    set_public_variables(
        &mut builder,
        &sum,
        hasher.digest.try_into().unwrap(),
        &withdrawal_credentials1,
        hash,
        &current_epoch1,
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
