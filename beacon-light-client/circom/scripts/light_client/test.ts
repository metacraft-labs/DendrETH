import { Tree } from '@chainsafe/persistent-merkle-tree';
import { bytesToHex } from '../../../../libs/typescript/ts-utils/bls';

(async () => {
  const { ssz } = await import('@lodestar/types');

  const beaconStateSZZ = await fetch(
    `http://unstable.prater.beacon-api.nimbus.team/eth/v2/debug/beacon/states/5349208`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const beaconState = ssz.capella.BeaconState.deserialize(beaconStateSZZ);
  const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  const stateTree = new Tree(beaconStateView.node);

  const proof = stateTree.getSingleProof(
    ssz.capella.BeaconState.getPathInfo(['fork', 'current_version']).gindex,
  );

  console.log('fork', bytesToHex(beaconState.fork.currentVersion));

  console.log('proof', proof.map(bytesToHex));


})();
