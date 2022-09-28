import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../libs/typescript/ts-utils/bls';
import { fastestTester } from './circuit_tester';
import { expect } from 'chai';
import * as update from '../../../vendor/eth2-light-client-updates/mainnet/updates/00290.json';

describe('Add public keys test', () => {
  it('Test1', async () => {
    let point: PointG1 = update.next_sync_committee.pubkeys.map(x =>
      PointG1.fromHex(x.slice(2)),
    )[0];
    const circuit = await fastestTester('./scripts/compress/compress.circom');

    const result = [
      bigint_to_array(55, 7, point.toAffine()[0].value),
      bigint_to_array(55, 7, point.toAffine()[1].value),
    ];

    let input = { point: result };
    const witness = await circuit.calculateWitness(input);
    let a = '';
    for (let i = 1; i <= 384; i++) {
      a += witness[i];
    }

    expect(point.toHex(true)).to.be.eq(BigInt('0b' + a).toString(16));
  });
});
