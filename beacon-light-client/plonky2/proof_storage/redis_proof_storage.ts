import { Redis } from '../../../relay/implementations/redis';
import { IProofStorage } from './proof_storage';

export class RedisStorage implements IProofStorage {
  private connection: Redis;

  constructor(host: string, port: number) {
    this.connection = new Redis(host, port);
  }

  async getProof(key: string): Promise<Buffer | null> {
    return this.connection.getBuffer(key);
  }

  setProof(key: string, proof: Buffer): Promise<void> {
    return this.connection.setBuffer(key, proof);
  }

  async delProof(key: string): Promise<void> {
    await this.connection.del(key);
  }

  async quit(): Promise<void> {
    this.connection.client.quit();
  }
}
