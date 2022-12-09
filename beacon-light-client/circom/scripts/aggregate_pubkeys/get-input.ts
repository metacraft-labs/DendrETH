import { PointG1 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
} from '../../../../libs/typescript/ts-utils/bls';
import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import * as validatorsJSON from '../../../../validators.json';
import { readFileSync, writeFileSync } from 'fs';

const SIZE = 4;

let points: PointG1[] = (validatorsJSON as any).data
  .slice(0, SIZE)
  .map(x => PointG1.fromHex(x.validator.pubkey.slice(2)));

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.map(x => x.validator),
);
let validatorsView = ssz.phase0.Validators.toViewDU(validators);
const validatorsTree = new Tree(validatorsView.node);
// pubkey 8n
// withdrawalCredentials: 9n
// effectiveBalance: 10n
// slashed 11n
// activationEligibilityEpoch 12n
// activationEpoch 13n
// exitEpoch 14n
// withdrawableEpoch 15n
const withdrawCredentials: string[][] = [];
const effectiveBalance: string[][] = [];
const slashed: string[] = [];
const activationEligibilityEpoch: string[] = [];
const activationEpoch: string[] = [];
const exitEpoch: string[] = [];
const withdrawableEpoch: string[][] = [];

for (let i = 0; i < SIZE; i++) {
  const validatorTree = new Tree(
    validatorsTree.getNode(ssz.phase0.Validators.getPathInfo([i]).gindex),
  );

  withdrawCredentials.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(9n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  effectiveBalance.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(10n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  slashed.push(Number(validators[i].slashed).toString());

  activationEligibilityEpoch.push(
    validators[i].activationEligibilityEpoch.toString(),
  );

  activationEpoch.push(validators[i].activationEpoch.toString());

  exitEpoch.push(
    validators[i].exitEpoch.toString() === 'Infinity'
      ? '18446744073709551615'
      : validators[i].exitEpoch.toString(),
  );

  withdrawableEpoch.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(15n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );
}

exitEpoch[2] = '160609';

let beaconStateJson = JSON.parse(
  readFileSync('../../beacon_state.json', 'utf-8'),
).data;

beaconStateJson.previousEpochParticipation =
  beaconStateJson.previous_epoch_participation;
beaconStateJson.currentEpochParticipation =
  beaconStateJson.current_epoch_participation;

let beaconState = ssz.altair.BeaconState.fromJson(beaconStateJson);
let beaconStateView = ssz.phase0.Validators.toViewDU(validators);
let beaconStateTree = new Tree(beaconStateView.node);
console.log(beaconStateTree.getSingleProof(ssz.phase0.BeaconState.getPathInfo(["validators"]).gindex).map(bytesToHex));
console.log(ssz.phase0.BeaconState.getPathInfo(["validators"]).gindex);
let input = {
  points: [
    ...points.map(x => [
      bigint_to_array(55, 7, x.toAffine()[0].value),
      bigint_to_array(55, 7, x.toAffine()[1].value),
    ]),
  ],
  zero: [
    ...[...Array(4).keys()].map(() => 1),
    ...[...Array(0).keys()].map(() => 0),
  ],
  withdrawCredentials,
  effectiveBalance,
  slashed,
  activationEligibilityEpoch,
  activationEpoch,
  exitEpoch,
  withdrawableEpoch,
  bitmask: [
    ...[...Array(4).keys()].map(() => 1),
    ...[...Array(0).keys()].map(() => 0),
  ],
  currentEpoch: Math.floor(beaconState.slot / 32),
};

writeFileSync('scripts/aggregate_pubkeys/input.json', JSON.stringify(input));
