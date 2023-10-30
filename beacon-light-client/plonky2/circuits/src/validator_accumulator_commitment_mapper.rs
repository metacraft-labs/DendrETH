use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    sha256::make_circuits,
    targets_serialization::{ReadTargets, WriteTargets},
    utils::ssz_num_to_bits,
};

pub struct ValidatorAccumulatorCommitmentTargets {
    pub validator_pubkey: [BoolTarget; 384],
    pub validator_eth1_deposit_index: BigUintTarget,
    pub sha256_hash_tree_root: [BoolTarget; 256],
    pub poseidon_hash_tree_root: HashOutTarget,
}

impl ReadTargets for ValidatorAccumulatorCommitmentTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        let validator_pubkey = data.read_target_bool_vec().unwrap();
        let validator_eth1_deposit_index = BigUintTarget::read_targets(data)?;
        let sha256_hash_tree_root = data.read_target_bool_vec()?;
        let poseidon_hash_tree_root = data.read_target_hash()?;

        Ok(ValidatorAccumulatorCommitmentTargets {
            validator_pubkey: validator_pubkey.try_into().unwrap(),
            validator_eth1_deposit_index,
            sha256_hash_tree_root: sha256_hash_tree_root.try_into().unwrap(),
            poseidon_hash_tree_root,
        })
    }
}

impl WriteTargets for ValidatorAccumulatorCommitmentTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::new();

        data.write_target_bool_vec(&self.validator_pubkey)?;
        data.extend(BigUintTarget::write_targets(
            &self.validator_eth1_deposit_index,
        )?);

        data.write_target_bool_vec(&self.sha256_hash_tree_root)?;
        data.write_target_hash(&self.poseidon_hash_tree_root)?;

        Ok(data)
    }
}

pub fn validator_accumulator_commitment_mapper<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorAccumulatorCommitmentTargets {
    let hasher = make_circuits(builder, 448);

    let validator_pubkey: [BoolTarget; 384] = (0..384)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let eth1_deposit_index = builder.add_virtual_biguint_target(2);

    let eth1_deposit_index_bits = ssz_num_to_bits(builder, &eth1_deposit_index, 64);

    for j in 0..384 {
        builder.connect(hasher.message[j].target, validator_pubkey[j].target);
    }

    for j in 384..448 {
        builder.connect(
            hasher.message[j].target,
            eth1_deposit_index_bits[j - 384].target,
        );
    }

    let poseidon_hash =
        get_validator_accumulator_poseidon_hash(builder, &validator_pubkey, &eth1_deposit_index);

    ValidatorAccumulatorCommitmentTargets {
        validator_pubkey: validator_pubkey,
        validator_eth1_deposit_index: eth1_deposit_index,
        sha256_hash_tree_root: hasher.digest.try_into().unwrap(),
        poseidon_hash_tree_root: poseidon_hash,
    }
}

pub fn get_validator_accumulator_poseidon_hash<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator_pubkey: &[BoolTarget; 384],
    eth1_deposit_indexes: &BigUintTarget,
) -> HashOutTarget {
    let pubkey_targets = validator_pubkey.iter().map(|x| x.target);
    let eth1_deposit_targets = eth1_deposit_indexes.limbs.iter().map(|x| x.0);

    builder.hash_n_to_hash_no_pad::<PoseidonHash>(
        pubkey_targets.chain(eth1_deposit_targets).collect_vec(),
    )
}

pub fn get_validators_accumulator_leaves<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator_pubkeys: &Vec<[BoolTarget; 384]>,
    eth1_deposit_indexes: &Vec<BigUintTarget>,
) -> Vec<HashOutTarget> {
    validator_pubkeys
        .iter()
        .zip(eth1_deposit_indexes.iter())
        .map(|(pubkey, deposit_index)| {
            get_validator_accumulator_poseidon_hash(builder, pubkey, deposit_index)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use num::BigUint;
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use crate::{
        biguint::WitnessBigUint,
        utils::{bytes_to_bools, SetBytesArray, ETH_SHA256_BIT_SIZE},
        validator_accumulator_commitment_mapper::validator_accumulator_commitment_mapper,
    };

    #[test]
    fn test_validator_hash_tree_root() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = validator_accumulator_commitment_mapper(&mut builder);

        let mut pw = PartialWitness::new();

        pw.set_bytes_array(
            &targets.validator_pubkey,
            &hex::decode("89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee").unwrap(),
        );

        pw.set_biguint_target(
            &targets.validator_eth1_deposit_index,
            &BigUint::from(1076099u64),
        );

        let validators_hash_tree_root = bytes_to_bools(
            &hex::decode("37a6de102958b6e12f4a42d0a95ba67208f2fbaee44127ceb13bc393d47304c6")
                .unwrap(),
        );

        for i in 0..ETH_SHA256_BIT_SIZE {
            if validators_hash_tree_root[i] {
                builder.assert_one(targets.sha256_hash_tree_root[i].target);
            } else {
                builder.assert_zero(targets.sha256_hash_tree_root[i].target);
            }
        }

        builder.register_public_inputs(&targets.poseidon_hash_tree_root.elements);

        builder.register_public_inputs(&targets.sha256_hash_tree_root.map(|x| x.target));

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }
}
