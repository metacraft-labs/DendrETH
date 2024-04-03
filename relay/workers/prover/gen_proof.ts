import { IProver } from '../../abstraction/prover-interface';
import { IRedis } from '../../abstraction/redis-interface';
import { ProofInputType } from '../../types/types';

export default async function genProof(
  redis: IRedis,
  prover: IProver,
  proofInput: ProofInputType,
) {
  const existingProof = await redis.getProof(
    proofInput.prevUpdateSlot,
    proofInput.updateSlot,
  );

  if (existingProof !== null) {
    await redis.notifyAboutNewProof();
    return;
  }

  let mock = Boolean(process.env.MOCK);

  const proof = await prover.genProof(proofInput);

  await redis.saveProof(proofInput.prevUpdateSlot, proofInput.updateSlot, {
    ...proofInput,
    proof,
  });

  await redis.notifyAboutNewProof();
}
