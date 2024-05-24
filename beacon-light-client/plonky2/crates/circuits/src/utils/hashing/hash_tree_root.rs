use itertools::Itertools;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    common_targets::{SSZLeafTarget, Sha256Target},
    utils::{
        hashing::sha256::sha256_pair,
        utils::{create_bool_target_array, ETH_SHA256_BIT_SIZE},
    },
};

use super::sha256::{make_circuits, Sha256Targets};

pub struct HashTreeRootTargets {
    pub leaves: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub hash_tree_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

pub fn hash_tree_root_new<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves: &[SSZLeafTarget],
) -> Sha256Target {
    assert!(leaves.len().is_power_of_two());

    let mut level = leaves.to_owned();

    while level.len() != 1 {
        level = level
            .iter()
            .tuples()
            .map(|(left, right)| sha256_pair(builder, left, right))
            .collect_vec();
    }

    level[0]
}

pub fn hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves_len: usize,
) -> HashTreeRootTargets {
    let leaves: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]> = (0..leaves_len)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let mut hashers: Vec<Sha256Targets> = Vec::new();

    for i in 0..(leaves_len / 2) {
        hashers.push(make_circuits(builder, 2 * ETH_SHA256_BIT_SIZE as u64));

        for j in 0..ETH_SHA256_BIT_SIZE {
            builder.connect(hashers[i].message[j].target, leaves[i * 2][j].target);
            builder.connect(
                hashers[i].message[j + 256].target,
                leaves[i * 2 + 1][j].target,
            );
        }
    }

    let mut k = 0;
    for i in leaves_len / 2..leaves_len - 1 {
        hashers.push(make_circuits(builder, 2 * ETH_SHA256_BIT_SIZE as u64));

        for j in 0..ETH_SHA256_BIT_SIZE {
            builder.connect(
                hashers[i].message[j].target,
                hashers[k * 2].digest[j].target,
            );
            builder.connect(
                hashers[i].message[j + ETH_SHA256_BIT_SIZE].target,
                hashers[k * 2 + 1].digest[j].target,
            );
        }

        k += 1;
    }

    HashTreeRootTargets {
        leaves,
        hash_tree_root: hashers[leaves_len - 2].digest.clone().try_into().unwrap(),
    }
}

#[cfg(test)]
mod tests {
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
        hashing::hash_tree_root::hash_tree_root,
        utils::{hash_bytes, SetBytesArray, ETH_SHA256_BIT_SIZE},
    };

    #[test]
    fn test_hash_tree_root() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = hash_tree_root(&mut builder, 4);

        let mut pw: PartialWitness<F> = PartialWitness::new();

        let first = hex::decode("67350f85c683777e5ec537cce1f6652302768c2636b85c6d8b1c44b67f981d2d")
            .unwrap();

        let second =
            hex::decode("35cd95976b68fad8d270a83a7a8092bb3f5a622508b3b6f97be8ede9eb03ddb2")
                .unwrap();

        let third = hex::decode("9dcc025d70596afc98d90e2aeaff64fb2f8cdc8cba67c743143a783627404734")
            .unwrap();

        let fourth =
            hex::decode("c3b6594b1b1343b69bc95a69190b85dadf776205921be8a1f3adbd39cc906f41")
                .unwrap();

        let hash_tree_root = vec![
            "1", "0", "0", "1", "0", "1", "0", "0", "0", "0", "1", "1", "1", "1", "0", "0", "1",
            "1", "1", "1", "1", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "1",
            "1", "1", "1", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "0", "1", "0", "1",
            "1", "1", "0", "0", "1", "1", "0", "1", "0", "1", "0", "1", "0", "1", "0", "1", "0",
            "1", "0", "1", "1", "1", "1", "0", "1", "0", "0", "1", "0", "1", "0", "1", "1", "1",
            "0", "1", "1", "0", "1", "0", "1", "1", "0", "1", "0", "0", "0", "1", "0", "0", "1",
            "1", "0", "0", "1", "0", "1", "0", "0", "1", "0", "1", "1", "0", "1", "1", "1", "1",
            "1", "0", "0", "1", "0", "0", "1", "0", "1", "1", "1", "0", "0", "0", "0", "1", "0",
            "0", "1", "1", "0", "1", "1", "1", "0", "0", "0", "0", "0", "1", "1", "0", "1", "0",
            "1", "0", "1", "1", "0", "1", "1", "1", "1", "1", "0", "1", "1", "0", "1", "1", "0",
            "0", "0", "0", "0", "1", "0", "0", "0", "0", "0", "1", "1", "1", "0", "1", "1", "1",
            "0", "0", "1", "0", "0", "0", "1", "1", "1", "0", "1", "1", "1", "1", "1", "1", "1",
            "0", "1", "0", "1", "1", "1", "1", "0", "1", "1", "0", "1", "1", "1", "1", "1", "1",
            "1", "0", "1", "1", "1", "1", "0", "0", "1", "1", "0", "0", "1", "0", "1", "0", "1",
            "1", "1", "0", "0", "0", "0", "0", "1", "0", "1", "1", "0", "1", "0", "0", "0", "1",
            "0",
        ];

        pw.set_bytes_array(&targets.leaves[0], &first);
        pw.set_bytes_array(&targets.leaves[1], &second);
        pw.set_bytes_array(&targets.leaves[2], &third);
        pw.set_bytes_array(&targets.leaves[3], &fourth);

        for i in 0..256 {
            if hash_tree_root[i] == "1" {
                builder.assert_one(targets.hash_tree_root[i].target);
            } else {
                builder.assert_zero(targets.hash_tree_root[i].target);
            }
        }

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    fn validators_hash_tree_root() -> Result<()> {
        let validator_pubkey =hex::decode("933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c9500000000000000000000000000000000").unwrap();
        let validator_pubkey = hash_bytes(&validator_pubkey);

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

        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = hash_tree_root(&mut builder, 8);

        let mut pw: PartialWitness<F> = PartialWitness::new();

        pw.set_bytes_array(&targets.leaves[0], &validator_pubkey);

        pw.set_bytes_array(&targets.leaves[1], &withdrawal_credentials);

        pw.set_bytes_array(&targets.leaves[2], &effective_balance);

        pw.set_bytes_array(&targets.leaves[3], &slashed);

        pw.set_bytes_array(&targets.leaves[4], &activation_eligibility_epoch);

        pw.set_bytes_array(&targets.leaves[5], &activation_epoch);

        pw.set_bytes_array(&targets.leaves[6], &exit_epoch);

        pw.set_bytes_array(&targets.leaves[7], &withdrawable_epoch);

        for i in 0..ETH_SHA256_BIT_SIZE {
            if validator_hash_tree_root[i] == "1" {
                builder.assert_one(targets.hash_tree_root[i].target);
            } else {
                builder.assert_zero(targets.hash_tree_root[i].target);
            }
        }

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }

    #[test]
    #[should_panic]
    fn test_hash_tree_root_failure() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = hash_tree_root(&mut builder, 4);

        let mut pw: PartialWitness<F> = PartialWitness::new();

        let first = hex::decode("67350f85c683777e5ec537cce1f6652302768c2636b85c6d8b1c44b67f981d2d")
            .unwrap();

        let second =
            hex::decode("35cd95976b68fad8d270a83a7a8092bb3f5a622508b3b6f97be8ede9eb03ddb2")
                .unwrap();

        let third = hex::decode("9dcc025d70596afc98d90e2aeaff64fb2f8cdc8cba67c743143a783627404734")
            .unwrap();

        let fourth =
            hex::decode("c3b6594b1b1343b69bc95a69190b85dadf776205921be8a1f3adbd39cc906f41")
                .unwrap();

        let hash_tree_root = vec![
            "1", "0", "0", "1", "0", "1", "0", "0", "0", "0", "1", "1", "1", "1", "0", "0", "1",
            "1", "1", "1", "1", "1", "0", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "1",
            "1", "1", "1", "1", "1", "0", "1", "1", "0", "0", "0", "1", "0", "0", "1", "0", "1",
            "1", "1", "0", "0", "1", "1", "0", "1", "0", "1", "0", "1", "0", "1", "0", "1", "0",
            "1", "0", "1", "1", "1", "1", "0", "1", "0", "0", "1", "0", "1", "0", "1", "1", "1",
            "0", "1", "1", "0", "1", "0", "1", "1", "0", "1", "0", "0", "0", "1", "0", "0", "1",
            "1", "0", "0", "1", "0", "1", "0", "0", "1", "0", "1", "1", "0", "1", "1", "1", "1",
            "1", "0", "0", "1", "0", "0", "1", "0", "1", "1", "1", "0", "0", "0", "0", "1", "0",
            "0", "1", "1", "0", "1", "1", "1", "0", "0", "0", "0", "0", "1", "1", "0", "1", "0",
            "1", "0", "1", "1", "0", "1", "1", "1", "1", "1", "0", "1", "1", "0", "1", "1", "0",
            "0", "0", "0", "0", "1", "0", "0", "0", "0", "0", "1", "1", "1", "0", "1", "1", "1",
            "0", "0", "1", "0", "0", "0", "1", "1", "1", "0", "1", "1", "1", "1", "1", "1", "1",
            "0", "1", "0", "1", "1", "1", "1", "0", "1", "1", "0", "1", "1", "1", "1", "1", "1",
            "1", "0", "1", "1", "1", "1", "0", "0", "1", "1", "0", "0", "1", "0", "1", "0", "1",
            "1", "1", "0", "0", "0", "0", "0", "1", "0", "1", "1", "0", "1", "0", "0", "0", "1",
            "1",
        ];

        pw.set_bytes_array(&targets.leaves[0], &first);
        pw.set_bytes_array(&targets.leaves[1], &second);
        pw.set_bytes_array(&targets.leaves[2], &third);
        pw.set_bytes_array(&targets.leaves[3], &fourth);

        for i in 0..ETH_SHA256_BIT_SIZE {
            if hash_tree_root[i] == "1" {
                builder.assert_one(targets.hash_tree_root[i].target);
            } else {
                builder.assert_zero(targets.hash_tree_root[i].target);
            }
        }

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof).expect("")
    }
}
