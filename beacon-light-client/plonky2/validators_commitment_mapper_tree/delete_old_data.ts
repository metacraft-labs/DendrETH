import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';

import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import { createProofStorage } from '../proof_storage/proof_storage';
import { CommandLineOptionsBuilder } from '../cmdline';

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
    `${validator_commitment_constants.validatorKey}:*:${validator_commitment_constants.slotLookupKey}`,
  );
  let validatorProofKeys = await redis.getAllKeys(
    `${validator_commitment_constants.validatorProofKey}:*:${validator_commitment_constants.slotLookupKey}`,
  );

  validatorKeys = validatorKeys.map(key =>
    key.substring(0, key.lastIndexOf(':')),
  );
  validatorProofKeys = validatorProofKeys.map(key =>
    key.substring(0, key.lastIndexOf(':')),
  );

  // delete proofs
  const proofKeys = validatorProofKeys.map(
    key =>
      validator_commitment_constants.validatorProofStorage +
      key.slice(key.indexOf(':')),
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
