import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';

import CONSTANTS from '../../../kv_db_constants.json';
import { createProofStorage } from '../../utils/proof_storage/proof_storage';
import { CommandLineOptionsBuilder } from '../../utils/cmdline';

require('dotenv').config({ path: '../.env' });

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withProofStorageOpts()
    .option('oldest-slot', {
      describe: 'The oldest slot to preserve data for',
      type: 'number',
      demandOption: true,
    })
    .build();

  const oldestSlot = BigInt(options['oldest-slot']);

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);
  const proofStorage = createProofStorage(options);

  let validatorKeys = await redis.getAllKeys(
    `${CONSTANTS.validatorKey}:*:${CONSTANTS.slotLookupKey}`,
  );
  let validatorProofKeys = await redis.getAllKeys(
    `${CONSTANTS.validatorProofKey}:*:${CONSTANTS.slotLookupKey}`,
  );

  validatorKeys = validatorKeys.map(key =>
    key.substring(0, key.lastIndexOf(':')),
  );
  validatorProofKeys = validatorProofKeys.map(key =>
    key.substring(0, key.lastIndexOf(':')),
  );

  // delete proofs
  const proofKeys = validatorProofKeys.map(
    key => CONSTANTS.validatorProofStorage + key.slice(key.indexOf(':')),
  );
  await Promise.all(
    proofKeys.map(async (proofKey, index) => {
      const outdatedSlots = await redis.collectOutdatedSlots(
        validatorProofKeys[index],
        oldestSlot,
      );
      const keysToDelete = outdatedSlots.map(slot => `${proofKey}:${slot}`);
      return Promise.all(keysToDelete.map(key => proofStorage.delProof(key)));
    }),
  );

  // delete redis data
  const redisKeys = [...validatorKeys, ...validatorProofKeys];
  const deleted = await Promise.all(
    redisKeys.map(async key => {
      return redis.pruneOldSlots(key, oldestSlot);
    }),
  );

  const deletedCount = deleted.reduce((sum, value) => sum + value);
  console.log(`Deleted ${deletedCount} database entries`);

  await proofStorage.quit();
  await redis.quit();
})();

// TODO: Delete `validators_root` and `validators_length`
