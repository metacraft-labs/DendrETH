import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { BeaconApi } from '../../../relay/implementations/beacon-api';

import yargs from 'yargs';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { getCommitmentMapperProof, gindexFromIndex } from './utils';

type HashAlgorithm = 'sha256' | 'poseidon';

function bitArrayToByteArray(hash: number[]): Uint8Array {
  const result = new Uint8Array(32);

  for (let byte = 0; byte < 32; ++byte) {
    let value = 0;
    for (let bit = 0; bit < 8; ++bit) {
      value += 2 ** (7 - bit) * hash[byte * 8 + bit];
    }
    result[byte] = value;
  }
  return result;
}

(async () => {
  const options = yargs
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .option('redis-host ', {
      alias: 'redis-host',
      describe: 'The Redis host',
      type: 'string',
      default: '127.0.0.1',
      description: 'Sets a custom redis connection',
    })
    .option('redis-port', {
      alias: 'redis-port',
      describe: 'The Redis port',
      type: 'number',
      default: 6379,
      description: 'Sets a custom redis connection',
    })
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: 'http://testing.mainnet.beacon-api.nimbus.team',
      description: 'Sets a custom beacon node url',
    })
    .option('validator-index', {
      alias: 'validator-index',
      describe: 'The index of the validator',
      type: 'number',
      demandOption: true,
      description: 'Gets merkle path for the given validator index',
    })
    .option('epoch', {
      alias: 'epoch',
      describe: 'The epoch for which to generate a merkle proof',
      type: 'number',
      default: undefined,
    })
    .option('hash-algorithm', {
      alias: 'hash-algorithm',
      describe: 'The type of hashes to return',
      type: 'string',
      default: 'sha256',
      choices: ['sha256', 'poseidon'],
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const beaconApi = new BeaconApi([options['beacon-node']]);
  const epoch = options['epoch']
    ? BigInt(options['epoch'])
    : (await beaconApi.getHeadSlot()) / 32n;
  let gindex = gindexFromIndex(
    BigInt(options['validator-index']),
    40n,
  );

  const hashAlg: HashAlgorithm = options['hash-algorithm'];
  let path = await getCommitmentMapperProof(
    epoch,
    gindex,
    hashAlg,
    redis,
  );

  console.log(path);

  await redis.disconnect();
})();
