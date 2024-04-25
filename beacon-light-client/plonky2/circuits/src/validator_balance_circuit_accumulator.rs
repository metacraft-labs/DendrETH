use itertools::{izip, Itertools};
use num::BigUint;
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
        assert_merkle_proof_is_valid, restore_merkle_root, MerkleBranch, Sha256,
    },
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{biguint_to_le_bits_target, create_bool_target_array, create_sha256_merkle_proof},
    validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
};

pub struct ValidatorStatusCountsTarget {
    pub active_validators_count: Target,
    pub exitted_validators_count: Target,
    pub not_activated_validators_count: Target,
}

impl ValidatorStatusCountsTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            active_validators_count: builder.zero(),
            exitted_validators_count: builder.zero(),
            not_activated_validators_count: builder.zero(),
        }
    }
}

impl ReadTargets for ValidatorStatusCountsTarget {
    fn read_targets(data: &mut Buffer) -> IoResult<Self> {
        Ok(Self {
            active_validators_count: data.read_target().unwrap(),
            exitted_validators_count: data.read_target().unwrap(),
            not_activated_validators_count: data.read_target().unwrap(),
        })
    }
}

impl WriteTargets for ValidatorStatusCountsTarget {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target(self.active_validators_count)?;
        data.write_target(self.exitted_validators_count)?;
        data.write_target(self.not_activated_validators_count)?;
        Ok(data)
    }
}

pub type BLSPubkey = [BoolTarget; 384];
pub type BLSSignature = [BoolTarget; 768];
pub type Bytes32 = [BoolTarget; 256];
pub type Gwei = BigUintTarget;

pub struct DepositDataTarget {
    pub pubkey: BLSPubkey,
    pub withdrawal_credentials: Bytes32,
    pub amount: Gwei,
    pub signature: BLSSignature,
}

impl DepositDataTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            pubkey: create_bool_target_array::<F, D, 384>(builder),
            withdrawal_credentials: create_bool_target_array::<F, D, 256>(builder),
            amount: builder.add_virtual_biguint_target(2),
            signature: create_bool_target_array::<F, D, 768>(builder),
        }
    }
}

impl ReadTargets for DepositDataTarget {
    fn read_targets(data: &mut Buffer) -> IoResult<Self> {
        // TODO: don't serialize the length
        let pubkey: BLSPubkey = data.read_target_bool_vec()?.try_into().unwrap();
        let withdrawal_credentials: Bytes32 = data.read_target_bool_vec()?.try_into().unwrap();
        let amount: Gwei = Gwei::read_targets(data)?;
        let signature: BLSSignature = data.read_target_bool_vec()?.try_into().unwrap();

        Ok(Self {
            pubkey,
            withdrawal_credentials,
            amount,
            signature,
        })
    }
}

impl WriteTargets for DepositDataTarget {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_target_bool_vec(&self.pubkey)?;
        data.write_target_bool_vec(&self.withdrawal_credentials)?;
        data.extend(Gwei::write_targets(&self.amount)?);
        data.write_target_bool_vec(&self.signature)?;

        Ok(data)
    }
}

// TODO: Reorder fields
pub struct ValidatorBalanceVerificationAccumulatorTargets {
    // Inputs
    pub balances_leaves: Vec<Sha256>,
    pub balances_root: Sha256,
    pub non_zero_validator_leaves_mask: Vec<BoolTarget>,
    // pub current_eth1_deposit_index: BigUintTarget,
    // pub validator_deposit_indexes: Vec<BigUintTarget>,
    pub balances_proofs: Vec<MerkleBranch<22>>,
    pub validators: Vec<ValidatorPoseidonTargets>,
    pub validator_indices: Vec<BigUintTarget>, // TODO: make this a Target vec / rename this to validator_gindices
    pub current_epoch: BigUintTarget,
    // pub range_start: Target,
    // pub range_end: Target,
    // pub range_deposit_count: Target,
    pub deposits_data: Vec<DepositDataTarget>,
    pub validators_poseidon_root: HashOutTarget,
    // pub validator_poseidon_proofs: HashOutTarget,

    // Outputs
    pub validators_commitment_in_range: HashOutTarget,
    pub validator_status_counts: ValidatorStatusCountsTarget,
    // pub accumulator_commitment_range_root: HashOutTarget, //
    pub range_total_balance: BigUintTarget,
}

impl ReadTargets for ValidatorBalanceVerificationAccumulatorTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<Self> {
        let validators_len = data.read_usize()?;

        Ok(Self {
            balances_leaves: (0..validators_len)
                .map(|_| {
                    data.read_target_bool_vec()
                        .expect("read target bool vec fails")
                        .try_into()
                        .expect("this fails")
                })
                .collect(),
            balances_root: data.read_target_bool_vec().unwrap().try_into().unwrap(),
            non_zero_validator_leaves_mask: data.read_target_bool_vec().unwrap(),
            balances_proofs: (0..validators_len)
                .map(|_| MerkleBranch::<DEPTH>::read_targets(data).unwrap())
                .collect_vec(),
            validators: (0..validators_len)
                .map(|_| {
                    ValidatorPoseidonTargets::read_targets(data)
                        .expect("ValidatorPoseidonTargets::read_targets failes")
                })
                .collect(),
            validator_indices: (0..validators_len)
                .map(|_| BigUintTarget::read_targets(data).unwrap())
                .collect_vec(),
            current_epoch: BigUintTarget::read_targets(data).unwrap(),
            deposits_data: (0..validators_len)
                .map(|_| DepositDataTarget::read_targets(data).unwrap())
                .collect_vec(),
            validators_poseidon_root: data.read_target_hash().unwrap(),
            validators_commitment_in_range: data.read_target_hash().unwrap(),
            validator_status_counts: ValidatorStatusCountsTarget::read_targets(data).unwrap(),
            range_total_balance: BigUintTarget::read_targets(data).unwrap(),
        })
    }
}

impl WriteTargets for ValidatorBalanceVerificationAccumulatorTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_usize(self.validators.len())?;

        for balance in &self.balances_leaves {
            data.write_target_bool_vec(balance)?;
        }

        data.write_target_bool_vec(&self.balances_root)?;

        data.write_target_bool_vec(&self.non_zero_validator_leaves_mask)?;

        for balances_proof in &self.balances_proofs {
            data.extend(&balances_proof.write_targets()?);
        }

        for validator in &self.validators {
            data.extend(ValidatorPoseidonTargets::write_targets(validator)?);
        }

        for gindex in &self.validator_indices {
            data.extend(gindex.write_targets()?);
        }

        data.extend(self.current_epoch.write_targets().unwrap());

        // TODO: encode validators_count in the circuit's serialized targets
        for deposit in &self.deposits_data {
            data.extend(deposit.write_targets()?);
        }

        data.write_target_hash(&self.validators_poseidon_root)?;

        // Outputs

        data.write_target_hash(&self.validators_commitment_in_range)?;

        data.extend(self.validator_status_counts.write_targets().unwrap());

        data.extend(self.range_total_balance.write_targets().unwrap());

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
    pub validator_indices: Vec<BigUintTarget>,
    pub current_epoch: BigUintTarget,
    pub deposits_data: Vec<DepositDataTarget>,
    pub validators_poseidon_root: HashOutTarget,
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

    let validator_indices: Vec<BigUintTarget> = (0..validators_count)
        .map(|_| builder.add_virtual_biguint_target(2)) // make this a target vector
        .collect_vec();

    let current_epoch = builder.add_virtual_biguint_target(2);

    let deposits_data = (0..validators_count)
        .map(|_| DepositDataTarget::new(builder))
        .collect_vec();

    let validators_poseidon_root = builder.add_virtual_hash();

    CircuitInputTargets {
        balances_leaves,
        balances_root,
        non_zero_validator_leaves_mask,
        balances_proofs,
        validators,
        validator_indices,
        current_epoch,
        deposits_data,
        validators_poseidon_root,
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
    let validator_hashes_targets = validators
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
        .flat_map(|hash| hash.elements)
        .collect_vec();

    hash_poseidon(builder, validator_hashes_targets)
}

struct ValidatorStatusTarget {
    is_active: BoolTarget,
    is_exitted: BoolTarget,
}

fn get_validator_status<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_epoch: &BigUintTarget,
    activation_epoch: &BigUintTarget,
    exit_epoch: &BigUintTarget,
) -> ValidatorStatusTarget {
    let current_gte_activation_epoch_pred = builder.gte_biguint(current_epoch, activation_epoch);
    let current_lt_exit_epoch_pred = builder.lt_biguint(current_epoch, exit_epoch);
    let is_active = builder.and(
        current_gte_activation_epoch_pred,
        current_lt_exit_epoch_pred,
    );

    let is_exitted = builder.gte_biguint(current_epoch, activation_epoch);

    ValidatorStatusTarget {
        is_active,
        is_exitted,
    }
}

fn increment_if_true<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    target: &mut Target,
    by: Target,
    predicate: BoolTarget,
) {
    let incr = builder.mul(by, predicate.target);
    *target = builder.add(*target, incr);
}

fn accumulate_validator_statuses<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_epoch: &BigUintTarget,
    validators: &[ValidatorPoseidonTargets],
    non_zero_validator_leaves_mask: &[BoolTarget],
) -> ValidatorStatusCountsTarget {
    let mut counts = ValidatorStatusCountsTarget::new(builder);

    for (validator, &is_non_zero_leaf) in validators.iter().zip(non_zero_validator_leaves_mask) {
        let validator_status = get_validator_status(
            builder,
            current_epoch,
            &validator.activation_epoch,
            &validator.exit_epoch,
        );

        let validator_is_active_or_exitted_pred =
            builder.or(validator_status.is_active, validator_status.is_exitted);

        let validator_is_not_active_or_exitted_pred =
            builder.not(validator_is_active_or_exitted_pred);

        counts.active_validators_count = builder.add(
            counts.active_validators_count,
            validator_status.is_active.target,
        );

        increment_if_true(
            builder,
            &mut counts.active_validators_count,
            validator_status.is_active.target,
            is_non_zero_leaf,
        );

        increment_if_true(
            builder,
            &mut counts.exitted_validators_count,
            validator_status.is_exitted.target,
            is_non_zero_leaf,
        );

        increment_if_true(
            builder,
            &mut counts.not_activated_validators_count,
            validator_is_not_active_or_exitted_pred.target,
            is_non_zero_leaf,
        );
    }

    counts
}

fn extract_balance_from_leaf<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256,
    offset: &BigUintTarget, // TODO: this doesn't need to be bigint
) -> BigUintTarget {
    let zero = builder.constant(F::from_canonical_u32(0));
    let one = builder.constant(F::from_canonical_u32(1));
    let two = builder.constant(F::from_canonical_u32(2));

    let const_64 = builder.constant_biguint(&BigUint::from(64u64));
    let range_begin = builder.mul_biguint(&const_64, offset);
    let range_end = builder.add_biguint(&range_begin, &const_64);

    let mut balance = builder.constant_biguint(&BigUint::from(0u64));
    let mut is_inside_range_pred = builder.zero();

    for i in 0..leaf.len() {
        let idx = builder.constant_biguint(&BigUint::from(i));
        let is_range_begin_pred = builder.eq_biguint(&idx, &range_begin);
        let is_range_end_pred = builder.eq_biguint(&idx, &range_end);

        is_inside_range_pred = builder.add(is_inside_range_pred, is_range_begin_pred.target);
        is_inside_range_pred = builder.sub(is_inside_range_pred, is_range_end_pred.target);

        let multiplier_target =
            builder.select(BoolTarget::new_unsafe(is_inside_range_pred), two, one);
        let multiplier = biguint_from_target(builder, multiplier_target);

        let addend_target = builder.select(BoolTarget::new_unsafe(is_inside_range_pred), one, zero);
        let addend = biguint_from_target(builder, addend_target);

        balance = builder.mul_add_biguint(&balance, &multiplier, &addend);
    }
    balance
}

fn biguint_from_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    target: Target,
) -> BigUintTarget {
    BigUintTarget {
        limbs: vec![U32Target(target), U32Target(builder.zero())],
    }
}

fn gindex_from_index_at_depth<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    index: &BigUintTarget,
    depth: u32,
) -> BigUintTarget {
    let first_leaf_gindex = builder.constant_biguint(&BigUint::from(2u64.pow(depth)));
    builder.add_biguint(&first_leaf_gindex, index)
}

fn prove_and_accumulate_balances<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves: &[Sha256],
    branches: &[MerkleBranch<22>],
    root: &Sha256,
    validator_indices: &[BigUintTarget],
) -> BigUintTarget {
    let mut total_balance = builder.constant_biguint(&BigUint::from(0u64));

    for (leaf, branch, validator_index) in izip!(leaves, branches, validator_indices) {
        let four = builder.constant_biguint(&BigUint::from(4u64));
        let validator_gindex = gindex_from_index_at_depth(builder, validator_index, 24);

        let balance_gindex = builder.div_biguint(&validator_gindex, &four);
        assert_merkle_proof_is_valid(builder, leaf, root, branch, &balance_gindex);

        let validator_balance_offset = builder.rem_biguint(&validator_index, &four);
        let balance = extract_balance_from_leaf(builder, leaf, &validator_balance_offset);
        total_balance = builder.add_biguint(&total_balance, &balance)
    }

    total_balance
}

pub fn validator_balance_verification_accumulator<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators_count: usize,
) -> ValidatorBalanceVerificationAccumulatorTargets {
    if !validators_count.is_power_of_two() {
        panic!("validators_count must be a power of two");
    }

    let input = read_input(builder, validators_count);

    // calculate hash tree root of validators input
    let validators_commitment_in_range = calc_validators_commitment(
        builder,
        &input.validators,
        &input.non_zero_validator_leaves_mask,
    );

    // count validators' statuses
    let validator_status_counts = accumulate_validator_statuses(
        builder,
        &input.current_epoch,
        &input.validators,
        &input.non_zero_validator_leaves_mask,
    );

    let range_total_balance = prove_and_accumulate_balances(
        builder,
        &input.balances_leaves,
        &input.balances_proofs,
        &input.balances_root,
        &input.validator_indices,
    );

    ValidatorBalanceVerificationAccumulatorTargets {
        balances_leaves: input.balances_leaves,
        balances_root: input.balances_root,
        non_zero_validator_leaves_mask: input.non_zero_validator_leaves_mask,
        balances_proofs: input.balances_proofs,
        validators: input.validators,
        validator_indices: input.validator_indices,
        current_epoch: input.current_epoch,
        deposits_data: input.deposits_data,
        validators_poseidon_root: input.validators_poseidon_root,
        validators_commitment_in_range,
        validator_status_counts,
        range_total_balance,
    }
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
