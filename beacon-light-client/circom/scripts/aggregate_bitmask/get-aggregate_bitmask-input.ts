import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '@dendreth/utils/ts-utils/bls';
import * as update from '../../../../vendor/eth2-light-client-updates/mainnet/updates/00290.json';
import { writeFileSync } from 'fs';
import { BitVectorType } from '@chainsafe/ssz';

let points: PointG1[] = update.next_sync_committee.pubkeys.map(x =>
  PointG1.fromHex(x.slice(2)),
);
const SyncCommitteeBits = new BitVectorType(512);
let bitmask = SyncCommitteeBits.fromJson(
  update.sync_aggregate.sync_committee_bits,
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

writeFileSync('scripts/aggregate_bitmask/input.json', JSON.stringify(input));
