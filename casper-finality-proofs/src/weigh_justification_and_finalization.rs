use std::io::Bytes;

use plonky2x::{
    backend::circuit::Circuit,
    frontend::vars::SSZVariable,
    prelude::{
        ArrayVariable, BoolVariable, ByteVariable, Bytes32Variable, CircuitBuilder,
        CircuitVariable, Div, PlonkParameters, Sub, U64Variable, Rem,
    },
    utils::bytes32,
};

use crate::checkpoint::CheckpointVariable;

#[derive(Debug, Clone)]
struct WeighJustificationAndFinalization;

impl Circuit for WeighJustificationAndFinalization {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let beacon_state = builder.read::<Bytes32Variable>();
        let total_active_balance = builder.read::<U64Variable>();
        let previous_epoch_target_balance = builder.read::<U64Variable>();
        let current_epoch_target_balance = builder.read::<U64Variable>();

        let slot = builder.read::<U64Variable>();
        let slot_proof = builder.read::<ArrayVariable<Bytes32Variable, 5>>();

        let slot_leaf = slot.hash_tree_root(builder);

        let slot_index = U64Variable::constant(builder, 34);

        builder.ssz_verify_proof(beacon_state, slot_leaf, slot_proof.as_slice(), slot_index);

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
    }
}

fn verify_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
) {
}
