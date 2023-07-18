use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_sha256::split_base::CircuitBuilderSplit;

use crate::{
    validator_hash_tree_root::{hash_tree_root_validator_sha256, Validator},
    validator_hash_tree_root_poseidon::{hash_tree_root_validator_poseidon, ValidatorPoseidon},
};
pub struct ValidatorCommitment {
    pub validator: Validator,
    pub sha256_hash_tree_root: [BoolTarget; 256],
    pub poseidon_hash_tree_root: HashOutTarget,
}

pub fn validator_commitment<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorCommitment {
    let hash_tree_root_sha256 = hash_tree_root_validator_sha256(builder);

    let validator_poseidon = hash_tree_root_validator_poseidon(builder);

    let validator = hash_tree_root_sha256.validator;

    let validator_poseidon_mapped = ValidatorPoseidon {
        pubkey: [
            builder.le_sum(validator.pubkey[0..63].iter()),
            builder.le_sum(validator.pubkey[63..126].iter()),
            builder.le_sum(validator.pubkey[126..189].iter()),
            builder.le_sum(validator.pubkey[189..252].iter()),
            builder.le_sum(validator.pubkey[252..315].iter()),
            builder.le_sum(validator.pubkey[315..378].iter()),
            builder.le_sum(validator.pubkey[378..381].iter()),
        ],
        withdrawal_credentials: [
            builder.le_sum(validator.withdrawal_credentials[0..63].iter()),
            builder.le_sum(validator.withdrawal_credentials[63..126].iter()),
            builder.le_sum(validator.withdrawal_credentials[126..189].iter()),
            builder.le_sum(validator.withdrawal_credentials[189..252].iter()),
            builder.le_sum(validator.withdrawal_credentials[252..256].iter()),
        ],
        activation_eligibility_epoch: [
            builder.le_sum(validator.activation_eligibility_epoch[0..63].iter()),
            builder.le_sum(validator.activation_eligibility_epoch[63..64].iter()),
        ],
        slashed: [builder.le_sum(validator.slashed[0..1].iter())],
        effective_balance: [
            builder.le_sum(validator.effective_balance[0..63].iter()),
            builder.le_sum(validator.effective_balance[63..64].iter()),
        ],
        activation_epoch: [
            builder.le_sum(validator.activation_epoch[0..63].iter()),
            builder.le_sum(validator.activation_epoch[63..64].iter()),
        ],
        exit_epoch: [
            builder.le_sum(validator.exit_epoch[0..63].iter()),
            builder.le_sum(validator.exit_epoch[63..64].iter()),
        ],
        withdrawable_epoch: [
            builder.le_sum(validator.withdrawable_epoch[0..63].iter()),
            builder.le_sum(validator.withdrawable_epoch[63..64].iter()),
        ],
    };

    for i in 0..7 {
        builder.connect(
            validator_poseidon.validator.pubkey[i],
            validator_poseidon_mapped.pubkey[i],
        );
    }

    for i in 0..5 {
        builder.connect(
            validator_poseidon.validator.withdrawal_credentials[i],
            validator_poseidon_mapped.withdrawal_credentials[i],
        )
    }

    builder.connect(
        validator_poseidon.validator.slashed[0],
        validator_poseidon_mapped.slashed[0],
    );

    for i in 0..2 {
        builder.connect(
            validator_poseidon.validator.activation_eligibility_epoch[i],
            validator_poseidon_mapped.activation_eligibility_epoch[i],
        );

        builder.connect(
            validator_poseidon.validator.effective_balance[i],
            validator_poseidon_mapped.effective_balance[i],
        );

        builder.connect(
            validator_poseidon.validator.activation_epoch[i],
            validator_poseidon_mapped.activation_epoch[i],
        );

        builder.connect(
            validator_poseidon.validator.exit_epoch[i],
            validator_poseidon_mapped.exit_epoch[i],
        );

        builder.connect(
            validator_poseidon.validator.withdrawable_epoch[i],
            validator_poseidon_mapped.withdrawable_epoch[i],
        );
    }

    ValidatorCommitment {
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

    use crate::validator_commitment::validator_commitment;

    #[test]
    fn test_validator_hash_tree_root() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = GoldilocksField;

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let targets = validator_commitment(&mut builder);

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

        for i in 0..256 {
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

        for i in 0..256 {
            if validator_hash_tree_root[i] == "1" {
                builder.assert_one(targets.sha256_hash_tree_root[i].target);
            } else {
                builder.assert_zero(targets.sha256_hash_tree_root[i].target);
            }
        }

        builder.register_public_inputs(&targets.sha256_hash_tree_root.map(|x| x.target));

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof)
    }
}
