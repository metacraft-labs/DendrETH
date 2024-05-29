use crate::{
    serializers::serde_bool_array_to_hex_string,
    utils::circuit::{
        biguint_to_bits_target,
        hashing::{
            merkle::poseidon::hash_tree_root_poseidon, poseidon::hash_poseidon, sha256::sha256,
        },
        reverse_endianness,
    },
};
use circuit::{Circuit, ToTargets};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};

use crate::common_targets::{DepositTargets, Sha256Target};

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositCommitmentMapperTargets {
    #[target(in)]
    pub deposit: DepositTargets,

    #[target(in)]
    pub is_real: BoolTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: Sha256Target,

    #[target(out)]
    pub poseidon_hash_tree_root: HashOutTarget,
}

pub fn hash_tree_root_deposit_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    deposit: &DepositTargets,
) -> HashOutTarget {
    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            deposit.pubkey.iter().map(|x| x.target).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            deposit.deposit_index.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            deposit.signature.iter().map(|x| x.target).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            deposit
                .deposit_message_root
                .iter()
                .map(|x| x.target)
                .collect(),
        ),
    ];

    hash_tree_root_poseidon(builder, &leaves)
}

pub struct DepositsCommitmentMapperFirstLevel {}

impl Circuit for DepositsCommitmentMapperFirstLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = DepositCommitmentMapperTargets;

    type Params = ();

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        _params: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let deposit_index_bits = reverse_endianness(&biguint_to_bits_target::<Self::F, 2, 2>(
            builder,
            &input.deposit.deposit_index,
        ));

        let sha256_hash_tree_root = sha256(
            builder,
            &[
                input.deposit.pubkey.as_slice(),
                deposit_index_bits.as_slice(),
                input.deposit.deposit_message_root.as_slice(),
                input.deposit.signature.as_slice(),
            ]
            .concat(),
        )
        .map(|x| builder.and(x, input.is_real));

        let poseidon_hash_tree_root = HashOutTarget {
            elements: hash_poseidon(builder, input.deposit.to_targets())
                .elements
                .map(|x| builder.mul(x, input.is_real.target)),
        };

        Self::Target {
            deposit: input.deposit,
            is_real: input.is_real,
            sha256_hash_tree_root,
            poseidon_hash_tree_root,
        }
    }
}

#[cfg(test)]
mod test {
    use circuit::{Circuit, CircuitInput, SetWitness};
    use plonky2::{field::goldilocks_field::GoldilocksField, iop::witness::PartialWitness};

    use crate::utils::bits_to_bytes;

    use super::DepositsCommitmentMapperFirstLevel;

    #[test]

    fn test_deposit_hash_tree_root() {
        let (targets, circuit) = DepositsCommitmentMapperFirstLevel::build(&());

        let input =
            serde_json::from_str::<CircuitInput<DepositsCommitmentMapperFirstLevel>>(r#"{
                "deposit": {
                  "pubkey": "89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee",
                  "depositIndex": "830987",
                  "depositMessageRoot": "9600f1c137423bfb703c9373918cfc299d7e939d2b428ec3eaf4b266d3638ef9",
                  "signature": "af92ccc88c4b1eca2f7dffb7c9288c014b2dc358d4846037a71f22a7ebab387795fd88fd29ab6304e25021fae7d99e320b8f9cbf6a5809a9b61e6612a2c838cea8f90a2e90172f111d17c429215d61452ee341ab17915c415696531ff9a69fe8"
                },
                "isReal": true
              }
              "#).unwrap();

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &input);

        let proof = circuit.prove(pw).unwrap();
        let public_inputs =
            DepositsCommitmentMapperFirstLevel::read_public_inputs(&proof.public_inputs);

        let hex = hex::encode(bits_to_bytes(
            public_inputs.sha256_hash_tree_root.as_slice(),
        ));

        assert_eq!(
            hex,
            "7f5050ba723dc67c0eb4cb07508ae0f4b81de30c846a5ddda55c218a821edf12"
        );
    }

    #[test]
    fn test_deposit_hash_tree_root_not_real() {
        let (targets, circuit) = DepositsCommitmentMapperFirstLevel::build(&());

        let input =
            serde_json::from_str::<CircuitInput<DepositsCommitmentMapperFirstLevel>>(r#"{
                "deposit": {
                  "pubkey": "89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee",
                  "depositIndex": "830987",
                  "depositMessageRoot": "9600f1c137423bfb703c9373918cfc299d7e939d2b428ec3eaf4b266d3638ef9",
                  "signature": "af92ccc88c4b1eca2f7dffb7c9288c014b2dc358d4846037a71f22a7ebab387795fd88fd29ab6304e25021fae7d99e320b8f9cbf6a5809a9b61e6612a2c838cea8f90a2e90172f111d17c429215d61452ee341ab17915c415696531ff9a69fe8"
                },
                "isReal": false
              }
              "#).unwrap();

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &input);

        let proof = circuit.prove(pw).unwrap();
        let public_inputs =
            DepositsCommitmentMapperFirstLevel::read_public_inputs(&proof.public_inputs);

        let hex = hex::encode(bits_to_bytes(
            public_inputs.sha256_hash_tree_root.as_slice(),
        ));

        assert_eq!(
            hex,
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
