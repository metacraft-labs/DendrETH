use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::CircuitBuilderBiguint, deposit_hash_tree_root::{hash_tree_root_deposit_sha256, DepositShaTargets}, deposit_hash_tree_root_poseidon::{hash_tree_root_deposit_poseidon, DepositPoseidonTargets}, targets_serialization::{ReadTargets, WriteTargets}, utils::ssz_num_from_bits
};

pub struct DepositCommitmentTargets {
    pub deposit: DepositShaTargets,
    pub sha256_hash_tree_root: [BoolTarget; 256],
    pub poseidon_hash_tree_root: HashOutTarget,
}

impl ReadTargets for DepositCommitmentTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(DepositCommitmentTargets {
            deposit: DepositShaTargets::read_targets(data)?,
            sha256_hash_tree_root: data.read_target_bool_vec()?.try_into().unwrap(),
            poseidon_hash_tree_root: data.read_target_hash()?,
        })
    }
}

impl WriteTargets for DepositCommitmentTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::new();

        data.extend(self.deposit.write_targets()?);
        data.write_target_bool_vec(&self.sha256_hash_tree_root)?;
        data.write_target_hash(&self.poseidon_hash_tree_root)?;

        Ok(data)
    }
}

pub fn deposit_commitment_mapper<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> DepositCommitmentTargets {
    let hash_tree_root_sha256 = hash_tree_root_deposit_sha256(builder);

    let deposit_poseidon = hash_tree_root_deposit_poseidon(builder);

    let deposit = hash_tree_root_sha256.deposit;

    let deposit_poseidon_mapped = DepositPoseidonTargets {
        pubkey: deposit.pubkey,
        deposit_index: ssz_num_from_bits(builder, &deposit.deposit_index[0..64]),
        signature: deposit.signature,
        deposit_message_hash_tree_root: deposit.deposit_message_hash_tree_root
    };

    for i in 0..384 {
        builder.connect(
            deposit_poseidon.deposit.pubkey[i].target,
            deposit_poseidon_mapped.pubkey[i].target,
        );
    }

    builder.connect_biguint(
        &deposit_poseidon.deposit.deposit_index,
        &deposit_poseidon_mapped.deposit_index,
    );

    let deposit_is_zero = builder.add_virtual_bool_target_safe();
    let zero = builder.zero();

    let poseidon_hash_tree_root = deposit_poseidon
        .hash_tree_root
        .elements
        .map(|x| builder._if(deposit_is_zero, zero, x));

    let sha256_hash_tree_root = hash_tree_root_sha256
        .hash_tree_root
        .map(|x| BoolTarget::new_unsafe(builder._if(deposit_is_zero, zero, x.target)));

    DepositCommitmentTargets {
        deposit,
        sha256_hash_tree_root,
        poseidon_hash_tree_root: HashOutTarget {
            elements: poseidon_hash_tree_root,
        },
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
        utils::{SetBytesArray, ETH_SHA256_BIT_SIZE},
        validator_commitment_mapper::validator_commitment_mapper,
    };

    #[test]
    fn test_validator_hash_tree_root() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = validator_commitment_mapper(&mut builder);

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

        let mut pw = PartialWitness::new();

        pw.set_bool_target(targets.validator_is_zero, false);

        pw.set_bytes_array(&targets.validator.pubkey, &validator_pubkey);

        pw.set_bytes_array(
            &targets.validator.withdrawal_credentials,
            &withdrawal_credentials,
        );

        pw.set_bytes_array(
            &targets.validator.activation_eligibility_epoch,
            &activation_eligibility_epoch,
        );

        pw.set_bytes_array(&targets.validator.activation_epoch, &activation_epoch);

        pw.set_bytes_array(&targets.validator.slashed, &slashed);

        pw.set_bytes_array(&targets.validator.effective_balance, &effective_balance);

        pw.set_bytes_array(&targets.validator.exit_epoch, &exit_epoch);

        pw.set_bytes_array(&targets.validator.withdrawable_epoch, &withdrawable_epoch);

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
