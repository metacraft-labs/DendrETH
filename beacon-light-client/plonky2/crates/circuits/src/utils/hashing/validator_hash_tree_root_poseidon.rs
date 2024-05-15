use crate::serializers::serde_bool_array_to_hex_string;
use circuit::add_virtual_target::AddVirtualTarget;
use circuit::public_inputs::field_reader::PublicInputsFieldReader;
use circuit::public_inputs::field_reader::PublicInputsReadable;
use circuit::public_inputs::target_reader::PublicInputsTargetReadable;
use circuit::public_inputs::target_reader::PublicInputsTargetReader;
use circuit::set_witness::SetWitness;
use circuit::target_primitive::TargetPrimitive;
use circuit::to_targets::ToTargets;
use circuit::SerdeCircuitTarget;
use circuit_derive::AddVirtualTarget;
use circuit_derive::SerdeCircuitTarget;
use circuit_derive::{PublicInputsReadable, SetWitness, TargetPrimitive};
use plonky2::iop::target::Target;
use plonky2::iop::witness::PartialWitness;
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
use serde::Deserialize;
use serde::Serialize;

use crate::{
    serialization::targets_serialization::{ReadTargets, WriteTargets},
    serializers::{biguint_to_str, parse_biguint},
    utils::{
        biguint::{BigUintTarget, CircuitBuilderBiguint},
        utils::{create_bool_target_array, ETH_SHA256_BIT_SIZE},
    },
};

use super::hash_tree_root_poseidon::hash_tree_root_poseidon;

#[derive(
    Clone,
    Debug,
    TargetPrimitive,
    SetWitness,
    PublicInputsReadable,
    AddVirtualTarget,
    SerdeCircuitTarget,
)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorTarget {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: [BoolTarget; 384],

    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials: [BoolTarget; ETH_SHA256_BIT_SIZE],

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub effective_balance: BigUintTarget,

    pub slashed: BoolTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub activation_eligibility_epoch: BigUintTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub activation_epoch: BigUintTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub exit_epoch: BigUintTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub withdrawable_epoch: BigUintTarget,
}

impl ReadTargets for ValidatorTarget {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorTarget> {
        Ok(ValidatorTarget {
            pubkey: data.read_target_bool_vec()?.try_into().unwrap(),
            withdrawal_credentials: data.read_target_bool_vec()?.try_into().unwrap(),
            effective_balance: BigUintTarget::read_targets(data)?,
            slashed: data.read_target_bool()?,
            activation_eligibility_epoch: BigUintTarget::read_targets(data)?,
            activation_epoch: BigUintTarget::read_targets(data)?,
            exit_epoch: BigUintTarget::read_targets(data)?,
            withdrawable_epoch: BigUintTarget::read_targets(data)?,
        })
    }
}

impl WriteTargets for ValidatorTarget {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_bool_vec(&self.pubkey)?;
        data.write_target_bool_vec(&self.withdrawal_credentials)?;
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

impl ValidatorTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> ValidatorTarget {
        let pubkey: [BoolTarget; 384] = (0..384)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        ValidatorTarget {
            pubkey,
            withdrawal_credentials: create_bool_target_array(builder),
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
    pub validator: ValidatorTarget,
    pub hash_tree_root: HashOutTarget,
}

pub fn hash_tree_root_validator_poseidon_new<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorPoseidonHashTreeRootTargets {
    let validator = ValidatorTarget::new(builder);

    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.pubkey.iter().map(|x| x.target).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawal_credentials
                .iter()
                .map(|x| x.target)
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

pub fn hash_validator_poseidon_or_zeroes<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorTarget,
    condition: BoolTarget,
) -> HashOutTarget {
    let validator_hash = hash_validator_poseidon(builder, validator);
    HashOutTarget {
        elements: validator_hash
            .elements
            .map(|element| builder.mul(condition.target, element)),
    }
}

pub fn hash_validator_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorTarget,
) -> HashOutTarget {
    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.pubkey.iter().map(|x| x.target).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawal_credentials
                .iter()
                .map(|x| x.target)
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

    hash_tree_root_poseidon.hash_tree_root
}

pub fn hash_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    targets: Vec<Target>,
) -> HashOutTarget {
    builder.hash_n_to_hash_no_pad::<PoseidonHash>(targets)
}

pub fn hash_tree_root_validator_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorPoseidonHashTreeRootTargets {
    let validator = ValidatorTarget::new(builder);

    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.pubkey.iter().map(|x| x.target).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawal_credentials
                .iter()
                .map(|x| x.target)
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
