import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../libs/typescript/ts-utils/bls';
import { wasm } from './circuit_tester';
import { expect } from 'chai';
import * as update from '../../../vendor/eth2-light-client-updates/mainnet/updates/00290.json';
import { BitVectorType } from '@chainsafe/ssz';

describe('Aggregate bitmask test', () => {
  it('Test1', async () => {
    let points: PointG1[] = update.next_sync_committee.pubkeys.map(x =>
      PointG1.fromHex(x.slice(2)),
    );
    const SyncCommitteeBits = new BitVectorType(512);
    let bitmask = SyncCommitteeBits.fromJson(
      update.sync_aggregate.sync_committee_bits,
    );

    let sum = points
      .filter((_, i) => bitmask.get(i))
      .reduce((prev, curr) => prev.add(curr), PointG1.ZERO);

    const expectedResult = bigint_to_array(55, 7, sum.toAffine()[0].value);
    expectedResult.push(...bigint_to_array(55, 7, sum.toAffine()[1].value));

    const circuit = await wasm(
      './scripts/aggregate_bitmask/aggregate_bitmask.circom',
    );

    let input = {
      points: [
        points.map(x => [
          bigint_to_array(55, 7, x.toAffine()[0].value),
          bigint_to_array(55, 7, x.toAffine()[1].value),
        ]),
      ],
      bitmask: bitmask.toBoolArray().map(x => (x ? 1 : 0)),
    };
    const witnes = await circuit.calculateWitness(input);
    for (let i = 0; i < expectedResult.length; i++) {
      expect(expectedResult[i]).to.be.eq(witnes[i + 1].toString());
    }
  });
});
