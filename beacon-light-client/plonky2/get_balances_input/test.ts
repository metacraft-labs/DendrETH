import Redis from 'ioredis';
import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import { sha256 } from 'ethers/lib/utils';

(async () => {
  const result = sha256(
    '0x89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee' +
      '836b100000000000',
  );

  console.log(result);
})();
