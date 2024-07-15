import Redis from 'ioredis';
import { CommandLineOptionsBuilder } from '../../utils/cmdline';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';

(async function () {
  const options = new CommandLineOptionsBuilder()
    .withBeaconNodeOpts()
    .withRedisOpts()
    .build();

  const redis = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  const beaconApi = await getBeaconApi(options['beacon-node']);
  const headSlot = await beaconApi.getHeadSlot();
  const validators = await beaconApi.getValidators(headSlot);
  console.log(validators);
  console.log(validators.length);

  console.log(validators.map(validator => bytesToHex(validator.pubkey)));

  // const allKeys = await redis.keys('validator_proof:*');
  // const keys = allKeys.filter((key) => !key.includes('slot_lookup'));
  // const proofs = keys.filter((key) => {
  //   const parts = key.split(':');
  //   const gindex = Number(parts[1]);
  //   const slot = Number(parts[2]);
  //   return slot == 9332704 && gindex >= 2n ** 40n;
  // });
  //
  // const jsons = await Promise.all(proofs.map((key) => redis.get(key)));
  // const objs = jsons.map((json) => JSON.parse(json!));
  //
  // const uncalculatedLeaves = objs.filter((leaf) => leaf.needsChange);
  // console.log(uncalculatedLeaves);
  // console.log(uncalculatedLeaves.length);

  // const proofs2 = new Set(proofs);

  // console.log(proofs2.size);

  redis.quit();
})();
