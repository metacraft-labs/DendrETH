import path from 'path';
import { Redis } from '../../../relay/implementations/redis';
import fs from 'fs/promises'

export interface IProofStorage {
  getProof(key: string): Promise<Buffer | null>;
  setProof(key: string, proof: Buffer): Promise<void>;
  delProof(key: string): Promise<void>;
  quit(): Promise<void>;
}

export function createProofStorage(options: any): IProofStorage {
  const type = options['proof-storage-type'];
  if (type === 'redis') {
    return new RedisStorage(options['redis-host'], options['redis-port']);
  } else if (type === 'file') {
    const folder = options['folder-name'];
    if (folder === undefined) {
      throw new Error('folder-name was not provided');

    }
    return new FileStorage(folder);
  } else {
    throw new Error(`Proof storage type not supported: ${type}`);
  }
}

export class FileStorage implements IProofStorage {
  constructor(private folderName: string) { }

  getPathFromKey(key: string): string {
    return path.join(this.folderName, key) + '.bin';
  }

  getProof(key: string): Promise<Buffer | null> {
    return fs.readFile(this.getPathFromKey(key));
  }

  setProof(key: string, proof: Buffer): Promise<void> {
    return fs.writeFile(this.getPathFromKey(key), proof);
  }

  delProof(key: string): Promise<void> {
    return fs.unlink(this.getPathFromKey(key));
  }

  async quit(): Promise<void> { }
}

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
