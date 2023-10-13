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
type BeaconStateLeafProof = ArrayVariable<Bytes32Variable, 5>;

#[derive(Debug, Clone)]
pub struct WeighJustificationAndFinalization;

fn verify_slot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    slot: Slot,
    proof: BeaconStateLeafProof,
) {
    let slot_leaf = slot.hash_tree_root(builder);
    let slot_index = Slot::constant(builder, 34);
    builder.ssz_verify_proof(beacon_state_root, slot_leaf, proof.as_slice(), slot_index);
}

#[allow(unused)]
fn compute_epoch_at_slot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) -> Epoch {
    let slots_per_epoch = builder.constant::<U64Variable>(32);
    builder.div(slot, slots_per_epoch)
}

#[allow(unused)]
fn get_current_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) -> Epoch {
    compute_epoch_at_slot(builder, slot)
}

#[allow(unused)]
fn get_previous_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) -> Epoch {
    let zero = builder.zero::<U64Variable>();
    let one = builder.one::<U64Variable>();

    let block_is_not_genesis = builder.gte(slot, one);
    let current_epoch = get_current_epoch(builder, slot);
    let previous_epoch = builder.sub(current_epoch, one);
    let previous_epoch = builder.select(block_is_not_genesis, previous_epoch, zero);
    compute_epoch_at_slot(builder, previous_epoch)
}
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

impl Circuit for WeighJustificationAndFinalization {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let beacon_state_root = builder.read::<Root>();

        let slot = builder.read::<U64Variable>();
        let slot_proof = builder.read::<BeaconStateLeafProof>();
        verify_slot(builder, beacon_state_root, slot, slot_proof);

        let previous_justified_checkpoint = builder.read::<CheckpointVariable>();
        let previous_justified_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        verify_previous_justified_checkpoint(
            builder,
            beacon_state_root,
            previous_justified_checkpoint,
            previous_justified_checkpoint_proof,
        );

        let current_justified_checkpoint = builder.read::<CheckpointVariable>();
        let current_justified_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        verify_current_justified_checkpoint(
            builder,
            beacon_state_root,
            current_justified_checkpoint,
            current_justified_checkpoint_proof,
        );

        let justification_bits = builder.read::<JustificationBitsVariable>();
        let justification_bits_proof = builder.read::<BeaconStateLeafProof>();
        verify_justification_bits(
            builder,
            beacon_state_root,
            justification_bits,
            justification_bits_proof,
        );

        // let total_active_balance = builder.read::<U64Variable>();
        // let previous_epoch_target_balance = builder.read::<U64Variable>();
        // let current_epoch_target_balance = builder.read::<U64Variable>();

        /*
        // Assert we are not in the genesis epoch
        // Maybe extract to another function get_current_epoch
        let current_epoch = slot.div(U64Variable::constant(builder, 32), builder);

        let prev_epoch = current_epoch.sub(U64Variable::constant(builder, 1), builder);

        // TODO: move to separate function
        let old_previous_justified_checkpoint = builder.read::<CheckpointVariable>();
        let old_previous_justified_checkpoint_proof =
            builder.read::<ArrayVariable<Bytes32Variable, 5>>();

        let old_previous_leaf = old_previous_justified_checkpoint.hash_tree_root(builder);
        let old_previous_justified_checkpoint_index = U64Variable::constant(builder, 50);

        builder.ssz_verify_proof(
            beacon_state,
            old_previous_leaf,
            old_previous_justified_checkpoint_proof.as_slice(),
            old_previous_justified_checkpoint_index,
        );

        let old_current_justified_checkpoint = builder.read::<U64Variable>();
        let old_current_justified_checkpoint_proof =
            builder.read::<ArrayVariable<Bytes32Variable, 5>>();

        let old_current_leaf = old_current_justified_checkpoint.hash_tree_root(builder);
        let old_current_justified_checkpoint_index = U64Variable::constant(builder, 51);

        builder.ssz_verify_proof(
            beacon_state,
            old_current_leaf,
            old_current_justified_checkpoint_proof.as_slice(),
            old_current_justified_checkpoint_index,
        );

        // TODO: implement
        //
        //   if previous_epoch_target_balance * 3 >= total_active_balance * 2:
        //        state.current_justified_checkpoint = Checkpoint(epoch=previous_epoch,
        //                                                        root=get_block_root(state, previous_epoch))
        //        state.justification_bits[1] = 0b1
        //   if current_epoch_target_balance * 3 >= total_active_balance * 2:
        //          state.current_justified_checkpoint = Checkpoint(epoch=current_epoch,
        //                                                          root=get_block_root(state, current_epoch))
        //          state.justification_bits[0] = 0b1

        let previous_epoch_justified_root = builder.read::<Bytes32Variable>();
        let previous_epoch_justified_root_proof =
            builder.read::<ArrayVariable<Bytes32Variable, 6>>();

        let previous_epoch_justified_checkpoint = CheckpointVariable {
            epoch: prev_epoch,
            root: previous_epoch_justified_root,
        };

        // let index = prev_epoch.rem(rhs, builder);

        // builder.ssz_verify_proof(root, leaf, branch, gindex)

        let current_epoch_justified_root = builder.read::<Bytes32Variable>();
        let current_epoch_justified_root_proof =
            builder.read::<ArrayVariable<Bytes32Variable, 6>>();

        let current_epoch_justified_checkpoint = CheckpointVariable {
            epoch: current_epoch,
            root: current_epoch_justified_root,
        };

        // Only if the above logic is true
        builder.write::<Bytes32Variable>(previous_epoch_justified_checkpoint.root);

        let justification_bits = builder.read::<ArrayVariable<BoolVariable, 4>>();
        let justification_bits_proof = builder.read::<ArrayVariable<Bytes32Variable, 5>>();

        let mut justification_bits_leaf = Bytes32Variable::constant(builder, bytes32!("0x0"));
        justification_bits_leaf.0.0[0] = ByteVariable([
            justification_bits[0],
            justification_bits[1],
            justification_bits[2],
            justification_bits[3],
            BoolVariable::constant(builder, false),
            BoolVariable::constant(builder, false),
            BoolVariable::constant(builder, false),
            BoolVariable::constant(builder, false),
        ]);

        let justification_bits_index = U64Variable::constant(builder, 49);

        builder.ssz_verify_proof(
            beacon_state,
            justification_bits_leaf,
            justification_bits_proof.as_slice(),
            justification_bits_index,
        );

        // TODO: implement
        // # The 2nd/3rd/4th most recent epochs are justified, the 2nd using the 4th as source
        // if all(bits[1:4]) and old_previous_justified_checkpoint.epoch + 3 == current_epoch:
        //     state.finalized_checkpoint = old_previous_justified_checkpoint
        // # The 2nd/3rd most recent epochs are justified, the 2nd using the 3rd as source
        // if all(bits[1:3]) and old_previous_justified_checkpoint.epoch + 2 == current_epoch:
        //     state.finalized_checkpoint = old_previous_justified_checkpoint
        // # The 1st/2nd/3rd most recent epochs are justified, the 1st using the 3rd as source
        // if all(bits[0:3]) and old_current_justified_checkpoint.epoch + 2 == current_epoch:
        //     state.finalized_checkpoint = old_current_justified_checkpoint
        // # The 1st/2nd most recent epochs are justified, the 1st using the 2nd as source
        // if all(bits[0:2]) and old_current_justified_checkpoint.epoch + 1 == current_epoch:
        //     state.finalized_checkpoint = old_current_justified_checkpoint

        builder.write::<Bytes32Variable>(current_epoch_justified_checkpoint.root);
        */
    }
}
