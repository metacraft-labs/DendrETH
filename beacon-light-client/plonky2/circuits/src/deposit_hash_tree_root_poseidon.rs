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
    utils::ETH_SHA256_BIT_SIZE,
};

#[derive(Clone, Debug)]
pub struct DepositPoseidonTargets {
    pub pubkey: [BoolTarget; 384],
    pub deposit_index: BigUintTarget,
    pub signature: [BoolTarget; 768],
    pub deposit_message_hash_tree_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

impl ReadTargets for DepositPoseidonTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<DepositPoseidonTargets> {
        Ok(DepositPoseidonTargets {
            pubkey: data.read_target_bool_vec()?.try_into().unwrap(),
            deposit_index: BigUintTarget::read_targets(data)?,
            signature: data.read_target_bool_vec()?.try_into().unwrap(),
            deposit_message_hash_tree_root: data.read_target_bool_vec()?.try_into().unwrap(),
        })
    }
}

impl WriteTargets for DepositPoseidonTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_bool_vec(&self.pubkey)?;
        data.extend(BigUintTarget::write_targets(&self.deposit_index)?);
        data.write_target_bool_vec(&self.signature)?;
        data.write_target_bool_vec(&self.deposit_message_hash_tree_root)?;

        Ok(data)
    }
}

impl DepositPoseidonTargets {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> DepositPoseidonTargets {
        let pubkey: [BoolTarget; 384] = (0..384)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let signature: [BoolTarget; 768] = (0..768)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let deposit_message_hash_tree_root: [BoolTarget; ETH_SHA256_BIT_SIZE] = (0..384)
            .map(|_| builder.add_virtual_bool_target_safe())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        DepositPoseidonTargets {
            pubkey: pubkey,
            deposit_index: builder.add_virtual_biguint_target(2),
            signature: signature,
            deposit_message_hash_tree_root: deposit_message_hash_tree_root,
        }
    }
}

pub struct DepositPoseidonHashTreeRootTargets {
    pub deposit: DepositPoseidonTargets,
    pub hash_tree_root: HashOutTarget,
}

pub fn hash_tree_root_deposit_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> DepositPoseidonHashTreeRootTargets {
    let deposit = DepositPoseidonTargets::new(builder);

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
                .deposit_message_hash_tree_root
                .iter()
                .map(|x| x.target)
                .collect(),
        ),
    ];

    let hash_tree_root_poseidon = hash_tree_root_poseidon(builder, leaves.len());

    for i in 0..leaves.len() {
        builder.connect_hashes(leaves[i], hash_tree_root_poseidon.leaves[i]);
    }

    DepositPoseidonHashTreeRootTargets {
        deposit,
        hash_tree_root: hash_tree_root_poseidon.hash_tree_root,
    }
}
