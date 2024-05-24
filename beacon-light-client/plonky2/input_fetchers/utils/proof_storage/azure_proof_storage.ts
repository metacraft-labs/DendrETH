import { BlobServiceClient, ContainerClient } from '@azure/storage-blob';
import { IProofStorage } from './proof_storage';

export class AzureBlobStorage implements IProofStorage {
  private containerClient: ContainerClient;

  constructor(container: string) {
    const blobServiceClient = BlobServiceClient.fromConnectionString(
      process.env.AZURE_CONNECTION_STRING!,
    );
    this.containerClient = blobServiceClient.getContainerClient(container);
  }

  async getProof(key: string): Promise<Buffer | null> {
    const blobClient = this.containerClient.getBlockBlobClient(key);

    try {
      const downloadResponse = await blobClient.download();
      const buffer = Buffer.from(
        await streamToBuffer(downloadResponse.readableStreamBody!),
      );
      return buffer;
    } catch (error: any) {
      if (error.statusCode === 404) {
        return null;
      }
      throw error;
    }
  }

  async setProof(key: string, proof: Buffer): Promise<void> {
    const blockBlobClient = this.containerClient.getBlockBlobClient(key);
    await blockBlobClient.uploadData(proof, {
      blobHTTPHeaders: { blobContentType: 'application/octet-stream' },
    });
  }

  async delProof(key: string): Promise<void> {
    const blobClient = this.containerClient.getBlockBlobClient(key);
    await blobClient.delete();
  }

  async quit(): Promise<void> {}
}

async function streamToBuffer(
  readableStream: NodeJS.ReadableStream,
): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const chunks: any[] = [];
    readableStream.on('data', data =>
      chunks.push(data instanceof Buffer ? data : Buffer.from(data)),
    );
    readableStream.on('end', () => resolve(Buffer.concat(chunks)));
    readableStream.on('error', reject);
  });
}
