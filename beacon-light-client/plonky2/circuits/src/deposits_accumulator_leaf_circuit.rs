use std::ops::Range;

use itertools::Itertools;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{
            CommonCircuitData, VerifierCircuitData, VerifierCircuitTarget, VerifierOnlyCircuitData,
        },
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    deposit_hash_tree_root_poseidon::{
        self, hash_tree_root_deposit_poseidon, DepositPoseidonTargets,
    },
    hash_tree_root::hash_tree_root,
    hash_tree_root_poseidon::hash_tree_root_poseidon,
    is_active_validator::get_validator_status,
    is_valid_merkle_branch::{self, is_valid_merkle_branch_sha256, MerkleBranch, Sha256},
    is_valid_merkle_branch_poseidon::{
        is_valid_merkle_branch_poseidon, is_valid_merkle_branch_poseidon_result,
    },
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{
        bool_target_equal, create_bool_target_array, if_biguint, ssz_num_from_bits,
        ETH_SHA256_BIT_SIZE,
    },
    validator_hash_tree_root,
    validator_hash_tree_root_poseidon::{
        hash_tree_root_validator_poseidon, ValidatorPoseidonHashTreeRootTargets,
        ValidatorPoseidonTargets,
    },
};

pub struct RangeObject {
    pub pubkey: [BoolTarget; 384],
    pub deposit_index: BigUintTarget,
    pub balance_sum: BigUintTarget,
    pub non_activated_count: Target,
    pub active_count: Target,
    pub exited_count: Target,
    pub slashed_count: Target,
    pub is_counted: BoolTarget,
    pub is_dummy: BoolTarget,
}

impl RangeObject {
    pub fn new(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        todo!()
        // RangeObject {
        //     pubkey: (0..384)
        //         .map(|_| builder.add_virtual_bool_target_safe())
        //         .collect::<Vec<_>>()
        //         .try_into()
        //         .unwrap(),
        //     deposit_index: builder.add_virtual_biguint_target(2),
        //     balance_sum: builder.add_virtual_biguint_target(2),
        //     non_activated_count: builder.add_virtual_target(),
        //     active_count: builder.add_virtual_target(),
        //     exited_count: builder.add_virtual_target(),
        //     slashed_count: builder.add_virtual_target(),
        //     is_counted: builder.add_virtual_bool_target_safe(),
        // }
    }
}

impl ReadTargets for RangeObject {
    fn read_targets(data: &mut Buffer) -> IoResult<Self>
    where
        Self: Sized,
    {
        todo!()
        // Ok(RangeObject {
        //     pubkey: data.read_target_bool_vec()?.try_into().unwrap(),
        //     deposit_index: BigUintTarget::read_targets(data)?,
        //     balance_sum: BigUintTarget::read_targets(data)?,
        //     non_activated_count: data.read_target()?,
        //     active_count: data.read_target()?,
        //     exited_count: data.read_target()?,
        //     slashed_count: data.read_target()?,
        //     is_counted: data.read_target_bool()?,
        // })
    }
}

impl WriteTargets for RangeObject {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_bool_vec(&self.pubkey)?;
        data.extend(BigUintTarget::write_targets(&self.deposit_index)?);
        data.extend(BigUintTarget::write_targets(&self.balance_sum)?);
        data.write_target(self.non_activated_count)?;
        data.write_target(self.active_count)?;
        data.write_target(self.exited_count)?;
        data.write_target(self.slashed_count)?;
        data.write_target_bool(self.is_counted)?;

        Ok(data)
    }
}

pub struct DepositAccumulatorInputs {
    pub validator: ValidatorPoseidonTargets,
    pub validator_deposit: DepositPoseidonTargets,
    pub commitment_mapper_hash_tree_root: HashOutTarget,
    pub commitment_mapper_proof: Vec<HashOutTarget>,
    pub validator_index: Target,
    pub validator_deposit_root: HashOutTarget,
    pub validator_deposit_proof: Vec<HashOutTarget>,
    pub validator_deposit_index: Target,
    pub balance_tree_root: Sha256,
    pub balance_leaf: Sha256,
    pub balance_proof: MerkleBranch<22>,
    pub bls_signature_proof: ProofWithPublicInputsTarget<2>,
    pub current_epoch: BigUintTarget,
    pub is_dummy: bool,
    pub eth1_deposit_index: BigUintTarget,
}

impl DepositAccumulatorInputs {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct DepositAccumulatorOutputs {
    pub current_epoch: BigUintTarget,
    pub left_most: RangeObject,
    pub right_most: RangeObject,
}

pub struct DepositAccumulatorLeafTargets {
    pub validator: ValidatorPoseidonTargets,
    pub validator_deposit: DepositPoseidonTargets,
    pub commitment_mapper_hash_tree_root: HashOutTarget,
    pub commitment_mapper_proof: Vec<HashOutTarget>,
    pub validator_index: Target,
    pub validator_deposit_root: HashOutTarget,
    pub validator_deposit_proof: Vec<HashOutTarget>,
    pub validator_deposit_index: Target,
    pub balance_tree_root: Sha256,
    pub balance_leaf: Sha256,
    pub balance_proof: MerkleBranch<22>,
    pub bls_signature_proof: ProofWithPublicInputsTarget<2>,
    pub current_epoch: BigUintTarget,
    pub is_dummy: bool,
    pub eth1_deposit_index: BigUintTarget,
    pub left_most: RangeObject,
    pub right_most: RangeObject,
}

// impl ReadTargets for DepositAccumulatorLeafTargets {
//     fn read_targets(data: &mut Buffer) -> IoResult<Self>
//     where
//         Self: Sized,
//     {
//         Ok(DepositAccumulatorLeafTargets {
//             validator: ValidatorPoseidonTargets::read_targets(data)?,
//             validator_deposit: DepositPoseidonTargets::read_targets(data)?,
//             commitment_mapper_hash_tree_root: data.read_target_hash()?,
//             commitment_mapper_proof: (0..22).map(|_| data.read_target_hash().unwrap()).collect(),
//             validator_index: data.read_target()?,
//             validator_deposit_root: data.read_target_hash()?,
//             validator_deposit_proof: (0..22).map(|_| data.read_target_hash().unwrap()).collect(),
//             validator_deposit_index: data.read_target()?,
//             balance_tree_root: data.read_target_bool_vec()?.try_into().unwrap(),
//             balance_leaf: data.read_target_bool_vec()?.try_into().unwrap(),
//             balance_proof: (0..22)
//                 .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
//                 .collect_vec()
//                 .try_into()
//                 .unwrap(),
//             bls_signature_proof: data.read_target_proof_with_public_inputs()?,
//             is_dummy: data.read_target_bool()?,
//             eth1_deposit_index: BigUintTarget::read_targets(data)?,
//             current_epoch: BigUintTarget::read_targets(data)?,
//             left_most: RangeObject::read_targets(data)?,
//             right_most: RangeObject::read_targets(data)?,
//         })
//     }
// }

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

// impl DepositAccumulatorLeafTargets {
//     pub fn new(
//         builder: &mut CircuitBuilder<GoldilocksField, 2>,
//         bls_common_data: &CommonCircuitData<GoldilocksField, 2>,
//         bls_verifier_data: &VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
//     ) -> Self {
//         DepositAccumulatorLeafTargets {
//             validator: ValidatorPoseidonTargets::new(builder),
//             validator_deposit: DepositPoseidonTargets::new(builder),
//             commitment_mapper_hash_tree_root: builder.add_virtual_hash(),
//             commitment_mapper_proof: (0..22).map(|_| builder.add_virtual_hash()).collect(),
//             validator_index: builder.add_virtual_target(),
//             validator_deposit_root: builder.add_virtual_hash(),
//             validator_deposit_proof: (0..22).map(|_| builder.add_virtual_hash()).collect(),
//             validator_deposit_index: builder.add_virtual_target(),
//             balance_tree_root: create_bool_target_array(builder),
//             balance_leaf: create_bool_target_array(builder),
//             balance_proof: (0..22)
//                 .map(|_| create_bool_target_array(builder))
//                 .collect_vec()
//                 .try_into()
//                 .unwrap(),
//             bls_signature_proof: builder.add_virtual_proof_with_pis(bls_common_data),
//             bls_verifier_circuit_targets: builder.constant_verifier_data(bls_verifier_data),
//             is_dummy: builder.add_virtual_bool_target_safe(),
//             eth1_deposit_index: builder.add_virtual_biguint_target(2),
//             current_epoch: builder.add_virtual_biguint_target(2),
//             left_most: RangeObject::new(builder),
//             right_most: RangeObject::new(builder),
//         }
//     }
// }

pub fn deposit_accumulator_leaf_circuit(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    bls_common_data: &CommonCircuitData<GoldilocksField, 2>,
    bls_verifier_data: &VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
) -> DepositAccumulatorLeafTargets {
    let deposit_hash_tree_root = hash_tree_root_deposit_poseidon(builder);
    let is_valid_merkle_tree_deposit_branch = is_valid_merkle_branch_poseidon(builder, 32);

    builder.connect_hashes(
        is_valid_merkle_tree_deposit_branch.leaf,
        deposit_hash_tree_root.hash_tree_root,
    );

    // TODO: compute message from deposit_message_hash_tree_root
    let message = deposit_hash_tree_root
        .deposit
        .deposit_message_hash_tree_root;
    let bls_signature_proof = builder.add_virtual_proof_with_pis(bls_common_data);

    verify_bls_signature(
        builder,
        &deposit_hash_tree_root.deposit.pubkey,
        &deposit_hash_tree_root.deposit.signature,
        &message,
        bls_signature_proof,
        bls_common_data,
        bls_verifier_data,
    );

    let deposit_index = builder.add_virtual_biguint_target(2);
    let eth1_deposit_index = builder.add_virtual_biguint_target(2);
    let deposit_is_processed = builder.cmp_biguint(&deposit_index, &eth1_deposit_index);

    let signature_is_valid =
        BoolTarget::new_unsafe(*bls_signature_proof.public_inputs.last().unwrap());
    let validator_is_definitely_on_chain = builder.and(deposit_is_processed, signature_is_valid);

    let is_valid_commitment_mapper_proof = is_valid_merkle_branch_poseidon_result(builder, 41);
    let validator_hash_tree_root = hash_tree_root_validator_poseidon(builder);

    // connect that validators are the same
    let is_dummy = builder.add_virtual_bool_target_safe();
    let one = builder.one();
    for i in 0..384 {
        builder.connect(
            validator_hash_tree_root.validator.pubkey[i].target,
            deposit_hash_tree_root.deposit.pubkey[i].target,
        );

        // connect if is dummy pubkey is max
        let is_one = builder.is_equal(validator_hash_tree_root.validator.pubkey[i].target, one);
        builder.and(is_dummy, one);
    }

    builder.connect_hashes(
        validator_hash_tree_root.hash_tree_root,
        is_valid_commitment_mapper_proof.leaf,
    );

    builder.connect(
        is_valid_commitment_mapper_proof.is_valid.target,
        validator_is_definitely_on_chain.target,
    );

    let is_valid_merkle_branch_balances = is_valid_merkle_branch_sha256(builder, 22);

    let balance = builder.zero_biguint();

    let non_activated_count = builder.zero();
    let active_count = builder.zero();
    let exited_count = builder.zero();
    let slashed_count = builder.zero();

    DepositAccumulatorLeafTargets {
        validator: validator_hash_tree_root.validator,
        validator_deposit: deposit_hash_tree_root.deposit,
        commitment_mapper_hash_tree_root: is_valid_commitment_mapper_proof.root,
        commitment_mapper_proof: is_valid_commitment_mapper_proof.branch,
        validator_index: is_valid_commitment_mapper_proof.index,
        validator_deposit_root: is_valid_merkle_tree_deposit_branch.root,
        validator_deposit_proof: is_valid_merkle_tree_deposit_branch.branch,
        validator_deposit_index: is_valid_merkle_tree_deposit_branch.index,
        balance_tree_root: is_valid_merkle_branch_balances.root,
        balance_leaf: is_valid_merkle_branch_balances.leaf,
        balance_proof: is_valid_merkle_branch_balances.branch.try_into().unwrap(),
        bls_signature_proof: bls_signature_proof,
        current_epoch: (),
        is_dummy: (),
        eth1_deposit_index: (),
        left_most: RangeObject {
            pubkey: validator_hash_tree_root.validator.pubkey,
            deposit_index: deposit_hash_tree_root.deposit.deposit_index,
            balance_sum: balance,
            non_activated_count: non_activated_count,
            active_count: active_count,
            exited_count: exited_count,
            slashed_count: slashed_count,
            is_counted: validator_is_definitely_on_chain,
            is_dummy: is_dummy,
        },
        right_most: (),
    }
}

fn verify_bls_signature(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    pubkey: &[BoolTarget; 384],
    signature: &[BoolTarget; 768],
    message: &[BoolTarget; 256],
    bls_signature_proof: ProofWithPublicInputsTarget<2>,
    bls_common_data: &CommonCircuitData<GoldilocksField, 2>,
    bls_verifier_data: &VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
) {
    let bls_verifier_circuit_targets = builder.constant_verifier_data(bls_verifier_data);

    builder.verify_proof::<PoseidonGoldilocksConfig>(
        &bls_signature_proof,
        &bls_verifier_circuit_targets,
        bls_common_data,
    );

    for i in (0..384).step_by(8) {
        let byte = builder.le_sum(pubkey[i..(i + 8)].iter().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i])
    }

    for i in (0..768).step_by(8) {
        let byte = builder.le_sum(signature[i..(i + 8)].iter().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i + 384]);
    }

    for i in (0..256).step_by(8) {
        let byte = builder.le_sum(message[i..(i + 8)].iter().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i + 384 + 768]);
    }
}

// pub fn validator_balance_verification<
//     F: RichField + Extendable<D>,
//     const D: usize,
//     const N: usize,
// >(
//     builder: &mut CircuitBuilder<F, D>,
//     validators_len: usize,
// ) -> ValidatorBalanceVerificationTargets<N> {
//     if !validators_len.is_power_of_two() {
//         panic!("validators_len must be a power of two");
//     }

//     let balances_len = validators_len / 4;

//     let balances_leaves: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]> = (0..balances_len)
//         .map(|_| create_bool_target_array(builder))
//         .collect();

//     let balances_hash_tree_root_targets = hash_tree_root(builder, balances_len);

//     for i in 0..balances_len {
//         for j in 0..ETH_SHA256_BIT_SIZE {
//             builder.connect(
//                 balances_hash_tree_root_targets.leaves[i][j].target,
//                 balances_leaves[i][j].target,
//             );
//         }
//     }

//     let validators_leaves: Vec<ValidatorPoseidonHashTreeRootTargets> = (0..validators_len)
//         .map(|_| hash_tree_root_validator_poseidon(builder))
//         .collect();

//     let hash_tree_root_poseidon_targets = hash_tree_root_poseidon(builder, validators_len);

//     let validator_is_zero: Vec<BoolTarget> = (0..validators_len)
//         .map(|_| builder.add_virtual_bool_target_safe())
//         .collect();

//     let zero_hash = builder.zero();

//     for i in 0..validators_len {
//         let mut elements = [zero_hash; 4];

//         for (j, _) in validators_leaves[i]
//             .hash_tree_root
//             .elements
//             .iter()
//             .enumerate()
//         {
//             elements[j] = builder._if(
//                 validator_is_zero[i],
//                 zero_hash,
//                 validators_leaves[i].hash_tree_root.elements[j],
//             );
//         }

//         builder.connect_hashes(
//             hash_tree_root_poseidon_targets.leaves[i],
//             HashOutTarget { elements },
//         );
//     }
//     let mut withdrawal_credentials = [[BoolTarget::default(); ETH_SHA256_BIT_SIZE]; N];

//     for i in 0..N {
//         withdrawal_credentials[i] = create_bool_target_array(builder);
//     }

//     let current_epoch = builder.add_virtual_biguint_target(2);

//     let mut sum = builder.zero_biguint();

//     let mut number_of_non_activated_validators = builder.zero();

//     let mut number_of_active_validators = builder.zero();

//     let mut number_of_exited_validators = builder.zero();

//     let mut number_of_slashed_validators = builder.zero();

//     for i in 0..validators_len {
//         let mut is_equal = builder._false();

//         for j in 0..N {
//             let is_equal_inner = bool_target_equal(
//                 builder,
//                 &validators_leaves[i].validator.withdrawal_credentials,
//                 &withdrawal_credentials[j],
//             );

//             is_equal = builder.or(is_equal_inner, is_equal);
//         }

//         let balance = ssz_num_from_bits(
//             builder,
//             &balances_leaves[i / 4][((i % 4) * 64)..(((i % 4) * 64) + 64)],
//         );

//         let zero = builder.zero_biguint();

//         let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
//             get_validator_status(
//                 builder,
//                 &validators_leaves[i].validator.activation_epoch,
//                 &current_epoch,
//                 &validators_leaves[i].validator.exit_epoch,
//             );

//         let will_be_counted = builder.and(is_equal, is_valid_validator);

//         let current = if_biguint(builder, will_be_counted, &balance, &zero);

//         sum = builder.add_biguint(&sum, &current);

//         number_of_active_validators =
//             builder.add(number_of_active_validators, will_be_counted.target);

//         let will_be_counted = builder.and(is_equal, is_non_activated_validator);

//         number_of_non_activated_validators =
//             builder.add(number_of_non_activated_validators, will_be_counted.target);

//         let will_be_counted = builder.and(is_equal, is_exited_validator);

//         number_of_exited_validators =
//             builder.add(number_of_exited_validators, will_be_counted.target);

//         number_of_slashed_validators = builder.add(
//             number_of_slashed_validators,
//             validators_leaves[i].validator.slashed.target,
//         );

//         sum.limbs.pop();
//     }

//     ValidatorBalanceVerificationTargets {
//         validator_is_zero: validator_is_zero,
//         range_total_value: sum,
//         range_balances_root: balances_hash_tree_root_targets.hash_tree_root,
//         range_validator_commitment: hash_tree_root_poseidon_targets.hash_tree_root,
//         validators: validators_leaves
//             .iter()
//             .map(|v| v.validator.clone())
//             .collect(),
//         balances: balances_leaves,
//         withdrawal_credentials: withdrawal_credentials,
//         current_epoch,
//         number_of_non_activated_validators,
//         number_of_active_validators,
//         number_of_exited_validators,
//         number_of_slashed_validators,
//     }
// }
