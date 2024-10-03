import { Redis } from '@dendreth/relay/implementations/redis';

import CONSTANTS from '../../../kv_db_constants.json';
import { createProofStorage } from '../../utils/proof_storage/proof_storage';
import { CommandLineOptionsBuilder } from '../../utils/cmdline';
import { getSlotWithLatestChange } from '../../redis_interactions';

require('dotenv').config({ path: '../.env' });

(async () => {
  const options = new CommandLineOptionsBuilder()
    .withProofStorageOpts()
    .option('oldest-slot', {
      describe: 'The oldest slot to preserve data for',
      type: 'number',
      demandOption: true,
    })
    .build();

  const oldestSlot = BigInt(options['oldest-slot']);

  const redis = new Redis(options['redis-host'], options['redis-port']);
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
      const outdatedSlots = await collectOutdatedSlots(
        redis,
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
      return pruneOldSlots(redis, key, oldestSlot);
    }),
  );

  const deletedCount = deleted.reduce((sum, value) => sum + value);
  console.log(`Deleted ${deletedCount} database entries`);

  await proofStorage.quit();
  await redis.quit();
})();

async function pruneOldSlots(
  redis: Redis,
  key: string,
  newOldestSlot: bigint,
): Promise<number> {
  const slots = await collectOutdatedSlots(redis, key, newOldestSlot);
  if (slots.length !== 0) {
    await removeFromSlotLookup(redis, key, ...slots);
  }
  return 0;
}

async function removeFromSlotLookup(
  redis: Redis,
  key: string,
  ...slots: bigint[]
) {
  await redis.client.zrem(
    `${key}:${CONSTANTS.slotLookupKey}`,
    slots.map(String),
  );
}

async function collectOutdatedSlots(
  redis: Redis,
  key: string,
  newOldestSlot: bigint,
): Promise<bigint[]> {
  const slotWithLatestChange = await getSlotWithLatestChange(
    redis,
    key,
    newOldestSlot,
  );
  if (slotWithLatestChange !== null) {
    return (
      await redis.client.zrange(
        `${key}:${CONSTANTS.slotLookupKey}`,
        0,
        (slotWithLatestChange - 1n).toString(),
        'BYSCORE',
      )
    ).map(BigInt);
  }
  return [];
}

// TODO: Delete `validators_root` and `validators_length`
