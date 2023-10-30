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
import { sha256 } from 'ethers/lib/utils';

let TAKE = 1000000;
const CIRCUIT_SIZE = 8;

(async () => {
  const result = sha256(
    '0x89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee' +
      '836b100000000000'
  );

  console.log(result);
})();
