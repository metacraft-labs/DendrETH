use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};
use sha2::Sha256;

use crate::{
    hash_tree_root::hash_tree_root, sha256::{make_circuits, Sha256Targets}, targets_serialization::{ReadTargets, WriteTargets}, utils::{create_bool_target_array, ETH_SHA256_BIT_SIZE}
};

pub struct DepositShaTargets {
    pub pubkey: [BoolTarget; 384],
    pub deposit_index: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub signature: [BoolTarget; 768],
    pub deposit_message_hash_tree_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

impl ReadTargets for DepositShaTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<DepositShaTargets> {
        Ok(DepositShaTargets {
            pubkey: data.read_target_bool_vec()?.try_into().unwrap(),
            deposit_index: data.read_target_bool_vec()?.try_into().unwrap(),
            signature: data.read_target_bool_vec()?.try_into().unwrap(),
            deposit_message_hash_tree_root: data.read_target_bool_vec()?.try_into().unwrap(),
        })
    }
}

impl WriteTargets for DepositShaTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut bytes = Vec::<u8>::new();

        bytes.write_target_bool_vec(&self.pubkey)?;
        bytes.write_target_bool_vec(&self.deposit_index)?;
        bytes.write_target_bool_vec(&self.signature)?;
        bytes.write_target_bool_vec(&self.deposit_message_hash_tree_root)?;

        Ok(bytes)
    }
}

// pub fn compress_big_target_sha256<F: RichField + Extendable<D>, const D: usize>( //Stefan TODO: Finish
//     builder: &mut CircuitBuilder<F,D>,
//     big_target: Vec<BoolTarget>,
//     target_size: usize,
// ) ->  Sha256Targets {
//     assert_eq!(big_target.len(), target_size);

//     let next_divisible_by_256 = (target_size + 255) & !255;

// }

pub struct DepositHashTreeRootTargets {
    pub deposit: DepositShaTargets,
    pub hash_tree_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

pub fn hash_tree_root_deposit_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> DepositHashTreeRootTargets {
    let hash_tree_root = hash_tree_root(builder, 4);

    let pubkey: [BoolTarget; 384] = (0..384)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let pubkey_hasher = make_circuits(builder, 512);

    for i in 0..384 {
        builder.connect(pubkey_hasher.message[i].target, pubkey[i].target);
    }

    for i in 384..512 {
        let zero = builder._false();
        builder.connect(pubkey_hasher.message[i].target, zero.target);
    }

    let signature: [BoolTarget; 768] = (0..768)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let signature_hasher = make_circuits(builder, 768);

    for i in 0..768 {
        builder.connect(signature_hasher.message[i].target, pubkey[i].target);
    }

    let deposit_index = create_bool_target_array(builder);
    let deposit_message_hash_tree_root = create_bool_target_array(builder);

    for i in 0..ETH_SHA256_BIT_SIZE {
        builder.connect(hash_tree_root.leaves[0][i].target, pubkey_hasher.digest[i].target);

        builder.connect(
            hash_tree_root.leaves[1][i].target,
            deposit_index[i].target,
        );

        builder.connect(hash_tree_root.leaves[3][i].target, signature_hasher.digest[i].target);

        builder.connect(
            hash_tree_root.leaves[4][i].target,
            deposit_message_hash_tree_root[i].target,
        );

    }

    DepositHashTreeRootTargets {
        deposit: DepositShaTargets {
            pubkey,
            deposit_index,
            signature,
            deposit_message_hash_tree_root,
        },
        hash_tree_root: hash_tree_root.hash_tree_root,
    }
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

    use crate::{
        utils::{SetBytesArray, ETH_SHA256_BIT_SIZE},
        validator_hash_tree_root::hash_tree_root_validator_sha256,
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
