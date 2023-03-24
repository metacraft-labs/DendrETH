import {
  BeaconBlockHeader,
  ExecutionPayloadHeader,
  SyncAggregate,
  SyncCommittee,
} from '../types/types';

export interface IBeaconApi {
  getCurrentHeadSlot(): Promise<number>;

  getBlockSlot(blockHash: string): Promise<number>;

  getExistingBlockHeader(slot: number): Promise<BeaconBlockHeader>;

  getBlockHeaderOrClosestExisting(
    slot: number,
    limitSlot: number,
  ): Promise<BeaconBlockHeader>;

  getBlockSyncAggregateOrClosestExisting(
    slot: number,
    limitSlot: number,
  ): Promise<{
    sync_aggregate: SyncAggregate;
    slot: number;
  }>;

  getPrevBlockHeaderStateInfo(
    prevSlot: number,
    nextSlot: number,
  ): Promise<{
    finalityHeader: BeaconBlockHeader;
    finalityHeaderBranch: string[];
    syncCommittee: SyncCommittee;
    syncCommitteeBranch: string[];
  }>;

  getFinalityBlockAndProof(slot: number): Promise<{
    finalityHeader: BeaconBlockHeader;
    finalityHeaderBranch: string[];
  }>;

  getBlockExecutionPayloadAndProof(slot: number): Promise<{
    executionPayloadHeader: ExecutionPayloadHeader;
    executionPayloadBranch: string[];
  }>;

  getFinalizedBlockHeader(slot: number): Promise<BeaconBlockHeader>;

  getExecutionStateRoot(slot: number): Promise<string>;
}
