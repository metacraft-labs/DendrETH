import { mimcsponge } from 'circomlibjs';
import { PointG1 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
} from '../../../../libs/typescript/ts-utils/bls';
import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import * as validatorsJSON from '../../../../validators.json';
import { readFileSync, writeFileSync } from 'fs';
import { sha256 } from 'ethers/lib/utils';

const SIZE = 2;

let points: PointG1[] = (validatorsJSON as any).data
  .slice(0, SIZE)
  .map(x => PointG1.fromHex(x.validator.pubkey.slice(2)));

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.slice(0, SIZE).map(x => x.validator),
);

let validatorsView = ssz.phase0.Validators.toViewDU(validators);
const validatorsTree = new Tree(validatorsView.node);
// console.log(validatorsTree.root);
let sum = points.reduce((prev, curr) => prev.add(curr), PointG1.ZERO);
const sumArr = bigint_to_array(55, 7, sum.toAffine()[0].value);
sumArr.push(...bigint_to_array(55, 7, sum.toAffine()[1].value));

let hash0 = ssz.phase0.Validator.hashTreeRoot(validators[0]);
let hash1 = ssz.phase0.Validator.hashTreeRoot(validators[1]);
let hash12 = sha256('0x' + bytesToHex(hash0) + bytesToHex(hash1));
let hash2 = ''.padStart(64, '0');
let hash3 = ''.padStart(64, '0');
let hash23 = sha256('0x' + hash2 + hash3);
let hash1234 = sha256(hash12 + formatHex(hash23));


// let secondOrderZero = sha256('0x' + ''.padStart(128, '0'));
// console.log(secondOrderZero);
// let thirdOrderZero = sha256(secondOrderZero + formatHex(secondOrderZero));
// console.log(thirdOrderZero);

// console.log(sha256(thirdOrderZero+formatHex(thirdOrderZero)));


let r = mimcsponge.multiHash(
  [
    160608,
    2,
    ...BigInt(hash1234).toString(2).padStart(256, '0').split(''),
    ...bigint_to_array(55, 7, sum.toAffine()[0].value),
    ...bigint_to_array(55, 7, sum.toAffine()[1].value),
    ...[...Array(144).keys()].map(() => 0)
  ],
  123,
  1,
);
console.log(r);
