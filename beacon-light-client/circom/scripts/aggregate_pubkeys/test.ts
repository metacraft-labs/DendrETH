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

const SIZE = 56;
const UPPER_SIZE = 64;

let points: PointG1[] = (validatorsJSON as any).data
  .slice(0, SIZE)
  .map(x => PointG1.fromHex(x.validator.pubkey.slice(2)));

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.slice(0, SIZE).map(x => x.validator),
);

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

console.log(
  BigInt(hashes[0]).toString(2).padStart(256, '0').split('').join(','),
);

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
