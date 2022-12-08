import { mimcsponge } from 'circomlibjs';
import { PointG1 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
} from '../../../../libs/typescript/ts-utils/bls';
import { ssz } from '@chainsafe/lodestar-types';
import * as validatorsJSON from '../../../../validators.json';
import { sha256 } from 'ethers/lib/utils';
import { Tree } from '@chainsafe/persistent-merkle-tree';

const SIZE = 4;
const UPPER_SIZE = 4;

let zeros: string[] = [];
zeros[0] = ''.padStart(64, '0');

for (let i = 1; i < 40; i++) {
  zeros[i] = formatHex(sha256('0x' + zeros[i - 1] + zeros[i - 1]));
}

let points: PointG1[] = (validatorsJSON as any).data
  .slice(0, SIZE)
  .map(x => PointG1.fromHex(x.validator.pubkey.slice(2)));

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.slice(0, SIZE).map(x => x.validator),
);

validators[2].exitEpoch = 160609;

let sum = points.reduce((prev, curr) => prev.add(curr), PointG1.ZERO);
const sumArr = bigint_to_array(55, 7, sum.toAffine()[0].value);
sumArr.push(...bigint_to_array(55, 7, sum.toAffine()[1].value));

const hashes: string[] = [];

for (let i = 0; i < SIZE; i++) {
  hashes.push(bytesToHex(ssz.phase0.Validator.hashTreeRoot(validators[i])));
}

for (let i = SIZE; i < UPPER_SIZE; i++) {
  hashes.push(''.padStart(64, '0'));
}

let n = 2;

while (UPPER_SIZE / n >= 1) {
  for (let i = 0; i < UPPER_SIZE / n; i++) {
    hashes[i] = sha256(
      '0x' + formatHex(hashes[2 * i]) + formatHex(hashes[2 * i + 1]),
    );
  }

  n *= 2;
}
let hash = formatHex(hashes[0]);
let i = 16;
while (i < 40) {
  hash = formatHex(sha256('0x' + hash + zeros[i]));
  i++;
}
hash = formatHex(
  sha256('0x' + hash + BigInt(SIZE).toString(16).padEnd(64, '0')),
);



console.log(BigInt(SIZE).toString(16).padEnd(64, '0'));
let validatorsView = ssz.phase0.Validators.toViewDU(validators);
const validatorsTree = new Tree(validatorsView.node);
let arr = validatorsTree.getSingleProof(ssz.phase0.Validators.getPathInfo([0]).gindex).map(bytesToHex);
console.log(arr[arr.length - 1]);
console.log(bytesToHex(ssz.phase0.Validators.hashTreeRoot(validators)));
console.log(hash);
// console.log(
//   validatorsTree
//     .getSingleProof(ssz.phase0.Validators.getPathInfo([0]).gindex)
//     .map(bytesToHex),
// );


console.log(bytesToHex(ssz.UintNum64.hashTreeRoot(SIZE)));

console.log(bytesToHex(ssz.UintNum64.serialize(SIZE)));
let r = mimcsponge.multiHash(
  [
    160608,
    SIZE,
    ...BigInt(hashes[0]).toString(2).padStart(256, '0').split(''),
    ...bigint_to_array(55, 7, sum.toAffine()[0].value),
    ...bigint_to_array(55, 7, sum.toAffine()[1].value),
    ...[...Array(144).keys()].map(() => 0),
  ],
  123,
  1,
);

console.log(r);
