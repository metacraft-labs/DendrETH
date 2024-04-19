import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '@dendreth/utils/ts-utils/bls';
import * as update from '../../../../vendor/eth2-light-client-updates/mainnet/updates/00290.json';
import { writeFileSync } from 'fs';

let point: PointG1 = update.next_sync_committee.pubkeys.map(x =>
  PointG1.fromHex(x.slice(2)),
)[0];
const result = [
  bigint_to_array(55, 7, point.toAffine()[0].value),
  bigint_to_array(55, 7, point.toAffine()[1].value),
];
let input = { point: result };

writeFileSync('input.json', JSON.stringify(input));
