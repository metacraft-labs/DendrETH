import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import Redis from 'ioredis';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { hexToBits } from '../../../libs/typescript/ts-utils/hex-utils';
import { bigint_to_array } from '../../solidity/test/utils/bls';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import validator_commitment_constants from '../constants/validator_commitment_constants.json';

let TAKE = 1000000;
const CIRCUIT_SIZE = 8;

(async () => {
  const db = new Redis('redis://127.0.0.1:6379');
  const queue = new WorkQueue(
    new KeyPrefix(
      `${validator_commitment_constants.balanceVerificationQueue}:${0}`,
    ),
  );

  const result = await queue.leaseExists(db, 0);

  console.log(result);
})();
