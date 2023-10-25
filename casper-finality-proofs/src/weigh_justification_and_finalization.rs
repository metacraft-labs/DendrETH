use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::Circuit,
    frontend::vars::SSZVariable,
    prelude::{
        ArrayVariable, BoolVariable, ByteVariable, Bytes32Variable, CircuitBuilder,
        CircuitVariable, PlonkParameters, U64Variable, Variable,
    },
};

type Epoch = U64Variable;
type Slot = U64Variable;
type Root = Bytes32Variable;
type Gwei = U64Variable;
type MerkleProof<const DEPTH: usize> = ArrayVariable<Bytes32Variable, DEPTH>;
type BeaconStateLeafProof = MerkleProof<5>;

const SLOTS_PER_EPOCH: u64 = 32;
const SLOTS_PER_HISTORICAL_ROOT: u64 = 8192;

#[derive(Debug, Clone)]
pub struct WeighJustificationAndFinalization;

fn verify_slot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    slot: Slot,
    proof: BeaconStateLeafProof,
) {
    let slot_leaf = slot.hash_tree_root(builder);
    let gindex = U64Variable::constant(builder, 34);
    builder.ssz_verify_proof(beacon_state_root, slot_leaf, proof.as_slice(), gindex);
}

fn compute_start_slot_at_epoch_in_block_roots<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    epoch: Epoch,
) -> Slot {
    let slots_per_epoch = builder.constant::<U64Variable>(SLOTS_PER_EPOCH);
    let slots_per_historical_root = builder.constant::<U64Variable>(SLOTS_PER_HISTORICAL_ROOT);
    let start_slot_at_epoch = builder.mul(epoch, slots_per_epoch);
    builder.rem(start_slot_at_epoch, slots_per_historical_root)
}

#[derive(Debug, Clone, CircuitVariable)]
#[value_name(CheckpointValue)]
pub struct CheckpointVariable {
    pub epoch: Epoch,
    pub root: Root,
}

impl SSZVariable for CheckpointVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> Bytes32Variable {
        let epoch_leaf = builder.ssz_hash_tree_root(self.epoch);
        let root_leaf = builder.ssz_hash_tree_root(self.root);
        builder.sha256_pair(epoch_leaf, root_leaf)
    }
}

#[derive(Debug, Clone, CircuitVariable)]
#[value_name(JustificationBitsValue)]
pub struct JustificationBitsVariable {
    pub bits: ArrayVariable<BoolVariable, 4>,
}

impl SSZVariable for JustificationBitsVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> Bytes32Variable {
        let zero_byte = builder.constant::<ByteVariable>(0);
        let zero_bit = builder.constant::<BoolVariable>(false);

        let first_byte = ByteVariable([
            zero_bit,
            zero_bit,
            zero_bit,
            zero_bit,
            self.bits[3],
            self.bits[2],
            self.bits[1],
            self.bits[0],
        ]);

        let mut justification_bits_vec = vec![first_byte];
        justification_bits_vec.extend(vec![zero_byte; 31]);
        let justification_bits_fixed_size: [ByteVariable; 32] =
            justification_bits_vec.try_into().unwrap();

        let justification_bits_leaf = Bytes32Variable::from(justification_bits_fixed_size);
        justification_bits_leaf
    }
}

fn verify_previous_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let checkpoint_leaf = builder.ssz_hash_tree_root(checkpoint);
    let gindex = builder.constant::<U64Variable>(50);
    builder.ssz_verify_proof(beacon_state_root, checkpoint_leaf, proof.as_slice(), gindex);
}

fn verify_current_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let checkpoint_leaf = builder.ssz_hash_tree_root(checkpoint);
    let gindex = builder.constant::<U64Variable>(51);
    builder.ssz_verify_proof(beacon_state_root, checkpoint_leaf, proof.as_slice(), gindex);
}

fn verify_justification_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    justification_bits: JustificationBitsVariable,
    proof: BeaconStateLeafProof,
) {
    let justification_bits_leaf = justification_bits.hash_tree_root(builder);
    let gindex = builder.constant::<U64Variable>(49);
    builder.ssz_verify_proof(
        beacon_state_root,
        justification_bits_leaf,
        proof.as_slice(),
        gindex,
    );
}

fn shift_justification_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    justification_bits: JustificationBitsVariable,
) -> JustificationBitsVariable {
    let mut new_bits = justification_bits.bits.as_vec();
    for i in 1..4 {
        new_bits[i] = new_bits[i - 1];
    }
    new_bits[0] = builder._false();

    JustificationBitsVariable {
        bits: ArrayVariable::new(new_bits),
    }
}

fn is_supermajority_link<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    target_balance: Gwei,
    total_active_balance: Gwei,
) -> BoolVariable {
    let three = builder.constant::<Gwei>(3);
    let two = builder.constant::<Gwei>(2);

    let target_balance_three_times = builder.mul(target_balance, three);
    let total_active_balance_two_times = builder.mul(total_active_balance, two);
    builder.gte(target_balance_three_times, total_active_balance_two_times)
}

fn set_first_justification_bit(
    justification_bits: JustificationBitsVariable,
    value: BoolVariable,
) -> JustificationBitsVariable {
    let mut new_bits = justification_bits.bits.as_vec();
    new_bits[0] = value;
    JustificationBitsVariable {
        bits: ArrayVariable::new(new_bits),
    }
}

fn determine_new_current_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_active_balance: Gwei,
    previous_epoch_target_balance: Gwei,
    current_epoch_target_balance: Gwei,
    justification_bits: JustificationBitsVariable,
    current_justified_checkpoint: &CheckpointVariable,
    current_epoch: Epoch,
    previous_epoch: Epoch, // probably don't need this
    previous_epoch_justified_checkpoint_root: Root,
    current_epoch_justified_checkpoint_root: Root,
) -> (CheckpointVariable, JustificationBitsVariable) {
    let previous_epoch_supermajority_link_pred =
        is_supermajority_link(builder, previous_epoch_target_balance, total_active_balance);
    let current_epoch_supermajority_link_pred =
        is_supermajority_link(builder, current_epoch_target_balance, total_active_balance);

    let previous_epoch_justified_checkpoint = CheckpointVariable {
        epoch: previous_epoch,
        root: previous_epoch_justified_checkpoint_root,
    };

    let current_epoch_justified_checkpoint = CheckpointVariable {
        epoch: current_epoch,
        root: current_epoch_justified_checkpoint_root,
    };

    let mut new_current_justified_checkpoint = builder.select(
        previous_epoch_supermajority_link_pred,
        previous_epoch_justified_checkpoint,
        current_justified_checkpoint.clone(),
    );

    new_current_justified_checkpoint = builder.select(
        current_epoch_supermajority_link_pred,
        current_epoch_justified_checkpoint,
        new_current_justified_checkpoint,
    );

    let current_justified_checkpoint_modified_pred = builder.and(
        previous_epoch_supermajority_link_pred,
        current_epoch_supermajority_link_pred,
    );

    let new_justification_bits = shift_justification_bits(builder, justification_bits);
    let new_justification_bits = set_first_justification_bit(
        new_justification_bits,
        current_justified_checkpoint_modified_pred,
    );

    (new_current_justified_checkpoint, new_justification_bits)
}

fn bits_test_range<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: &[BoolVariable],
    lower_bound: usize,
    upper_bound_non_inclusive: usize,
) -> BoolVariable {
    let mut result = builder._true();
    for i in lower_bound..upper_bound_non_inclusive {
        result = builder.and(result, bits[i]);
    }
    result
}

fn process_finalizations<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    justification_bits: JustificationBitsVariable,
    previous_justified_checkpoint: CheckpointVariable,
    current_justified_checkpoint: CheckpointVariable,
    current_epoch: Epoch,
    finalized_checkpoint: CheckpointVariable,
) -> CheckpointVariable {
    let one = builder.constant::<U64Variable>(1);
    let two = builder.constant::<U64Variable>(2);
    let three = builder.constant::<U64Variable>(3);

    let bits = justification_bits.bits.as_vec();
    let bits = bits.as_slice();

    let bits_set_1_through_4_pred = bits_test_range(builder, bits, 1, 4);
    let bits_set_1_through_3_pred = bits_test_range(builder, bits, 1, 3);
    let bits_set_0_through_3_pred = bits_test_range(builder, bits, 0, 3);
    let bits_set_0_through_2_pred = bits_test_range(builder, bits, 0, 2);

    let previous_justified_checkpoint_epoch_plus_three =
        builder.add(previous_justified_checkpoint.epoch, three);
    let previous_justified_checkpoint_epoch_plus_two =
        builder.add(previous_justified_checkpoint.epoch, two);
    let current_justified_checkpoint_epoch_plus_two =
        builder.add(current_justified_checkpoint.epoch, two);
    let current_justified_checkpoint_epoch_plus_one =
        builder.add(current_justified_checkpoint.epoch, one);

    let second_using_fourth_as_source_pred = builder.is_equal(
        previous_justified_checkpoint_epoch_plus_three,
        current_epoch,
    );

    let second_using_third_as_source_pred =
        builder.is_equal(previous_justified_checkpoint_epoch_plus_two, current_epoch);

    let first_using_third_as_source_pred =
        builder.is_equal(current_justified_checkpoint_epoch_plus_two, current_epoch);

    let first_using_second_as_source_pred =
        builder.is_equal(current_justified_checkpoint_epoch_plus_one, current_epoch);

    let should_finalize_previous_justified_checkpoint_1_pred = builder.and(
        bits_set_1_through_4_pred,
        second_using_fourth_as_source_pred,
    );
    let should_finalize_previous_justified_checkpoint_2_pred =
        builder.and(bits_set_1_through_3_pred, second_using_third_as_source_pred);
    let should_finalize_previous_justified_checkpoint_pred = builder.or(
        should_finalize_previous_justified_checkpoint_1_pred,
        should_finalize_previous_justified_checkpoint_2_pred,
    );

    let should_finalize_current_justified_checkpoint_1_pred =
        builder.and(bits_set_0_through_3_pred, first_using_third_as_source_pred);
    let should_finalize_current_justified_checkpoint_2_pred =
        builder.and(bits_set_0_through_2_pred, first_using_second_as_source_pred);
    let should_finalize_current_justified_checkpoint_pred = builder.or(
        should_finalize_current_justified_checkpoint_1_pred,
        should_finalize_current_justified_checkpoint_2_pred,
    );

    let mut new_finalized_checkpoint = builder.select(
        should_finalize_previous_justified_checkpoint_pred,
        previous_justified_checkpoint,
        finalized_checkpoint,
    );

    new_finalized_checkpoint = builder.select(
        should_finalize_current_justified_checkpoint_pred,
        current_justified_checkpoint,
        new_finalized_checkpoint,
    );

    new_finalized_checkpoint
}

fn verify_epoch_start_slot_root_in_block_roots<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    epoch: Epoch,
    block_root: Root,
    proof: MerkleProof<18>,
) {
    let first_block_roots_gindex = builder.constant::<U64Variable>(303104);
    let index_in_block_roots = compute_start_slot_at_epoch_in_block_roots(builder, epoch);
    let gindex = builder.add(first_block_roots_gindex, index_in_block_roots);
    builder.ssz_verify_proof(beacon_state_root, block_root, proof.as_slice(), gindex);
}

fn verify_finalized_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    finalized_checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let finalized_checkpoint_leaf = finalized_checkpoint.hash_tree_root(builder);
    let gindex = builder.constant::<U64Variable>(52);
    builder.ssz_verify_proof(
        beacon_state_root,
        finalized_checkpoint_leaf,
        proof.as_slice(),
        gindex,
    );
}

fn assert_epoch_is_not_genesis_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    epoch: Epoch,
) {
    let one = builder.one();
    let condition = builder.gte(epoch, one);
    let one = builder.one();
    builder.assert_is_equal(condition.variable, one);
}

fn calculate_current_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) -> Epoch {
    let slots_per_epoch = builder.constant::<U64Variable>(SLOTS_PER_EPOCH);
    builder.div(slot, slots_per_epoch)
}

impl Circuit for WeighJustificationAndFinalization {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let beacon_state_root = builder.read::<Root>();
        let slot = builder.read::<Slot>();
        let slot_proof = builder.read::<BeaconStateLeafProof>();
        let previous_justified_checkpoint = builder.read::<CheckpointVariable>();
        let previous_justified_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        let current_justified_checkpoint = builder.read::<CheckpointVariable>();
        let current_justified_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        let justification_bits = builder.read::<JustificationBitsVariable>();
        let justification_bits_proof = builder.read::<BeaconStateLeafProof>();
        let total_active_balance = builder.read::<Gwei>();
        let previous_epoch_target_balance = builder.read::<Gwei>();
        let current_epoch_target_balance = builder.read::<Gwei>();
        let previous_epoch_start_slot_root_in_block_roots = builder.read::<Root>();
        let previous_epoch_start_slot_root_in_block_roots_proof = builder.read::<MerkleProof<18>>();
        let current_epoch_start_slot_root_in_block_roots = builder.read::<Root>();
        let current_epoch_start_slot_root_in_block_roots_proof = builder.read::<MerkleProof<18>>();
        let finalized_checkpoint = builder.read::<CheckpointVariable>();
        let finalized_checkpoint_proof = builder.read::<BeaconStateLeafProof>();

        verify_slot(builder, beacon_state_root, slot, slot_proof);

        verify_previous_justified_checkpoint(
            builder,
            beacon_state_root,
            previous_justified_checkpoint.clone(),
            previous_justified_checkpoint_proof,
        );

        verify_current_justified_checkpoint(
            builder,
            beacon_state_root,
            current_justified_checkpoint.clone(),
            current_justified_checkpoint_proof,
        );

        verify_justification_bits(
            builder,
            beacon_state_root,
            justification_bits.clone(),
            justification_bits_proof,
        );

        let current_epoch = calculate_current_epoch(builder, slot);
        assert_epoch_is_not_genesis_epoch(builder, current_epoch);

        let one = builder.one();
        let previous_epoch = builder.sub(current_epoch, one);

        let new_previous_justified_checkpoint = current_justified_checkpoint.clone();

        verify_epoch_start_slot_root_in_block_roots(
            builder,
            beacon_state_root,
            previous_epoch,
            previous_epoch_start_slot_root_in_block_roots,
            previous_epoch_start_slot_root_in_block_roots_proof,
        );

        verify_epoch_start_slot_root_in_block_roots(
            builder,
            beacon_state_root,
            current_epoch,
            current_epoch_start_slot_root_in_block_roots,
            current_epoch_start_slot_root_in_block_roots_proof,
        );

        verify_finalized_checkpoint(
            builder,
            beacon_state_root,
            finalized_checkpoint.clone(),
            finalized_checkpoint_proof,
        );

        let (new_current_justified_checkpoint, new_justification_bits) =
            determine_new_current_justified_checkpoint(
                builder,
                total_active_balance,
                previous_epoch_target_balance,
                current_epoch_target_balance,
                justification_bits.clone(),
                &current_justified_checkpoint,
                current_epoch,
                previous_epoch,
                previous_epoch_start_slot_root_in_block_roots,
                current_epoch_start_slot_root_in_block_roots,
            );

        let new_finalized_checkpoint = process_finalizations(
            builder,
            justification_bits.clone(),
            previous_justified_checkpoint,
            current_justified_checkpoint,
            current_epoch,
            finalized_checkpoint.clone(),
        );

        builder.write::<CheckpointVariable>(new_previous_justified_checkpoint);
        builder.write::<CheckpointVariable>(new_current_justified_checkpoint);
        builder.write::<CheckpointVariable>(new_finalized_checkpoint);
        builder.write::<JustificationBitsVariable>(new_justification_bits);
    }
}
