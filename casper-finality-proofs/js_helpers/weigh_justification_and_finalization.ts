import { Tree } from '@chainsafe/persistent-merkle-tree';
import { BeaconApi } from '../../relay/implementations/beacon-api';
import { bytesToHex } from '../../libs/typescript/ts-utils/bls';
import { hexToBits } from '../../libs/typescript/ts-utils/hex-utils';

(async () => {
  const beaconApi = new BeaconApi([
    'http://unstable.mainnet.beacon-api.nimbus.team',
  ]);
  const { beaconState } = await beaconApi.getBeaconState(6953401);

  // console.log(beaconState.justificationBits.get(0));
  // console.log(beaconState.justificationBits.get(1));
  // console.log(beaconState.justificationBits.get(2));
  // console.log(beaconState.justificationBits.get(3));

  const { ssz } = await import('@lodestar/types');
  const beaconStateHash = bytesToHex(ssz.capella.BeaconState.hashTreeRoot(beaconState));
  console.log('beacon state hash', beaconStateHash);
  /*
  console.log(
    'justification bits index',
    ssz.capella.BeaconState.getPathInfo(['justification_bits']).gindex,
  );

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
  console.log('epoch', beaconState.previousJustifiedCheckpoint.epoch);
  console.log('root', bytesToHex(beaconState.previousJustifiedCheckpoint.root));
  console.log('gindex', previousJustifiedCheckpointPathInfo.gindex);
  console.log(previousJustifiedCheckpointProof.map(bytesToHex));
})();
