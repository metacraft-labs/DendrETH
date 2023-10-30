use itertools::Itertools;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field64},
    hash::hash_types::HashOutTarget,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use crate::{
    utils::{ETH_SHA256_BIT_SIZE, POSEIDON_HASH_SIZE},
    validator_accumulator_commitment_mapper::{
        validator_accumulator_commitment_mapper, ValidatorAccumulatorCommitmentTargets,
    },
    validator_commitment_mapper::{validator_commitment_mapper, ValidatorCommitmentTargets},
};

pub const POSEIDON_HASH_PUB_INDEX: usize = 0;
pub const SHA256_HASH_PUB_INDEX: usize = 4;

pub type CommitmentMapperProof =
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>;

pub trait CommitmentMapperProofExt {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> Vec<u64>;

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> Vec<u64>;
}

impl CommitmentMapperProofExt for CommitmentMapperProof {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> Vec<u64> {
        return self.public_inputs
            [POSEIDON_HASH_PUB_INDEX..POSEIDON_HASH_PUB_INDEX + POSEIDON_HASH_SIZE]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect_vec();
    }

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> Vec<u64> {
        return self.public_inputs
            [SHA256_HASH_PUB_INDEX..SHA256_HASH_PUB_INDEX + ETH_SHA256_BIT_SIZE]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect_vec();
    }
}

pub type CommitmentMapperProofTarget = ProofWithPublicInputsTarget<2>;

pub trait CommitmentMapperProofTargetExt {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> HashOutTarget;

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE];
}

impl CommitmentMapperProofTargetExt for CommitmentMapperProofTarget {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> HashOutTarget {
        HashOutTarget {
            elements: self.public_inputs
                [POSEIDON_HASH_PUB_INDEX..POSEIDON_HASH_PUB_INDEX + POSEIDON_HASH_SIZE]
                .try_into()
                .unwrap(),
        }
    }

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
        self.public_inputs[SHA256_HASH_PUB_INDEX..SHA256_HASH_PUB_INDEX + ETH_SHA256_BIT_SIZE]
            .iter()
            .cloned()
            .map(|x| BoolTarget::new_unsafe(x))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

pub trait CommitmentMapperTargets {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> HashOutTarget;

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE];
}

impl CommitmentMapperTargets for ValidatorCommitmentTargets {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> HashOutTarget {
        self.poseidon_hash_tree_root
    }

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
        self.sha256_hash_tree_root
    }
}

impl CommitmentMapperTargets for ValidatorAccumulatorCommitmentTargets {
    fn get_commitment_mapper_poseidon_hash_tree_root(&self) -> HashOutTarget {
        self.poseidon_hash_tree_root
    }

    fn get_commitment_mapper_sha256_hash_tree_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
        self.sha256_hash_tree_root
    }
}

fn build_commitment_mapper_first_level_circuit_generic<F, T>(
    mapper_function: F,
) -> (
    T,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
)
where
    F: FnOnce(&mut CircuitBuilder<GoldilocksField, 2>) -> T,
    T: CommitmentMapperTargets,
{
    let standard_recursion_config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(standard_recursion_config);

    let validator_commitment_result = mapper_function(&mut builder);

    // Register public inputs using the trait methods
    builder.register_public_inputs(
        &validator_commitment_result
            .get_commitment_mapper_poseidon_hash_tree_root()
            .elements,
    );
    builder.register_public_inputs(
        &validator_commitment_result
            .get_commitment_mapper_sha256_hash_tree_root()
            .map(|x| x.target),
    );

    let data = builder.build::<PoseidonGoldilocksConfig>();

    (validator_commitment_result, data)
}

pub fn build_accumulator_commitment_mapper_first_level_circuit() -> (
    ValidatorAccumulatorCommitmentTargets,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    build_commitment_mapper_first_level_circuit_generic(validator_accumulator_commitment_mapper)
}

pub fn build_commitment_mapper_first_level_circuit() -> (
    ValidatorCommitmentTargets,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    build_commitment_mapper_first_level_circuit_generic(validator_commitment_mapper)
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use plonky2::iop::witness::PartialWitness;

    use crate::{
        build_commitment_mapper_first_level_circuit::build_commitment_mapper_first_level_circuit,
        utils::{SetBytesArray, ETH_SHA256_BIT_SIZE},
    };

    #[test]
    fn test_validator_hash_tree_root() -> Result<()> {
        let (validator_commitment, data) = build_commitment_mapper_first_level_circuit();

        let mut pw = PartialWitness::new();

        let validator_pubkey =hex::decode("933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95").unwrap();
        let withdrawal_credentials =
            hex::decode("0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50")
                .unwrap();

        let effective_balance =
            hex::decode("0040597307000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let slashed =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let activation_eligibility_epoch =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let activation_epoch =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let exit_epoch =
            hex::decode("ffffffffffffffff000000000000000000000000000000000000000000000000")
                .unwrap();

        let withdrawable_epoch =
            hex::decode("ffffffffffffffff000000000000000000000000000000000000000000000000")
                .unwrap();

        let validator_hash_tree_root = [
            "0", "0", "1", "0", "1", "0", "1", "1", "1", "0", "1", "0", "1", "1", "1", "1", "0",
            "1", "0", "0", "0", "0", "0", "0", "0", "1", "1", "0", "0", "1", "0", "1", "1", "0",
            "1", "1", "0", "1", "0", "1", "1", "1", "0", "1", "0", "1", "1", "0", "0", "0", "1",
            "0", "0", "1", "0", "0", "0", "1", "1", "0", "0", "1", "0", "0", "0", "0", "0", "1",
            "0", "0", "0", "0", "0", "1", "0", "1", "0", "0", "0", "1", "1", "0", "0", "0", "1",
            "1", "0", "0", "0", "1", "1", "1", "1", "0", "0", "1", "1", "0", "0", "0", "0", "0",
            "0", "1", "1", "1", "1", "0", "0", "1", "0", "1", "0", "1", "0", "1", "0", "0", "0",
            "0", "0", "1", "1", "1", "1", "1", "0", "0", "1", "1", "1", "0", "1", "0", "0", "0",
            "0", "0", "1", "0", "1", "1", "0", "1", "0", "1", "0", "0", "0", "1", "1", "0", "1",
            "1", "0", "1", "1", "0", "0", "0", "0", "1", "1", "1", "1", "1", "1", "1", "1", "0",
            "0", "0", "0", "0", "0", "0", "1", "0", "0", "1", "1", "0", "0", "1", "1", "1", "0",
            "1", "1", "1", "1", "1", "0", "1", "0", "1", "0", "0", "1", "0", "1", "1", "0", "0",
            "0", "0", "1", "1", "1", "0", "0", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0",
            "0", "1", "1", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "1", "0",
            "1", "1", "0", "0", "0", "0", "1", "1", "1", "1", "1", "1", "0", "1", "0", "1", "0",
            "1",
        ];

        pw.set_bytes_array(&validator_commitment.validator.pubkey, &validator_pubkey);

        pw.set_bytes_array(
            &validator_commitment.validator.withdrawal_credentials,
            &withdrawal_credentials,
        );

        pw.set_bytes_array(
            &validator_commitment.validator.effective_balance,
            &effective_balance,
        );

        pw.set_bytes_array(&validator_commitment.validator.slashed, &slashed);

        pw.set_bytes_array(
            &validator_commitment.validator.activation_eligibility_epoch,
            &activation_eligibility_epoch,
        );

        pw.set_bytes_array(
            &validator_commitment.validator.activation_epoch,
            &activation_epoch,
        );

        pw.set_bytes_array(&validator_commitment.validator.exit_epoch, &exit_epoch);

        pw.set_bytes_array(
            &validator_commitment.validator.withdrawable_epoch,
            &withdrawable_epoch,
        );

        let proof = data.prove(pw).unwrap();

        for i in 0..ETH_SHA256_BIT_SIZE {
            assert_eq!(
                proof.public_inputs[i + 4].to_string(),
                validator_hash_tree_root[i].to_string()
            )
        }

        Ok(())
    }
}
