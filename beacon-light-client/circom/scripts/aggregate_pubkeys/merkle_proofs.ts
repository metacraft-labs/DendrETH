import { ssz } from '@chainsafe/lodestar-types';
// import * as everything from '@lodestar/types';
// import { LeafNode, BranchNode } from '@chainsafe/persistent-merkle-tree';
import {
  Tree,
  zeroNode,
  ProofType,
  Proof,
} from '@chainsafe/persistent-merkle-tree';
import { readFileSync } from 'fs';
import {
  bytesToHex,
  hexToBytes,
} from '../../../../libs/typescript/ts-utils/bls';
import { sha256, stripZeros } from 'ethers/lib/utils';

console.log(ssz.altair.BeaconState.getPropertyGindex('validators'));

let validatorsJSON = JSON.parse(readFileSync('../../validators.json', 'utf-8'));

let validators = ssz.phase0.Validators.fromJson(
  validatorsJSON.data.map(x => x.validator),
);
let validatorsView = ssz.phase0.Validators.toViewDU(validators);
console.log(ssz.phase0.Validators.getPathInfo([1, 'pubkey']).gindex);
const validatorsTree = new Tree(validatorsView.node);
for (let i = 0; i < 64; i++) {
  const proof = validatorsTree
    .getSingleProof(ssz.phase0.Validators.getPathInfo([i, 'pubkey']).gindex)
    .map(x => bytesToHex(x));
  if (
    !proof.some(
      x =>
        x ===
        '8e2d14c4248791db6fc6b0779764905514819ffc38260aae9abd4ac03921beb3',
    )
  ) {
    console.log('Stop');
    throw new Error('Not every one contains this');
  }
  // console.log(proof);
}
// new Tree(validatorsView.node).getSingleProof();

console.log(
  'Validators hashtreeroot: ',
  ssz.phase0.Validators.hashTreeRoot(validators),
);

let beaconStateJson = JSON.parse(
  readFileSync('../../beacon_state.json', 'utf-8'),
).data;
beaconStateJson.previousEpochParticipation =
  beaconStateJson.previous_epoch_participation;
beaconStateJson.currentEpochParticipation =
  beaconStateJson.current_epoch_participation;

let beaconState = ssz.altair.BeaconState.fromJson(beaconStateJson);
console.log(
  'Beacon state validators hashtree root: ',
  ssz.phase0.Validators.hashTreeRoot(beaconState.validators),
);
console.log(bytesToHex(ssz.altair.BeaconState.hashTreeRoot(beaconState)));
console.log('Last block hashRoot: ', bytesToHex(beaconState.blockRoots[0]));
console.log(beaconState.slot);
console.log(bytesToHex(beaconState.eth1Data.blockHash));

console.log(bytesToHex(beaconState.latestBlockHeader.parentRoot));

console.log(bytesToHex(beaconState.latestBlockHeader.bodyRoot));

let view = ssz.altair.BeaconState.toViewDU(beaconState);
console.log(bytesToHex(view.hashTreeRoot()));

const validatorsBranch = new Tree(view.node).getSingleProof(43n);

console.log(validatorsBranch.map(x => bytesToHex(x)));

let tree = Tree.createFromProof({
  type: ProofType.single,
  gindex: 43n,
  leaf: ssz.phase0.Validators.hashTreeRoot(beaconState.validators),
  witnesses: validatorsBranch,
});

console.log(bytesToHex(tree.root));
