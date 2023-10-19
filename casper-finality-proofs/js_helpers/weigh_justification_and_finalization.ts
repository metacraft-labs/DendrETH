import { Tree } from '@chainsafe/persistent-merkle-tree';
import { BeaconApi } from '../../relay/implementations/beacon-api';
import { bytesToHex } from '../../libs/typescript/ts-utils/bls';
import { hexToBits } from '../../libs/typescript/ts-utils/hex-utils';

function dumpBlockRootsInfo(beaconState, ssz, slot) {
}

(async () => {
  const beaconApi = new BeaconApi([
    'http://unstable.mainnet.beacon-api.nimbus.team',
  ]);
  let slot = 6953401;
  const { beaconState } = await beaconApi.getBeaconState(slot);

  const { ssz } = await import('@lodestar/types');
  const beaconStateHash = bytesToHex(ssz.capella.BeaconState.hashTreeRoot(beaconState));
  console.log('beacon state hash', beaconStateHash);
  /*
  console.log(
    'previous justified checkpoint index',
    ssz.capella.BeaconState.getPathInfo(['previous_justified_checkpoint'])
      .gindex,
  );

  console.log(
    'current justified checkpoint index',
    ssz.capella.BeaconState.getPathInfo(['current_justified_checkpoint'])
      .gindex,
  );

  console.log(
    'blocks roots index',
    ssz.capella.BeaconState.getPathInfo(['block_roots']).gindex,
  );

  console.log(
    'index in the block roots of 123',
    ssz.capella.BeaconState.fields.blockRoots.getPathInfo([123]).gindex,
  );

  let blocks_root_index = ssz.capella.BeaconState.getPathInfo([
    'block_roots',
  ]).gindex;

  let epoch_index = ssz.capella.BeaconState.fields.blockRoots.getPathInfo([
    123,
  ]).gindex;

  console.log(
    'combined index',
    BigInt('0b' + blocks_root_index.toString(2) + epoch_index.toString(2)),
  );

  const beaconStateViewDU = ssz.capella.BeaconState.toViewDU(beaconState);

  const tree = new Tree(beaconStateViewDU.node);

  console.log(tree.getSingleProof(blocks_root_index).map(bytesToHex));

  const blocksRootViewDU = ssz.capella.BeaconState.fields.blockRoots.toViewDU(
    beaconState.blockRoots,
  );
  const blocksRootTree = new Tree(blocksRootViewDU.node);

  console.log(blocksRootTree.getSingleProof(epoch_index).map(bytesToHex));

  console.log(
    'combined proof',
    [
      ...tree.getSingleProof(blocks_root_index),
      ...blocksRootTree.getSingleProof(epoch_index),
    ].map(bytesToHex),
  );
  */

  // beaconState.slot = 12;
  // beaconState.balances = [1234];

  // const { ssz } = await import('@lodestar/types');
  // const pathInfo = ssz.capella.BeaconState.getPathInfo(['historical_summaries']);
  const slotPathInfo = ssz.capella.BeaconState.getPathInfo(['slot']);
  console.log(slotPathInfo);
  console.log(slotPathInfo.gindex);
  console.log(beaconState.slot);

  console.log(bytesToHex(ssz.capella.BeaconState.fields.slot.hashTreeRoot(beaconState.slot)));
  // console.log(ssz.capella.BeaconState.hashTreeRoot(beaconState))

  const beaconStateViewDU = ssz.capella.BeaconState.toViewDU(beaconState);
  const tree = new Tree(beaconStateViewDU.node);

  const slot_proof = tree.getSingleProof(slotPathInfo.gindex);
  console.log(slot_proof.map(bytesToHex));

  console.log('previous_justified_checkpoint');
  const previousJustifiedCheckpointPathInfo = ssz.capella.BeaconState.getPathInfo(['previous_justified_checkpoint']);
  const previousJustifiedCheckpointProof = tree.getSingleProof(previousJustifiedCheckpointPathInfo.gindex);
  const previousJustifiedCheckpointLeaf = ssz.capella.BeaconState.fields.previousJustifiedCheckpoint.hashTreeRoot(beaconState.previousJustifiedCheckpoint);
  console.log('previous_justified_checkpoint_leaf', bytesToHex(previousJustifiedCheckpointLeaf));
  console.log('previous justified epoch', beaconState.previousJustifiedCheckpoint.epoch);
  console.log('root', bytesToHex(beaconState.previousJustifiedCheckpoint.root));
  console.log('gindex', previousJustifiedCheckpointPathInfo.gindex);
  console.log(previousJustifiedCheckpointProof.map(bytesToHex));


  console.log('current_justified_checkpoint');
  const currentJustifiedCheckpointPathInfo = ssz.capella.BeaconState.getPathInfo(['current_justified_checkpoint']);
  const currentJustifiedCheckpointProof = tree.getSingleProof(currentJustifiedCheckpointPathInfo.gindex);
  const currentJustifiedCheckpointLeaf = ssz.capella.BeaconState.fields.currentJustifiedCheckpoint.hashTreeRoot(beaconState.currentJustifiedCheckpoint);
  console.log('current_justified_checkpoint_leaf', bytesToHex(currentJustifiedCheckpointLeaf));
  console.log('current justified epoch', beaconState.currentJustifiedCheckpoint.epoch);
  console.log('root', bytesToHex(beaconState.currentJustifiedCheckpoint.root));
  console.log('gindex', currentJustifiedCheckpointPathInfo.gindex);
  console.log(currentJustifiedCheckpointProof.map(bytesToHex));

  console.log('justification_bits')
  console.log(beaconState.justificationBits.get(0));
  console.log(beaconState.justificationBits.get(1));
  console.log(beaconState.justificationBits.get(2));
  console.log(beaconState.justificationBits.get(3));

  const justificationBitsPathInfo = ssz.capella.BeaconState.getPathInfo(['justification_bits']);
  console.log(
    'justification bits index',
    justificationBitsPathInfo.gindex
  );

  console.log('justification_bits_leaf', bytesToHex(ssz.capella.BeaconState.fields.justificationBits.hashTreeRoot(beaconState.justificationBits)));

  console.log('justification_bits proof', tree.getSingleProof(justificationBitsPathInfo.gindex).map(bytesToHex));

  console.log('block_roots');
  const blockRootsPathInfo = ssz.capella.BeaconState.getPathInfo(['block_roots']);
  console.log('block_roots gindex', blockRootsPathInfo.gindex);
  console.log('block_roots proof', tree.getSingleProof(blockRootsPathInfo.gindex).map(bytesToHex));
  console.log('block roots leaf', bytesToHex(ssz.capella.BeaconState.fields.blockRoots.hashTreeRoot(beaconState.blockRoots)));

  let current_epoch = Math.floor(beaconState.slot / 32);
  let previous_epoch = current_epoch - 1;

  const blockRootsSSZ = ssz.capella.BeaconState.fields.blockRoots.hashTreeRoot(beaconState.blockRoots);

  const blocksRootViewDU = ssz.capella.BeaconState.fields.blockRoots.toViewDU(
    beaconState.blockRoots,
  );
  const blockRoots = new Tree(blocksRootViewDU.node);
  // let epoch = previous_epoch;
  /*
  let previous_epoch_gindex_in_blockRoots = ssz.capella.BeaconState.fields.blockRoots.getPathInfo([
    previous_epoch,
  ]).gindex;
  */

  // let current_epoch_gindex_in_blockRoots = ssz.capella.BeaconState.fields.blockRoots.getPathInfo([current_epoch]).gindex;

  const SLOTS_PER_HISTORICAL_ROOT = 8192;
  const computeStartSlotAtEpoch = (epoch: number) => (epoch * 32) % SLOTS_PER_HISTORICAL_ROOT;


  const previous_epoch_index_in_block_roots = computeStartSlotAtEpoch(previous_epoch);
  const current_epoch_index_in_block_roots = computeStartSlotAtEpoch(current_epoch);
  console.log(previous_epoch_index_in_block_roots);
  console.log(current_epoch_index_in_block_roots);

  let previous_epoch_gindex_in_block_roots = ssz.capella.BeaconState.fields.blockRoots.getPathInfo([previous_epoch_index_in_block_roots]).gindex;
  let current_epoch_gindex_in_block_roots = ssz.capella.BeaconState.fields.blockRoots.getPathInfo([current_epoch_index_in_block_roots]).gindex;

  const previous_epoch_in_block_roots_proof = [
    ...blockRoots.getSingleProof(previous_epoch_gindex_in_block_roots),
    ...tree.getSingleProof(blockRootsPathInfo.gindex),
  ].map(bytesToHex);

  const current_epoch_in_block_roots_proof = [
    ...blockRoots.getSingleProof(current_epoch_gindex_in_block_roots),
    ...tree.getSingleProof(blockRootsPathInfo.gindex),
  ].map(bytesToHex);

  const previous_epoch_in_block_roots = bytesToHex(beaconState.blockRoots[previous_epoch_index_in_block_roots]);
  const current_epoch_in_block_roots = bytesToHex(beaconState.blockRoots[current_epoch_index_in_block_roots]);

  console.log('previous_epoch_in_block_roots', previous_epoch_in_block_roots);
  console.log('current_epoch_in_block_roots', current_epoch_in_block_roots);

  console.log('previous_epoch_in_block_roots_proof', previous_epoch_in_block_roots_proof);
  console.log('current_epoch_in_block_roots_proof', current_epoch_in_block_roots_proof);

  const finalizedCheckpointPathInfo = ssz.capella.BeaconState.getPathInfo(['finalized_checkpoint'])
  console.log('finalized_checkpoint epoch', beaconState.finalizedCheckpoint.epoch);
  console.log('finalized_checkpoint root', bytesToHex(beaconState.finalizedCheckpoint.root));
  console.log('finalized_checkpoint ssz', ssz.capella.BeaconState.fields.finalizedCheckpoint.hashTreeRoot(beaconState.finalizedCheckpoint));
  console.log('finalized_checkpoint gindex', finalizedCheckpointPathInfo.gindex);
  console.log('finalized_checkpoint proof', tree.getSingleProof(finalizedCheckpointPathInfo.gindex).map(bytesToHex));

  /*
  console.log('previous_epoch_gindex_in_blockRoots', previous_epoch_gindex_in_block_roots);
  console.log('current_epoch_gindex_in_blockRoots', current_epoch_gindex_in_block_roots);

  console.log('previous_epoch_root_in_blockRoots', beaconState.blockRoots[previous_epoch_index_in_block_roots]);
  console.log('current_epoch_root_in_blockRoots', beaconState.blockRoots[current_epoch_index_in_block_roots]);
  */

  // console.log('block_roots value for epoch', bytesToHex(beaconState.blockRoots[epoch]));
  // console.log('epoch_index gindex', epoch_index);
  // console.log('block_roots at index proof', blockRoots.getSingleProof(epoch_index).map(bytesToHex));

  // console.log('previous_epoch_justified_checkpoint',)
  // console.log('block_roots content', beaconState.blockRoots.map(bytesToHex));
})();
