import * as validators from '../../../../validators.json';
import { Fp, PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../../libs/typescript/ts-utils/bls';
import { poseidon } from 'circomlibjs';

(async () => {
  let sum = PointG1.ZERO;

  for (let validator of (validators as any).data.slice(0, 2048)) {
    sum = sum.add(PointG1.fromHex(validator.validator.pubkey.slice(2)));
  }

  console.log(
    poseidon([
      ...bigint_to_array(55, 7, sum.toAffine()[0].value),
      ...bigint_to_array(55, 7, sum.toAffine()[1].value),
    ]),
  );
})();
