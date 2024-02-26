import { S3Client, GetObjectCommand, PutObjectCommand, DeleteObjectCommand } from "@aws-sdk/client-s3";
import { IProofStorage } from "./proof_storage";
import { Readable } from "stream";

export class S3Storage implements IProofStorage {
  private s3: S3Client;
  private bucketName: string;

  constructor(endpoint: string, region: string, bucketName: string) {
    this.s3 = new S3Client({
      region,
      endpoint,
      credentials: {
        accessKeyId: process.env.AWS_ACCESS_KEY_ID || "unset",
        secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY || "unset",
      },
    });
    this.bucketName = bucketName;
  }

  async getProof(key: string): Promise<Buffer | null> {
    try {
      const params = {
        Bucket: this.bucketName,
        Key: key,
      };

      const response = await this.s3.send(new GetObjectCommand(params));
      const bodyStream = response.Body as Readable;
      const chunks: Uint8Array[] = [];

      for await (const chunk of bodyStream) {
        chunks.push(chunk);
      }

      return Buffer.concat(chunks);
    } catch (error: any) {
      if (error.name === 'NoSuchKey') {
        return null;
      }
      throw error;
    }
  }

  async setProof(key: string, proof: Buffer): Promise<void> {
    const params = {
      Bucket: this.bucketName,
      Key: key,
      Body: proof,
    };

    await this.s3.send(new PutObjectCommand(params));
  }

  async delProof(key: string): Promise<void> {
    const params = {
      Bucket: this.bucketName,
      Key: key,
    };

    await this.s3.send(new DeleteObjectCommand(params));
  }

  async quit(): Promise<void> {
    // Additional cleanup logic can be added if needed.
  }
}
