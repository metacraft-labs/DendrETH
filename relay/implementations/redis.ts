import { IRedis } from '../abstraction/redis-interface';
import { ProofResultType } from '../types/types';
import { RedisClientType, createClient } from 'redis';
import { Redis as RedisClient, Result } from 'ioredis';
import fs from 'fs';
import path from 'path';

declare module 'ioredis' {
  interface RedisCommander<Context> {
    deletePattern(pattern: string): Result<string, Context>;
    rebaseValidatorsCommitmentMapper(gindex: number): Result<string, Context>;
    recomputeSlot(slot: number): Result<string, Context>;
  }
}

function makeRedisURL(host: string, port: number, auth?: string): string {
  const at: string = auth != null && auth.length > 0 ? `${auth}@` : '';
  return `redis://${at}${host}:${port}`;
}

export class Redis implements IRedis {
  public readonly client: RedisClient;
  private readonly pubSub: RedisClientType;

  constructor(redisHost: string, redisPort: number, redisAuth?: string) {
    const url: string = makeRedisURL(redisHost, redisPort, redisAuth);
    this.client = new RedisClient(url);
    this.pubSub = createClient({ url });

    this.client.defineCommand('deletePattern', {
      numberOfKeys: 0,
      lua: fs.readFileSync(
        path.resolve(__dirname, 'redis-scripts', 'deletePattern.lua'),
        'utf8',
      ),
    });

    this.client.defineCommand('rebaseValidatorsCommitmentMapper', {
      numberOfKeys: 0,
      lua: fs.readFileSync(
        path.resolve(
          __dirname,
          'redis-scripts',
          'rebaseValidatorsCommitmentMapper.lua',
        ),
        'utf8',
      ),
    });

    this.client.defineCommand('recomputeSlot', {
      numberOfKeys: 0,
      lua: fs.readFileSync(
        path.resolve(__dirname, 'redis-scripts', 'recomputeSlot.lua'),
        'utf8',
      ),
    });
  }

  async quit() {
    await this.waitForConnection();
    await this.pubSub.quit();
    this.client.quit();
  }

  async getAllKeys(pattern: string): Promise<string[]> {
    await this.waitForConnection();
    return this.client.keys(pattern);
  }

  async notifyAboutNewProof(): Promise<void> {
    await this.waitForConnection();

    this.pubSub.publish('proofs_channel', 'proof');
  }

  async getNextProof(slot: number): Promise<ProofResultType | null> {
    await this.waitForConnection();

    const keys = await this.client.keys(`proof:${slot}:*`);

    if (keys.length == 0) {
      return null;
    }

    return JSON.parse((await this.client.get(keys[0]))!);
  }

  async getProof(
    prevSlot: number,
    nextSlot: number,
  ): Promise<ProofResultType | null> {
    await this.waitForConnection();

    let proof = await this.client.get(`proof:${prevSlot}:${nextSlot}`);

    if (proof == null) {
      return null;
    }

    return JSON.parse(proof);
  }

  async get(key: string): Promise<string | null> {
    await this.waitForConnection();
    return this.client.get(key);
  }

  async getBuffer(key: string): Promise<Buffer | null> {
    await this.waitForConnection();
    return this.client.getBuffer(key);
  }

  async setBuffer(key: string, buffer: Buffer): Promise<void> {
    await this.waitForConnection();
    await this.client.set(key, buffer);
  }

  async set(key: string, value: string): Promise<void> {
    await this.waitForConnection();
    await this.client.set(key, value);
  }

  async del(key: string): Promise<number> {
    await this.waitForConnection();
    return this.client.del(key);
  }

  async saveProof(
    prevSlot: number,
    nextSlot: number,
    proof: ProofResultType,
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
      `proof:${prevSlot}:${nextSlot}`,
      JSON.stringify(proof),
    );
  }

  async subscribeForProofs(
    listener: (message: string, channel: string) => unknown,
  ): Promise<void> {
    await this.waitForConnection();

    await this.pubSub.subscribe('proofs_channel', listener);
  }

  async subscribeForGnarkProofs(
    protocol: string,
    listener: (message: string, channel: string) => unknown,
  ): Promise<void> {
    await this.waitForConnection();

    await this.pubSub.subscribe(`${protocol}:gnark_proofs_channel`, listener);
  }

  private async waitForConnection() {
    if (!['connect', 'connecting', 'ready'].includes(this.client.status)) {
      await this.client.connect();
    }

    if (!this.pubSub.isOpen) {
      await this.pubSub.connect();
    }
  }
}
