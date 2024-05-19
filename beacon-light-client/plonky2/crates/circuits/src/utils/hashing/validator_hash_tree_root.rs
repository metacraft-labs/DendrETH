use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    common_targets::{SSZLeafTarget, Sha256Target},
    serialization::targets_serialization::{ReadTargets, WriteTargets},
    utils::utils::{create_bool_target_array, ETH_SHA256_BIT_SIZE},
    validators_commitment_mapper::first_level::MerklelizedValidatorTarget,
};

use super::{
    hash_tree_root::{hash_tree_root, hash_tree_root_new},
    sha256::make_circuits,
};

pub struct ValidatorShaTargets {
    pub pubkey: [BoolTarget; 384],
    pub withdrawal_credentials: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub effective_balance: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub slashed: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub activation_eligibility_epoch: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub activation_epoch: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub exit_epoch: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub withdrawable_epoch: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

impl ReadTargets for ValidatorShaTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorShaTargets> {
        Ok(ValidatorShaTargets {
            pubkey: data.read_target_bool_vec()?.try_into().unwrap(),
            withdrawal_credentials: data.read_target_bool_vec()?.try_into().unwrap(),
            effective_balance: data.read_target_bool_vec()?.try_into().unwrap(),
            slashed: data.read_target_bool_vec()?.try_into().unwrap(),
            activation_eligibility_epoch: data.read_target_bool_vec()?.try_into().unwrap(),
            activation_epoch: data.read_target_bool_vec()?.try_into().unwrap(),
            exit_epoch: data.read_target_bool_vec()?.try_into().unwrap(),
            withdrawable_epoch: data.read_target_bool_vec()?.try_into().unwrap(),
        })
    }
}

impl WriteTargets for ValidatorShaTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut bytes = Vec::<u8>::new();

        bytes.write_target_bool_vec(&self.pubkey)?;
        bytes.write_target_bool_vec(&self.withdrawal_credentials)?;
        bytes.write_target_bool_vec(&self.effective_balance)?;
        bytes.write_target_bool_vec(&self.slashed)?;
        bytes.write_target_bool_vec(&self.activation_eligibility_epoch)?;
        bytes.write_target_bool_vec(&self.activation_epoch)?;
        bytes.write_target_bool_vec(&self.exit_epoch)?;
        bytes.write_target_bool_vec(&self.withdrawable_epoch)?;

        Ok(bytes)
    }
}

pub struct ValidatorHashTreeRootTargets {
    pub validator: ValidatorShaTargets,
    pub hash_tree_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

pub fn hash_tree_root_validator_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &MerklelizedValidatorTarget,
) -> Sha256Target {
    hash_tree_root_new(
        builder,
        &[
            validator.pubkey[0],
            validator.pubkey[1],
            validator.withdrawal_credentials,
            validator.effective_balance,
            validator.slashed,
            validator.activation_eligibility_epoch,
            validator.activation_epoch,
            validator.exit_epoch,
            validator.withdrawable_epoch,
        ],
    )
}

pub fn hash_tree_root_validator_sha256_or_zeroes<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &MerklelizedValidatorTarget,
    condition: BoolTarget,
) -> Sha256Target {
    let zero_leaf: SSZLeafTarget = [builder._false(); 256];
    let zero_leaves = [zero_leaf; 7];

    let mut leaves = vec![
        validator.pubkey[0],
        validator.pubkey[1],
        validator.withdrawal_credentials,
        validator.effective_balance,
        validator.slashed,
        validator.activation_eligibility_epoch,
        validator.activation_epoch,
        validator.exit_epoch,
        validator.withdrawable_epoch,
    ];

    leaves.extend(zero_leaves);

    let validator_hash = hash_tree_root_new(builder, &leaves);
    validator_hash.map(|bit| builder.and(condition, bit))
}

#[cfg(test)]
mod test {
    use std::println;

    use anyhow::Result;
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use crate::utils::{
        hashing::validator_hash_tree_root::hash_tree_root_validator_sha256,
        utils::{SetBytesArray, ETH_SHA256_BIT_SIZE},
    };

    #[test]
    fn test_validator_hash_tree_root() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = hash_tree_root_validator_sha256(&mut builder);

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
                builder.assert_one(targets.hash_tree_root[i].target);
            } else {
                builder.assert_zero(targets.hash_tree_root[i].target);
            }
        }

        builder.register_public_inputs(&targets.hash_tree_root.map(|x| x.target));

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        println!("public outputs {:?}", proof.public_inputs);

        data.verify(proof)
    }
}
