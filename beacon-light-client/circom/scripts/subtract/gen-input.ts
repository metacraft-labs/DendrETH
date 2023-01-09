import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../../libs/typescript/ts-utils/bls';
import { writeFileSync } from 'fs';

let a = PointG1.fromHex(
  '8e3d02b1ae8ed167854603a572ddf5c2044ef3f9a6996bb57ba00ffe3797f250c2cd75d4b10cdd5c95344205053f5c04',
);

let sub = a.subtract(a);

let input = {
  a: [
    bigint_to_array(55, 7, a.toAffine()[0].value),
    bigint_to_array(55, 7, a.toAffine()[1].value),
  ],
  b: [
    bigint_to_array(55, 7, a.toAffine()[0].value),
    bigint_to_array(55, 7, a.toAffine()[1].value),
  ],
  aIsInfinity: 0,
  bIsInfinity: 0,
};

let output = {
  out: [
    bigint_to_array(55, 7, a.toAffine()[0].value),
    bigint_to_array(55, 7, a.toAffine()[1].value),
  ],
  isInfinity: 1,
};

writeFileSync('input.json', JSON.stringify(input));
writeFileSync('output.json', JSON.stringify(output));
