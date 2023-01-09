import { PointG1 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
} from '../../../../libs/typescript/ts-utils/bls';
import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import * as validatorsJSON from '../../../../validators.json';
import { randomBytes } from 'crypto';
import { writeFileSync } from 'fs';
import { mimcsponge } from 'circomlibjs';

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.map(x => x.validator),
).slice(0, 2);

let aggregatedKey = PointG1.fromHex(
  '97cbc158beb5d212de83dd8332bdb7d56da3efeceb5cb8dd46f9358dfd8e67399c1012671c2561ca81f7bea56295d156',
);

let points: PointG1[] = validators.map(x => PointG1.fromHex(x.pubkey));

let r = aggregatedKey;
for (let i = 0; i < points.length; i++) {
  r = r.subtract(points[i]);
}

let validatorsView = ssz.phase0.Validators.toViewDU(validators);
const validatorsTree = new Tree(validatorsView.node);

let branches = [...Array(2).keys()].map(x =>
  validatorsTree
    .getSingleProof(ssz.phase0.Validators.getPathInfo([x]).gindex)
    .map(x =>
      BigInt('0x' + bytesToHex(x))
        .toString(2)
        .padStart(256)
        .split(''),
    ),
);

let indexes = [...Array(2).keys()].map(x =>
  ssz.phase0.Validators.getPathInfo([x]).gindex.toString(),
);

let state_root = BigInt(
  '0x' + bytesToHex(ssz.phase0.Validators.hashTreeRoot(validators)),
)
  .toString(2)
  .padStart(256, '0')
  .split('');

const withdrawCredentials: string[][] = [];
const effectiveBalance: string[][] = [];
const slashed: string[] = [];
const activationEligibilityEpoch: string[] = [];
const activationEpoch: string[] = [];
const exitEpoch: string[] = [];
const withdrawableEpoch: string[][] = [];

for (let i = 0; i < 2; i++) {
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

let input = {
  points: [
    ...points.map(x => [
      bigint_to_array(55, 7, x.toAffine()[0].value),
      bigint_to_array(55, 7, x.toAffine()[1].value),
    ]),
  ],
  aggregatedKey: [
    bigint_to_array(55, 7, aggregatedKey.toAffine()[0].value),
    bigint_to_array(55, 7, aggregatedKey.toAffine()[1].value),
  ],
  withdrawCredentials,
  branches,
  indexes,
  state_root,
  effectiveBalance,
  slashed,
  activationEligibilityEpoch,
  activationEpoch,
  exitEpoch,
  withdrawableEpoch,
  currentEpoch: 173459,
};

let mimHash = mimcsponge.multiHash(
  [
    173459,
    indexes[1],
    ...state_root,
    ...bigint_to_array(55, 7, aggregatedKey.toAffine()[0].value),
    ...bigint_to_array(55, 7, aggregatedKey.toAffine()[1].value),
    ...bigint_to_array(55, 7, r.toAffine()[0].value),
    ...bigint_to_array(55, 7, r.toAffine()[1].value),
    ...[...Array(144).keys()].map(() => 0),
  ],
  123,
  1,
);

writeFileSync(`output.json`, JSON.stringify({ output_commitment: mimHash.toString() }));
writeFileSync(`input.json`, JSON.stringify(input));
