import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';

(async () => {
  const { ssz } = await import('@lodestar/types');

  console.log(
    bytesToHex(ssz.phase0.Validator.fields.activationEpoch.hashTreeRoot(2301)),
  );
  //   const redis = new RedisLocal('localhost', 6381);

  //   const db = new Redis('redis://localhost:6381');

  //   const work_queue = new WorkQueue(new KeyPrefix('first_level_proofs'));

  //   console.log(await work_queue.queueLen(db));
  //   const buffer = new ArrayBuffer(8);
  //   const dataView = new DataView(buffer);
  //   dataView.setFloat64(0, 123, false);
  //   console.log('Buffer', buffer);
  //   await work_queue.addItem(db, new Item(buffer));

  //   const item = await work_queue.lease(db, 200);
})();
