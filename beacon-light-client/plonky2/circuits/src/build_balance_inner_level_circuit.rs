use plonky2::{
    hash::poseidon::PoseidonHash,
    iop::target::{Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};

use crate::{sha256::make_circuits, utils::ETH_SHA256_BIT_SIZE};

pub struct BalanceInnerCircuitTargets {
    pub proof1: ProofWithPublicInputsTarget<2>,
    pub proof2: ProofWithPublicInputsTarget<2>,
    pub verifier_circuit_target: VerifierCircuitTarget,
}

pub fn build_balance_inner_circuit(
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

    let poseidon_hash: &[Target] = &pt1.public_inputs[262..266];

    let sha256_hash = &pt1.public_inputs[1..257];

    let poseidon_hash2 = &pt2.public_inputs[262..266];

    let sha256_hash2 = &pt2.public_inputs[1..257];

    let hasher = make_circuits(&mut builder, 512);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(hasher.message[i].target, sha256_hash[i]);
        builder.connect(hasher.message[i + ETH_SHA256_BIT_SIZE].target, sha256_hash2[i]);
    }

    let hash = builder.hash_n_to_hash_no_pad::<PoseidonHash>(
        poseidon_hash
            .iter()
            .chain(poseidon_hash2.iter())
            .cloned()
            .collect(),
    );

    let sum1 = pt1.public_inputs[0];

    let sum2 = pt2.public_inputs[0];

    let sum = builder.add(sum1, sum2);

    let withdrawal_credentials1 = &pt1.public_inputs[257..262];
    let withdrawal_credentials2 = &pt2.public_inputs[257..262];

    for i in 0..5 {
        builder.connect(withdrawal_credentials1[i], withdrawal_credentials2[i]);
    }

    let current_epoch1 = &pt1.public_inputs[266..268];
    let current_epoch2 = &pt2.public_inputs[266..268];

    for i in 0..2 {
        builder.connect(current_epoch1[i], current_epoch2[i]);
    }

    builder.register_public_input(sum);

    builder.register_public_inputs(
        &hasher
            .digest
            .iter()
            .map(|x| x.target)
            .collect::<Vec<Target>>(),
    );

    builder.register_public_inputs(withdrawal_credentials1);

    builder.register_public_inputs(&hash.elements);

    builder.register_public_inputs(current_epoch1);

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
