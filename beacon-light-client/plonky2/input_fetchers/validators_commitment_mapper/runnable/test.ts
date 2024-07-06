import Redis from "ioredis";
import { CommandLineOptionsBuilder } from "../../utils/cmdline";

(async function() {
  const options = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .build();

  const redis = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  const allKeys = await redis.keys('validator_proof:*');
  const keys = allKeys.filter((key) => !key.includes('slot_lookup'));
  const proofs = keys.filter((key) => {
    const parts = key.split(':');
    const gindex = Number(parts[1]);
    const slot = Number(parts[2]);
    return slot == 9332704 && gindex >= 2n ** 40n;
  });

  const jsons = await Promise.all(proofs.map((key) => redis.get(key)));
  const objs = jsons.map((json) => JSON.parse(json!));

  const uncalculatedLeaves = objs.filter((leaf) => leaf.needsChange);
  console.log(uncalculatedLeaves);
  console.log(uncalculatedLeaves.length);

  // const proofs2 = new Set(proofs);

  // console.log(proofs2.size);

  redis.quit();
})();
