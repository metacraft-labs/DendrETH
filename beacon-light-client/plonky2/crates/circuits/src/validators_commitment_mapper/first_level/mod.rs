use crate::serializers::serde_bool_array_to_hex_string;
use crate::serializers::serde_bool_array_to_hex_string_nested;
use crate::utils::hashing::validator_hash_tree_root::hash_validator_sha256_or_zeroes;
use circuit::Circuit;
use circuit_derive::{CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};

use crate::{
    common_targets::{SSZLeafTarget, Sha256Target},
    utils::{
        hashing::validator_hash_tree_root_poseidon::{
            hash_validator_poseidon_or_zeroes, ValidatorTarget,
        },
        utils::{ssz_merklelize_bool, ssz_num_to_bits},
    },
};

#[derive(TargetPrimitive, PublicInputsReadable)]
pub struct MerklelizedValidatorTarget {
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub pubkey: [SSZLeafTarget; 2],
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials: SSZLeafTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub effective_balance: SSZLeafTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub slashed: SSZLeafTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub activation_eligibility_epoch: SSZLeafTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub activation_epoch: SSZLeafTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub exit_epoch: SSZLeafTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawable_epoch: SSZLeafTarget,
}

pub fn merklelize_validator_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorTarget,
) -> MerklelizedValidatorTarget {
    let zero_bits_128 = [BoolTarget::new_unsafe(builder.zero()); 128];

    let first_pubkey_leaf: SSZLeafTarget = (&validator.pubkey[0..256]).try_into().unwrap();
    let second_pubkey_leaf: SSZLeafTarget = [&validator.pubkey[256..], &zero_bits_128[..]]
        .concat()
        .try_into()
        .unwrap();

    MerklelizedValidatorTarget {
        pubkey: [first_pubkey_leaf, second_pubkey_leaf],
        withdrawal_credentials: validator.withdrawal_credentials,
        effective_balance: ssz_num_to_bits(builder, &validator.effective_balance, 64),
        slashed: ssz_merklelize_bool(builder, validator.slashed),
        activation_eligibility_epoch: ssz_num_to_bits(
            builder,
            &validator.activation_eligibility_epoch,
            64,
        ),
        activation_epoch: ssz_num_to_bits(builder, &validator.activation_epoch, 64),
        exit_epoch: ssz_num_to_bits(builder, &validator.exit_epoch, 64),
        withdrawable_epoch: ssz_num_to_bits(builder, &validator.withdrawable_epoch, 64),
    }
}

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorsCommitmentMapperTarget {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(in)]
    pub is_real: BoolTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: Sha256Target,

    #[target(out)]
    pub poseidon_hash_tree_root: HashOutTarget,
}

pub struct ValidatorsCommitmentMapperFirstLevel {}

impl Circuit for ValidatorsCommitmentMapperFirstLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = ValidatorsCommitmentMapperTarget;

    type Params = ();

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        _params: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let merklelized_validator = merklelize_validator_target(builder, &input.validator);
        let sha256_hash_tree_root =
            hash_validator_sha256_or_zeroes(builder, &merklelized_validator, input.is_real);

        let poseidon_hash_tree_root =
            hash_validator_poseidon_or_zeroes(builder, &input.validator, input.is_real);

        Self::Target {
            validator: input.validator,
            is_real: input.is_real,
            sha256_hash_tree_root,
            poseidon_hash_tree_root,
        }
    }
}

// #[cfg(test)]
// mod test {
//     use anyhow::Result;
//     use plonky2::{
//         field::goldilocks_field::GoldilocksField,
//         iop::witness::{PartialWitness, WitnessWrite},
//         plonk::{
//             circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
//             config::PoseidonGoldilocksConfig,
//         },
//     };
//
//     use crate::{
//         utils::utils::{SetBytesArray, ETH_SHA256_BIT_SIZE},
//         validators_commitment_mapper::validator_commitment_mapper::validator_commitment_mapper,
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
//         let targets = validator_commitment_mapper(&mut builder);
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
//         let mut pw = PartialWitness::new();
//
//         pw.set_bool_target(targets.validator_is_zero, false);
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
//                 builder.assert_one(targets.sha256_hash_tree_root[i].target);
//             } else {
//                 builder.assert_zero(targets.sha256_hash_tree_root[i].target);
//             }
//         }
//
//         builder.register_public_inputs(&targets.poseidon_hash_tree_root.elements);
//
//         builder.register_public_inputs(&targets.sha256_hash_tree_root.map(|x| x.target));
//
//         let data = builder.build::<C>();
//         let proof = data.prove(pw).unwrap();
//
//         data.verify(proof)
//     }
// }
