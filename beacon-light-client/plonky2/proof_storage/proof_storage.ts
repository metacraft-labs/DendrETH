import { S3Storage } from './aws_proof_storage';
import { AzureStorage } from './azure_proof_storage';
import { FileStorage } from './file_proof_storage';
import { RedisStorage } from './redis_proof_storage';

export interface IProofStorage {
  getProof(key: string): Promise<Buffer | null>;
  setProof(key: string, proof: Buffer): Promise<void>;
  delProof(key: string): Promise<void>;
  quit(): Promise<void>;
}

export function createProofStorage(options: any): IProofStorage {
  const type = options['proof-storage-type'];

  switch (type) {
    case 'redis': {
      const redisHost = options['redis-host'];
      const redisPort = options['redis-port'];

      if (redisHost === undefined) {
        throw new Error('redis-host was not provided');
      }
      if (redisPort === undefined) {
        throw new Error('redis-port was not provided');
      }

      return new RedisStorage(redisHost, redisPort);
    }
    case 'file': {
      const folder = options['folder-name'];

      if (folder === undefined) {
        throw new Error('folder-name was not provided');
      }

      return new FileStorage(folder);
    }
    case 'azure': {
      const account = options['azure-account'];
      const container = options['azure-container'];

      if (account === undefined) {
        throw new Error('azure-account was not provided');
      }
      if (container === undefined) {
        throw new Error('azure-container was not provided');
      }

      return new AzureStorage('placeholder', 'placeholder');
    }
    case 'aws': {
      const endpoint = options['aws-endpoint-url'];
      const region = options['aws-region'];
      const bucket = options['aws-bucket-name'];

      if (endpoint === undefined) {
        throw new Error('aws-endpoint-url was not provided');
      }

      if (region === undefined) {
        throw new Error('aws-region was not provided');
      }

      if (bucket === undefined) {
        throw new Error('aws-bucket-name was not provided');
      }

      return new S3Storage(endpoint, region, bucket);
    }
    default:
      throw new Error(`Proof storage type not supported: ${type}`);
  }
}
