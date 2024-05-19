// #[cfg(test)]
// mod test {
//     use anyhow::Result;
//     use plonky2::iop::witness::{PartialWitness, WitnessWrite};
//
//     use crate::utils::utils::{SetBytesArray, ETH_SHA256_BIT_SIZE};
//
//     #[test]
//     fn test_validator_hash_tree_root() -> Result<()> {
//         let (validator_commitment, data) = build_commitment_mapper_first_level_circuit();
//
//         let mut pw = PartialWitness::new();
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
//         pw.set_bool_target(validator_commitment.validator_is_zero, false);
//
//         pw.set_bytes_array(&validator_commitment.validator.pubkey, &validator_pubkey);
//
//         pw.set_bytes_array(
//             &validator_commitment.validator.withdrawal_credentials,
//             &withdrawal_credentials,
//         );
//
//         pw.set_bytes_array(
//             &validator_commitment.validator.effective_balance,
//             &effective_balance,
//         );
//
//         pw.set_bytes_array(&validator_commitment.validator.slashed, &slashed);
//
//         pw.set_bytes_array(
//             &validator_commitment.validator.activation_eligibility_epoch,
//             &activation_eligibility_epoch,
//         );
//
//         pw.set_bytes_array(
//             &validator_commitment.validator.activation_epoch,
//             &activation_epoch,
//         );
//
//         pw.set_bytes_array(&validator_commitment.validator.exit_epoch, &exit_epoch);
//
//         pw.set_bytes_array(
//             &validator_commitment.validator.withdrawable_epoch,
//             &withdrawable_epoch,
//         );
//
//         let proof = data.prove(pw).unwrap();
//
//         for i in 0..ETH_SHA256_BIT_SIZE {
//             assert_eq!(
//                 proof.public_inputs[i + 4].to_string(),
//                 validator_hash_tree_root[i].to_string()
//             )
//         }
//
//         Ok(())
//     }
// }
