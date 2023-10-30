use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::CircuitBuilderBiguint,
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{bits_to_biguint_target, ssz_num_from_bits, ETH_SHA256_BIT_SIZE},
    validator_hash_tree_root::{hash_tree_root_validator_sha256, ValidatorShaTargets},
    validator_hash_tree_root_poseidon::{
        hash_tree_root_validator_poseidon, ValidatorPoseidonTargets,
    },
};
pub struct ValidatorCommitmentTargets {
    pub validator: ValidatorShaTargets,
    pub sha256_hash_tree_root: [BoolTarget; 256],
    pub poseidon_hash_tree_root: HashOutTarget,
}

impl ReadTargets for ValidatorCommitmentTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(ValidatorCommitmentTargets {
            validator: ValidatorShaTargets::read_targets(data)?,
            sha256_hash_tree_root: data.read_target_bool_vec()?.try_into().unwrap(),
            poseidon_hash_tree_root: data.read_target_hash()?,
        })
    }
}

impl WriteTargets for ValidatorCommitmentTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = self.validator.write_targets()?;

        data.write_target_bool_vec(&self.sha256_hash_tree_root)?;
        data.write_target_hash(&self.poseidon_hash_tree_root)?;

        Ok(data)
    }
}

pub fn validator_commitment_mapper<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorCommitmentTargets {
    let hash_tree_root_sha256 = hash_tree_root_validator_sha256(builder);

    let validator_poseidon = hash_tree_root_validator_poseidon(builder);

    let validator = hash_tree_root_sha256.validator;

    let validator_poseidon_mapped = ValidatorPoseidonTargets {
        pubkey: validator.pubkey,
        withdrawal_credentials: validator.withdrawal_credentials,
        activation_eligibility_epoch: ssz_num_from_bits(
            builder,
            &validator.activation_eligibility_epoch[0..64],
        ),
        slashed: validator.slashed[0],
        effective_balance: ssz_num_from_bits(builder, &validator.effective_balance[0..64]),
        activation_epoch: ssz_num_from_bits(builder, &validator.activation_epoch[0..64]),
        exit_epoch: ssz_num_from_bits(builder, &validator.exit_epoch[0..64]),
        withdrawable_epoch: ssz_num_from_bits(builder, &validator.withdrawable_epoch[0..64]),
    };

    for i in 0..384 {
        builder.connect(
            validator_poseidon.validator.pubkey[i].target,
            validator_poseidon_mapped.pubkey[i].target,
        );
    }

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(
            validator_poseidon.validator.withdrawal_credentials[i].target,
            validator_poseidon_mapped.withdrawal_credentials[i].target,
        );
    }

    builder.connect(
        validator_poseidon.validator.slashed.target,
        validator_poseidon_mapped.slashed.target,
    );

    builder.connect_biguint(
        &validator_poseidon.validator.activation_eligibility_epoch,
        &validator_poseidon_mapped.activation_eligibility_epoch,
    );

    builder.connect_biguint(
        &validator_poseidon.validator.effective_balance,
        &validator_poseidon_mapped.effective_balance,
    );

    builder.connect_biguint(
        &validator_poseidon.validator.activation_epoch,
        &validator_poseidon_mapped.activation_epoch,
    );

    builder.connect_biguint(
        &validator_poseidon.validator.exit_epoch,
        &validator_poseidon_mapped.exit_epoch,
    );

    builder.connect_biguint(
        &validator_poseidon.validator.withdrawable_epoch,
        &validator_poseidon_mapped.withdrawable_epoch,
    );

    ValidatorCommitmentTargets {
        validator: validator,
        sha256_hash_tree_root: hash_tree_root_sha256.hash_tree_root,
        poseidon_hash_tree_root: validator_poseidon.hash_tree_root,
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use crate::{
        utils::ETH_SHA256_BIT_SIZE, validator_commitment_mapper::validator_commitment_mapper,
    };

    #[test]
    fn test_validator_hash_tree_root() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = validator_commitment_mapper(&mut builder);

        let mut pw = PartialWitness::new();

        let validator_pubkey = [
            "1", "0", "0", "1", "0", "0", "1", "1", "0", "0", "1", "1", "1", "0", "1", "0", "1",
            "1", "0", "1", "1", "0", "0", "1", "0", "1", "0", "0", "1", "0", "0", "1", "0", "0",
            "0", "1", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "0", "0", "0",
            "0", "0", "1", "0", "1", "1", "0", "0", "1", "1", "1", "0", "1", "1", "1", "0", "1",
            "0", "0", "0", "0", "0", "1", "1", "0", "0", "1", "0", "1", "1", "0", "1", "1", "0",
            "1", "0", "1", "0", "1", "1", "0", "0", "0", "0", "0", "1", "1", "0", "1", "0", "0",
            "1", "0", "0", "1", "0", "1", "0", "1", "1", "0", "1", "1", "0", "1", "1", "0", "0",
            "0", "1", "0", "0", "1", "0", "1", "0", "1", "0", "1", "1", "1", "1", "0", "1", "0",
            "1", "0", "0", "0", "1", "1", "0", "0", "0", "1", "0", "0", "0", "0", "0", "0", "0",
            "0", "1", "0", "1", "1", "0", "0", "1", "1", "0", "0", "0", "1", "1", "0", "1", "1",
            "1", "0", "1", "0", "0", "0", "1", "1", "0", "1", "1", "0", "0", "0", "1", "1", "1",
            "0", "1", "1", "1", "0", "0", "1", "1", "1", "0", "0", "1", "0", "1", "0", "0", "1",
            "0", "0", "0", "0", "1", "0", "1", "0", "1", "1", "1", "0", "0", "0", "0", "1", "0",
            "0", "0", "1", "1", "1", "1", "0", "1", "0", "0", "0", "1", "1", "1", "1", "0", "1",
            "1", "1", "0", "0", "1", "1", "0", "0", "1", "0", "1", "0", "0", "1", "0", "0", "1",
            "0", "0", "1", "1", "0", "0", "1", "1", "1", "1", "0", "1", "0", "1", "0", "0", "0",
            "1", "0", "0", "0", "0", "0", "0", "1", "0", "0", "0", "1", "1", "1", "0", "0", "0",
            "0", "1", "1", "1", "0", "0", "1", "0", "1", "1", "1", "0", "1", "0", "1", "0", "0",
            "1", "0", "1", "0", "0", "1", "1", "1", "0", "1", "1", "0", "1", "0", "1", "1", "0",
            "0", "0", "1", "0", "1", "0", "0", "1", "0", "1", "0", "1", "0", "1", "1", "1", "0",
            "0", "0", "0", "1", "0", "0", "1", "1", "0", "1", "0", "0", "0", "0", "1", "0", "1",
            "0", "1", "1", "1", "0", "1", "0", "0", "1", "0", "1", "1", "0", "0", "0", "1", "1",
            "0", "0", "1", "0", "0", "1", "0", "1", "0", "1",
        ];

        let withdraw_credentials = [
            "0", "0", "0", "0", "0", "0", "0", "1", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "1", "1",
            "0", "1", "0", "0", "1", "1", "0", "1", "1", "0", "1", "0", "0", "1", "1", "0", "1",
            "1", "1", "0", "1", "1", "0", "1", "0", "0", "1", "0", "0", "1", "1", "1", "1", "0",
            "1", "1", "1", "1", "1", "0", "1", "0", "0", "1", "0", "1", "0", "0", "0", "1", "0",
            "0", "0", "0", "0", "0", "0", "0", "1", "1", "1", "1", "1", "1", "0", "1", "0", "0",
            "1", "1", "1", "0", "1", "1", "1", "0", "0", "0", "0", "1", "1", "0", "1", "0", "1",
            "0", "1", "0", "0", "1", "1", "1", "1", "1", "1", "0", "0", "0", "0", "0", "1", "0",
            "1", "0", "0", "0", "1", "1", "0", "0", "0", "1", "0", "1", "0", "1", "0", "1", "1",
            "1", "0", "1", "1", "0", "1", "0", "0", "0", "0", "0", "0", "1", "0", "0", "1", "1",
            "0", "1", "0", "0", "1", "0", "1", "1", "0", "1", "0", "1", "0", "1", "0", "0", "0",
            "0",
        ];

        let effective_balance = [
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "1", "0", "0", "0", "0", "0", "0", "0",
            "1", "0", "1", "1", "0", "0", "1", "0", "1", "1", "1", "0", "0", "1", "1", "0", "0",
            "0", "0", "0", "1", "1", "1", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0",
        ];

        let slashed = [
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0",
        ];

        let activation_eligibility_epoch = [
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0",
        ];

        let withdrawable_epoch = [
            "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1",
            "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1",
            "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1",
            "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "1", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0",
        ];

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

        for i in 0..384 {
            pw.set_bool_target(targets.validator.pubkey[i], validator_pubkey[i] == "1");
        }

        for i in 0..ETH_SHA256_BIT_SIZE {
            pw.set_bool_target(
                targets.validator.withdrawal_credentials[i],
                withdraw_credentials[i] == "1",
            );

            pw.set_bool_target(
                targets.validator.effective_balance[i],
                effective_balance[i] == "1",
            );

            pw.set_bool_target(targets.validator.slashed[i], slashed[i] == "1");

            pw.set_bool_target(
                targets.validator.activation_eligibility_epoch[i],
                activation_eligibility_epoch[i] == "1",
            );

            pw.set_bool_target(targets.validator.activation_epoch[i], false);

            pw.set_bool_target(
                targets.validator.exit_epoch[i],
                if i < 64 { true } else { false },
            );

            pw.set_bool_target(
                targets.validator.withdrawable_epoch[i],
                withdrawable_epoch[i] == "1",
            );
        }

        for i in 0..ETH_SHA256_BIT_SIZE {
            if validator_hash_tree_root[i] == "1" {
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
