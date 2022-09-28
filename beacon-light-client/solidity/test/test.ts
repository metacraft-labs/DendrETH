import path from 'path';
import { getFilesInDir, getMessage, getSolidityProof } from './utils';
import {
  formatJSONBlockHeader,
  formatJSONUpdate,
  hashTreeRootSyncCommitee,
} from './utils/format';
import * as constants from './utils/constants';
import { ssz } from '@chainsafe/lodestar-types';
import { bigint_to_array, bytesToHex, hexToBytes, utils } from './utils/bls';

(async () => {
  const UPDATES = getFilesInDir(
    path.join(
      __dirname,
      '..',
      '..',
      '..',
      'vendor',
      'eth2-light-client-updates',
      'mainnet',
      'updates',
    ),
  ).map(u =>
    formatJSONUpdate(
      JSON.parse(u.toString()),
      constants.GENESIS_FORK_VERSION.join(''),
    ),
  );

  console.log(
    BigInt(UPDATES[1].attested_header.state_root)
      .toString(2)
      .padStart(256, '0')
      .split('')
      .join(','),
  );
  console.log(
    BigInt(hashTreeRootSyncCommitee(UPDATES[1].next_sync_committee))
      .toString(2)
      .padStart(256, '0')
      .split('')
      .join(','),
  );
  console.log(
    UPDATES[1].next_sync_committee_branch
      .map(BigInt)
      .map(x => x.toString(2).padStart(256, '0').split('').join(',')),
  );

  const block_header = formatJSONBlockHeader(UPDATES[0].attested_header);
  const hash = ssz.phase0.BeaconBlockHeader.hashTreeRoot(block_header);

  const message = getMessage(hash, constants.ALTAIR_FORK_VERSION);
  console.log(bytesToHex(message));
  const u = await utils.hashToField(message, 2);

  // console.log([
  //   [
  //     bigint_to_array(55, 7, u[0][0]),
  //     bigint_to_array(55, 7, u[0][1])
  //   ],
  //   [
  //     bigint_to_array(55, 7, u[1][0]),
  //     bigint_to_array(55, 7, u[1][1])
  //   ]
  // ])
})();
