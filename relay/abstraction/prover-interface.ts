import { Proof, ProofInputType } from '@/types/types';

export interface IProver {
  genProof(proofInput: ProofInputType): Promise<Proof>;
}
