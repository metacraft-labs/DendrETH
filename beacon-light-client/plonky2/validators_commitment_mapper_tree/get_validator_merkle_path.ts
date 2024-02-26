import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { BeaconApi } from '../../../relay/implementations/beacon-api';

import yargs from 'yargs';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';

type HashAlgorithm = 'sha256' | 'poseidon';

function bitArrayToByteArray(hash: number[]): Uint8Array {
  const result = new Uint8Array(32);

  for (let byte = 0; byte < 32; ++byte) {
    let value = 0;
    for (let bit = 0; bit < 8; ++bit) {
      value += (2 ** (7 - bit)) * hash[byte * 8 + bit];
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
    })
    .argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const beaconApi = new BeaconApi([options['beacon-node']]);
  const epoch = options['epoch'] ? BigInt(options['epoch']) : await beaconApi.getHeadSlot() / 32n;
  let gindex = 2n ** 40n - 1n + BigInt(options['validator-index']);

  const hashAlg: HashAlgorithm = options['hash-algorithm'];
  let path: (number[] | string)[] = [];

  while (gindex !== 0n) {
    const siblingGindex = (gindex % 2n === 0n)
      ? gindex - 1n
      : gindex + 1n;

    const hash = await redis.extractHashFromCommitmentMapperProof(siblingGindex, epoch, hashAlg)
    if (hash !== null) {
      path.push(hash);
    }

    gindex = (gindex - 1n) / 2n;
  }

  if (hashAlg == 'sha256') {
    path = (path as number[][]).map(bitArrayToByteArray).map(bytesToHex);
  }

  console.log(path)

  await redis.quit();
})();
