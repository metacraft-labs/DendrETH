import { IRedis } from '../abstraction/redis-interface';
import { ProofResultType } from '../types/types';
import { createClient, RedisClientType } from 'redis';
import { v4 as uuidv4 } from 'uuid';

export interface Item {
  id: string;
  data: string | null;
}

export class RedisWorkQueue {
  private redisClient: RedisClientType;
  private pubSub: RedisClientType;
  private prefix: string;

  private session: string;
  private mainQueueKey: string;
  private processingQueueKey: string;
  private cleaningKey: string;
  private leaseKey: string;
  private itemDataKey: string;

  constructor(redisHost: string, redisPort: number, prefix: string) {
    this.redisClient = createClient({
      url: `redis://${redisHost}:${redisPort}`,
    });

    this.pubSub = this.redisClient.duplicate();

    this.prefix = prefix;

    this.session = `${this.prefix}:${uuidv4()}`;
    this.mainQueueKey = `${this.prefix}:queue`;
    this.processingQueueKey = `${this.prefix}:processing`;
    this.cleaningKey = `${this.prefix}:cleaning`;
    this.leaseKey = `${this.prefix}:leased_by_session:`;
    this.itemDataKey = `${this.prefix}:item:`;
  }

  async addItem(item: Item) {
    await this.waitForConnection();

    await Promise.all([
      this.redisClient.set(
        `${this.itemDataKey}${item.id}`,
        JSON.stringify(item.data),
      ),
      this.redisClient.lPush(this.mainQueueKey, item.id),
    ]);
  }

  async queueLength(): Promise<number> {
    await this.waitForConnection();

    return await this.redisClient.lLen(this.mainQueueKey);
  }

  async processing(): Promise<number> {
    await this.waitForConnection();

    return await this.redisClient.lLen(this.processingQueueKey);
  }

  async leaseExists(itemId: string): Promise<boolean> {
    await this.waitForConnection();

    return (await this.redisClient.exists(`${this.leaseKey}:${itemId}`)) > 0;
  }

  async lease(leaseSeconds: number, block: boolean, timeout = 0): Promise<Item | null> {
    await this.waitForConnection();

    let maybeItemId: string | null;

    if (block) {
      maybeItemId = await this.redisClient.brPopLPush(
        this.mainQueueKey,
        this.processingQueueKey,
        timeout,
      );
    } else {
      maybeItemId = await this.redisClient.rPopLPush(
        this.mainQueueKey,
        this.processingQueueKey,
      );
    }

    if (!maybeItemId) return null;

    let data = await this.redisClient.get(`${this.itemDataKey}:${maybeItemId}`);

    await this.redisClient.setEx(`${this.leaseKey}:${maybeItemId}`, leaseSeconds, this.session);

    return { id: maybeItemId, data: data };
  }

  async complete(item: Item): Promise<boolean> {
    await this.waitForConnection();

    let remove = await this.redisClient.lRem(this.processingQueueKey, 0, item.id);

    if(remove == 0) {
      return false;
    }

    await Promise.all([
      this.redisClient.del(`${this.itemDataKey}:${item.id}`),
      this.redisClient.del(`${this.leaseKey}:${item.id}`),
    ]);

    return true;
  }

  private async waitForConnection() {
    if (!this.redisClient.isOpen) {
      await this.redisClient.connect();
    }

    if (!this.pubSub.isOpen) {
      await this.pubSub.connect();
    }
  }
}
