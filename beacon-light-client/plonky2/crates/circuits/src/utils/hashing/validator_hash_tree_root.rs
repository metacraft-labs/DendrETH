use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    common_targets::Sha256Target,
    validators_commitment_mapper::first_level::MerklelizedValidatorTarget,
};

use super::{hash_tree_root::hash_tree_root_new, sha256::sha256_pair};
pub fn hash_validator_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &MerklelizedValidatorTarget,
) -> Sha256Target {
    let leaves = vec![
        sha256_pair(builder, &validator.pubkey[0], &validator.pubkey[1]),
        validator.withdrawal_credentials,
        validator.effective_balance,
        validator.slashed,
        validator.activation_eligibility_epoch,
        validator.activation_epoch,
        validator.exit_epoch,
        validator.withdrawable_epoch,
    ];

    hash_tree_root_new(builder, &leaves)
}

pub fn hash_validator_sha256_or_zeroes<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &MerklelizedValidatorTarget,
    condition: BoolTarget,
) -> Sha256Target {
    let validator_hash = hash_validator_sha256(builder, validator);
    validator_hash.map(|bit| builder.and(condition, bit))
}

// #[cfg(test)]
// mod test {
//     use std::println;
//
//     use anyhow::Result;
//     use plonky2::{
//         field::goldilocks_field::GoldilocksField,
//         iop::witness::PartialWitness,
//         plonk::{
//             circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
//             config::PoseidonGoldilocksConfig,
//         },
//     };
//
//     use crate::utils::{
//         hashing::validator_hash_tree_root::hash_tree_root_validator_sha256,
//         utils::{SetBytesArray, ETH_SHA256_BIT_SIZE},
//     };
//
//     #[test]
//     fn test_validator_hash_tree_root() -> Result<()> {
//         const D: usize = 2;
//         type C = PoseidonGoldilocksConfig;
//         type F = GoldilocksField;
//
//         let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());
//
//         let targets = hash_tree_root_validator_sha256(&mut builder, leaves);
//
//         let mut pw = PartialWitness::new();
//
//         let input = r#"
//             {
//                 pubkey: "933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95",
//                 effective_balance:
//                 slashed: false,
//             }
//         "#;
//
//         let validator_pubkey =hex::decode("933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95").unwrap();
//         let withdrawal_credentials =
//             hex::decode("0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50")
//                 .unwrap();
//
//         let effective_balance =
//             hex::decode("0040597307000000000000000000000000000000000000000000000000000000")
//                 .unwrap();
//
//         let slashed =
//             hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
//                 .unwrap();
//
//         let activation_eligibility_epoch =
//             hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
//                 .unwrap();
//
//         let activation_epoch =
//             hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
//                 .unwrap();
//
//         let exit_epoch =
//             hex::decode("ffffffffffffffff000000000000000000000000000000000000000000000000")
//                 .unwrap();
//
//         let withdrawable_epoch =
//             hex::decode("ffffffffffffffff000000000000000000000000000000000000000000000000")
//                 .unwrap();
//
//         let validator_hash_tree_root = [
//             "0", "0", "1", "0", "1", "0", "1", "1", "1", "0", "1", "0", "1", "1", "1", "1", "0",
//             "1", "0", "0", "0", "0", "0", "0", "0", "1", "1", "0", "0", "1", "0", "1", "1", "0",
//             "1", "1", "0", "1", "0", "1", "1", "1", "0", "1", "0", "1", "1", "0", "0", "0", "1",
//             "0", "0", "1", "0", "0", "0", "1", "1", "0", "0", "1", "0", "0", "0", "0", "0", "1",
//             "0", "0", "0", "0", "0", "1", "0", "1", "0", "0", "0", "1", "1", "0", "0", "0", "1",
//             "1", "0", "0", "0", "1", "1", "1", "1", "0", "0", "1", "1", "0", "0", "0", "0", "0",
//             "0", "1", "1", "1", "1", "0", "0", "1", "0", "1", "0", "1", "0", "1", "0", "0", "0",
//             "0", "0", "1", "1", "1", "1", "1", "0", "0", "1", "1", "1", "0", "1", "0", "0", "0",
//             "0", "0", "1", "0", "1", "1", "0", "1", "0", "1", "0", "0", "0", "1", "1", "0", "1",
//             "1", "0", "1", "1", "0", "0", "0", "0", "1", "1", "1", "1", "1", "1", "1", "1", "0",
//             "0", "0", "0", "0", "0", "0", "1", "0", "0", "1", "1", "0", "0", "1", "1", "1", "0",
//             "1", "1", "1", "1", "1", "0", "1", "0", "1", "0", "0", "1", "0", "1", "1", "0", "0",
//             "0", "0", "1", "1", "1", "0", "0", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0",
//             "0", "1", "1", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "1", "0",
//             "1", "1", "0", "0", "0", "0", "1", "1", "1", "1", "1", "1", "0", "1", "0", "1", "0",
//             "1",
//         ];
//
//         pw.set_bytes_array(&targets.validator.pubkey, &validator_pubkey);
//
//         pw.set_bytes_array(
//             &targets.validator.withdrawal_credentials,
//             &withdrawal_credentials,
//         );
//
//         pw.set_bytes_array(
//             &targets.validator.activation_eligibility_epoch,
//             &activation_eligibility_epoch,
//         );
//
//         pw.set_bytes_array(&targets.validator.activation_epoch, &activation_epoch);
//
//         pw.set_bytes_array(&targets.validator.slashed, &slashed);
//
//         pw.set_bytes_array(&targets.validator.effective_balance, &effective_balance);
//
//         pw.set_bytes_array(&targets.validator.exit_epoch, &exit_epoch);
//
//         pw.set_bytes_array(&targets.validator.withdrawable_epoch, &withdrawable_epoch);
//
//         for i in 0..ETH_SHA256_BIT_SIZE {
//             if validator_hash_tree_root[i] == "1" {
//                 builder.assert_one(targets.hash_tree_root[i].target);
//             } else {
//                 builder.assert_zero(targets.hash_tree_root[i].target);
//             }
//         }
//
//         builder.register_public_inputs(&targets.hash_tree_root.map(|x| x.target));
//
//         let data = builder.build::<C>();
//         let proof = data.prove(pw).unwrap();
//
//         println!("public outputs {:?}", proof.public_inputs);
//
//         data.verify(proof)
//     }
// }
