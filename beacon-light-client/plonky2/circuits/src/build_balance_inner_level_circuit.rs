use itertools::Itertools;
use plonky2::{
    hash::poseidon::PoseidonHash,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    build_validator_balance_circuit::{
        set_public_variables, CURRENT_EPOCH_PUB_INDEX, RANGE_BALANCES_ROOT_PUB_INDEX,
        RANGE_TOTAL_VALUE_PUB_INDEX, RANGE_VALIDATOR_COMMITMENT_PUB_INDEX,
        WITHDRAWAL_CREDENTIALS_PUB_INDEX, WITHDRAWAL_CREDENTIALS_SIZE,
    },
    sha256::make_circuits,
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{ETH_SHA256_BIT_SIZE, POSEIDON_HASH_SIZE},
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
            .add_virtual_cap(inner_circuit_data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };

    let pt1: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&inner_circuit_data.common);
    let pt2: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&inner_circuit_data.common);

    builder.verify_proof::<C>(&pt1, &verifier_circuit_target, &inner_circuit_data.common);

    builder.verify_proof::<C>(&pt2, &verifier_circuit_target, &inner_circuit_data.common);

    let poseidon_hash: &[Target] = &pt1.public_inputs[RANGE_VALIDATOR_COMMITMENT_PUB_INDEX
        ..RANGE_VALIDATOR_COMMITMENT_PUB_INDEX + POSEIDON_HASH_SIZE];

    let sha256_hash = &pt1.public_inputs
        [RANGE_BALANCES_ROOT_PUB_INDEX..RANGE_BALANCES_ROOT_PUB_INDEX + ETH_SHA256_BIT_SIZE];

    let poseidon_hash2 = &pt2.public_inputs[RANGE_VALIDATOR_COMMITMENT_PUB_INDEX
        ..RANGE_VALIDATOR_COMMITMENT_PUB_INDEX + POSEIDON_HASH_SIZE];

    let sha256_hash2 = &pt2.public_inputs
        [RANGE_BALANCES_ROOT_PUB_INDEX..RANGE_BALANCES_ROOT_PUB_INDEX + ETH_SHA256_BIT_SIZE];

    let hasher = make_circuits(&mut builder, (2 * ETH_SHA256_BIT_SIZE) as u64);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(hasher.message[i].target, sha256_hash[i]);
        builder.connect(
            hasher.message[i + ETH_SHA256_BIT_SIZE].target,
            sha256_hash2[i],
        );
    }

    let hash = builder.hash_n_to_hash_no_pad::<PoseidonHash>(
        poseidon_hash
            .iter()
            .chain(poseidon_hash2.iter())
            .cloned()
            .collect(),
    );

    let sum1 = BigUintTarget {
        limbs: pt1.public_inputs[RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
            .iter()
            .map(|x| U32Target(*x))
            .collect_vec(),
    };

    let sum2 = BigUintTarget {
        limbs: pt2.public_inputs[RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
            .iter()
            .map(|x| U32Target(*x))
            .collect_vec(),
    };

    let mut sum = builder.add_biguint(&sum1, &sum2);

    // pop carry
    sum.limbs.pop();

    let withdrawal_credentials1 = &pt1.public_inputs
        [WITHDRAWAL_CREDENTIALS_PUB_INDEX..WITHDRAWAL_CREDENTIALS_PUB_INDEX + WITHDRAWAL_CREDENTIALS_SIZE];
    let withdrawal_credentials2 = &pt2.public_inputs
        [WITHDRAWAL_CREDENTIALS_PUB_INDEX..WITHDRAWAL_CREDENTIALS_PUB_INDEX + WITHDRAWAL_CREDENTIALS_SIZE];

    for i in 0..5 {
        builder.connect(withdrawal_credentials1[i], withdrawal_credentials2[i]);
    }

    let withdrawal_credentials = BigUintTarget {
        limbs: withdrawal_credentials1
            .iter()
            .cloned()
            .map(|x| U32Target(x))
            .collect_vec(),
    };

    let current_epoch1 = &pt1.public_inputs[CURRENT_EPOCH_PUB_INDEX..CURRENT_EPOCH_PUB_INDEX + 2];
    let current_epoch2 = &pt2.public_inputs[CURRENT_EPOCH_PUB_INDEX..CURRENT_EPOCH_PUB_INDEX + 2];

    for i in 0..2 {
        builder.connect(current_epoch1[i], current_epoch2[i]);
    }

    let current_epoch = BigUintTarget {
        limbs: current_epoch1
            .iter()
            .cloned()
            .map(|x| U32Target(x))
            .collect_vec(),
    };

    set_public_variables(
        &mut builder,
        &sum,
        hasher.digest.try_into().unwrap(),
        &withdrawal_credentials,
        hash,
        &current_epoch,
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
