use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    hash_tree_root_poseidon::hash_tree_root_poseidon,
    targets_serialization::{ReadTargets, WriteTargets},
};

#[derive(Clone, Debug)]
pub struct ValidatorPoseidonTargets {
    pub pubkey: BigUintTarget,                 // [BoolTarget; 384]
    pub withdrawal_credentials: BigUintTarget, // [BoolTarget; 256]
    pub effective_balance: BigUintTarget,      // u64
    pub slashed: BoolTarget,
    pub activation_eligibility_epoch: BigUintTarget, // u64
    pub activation_epoch: BigUintTarget,
    pub exit_epoch: BigUintTarget,
    pub withdrawable_epoch: BigUintTarget,
}

impl ReadTargets for ValidatorPoseidonTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorPoseidonTargets> {
        Ok(ValidatorPoseidonTargets {
            pubkey: BigUintTarget::read_targets(data)?,
            withdrawal_credentials: BigUintTarget::read_targets(data)?,
            effective_balance: BigUintTarget::read_targets(data)?,
            slashed: data.read_target_bool()?,
            activation_eligibility_epoch: BigUintTarget::read_targets(data)?,
            activation_epoch: BigUintTarget::read_targets(data)?,
            exit_epoch: BigUintTarget::read_targets(data)?,
            withdrawable_epoch: BigUintTarget::read_targets(data)?,
        })
    }
}

impl WriteTargets for ValidatorPoseidonTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.extend(BigUintTarget::write_targets(&self.pubkey)?);
        data.extend(BigUintTarget::write_targets(&self.withdrawal_credentials)?);
        data.extend(BigUintTarget::write_targets(&self.effective_balance)?);
        data.write_target_bool(self.slashed)?;
        data.extend(BigUintTarget::write_targets(
            &self.activation_eligibility_epoch,
        )?);
        data.extend(BigUintTarget::write_targets(&self.activation_epoch)?);
        data.extend(BigUintTarget::write_targets(&self.exit_epoch)?);
        data.extend(BigUintTarget::write_targets(&self.withdrawable_epoch)?);

        Ok(data)
    }
}

impl ValidatorPoseidonTargets {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> ValidatorPoseidonTargets {
        ValidatorPoseidonTargets {
            pubkey: builder.add_virtual_biguint_target(12),
            withdrawal_credentials: builder.add_virtual_biguint_target(8),
            effective_balance: builder.add_virtual_biguint_target(2),
            slashed: builder.add_virtual_bool_target_safe(),
            activation_eligibility_epoch: builder.add_virtual_biguint_target(2),
            activation_epoch: builder.add_virtual_biguint_target(2),
            exit_epoch: builder.add_virtual_biguint_target(2),
            withdrawable_epoch: builder.add_virtual_biguint_target(2),
        }
    }
}

pub struct ValidatorPoseidonHashTreeRootTargets {
    pub validator: ValidatorPoseidonTargets,
    pub hash_tree_root: HashOutTarget,
}

pub fn hash_tree_root_validator_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorPoseidonHashTreeRootTargets {
    let validator = ValidatorPoseidonTargets::new(builder);

    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.pubkey.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawal_credentials
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .effective_balance
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![validator.slashed.target]),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .activation_eligibility_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .activation_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.exit_epoch.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawable_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
    ];

    let hash_tree_root_poseidon = hash_tree_root_poseidon(builder, leaves.len());

    for i in 0..leaves.len() {
        builder.connect_hashes(leaves[i], hash_tree_root_poseidon.leaves[i]);
    }

    ValidatorPoseidonHashTreeRootTargets {
        validator,
        hash_tree_root: hash_tree_root_poseidon.hash_tree_root,
    }
}
