interface ProofStorage {
  getProof(slot: number, merkleTreeIndex: number, formatVersion: number): Promise<Buffer>;
  setProof(slot: number, merkleTreeIndex: number, formatVersion: number, proof: Buffer): Promise<void>;
}
