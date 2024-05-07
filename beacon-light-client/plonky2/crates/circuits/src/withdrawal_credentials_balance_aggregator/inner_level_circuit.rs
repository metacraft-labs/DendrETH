use crate::{
    serialization::targets_serialization::{ReadTargets, WriteTargets},
    utils::{
        biguint::CircuitBuilderBiguint,
        hashing::sha256::{connect_bool_arrays, sha256_pair},
    },
    withdrawal_credentials_balance_aggregator::first_level::circuit::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::CircuitWithPublicInputs;
use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};

// TODO: move this to a different file
fn poseidon_pair<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: HashOutTarget,
    right: HashOutTarget,
) -> HashOutTarget {
    builder.hash_n_to_hash_no_pad::<PoseidonHash>(
        left.elements
            .iter()
            .chain(right.elements.iter())
            .cloned()
            .collect(),
    )
}

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

pub fn build_inner_level_circuit<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
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
)
where
    [(); VALIDATORS_COUNT / 4]:,
{
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

    let proof1 = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);
    let proof2 = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);

    builder.verify_proof::<C>(
        &proof1,
        &verifier_circuit_target,
        &inner_circuit_data.common,
    );
    builder.verify_proof::<C>(
        &proof2,
        &verifier_circuit_target,
        &inner_circuit_data.common,
    );

    let l_input = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::read_public_inputs_target(&proof1.public_inputs);

    let r_input = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::read_public_inputs_target(&proof2.public_inputs);

    let range_validator_commitment = poseidon_pair(
        &mut builder,
        l_input.range_validator_commitment,
        r_input.range_validator_commitment,
    );

    let range_balances_root = sha256_pair(
        &mut builder,
        &l_input.range_balances_root,
        &r_input.range_balances_root,
    );

    let number_of_non_activated_validators = builder.add(
        l_input.number_of_non_activated_validators,
        r_input.number_of_non_activated_validators,
    );

    let number_of_active_validators = builder.add(
        l_input.number_of_active_validators,
        r_input.number_of_active_validators,
    );

    let number_of_exitted_validators = builder.add(
        l_input.number_of_exitted_validators,
        r_input.number_of_exitted_validators,
    );

    let mut sum = builder.add_biguint(&l_input.range_total_value, &r_input.range_total_value);

    // pop carry
    sum.limbs.pop();

    for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
        connect_bool_arrays(
            &mut builder,
            &l_input.withdrawal_credentials[i],
            &r_input.withdrawal_credentials[i],
        );
    }

    builder.connect_biguint(&l_input.current_epoch, &r_input.current_epoch);

    // TODO: fix this later
    // set_public_inputs(
    //     &mut builder,
    //     &sum,
    //     range_balances_root,
    //     &l_input.withdrawal_credentials,
    //     range_validator_commitment,
    //     &l_input.current_epoch,
    //     number_of_non_activated_validators,
    //     number_of_active_validators,
    //     number_of_exitted_validators,
    // );

    let data = builder.build::<C>();

    (
        BalanceInnerCircuitTargets {
            proof1,
            proof2,
            verifier_circuit_target,
        },
        data,
    )
}
