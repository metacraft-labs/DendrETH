use plonky2::{
    hash::poseidon::PoseidonHash,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    build_commitment_mapper_first_level_circuit::CommitmentMapperProofTargetExt,
    sha256::make_circuits,
    targets_serialization::{ReadTargets, WriteTargets},
    utils::ETH_SHA256_BIT_SIZE,
};

pub struct CommitmentMapperInnerCircuitTargets {
    pub proof1: ProofWithPublicInputsTarget<2>,
    pub proof2: ProofWithPublicInputsTarget<2>,
    pub verifier_circuit_target: VerifierCircuitTarget,
    pub is_zero: BoolTarget,
}

impl ReadTargets for CommitmentMapperInnerCircuitTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<CommitmentMapperInnerCircuitTargets> {
        Ok(CommitmentMapperInnerCircuitTargets {
            proof1: data.read_target_proof_with_public_inputs()?,
            proof2: data.read_target_proof_with_public_inputs()?,
            verifier_circuit_target: data.read_target_verifier_circuit()?,
            is_zero: data.read_target_bool()?,
        })
    }
}

impl WriteTargets for CommitmentMapperInnerCircuitTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_proof_with_public_inputs(&self.proof1)?;
        data.write_target_proof_with_public_inputs(&self.proof2)?;
        data.write_target_verifier_circuit(&self.verifier_circuit_target)?;
        data.write_target_bool(self.is_zero)?;

        Ok(data)
    }
}

pub fn build_commitment_mapper_inner_circuit(
    inner_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) -> (
    CommitmentMapperInnerCircuitTargets,
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

    let pt1 = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);
    let pt2: ProofWithPublicInputsTarget<2> =
        builder.add_virtual_proof_with_pis(&inner_circuit_data.common);

    builder.verify_proof::<C>(&pt1, &verifier_circuit_target, &inner_circuit_data.common);

    builder.verify_proof::<C>(&pt2, &verifier_circuit_target, &inner_circuit_data.common);

    let is_zero = builder.add_virtual_bool_target_safe();
    let is_one = builder.not(is_zero);

    let poseidon_hash = pt1
        .get_commitment_mapper_poseidon_hash_tree_root()
        .elements
        .map(|x| builder.mul(x, is_one.target));

    let sha256_hash = pt1
        .get_commitment_mapper_sha256_hash_tree_root()
        .map(|x| builder.mul(x.target, is_one.target));

    let poseidon_hash2 = pt2
        .get_commitment_mapper_poseidon_hash_tree_root()
        .elements
        .map(|x| builder.mul(x, is_one.target));

    let sha256_hash2 = pt2
        .get_commitment_mapper_sha256_hash_tree_root()
        .map(|x| builder.mul(x.target, is_one.target));

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

    builder.register_public_inputs(&hash.elements);
    builder.register_public_inputs(
        &hasher
            .digest
            .iter()
            .map(|x| x.target)
            .collect::<Vec<Target>>(),
    );

    let data = builder.build::<C>();

    (
        CommitmentMapperInnerCircuitTargets {
            proof1: pt1,
            proof2: pt2,
            verifier_circuit_target: verifier_circuit_target,
            is_zero,
        },
        data,
    )
}
