import path from "path";
import { IProofStorage } from "./proof_storage";
import fs from 'fs/promises';

export class FileStorage implements IProofStorage {
  constructor(private folderName: string) { }

  getPathFromKey(key: string): string {
    return path.join(this.folderName, key);
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
