import { writeFileSync } from 'fs';
let input = {
  currentEpoch: 1,
  participantsCount: 1,
  hash: [...Array(256).keys()].map(() => 1),
  aggregatedKey: [
    [...Array(7).keys()].map(() => 1),
    [...Array(7).keys()].map(() => 1),
  ],

  // verification key
  negalfa1xbeta2: [...Array(6).keys()].map(() =>
    [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 1)),
  ),
  gamma2: [...Array(2).keys()].map(() =>
    [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 1)),
  ),
  delta2: [...Array(2).keys()].map(() =>
    [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 1)),
  ),
  IC: [...Array(2).keys()].map(() =>
    [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 1)),
  ),
};

writeFileSync('scripts/output_commitment/input.json', JSON.stringify(input));
