use itertools::{izip, Itertools};
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    hash_tree_root_poseidon::hash_tree_root_poseidon,
    is_valid_merkle_branch::{
        assert_merkle_proof_is_valid, restore_merkle_root, validate_merkle_proof, MerkleBranch,
        Sha256,
    },
    sha256::{make_circuits, sha256, sha256_pair},
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{
        biguint_to_bits_target, biguint_to_le_bits_target, create_bool_target_array,
        create_sha256_merkle_proof,
    },
    validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
};

pub struct ValidatorBalanceVerificationTargetsAccumulator {
    // Inputs
    pub balances_leaves: Vec<Sha256>,
    pub balances_root: Sha256,
    pub non_zero_validator_leaves_mask: Vec<BoolTarget>,
    // pub current_eth1_deposit_index: BigUintTarget,
    // pub validator_deposit_indexes: Vec<BigUintTarget>,
    pub balances_proofs: Vec<MerkleBranch<22>>,
    pub validators: Vec<ValidatorPoseidonTargets>,
    pub validators_gindices: Vec<BigUintTarget>,
    // delete this
    // pub validators_commitment_root: HashOutTarget,
    // pub validator_commitment_proofs: Vec<Vec<HashOutTarget>>,
    // pub current_epoch: BigUintTarget,

    // Outputs
    pub validators_range_commitment: HashOutTarget,
    // pub accumulator_commitment_range_root: HashOutTarget, //
    // pub number_of_non_activated_validators: Target,
    // pub number_of_active_validators: Target,
    // pub number_of_exited_validators: Target,
    // pub range_total_value: BigUintTarget,
    // pub range_start: Target,
    // pub range_end: Target,
    // pub range_deposit_count: Target,
}

impl ReadTargets for ValidatorBalanceVerificationTargetsAccumulator {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorBalanceVerificationTargetsAccumulator> {
        let validators_len = data.read_usize()?;

        Ok(ValidatorBalanceVerificationTargetsAccumulator {
            /*
                        range_total_value: BigUintTarget::read_targets(data)?,
                        range_start: data.read_target()?,
                        range_end: data.read_target()?,
                        range_deposit_count: data.read_target()?,
                        balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            */
            balances_leaves: (0..validators_len)
                .map(|_| {
                    data.read_target_bool_vec()
                        .expect("read target bool vec fails")
                        .try_into()
                        .expect("this fails")
                })
                .collect(),
            balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            non_zero_validator_leaves_mask: data.read_target_bool_vec()?,
            balances_proofs: (0..validators_len)
                .map(|_| MerkleBranch::<DEPTH>::read_targets(data).unwrap())
                .collect_vec(),
            /*
                        balances_proofs: (0..validators_len)
                            .map(|_| [(); 22].map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap()))
                            .collect_vec(),
                        validator_commitment_root: data.read_target_hash()?,
                        accumulator_commitment_range_root: data.read_target_hash()?,
            */
            validators: (0..validators_len)
                .map(|_| {
                    ValidatorPoseidonTargets::read_targets(data)
                        .expect("ValidatorPoseidonTargets::read_targets failes")
                })
                .collect(),
            validators_gindices: (0..validators_len)
                .map(|_| BigUintTarget::read_targets(data).unwrap())
                .collect_vec(),
            validators_range_commitment: data.read_target_hash().unwrap(),
            /*
                        validator_deposit_indexes: (0..validators_len)
                            .map(|_| BigUintTarget::read_targets(data).unwrap())
                            .collect_vec(),
                        validator_indexes: data.read_target_vec()?,
                        validator_commitment_proofs: (0..validators_len)
                            .map(|_| {
                                (0..24)
                                    .map(|_| data.read_target_hash().unwrap())
                                    .collect_vec()
                            })
                            .collect_vec(),

                        current_epoch: BigUintTarget::read_targets(data)?,
                        current_eth1_deposit_index: BigUintTarget::read_targets(data)?,
                        number_of_non_activated_validators: data.read_target()?,
                        number_of_active_validators: data.read_target()?,
                        number_of_exited_validators: data.read_target()?,
            */
        })
    }
}

impl WriteTargets for ValidatorBalanceVerificationTargetsAccumulator {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_usize(self.validators.len())?;
        /*
                data.extend(BigUintTarget::write_targets(&self.range_total_value)?);
                data.write_target(self.range_start)?;
                data.write_target(self.range_end)?;
                data.write_target(self.range_deposit_count)?;
                data.write_target_bool_vec(&self.balances_root)?;
        */

        for balance in &self.balances_leaves {
            data.write_target_bool_vec(balance)?;
        }

        data.write_target_bool_vec(&self.balances_root)?;

        data.write_target_bool_vec(&self.non_zero_validator_leaves_mask)?;

        // self.balances_proofs.write_targets();
        for balances_proof in &self.balances_proofs {
            data.extend(&MerkleBranch::<DEPTH>::write_targets(balances_proof)?);
        }

        /*
                for balance_proof in &self.balances_proofs {
                    for element in balance_proof {
                        data.write_target_bool_vec(element)?;
                    }
                }

                data.write_target_hash(&self.validator_commitment_root)?;
                data.write_target_hash(&self.accumulator_commitment_range_root)?;
        */

        for validator in &self.validators {
            data.extend(ValidatorPoseidonTargets::write_targets(validator)?);
        }

        for gindex in &self.validators_gindices {
            data.extend(gindex.write_targets()?);
        }

        data.write_target_hash(&self.validators_range_commitment)?;

        /*

                for validator_deposit_index in &self.validator_deposit_indexes {
                    data.extend(BigUintTarget::write_targets(validator_deposit_index)?);
                }

                data.write_target_vec(&self.validator_indexes)?;

                for validator_proof in &self.validator_commitment_proofs {
                    for element in validator_proof {
                        data.write_target_hash(element)?;
                    }
                }


                data.extend(BigUintTarget::write_targets(&self.current_epoch)?);

                data.extend(BigUintTarget::write_targets(
                    &self.current_eth1_deposit_index,
                )?);

                data.write_target(self.number_of_non_activated_validators)?;
                data.write_target(self.number_of_active_validators)?;
                data.write_target(self.number_of_exited_validators)?;
        */

        Ok(data)
    }
}

const DEPTH: usize = 22;

struct CircuitInputTargets {
    pub balances_leaves: Vec<Sha256>,
    pub balances_root: Sha256,
    pub non_zero_validator_leaves_mask: Vec<BoolTarget>,
    pub balances_proofs: Vec<MerkleBranch<DEPTH>>,
    pub validators: Vec<ValidatorPoseidonTargets>,
    pub validators_gindices: Vec<BigUintTarget>,
}

fn read_input<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators_count: usize,
) -> CircuitInputTargets {
    let balances_leaves: Vec<Sha256> = (0..validators_count)
        .map(|_| create_bool_target_array(builder))
        .collect_vec();

    let balances_root: Sha256 = create_bool_target_array(builder);

    let non_zero_validator_leaves_mask = (0..validators_count)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect_vec();

    let balances_proofs: Vec<MerkleBranch<DEPTH>> = (0..validators_count)
        .map(|_| create_sha256_merkle_proof(builder))
        .collect_vec();

    let validators: Vec<ValidatorPoseidonTargets> = (0..validators_count)
        .map(|_| ValidatorPoseidonTargets::new(builder))
        .collect_vec();

    let validators_gindices: Vec<BigUintTarget> = (0..validators_count)
        .map(|_| builder.add_virtual_biguint_target(2))
        .collect_vec();

    CircuitInputTargets {
        balances_leaves,
        balances_root,
        non_zero_validator_leaves_mask,
        balances_proofs,
        validators,
        validators_gindices,
    }
}

fn hash_poseidon_validator<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorPoseidonTargets,
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

fn hash_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    targets: Vec<Target>,
) -> HashOutTarget {
    builder.hash_n_to_hash_no_pad::<PoseidonHash>(targets)
}

fn calc_validators_commitment<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators: &[ValidatorPoseidonTargets],
    non_zero_validator_leaves_mask: &[BoolTarget],
) -> HashOutTarget {
    let validator_hashes = validators
        .iter()
        .zip(non_zero_validator_leaves_mask)
        .map(|(validator, is_non_zero_leaf)| {
            HashOutTarget::from_vec(
                hash_poseidon_validator(builder, validator)
                    .elements
                    .iter()
                    .map(|&element| builder.mul(element, is_non_zero_leaf.target))
                    .collect_vec(),
            )
        })
        .collect_vec();

    let validator_targets = validator_hashes
        .iter()
        .flat_map(|hash| hash.elements)
        .collect_vec();

    hash_poseidon(builder, validator_targets)
}

pub fn validator_balance_accumulator_verification<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators_count: usize,
) -> ValidatorBalanceVerificationTargetsAccumulator {
    if !validators_count.is_power_of_two() {
        panic!("validators_count must be a power of two");
    }

    let input = read_input(builder, validators_count);

    let validators_range_commitment = calc_validators_commitment(
        builder,
        &input.validators,
        &input.non_zero_validator_leaves_mask,
    );

    for (leaf, proof, validator_gindex) in izip!(
        &input.balances_leaves,
        &input.balances_proofs,
        &input.validators_gindices,
    ) {
        let four = builder.constant_biguint(&BigUint::from(4u64));
        let balance_gindex = builder.div_biguint(validator_gindex, &four);
        assert_merkle_proof_is_valid(builder, leaf, &input.balances_root, proof, &balance_gindex);
    }

    return ValidatorBalanceVerificationTargetsAccumulator {
        balances_leaves: input.balances_leaves,
        balances_root: input.balances_root,
        non_zero_validator_leaves_mask: input.non_zero_validator_leaves_mask,
        balances_proofs: input.balances_proofs,
        validators: input.validators,
        validators_gindices: input.validators_gindices,
        validators_range_commitment,
    };
}

pub struct Targets {
    pub leaf: Sha256,
    pub proof: MerkleBranch<1>,
    pub gindex: BigUintTarget,
}

pub fn test_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> Targets {
    let leaf = create_bool_target_array(builder);
    let proof: MerkleBranch<1> = create_sha256_merkle_proof(builder);
    let gindex = builder.add_virtual_biguint_target(2);
    let root = restore_merkle_root(builder, &leaf, &proof, &gindex);

    let bits = biguint_to_le_bits_target::<F, D, 2>(builder, &gindex);

    builder.register_public_inputs(root.map(|bool_target| bool_target.target).as_slice());
    builder.register_public_inputs(
        bits.iter()
            .map(|bool_target| bool_target.target)
            .collect_vec()
            .as_slice(),
    );

    Targets {
        leaf,
        proof,
        gindex,
    }
}
