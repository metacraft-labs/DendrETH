import { writeFile, writeFileSync } from 'fs';
import { Redis } from '@/implementations/redis';

(async () => {
  const redis = new Redis('localhost', 6379);

  let proof = await redis.getNextProof(5644385);

  while (proof != null) {
    const proofInput = proof.proofInput;

    writeFileSync(
      `../vendor/eth2-light-client-updates/prater/capella-94-proof-inputs/public_output_${proof.prevUpdateSlot}_${proof.updateSlot}.json`,
      JSON.stringify(proof.proof.public, null, 2),
    );

    console.log('Saved proof input for slot', proof.updateSlot);

    proof = await redis.getNextProof(proof.updateSlot);
  }

  console.log('Finished.');
})();
