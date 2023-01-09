import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../../libs/typescript/ts-utils/bls';
import { writeFileSync } from 'fs';

let a = PointG1.ZERO;

let input = {
  a: [
    bigint_to_array(55, 7, PointG1.BASE.toAffine()[0].value),
    bigint_to_array(55, 7, PointG1.BASE.toAffine()[1].value),
  ],
  b: [
    bigint_to_array(55, 7, PointG1.BASE.toAffine()[0].value),
    bigint_to_array(55, 7, PointG1.BASE.toAffine()[1].value),
  ],
  aIsInfinity: 1,
  bIsInfinity: 1,
};

let output = {
  out: [
    bigint_to_array(55, 7, PointG1.BASE.toAffine()[0].value),
    bigint_to_array(55, 7, PointG1.BASE.toAffine()[1].value),
  ],
  isInfinity: 1,
};

writeFileSync('input.json', JSON.stringify(input));
writeFileSync('output.json', JSON.stringify(output));
