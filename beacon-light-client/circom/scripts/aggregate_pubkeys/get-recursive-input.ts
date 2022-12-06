import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../../libs/typescript/ts-utils/bls';
import * as validators from '../../../../validators.json';
import { readFileSync, writeFileSync } from 'fs';
import { promisify } from 'util';
import { exec } from 'child_process';
import * as path from 'path';
import * as vkey from './converted-vkey.json';
const promiseExec = promisify(exec);
const proofsDir = path.join(__dirname, `proofs`);

function getAggregatedPoint(points: any[]): string[][] {
  let sum = points
    .map(x => PointG1.fromHex(x.validator.pubkey.slice(2)))
    .reduce((acc, c) => acc.add(c), PointG1.ZERO);

  return [
    bigint_to_array(55, 7, sum.toAffine()[0].value),
    bigint_to_array(55, 7, sum.toAffine()[1].value),
  ];
}

(async () => {
  let points: string[][][] = [];
  points.push(getAggregatedPoint((validators as any).data.slice(0, 2048)));
  points.push(getAggregatedPoint((validators as any).data.slice(2048, 4096)));

  console.log('Proof convertion...');
  await promiseExec(
    `python ${path.join(
      __dirname,
      '../../utils/proof_converter.py',
    )} ${proofsDir}/proof${0}.json ${proofsDir}/public${0}.json`,
  );
  console.log('Input generation...');
  const proof1 = JSON.parse(readFileSync(`proof.json`).toString());

  console.log('Proof convertion...');
  await promiseExec(
    `python ${path.join(
      __dirname,
      '../../utils/proof_converter.py',
    )} ${proofsDir}/proof${1}.json ${proofsDir}/public${1}.json`,
  );
  console.log('Input generation...');
  const proof2 = JSON.parse(readFileSync(`proof.json`).toString());
  let input = {
    // proof
    negpa: [proof1.negpa, proof2.negpa],
    pb: [proof1.pb, proof2.pb],
    pc: [proof1.pc, proof2.pc],
    pubInput: ['1', '2'],

    points,
    negalfa1xbeta2: [vkey.negalfa1xbeta2, vkey.negalfa1xbeta2],
    gamma2: [vkey.gamma2, vkey.gamma2],
    delta2: [vkey.delta2, vkey.delta2],
    IC: [vkey.IC, vkey.IC],
  };

  writeFileSync(
    'scripts/aggregate_pubkeys/recursive-input.json',
    JSON.stringify(input),
  );
})();
