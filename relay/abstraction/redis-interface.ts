import { ProofResultType, WitnessGeneratorInput } from '@/types/types';

export interface IRedis {
  getNextProof(slot: number): Promise<ProofResultType | null>;

  getProof(prevSlot: number, nextSlot: number): Promise<ProofResultType | null>;

  get(key: string): Promise<string | null>;

  set(key: string, value: string | number): Promise<void>;

  saveProof(prevSlot: number, nextSlot: number, proof: ProofResultType);

  notifyAboutNewProof(): Promise<void>;

  subscribeForProofs(
    listener: (message: string, channel: string) => unknown,
  ): Promise<void>;
}
