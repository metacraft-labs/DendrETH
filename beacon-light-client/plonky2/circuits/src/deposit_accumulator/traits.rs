use itertools::Itertools;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CommonCircuitData, VerifierOnlyCircuitData},
        config::PoseidonGoldilocksConfig,
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    targets_serialization::{ReadTargets, WriteTargets},
    types::DepositAccumulatorProofTarget,
    utils::create_bool_target_array,
    validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
};

use super::{
    deposit_hash_tree_root_poseidon::DepositPoseidonTargets,
    objects::{
        AccumulatedDataTargets, DepositAccumulatorLeafTargets, NodeTargets, RangeObject,
        ValidatorStatsTargets,
    },
};

pub trait DepositAccumulatorNodeTargetExt {
    fn get_node(&self) -> NodeTargets;
}

impl NodeTargets {
    pub fn new(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        NodeTargets {
            leftmost: RangeObject::new(builder),
            rightmost: RangeObject::new(builder),
            accumulated: AccumulatedDataTargets::new(builder),

            current_epoch: builder.add_virtual_biguint_target(2),
            eth1_deposit_index: builder.add_virtual_biguint_target(2),

            commitment_mapper_proof_root: builder.add_virtual_hash(),
            merkle_tree_deposit_branch_root: builder.add_virtual_hash(),
        }
    }
}

impl RangeObject {
    fn new(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        RangeObject {
            pubkey: (0..384)
                .map(|_| builder.add_virtual_bool_target_safe())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            deposit_index: builder.add_virtual_biguint_target(2),

            is_counted: builder.add_virtual_bool_target_safe(),
            is_dummy: builder.add_virtual_bool_target_safe(),
        }
    }
}

impl AccumulatedDataTargets {
    fn new(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        AccumulatedDataTargets {
            balance_sum: builder.add_virtual_biguint_target(2),
            deposits_count: builder.add_virtual_target(),
            validator_stats: ValidatorStatsTargets::new(builder),
        }
    }
}

impl ValidatorStatsTargets {
    fn new(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        ValidatorStatsTargets {
            non_activated_validators_count: builder.add_virtual_target(),
            active_validators_count: builder.add_virtual_target(),
            exited_validators_count: builder.add_virtual_target(),
            slashed_validators_count: builder.add_virtual_target(),
        }
    }
}

impl DepositAccumulatorLeafTargets {
    pub fn new(
        builder: &mut CircuitBuilder<GoldilocksField, 2>,
        bls_common_data: &CommonCircuitData<GoldilocksField, 2>,
        bls_verifier_data: &VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
    ) -> Self {
        DepositAccumulatorLeafTargets {
            validator: ValidatorPoseidonTargets::new(builder),
            validator_deposit: DepositPoseidonTargets::new(builder),
            commitment_mapper_proof: (0..22).map(|_| builder.add_virtual_hash()).collect(),
            validator_index: builder.add_virtual_target(),
            validator_deposit_proof: (0..22).map(|_| builder.add_virtual_hash()).collect(),
            validator_deposit_index: builder.add_virtual_target(),
            balance_tree_root: create_bool_target_array(builder),
            balance_leaf: create_bool_target_array(builder),
            balance_proof: (0..22)
                .map(|_| create_bool_target_array(builder))
                .collect_vec()
                .try_into()
                .unwrap(),
            bls_signature_proof: builder.add_virtual_proof_with_pis(bls_common_data),
            // bls_verifier_circuit_targets: builder.constant_verifier_data(bls_verifier_data),
            is_dummy: builder.add_virtual_bool_target_safe(),
            node: NodeTargets::new(builder),
        }
    }
}

impl ReadTargets for RangeObject {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(RangeObject {
            pubkey: data.read_target_bool_vec()?.try_into().unwrap(),
            deposit_index: BigUintTarget::read_targets(data)?,
            is_counted: data.read_target_bool()?,
            is_dummy: data.read_target_bool()?,
        })
    }
}

impl ReadTargets for AccumulatedDataTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(AccumulatedDataTargets {
            balance_sum: BigUintTarget::read_targets(data)?,
            deposits_count: data.read_target()?,
            validator_stats: ValidatorStatsTargets::read_targets(data)?,
        })
    }
}

impl ReadTargets for ValidatorStatsTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(ValidatorStatsTargets {
            non_activated_validators_count: data.read_target()?,
            active_validators_count: data.read_target()?,
            exited_validators_count: data.read_target()?,
            slashed_validators_count: data.read_target()?,
        })
    }
}

impl ReadTargets for NodeTargets {
    fn read_targets(data: &mut plonky2::util::serialization::Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(NodeTargets {
            leftmost: RangeObject::read_targets(data)?,
            rightmost: RangeObject::read_targets(data)?,
            accumulated: AccumulatedDataTargets::read_targets(data)?,
            current_epoch: BigUintTarget::read_targets(data)?,
            eth1_deposit_index: BigUintTarget::read_targets(data)?,
            commitment_mapper_proof_root: data.read_target_hash()?,
            merkle_tree_deposit_branch_root: data.read_target_hash()?,
        })
    }
}

impl ReadTargets for DepositAccumulatorLeafTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        Ok(DepositAccumulatorLeafTargets {
            validator: ValidatorPoseidonTargets::read_targets(data)?,
            validator_deposit: DepositPoseidonTargets::read_targets(data)?,
            commitment_mapper_proof: (0..22).map(|_| data.read_target_hash().unwrap()).collect(),
            validator_index: data.read_target()?,
            validator_deposit_proof: (0..22).map(|_| data.read_target_hash().unwrap()).collect(),
            validator_deposit_index: data.read_target()?,
            balance_tree_root: data.read_target_bool_vec()?.try_into().unwrap(),
            balance_leaf: data.read_target_bool_vec()?.try_into().unwrap(),
            balance_proof: (0..22)
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
                .collect_vec()
                .try_into()
                .unwrap(),
            bls_signature_proof: data.read_target_proof_with_public_inputs()?,
            is_dummy: data.read_target_bool()?,
            node: NodeTargets::read_targets(data)?,
        })
    }
}

impl WriteTargets for RangeObject {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_bool_vec(&self.pubkey)?;
        data.extend(BigUintTarget::write_targets(&self.deposit_index)?);
        data.write_target_bool(self.is_counted)?;

        Ok(data)
    }
}

// impl WriteTargets for DepositAccumulatorLeafTargets {
//     fn write_targets(&self) -> IoResult<Vec<u8>> {
//         let mut data = Vec::<u8>::new();

//         data.extend(ValidatorPoseidonTargets::write_targets(&self.validator)?);
//         data.extend(DepositPoseidonTargets::write_targets(
//             &self.validator_deposit,
//         )?);
//         data.write_target_hash(&self.commitment_mapper_hash_tree_root)?;
//         for proof in &self.commitment_mapper_proof {
//             data.write_target_hash(proof)?;
//         }

//         data.write_target(self.validator_index)?;
//         data.write_target_hash(&self.validator_deposit_root)?;

//         for proof in &self.validator_deposit_proof {
//             data.write_target_hash(proof)?;
//         }

//         data.write_target(self.validator_deposit_index)?;
//         data.write_target_bool_vec(&self.balance_tree_root)?;
//         data.write_target_bool_vec(&self.balance_leaf)?;

//         for proof in &self.balance_proof {
//             data.write_target_bool_vec(proof)?;
//         }

//         data.write_target_proof_with_public_inputs(&self.bls_signature_proof)?;
//         data.write_target_bool(self.is_dummy)?;
//         data.extend(BigUintTarget::write_targets(&self.eth1_deposit_index)?);
//         data.extend(BigUintTarget::write_targets(&self.current_epoch)?);
//         data.extend(RangeObject::write_targets(&self.left_most)?);
//         data.extend(RangeObject::write_targets(&self.right_most)?);

//         Ok(data)
//     }
// }

impl WriteTargets for ValidatorStatsTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target(self.non_activated_validators_count)?;
        data.write_target(self.active_validators_count)?;
        data.write_target(self.exited_validators_count)?;
        data.write_target(self.slashed_validators_count)?;

        Ok(data)
    }
}

impl WriteTargets for AccumulatedDataTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.extend(BigUintTarget::write_targets(&self.balance_sum)?);
        data.write_target(self.deposits_count)?;
        data.extend(ValidatorStatsTargets::write_targets(&self.validator_stats)?);

        Ok(data)
    }
}

impl WriteTargets for NodeTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.extend(RangeObject::write_targets(&self.leftmost)?);
        data.extend(RangeObject::write_targets(&self.rightmost)?);
        data.extend(AccumulatedDataTargets::write_targets(&self.accumulated)?);

        data.extend(BigUintTarget::write_targets(&self.current_epoch)?);
        data.extend(BigUintTarget::write_targets(&self.eth1_deposit_index)?);

        Ok(data)
    }
}

//LEFTMOST
pub const LEFTMOST_PUBKEY_INDEX: usize = 0; // Size of 384
pub const LEFTMOST_DEPOSIT_INDEX_INDEX: usize = 384; // Size of 2
pub const LEFTMOST_IS_COUNTED_INDEX: usize = 386; // Size of 1
pub const LEFTMOST_IS_DUMMY_INDEX: usize = 387; // Size of 1

//RIGHTMOST
pub const RIGHTMOST_PUBKEY_INDEX: usize = 388; // Size of 384
pub const RIGHTMOST_DEPOSIT_INDEX_INDEX: usize = 772; // Size of 2
pub const RIGHTMOST_IS_COUNTED_INDEX: usize = 774; // Size of 1
pub const RIGHTMOST_IS_DUMMY_INDEX: usize = 775; // Size of 1

//ACCUMULATED
pub const BALANCE_SUM_INDEX: usize = 776; // Size of 2
pub const DEPOSITS_COUNT: usize = 778; // Size of 1
pub const NON_ACTIVATED_VALIDATORS_COUNT: usize = 779; // Size of 1
pub const ACTIVE_VALIDATORS_COUNT: usize = 780; // Size of 1
pub const EXITED_VALIDATORS_COUNT: usize = 781; // Size of 1
pub const SLASHED_VALIDATORS_COUNT: usize = 782; // Size of 1

//CONSTS
pub const CURRENT_EPOCH_INDEX: usize = 783; // Size of 2
pub const ETH1_DEPOSIT_INDEX: usize = 785; // Size of 2
pub const COMMITMENT_MAPPER_PROOF_ROOT: usize = 787; // Size of 4
pub const MERKLE_TREE_DEPOSIT_BRANCH_ROOT: usize = 791; // Size of 4
pub const END: usize = 795; // End

//TODO: Review
impl DepositAccumulatorNodeTargetExt for DepositAccumulatorProofTarget {
    fn get_node(&self) -> NodeTargets {
        NodeTargets {
            leftmost: RangeObject {
                pubkey: self.public_inputs[LEFTMOST_PUBKEY_INDEX..LEFTMOST_DEPOSIT_INDEX_INDEX]
                    .iter()
                    .cloned()
                    .map(|x| BoolTarget::new_unsafe(x)) //TODO: Review is this safe?
                    .collect_vec()
                    .try_into()
                    .unwrap(),
                deposit_index: BigUintTarget {
                    limbs: self.public_inputs
                        [LEFTMOST_DEPOSIT_INDEX_INDEX..LEFTMOST_IS_COUNTED_INDEX]
                        .iter()
                        .cloned()
                        .map(|x| U32Target(x))
                        .collect_vec(),
                },
                is_counted: BoolTarget::new_unsafe(self.public_inputs[LEFTMOST_IS_COUNTED_INDEX]),
                is_dummy: BoolTarget::new_unsafe(self.public_inputs[LEFTMOST_IS_DUMMY_INDEX]),
            },
            rightmost: RangeObject {
                pubkey: self.public_inputs[RIGHTMOST_PUBKEY_INDEX..RIGHTMOST_DEPOSIT_INDEX_INDEX]
                    .iter()
                    .cloned()
                    .map(|x| BoolTarget::new_unsafe(x)) //TODO: Review is this safe?
                    .collect_vec()
                    .try_into()
                    .unwrap(),
                deposit_index: BigUintTarget {
                    limbs: self.public_inputs
                        [RIGHTMOST_DEPOSIT_INDEX_INDEX..RIGHTMOST_IS_COUNTED_INDEX]
                        .iter()
                        .cloned()
                        .map(|x| U32Target(x))
                        .collect_vec(),
                },
                is_counted: BoolTarget::new_unsafe(self.public_inputs[RIGHTMOST_IS_COUNTED_INDEX]),
                is_dummy: BoolTarget::new_unsafe(self.public_inputs[RIGHTMOST_IS_DUMMY_INDEX]),
            },
            accumulated: AccumulatedDataTargets {
                balance_sum: BigUintTarget {
                    limbs: self.public_inputs[BALANCE_SUM_INDEX..DEPOSITS_COUNT]
                        .iter()
                        .cloned()
                        .map(|x| U32Target(x))
                        .collect_vec(),
                },
                deposits_count: self.public_inputs[DEPOSITS_COUNT],
                validator_stats: ValidatorStatsTargets {
                    non_activated_validators_count: self.public_inputs
                        [NON_ACTIVATED_VALIDATORS_COUNT],
                    active_validators_count: self.public_inputs[ACTIVE_VALIDATORS_COUNT],
                    exited_validators_count: self.public_inputs[EXITED_VALIDATORS_COUNT],
                    slashed_validators_count: self.public_inputs[SLASHED_VALIDATORS_COUNT],
                },
            },

            current_epoch: BigUintTarget {
                limbs: self.public_inputs[CURRENT_EPOCH_INDEX..ETH1_DEPOSIT_INDEX]
                    .iter()
                    .cloned()
                    .map(|x| U32Target(x))
                    .collect_vec(),
            },

            eth1_deposit_index: BigUintTarget {
                limbs: self.public_inputs[ETH1_DEPOSIT_INDEX..COMMITMENT_MAPPER_PROOF_ROOT]
                    .iter()
                    .cloned()
                    .map(|x| U32Target(x))
                    .collect_vec(),
            },

            commitment_mapper_proof_root: HashOutTarget {
                elements: self.public_inputs
                    [COMMITMENT_MAPPER_PROOF_ROOT..MERKLE_TREE_DEPOSIT_BRANCH_ROOT]
                    .try_into()
                    .unwrap(),
            },

            merkle_tree_deposit_branch_root: HashOutTarget {
                elements: self.public_inputs[MERKLE_TREE_DEPOSIT_BRANCH_ROOT..END]
                    .try_into()
                    .unwrap(),
            },
        }
    }
}
