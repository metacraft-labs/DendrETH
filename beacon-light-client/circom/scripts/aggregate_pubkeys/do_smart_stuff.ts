import * as validatorsJSON from '../../../../validators.json';
import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../../libs/typescript/ts-utils/bls';
import { writeFileSync } from 'fs';
const validators = validatorsJSON as any;
let start = Number.parseInt(process.argv[2]);
let end = Number.parseInt(process.argv[3]);
for (let i = start; i < end; i++) {
  let points: PointG1[] = (validators as any).data
    .slice(i * 2048, i * 2048 + 2048)
    .map(x => PointG1.fromHex(x.validator.pubkey.slice(2)));

  if (points.length < 2048) {
    break;
  }

  let input = {
    points: [
      points.map(x => [
        bigint_to_array(55, 7, x.toAffine()[0].value),
        bigint_to_array(55, 7, x.toAffine()[1].value),
      ]),
    ],
  };

  writeFileSync(`./inputs/input${i}.json`, JSON.stringify(input));
}
