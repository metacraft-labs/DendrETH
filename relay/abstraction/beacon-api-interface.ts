import { SyncAggregate, SyncCommittee } from '../types/types';
import { CapellaOrDeneb } from '@dendreth/utils/ts-utils/ssz-utils';
import { BeaconBlockHeader } from '@lodestar/types/phase0';
import { ExecutionPayloadHeader } from '@lodestar/types/deneb';
export interface IBeaconApi {
  getCurrentSSZ(slot: bigint): Promise<CapellaOrDeneb>;

  getBeaconRestApis(): string[];

  getHashiAdapterInfo(slot: number): Promise<{
    slotProof: string[];
    blockNumber: number;
    blockHash: string;
    blockNumberProof: string[];
    blockHashProof: string[];
  }>;

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
