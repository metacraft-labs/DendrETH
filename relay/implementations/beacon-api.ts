import { UintNumberType, ByteVectorType } from '@chainsafe/ssz';
import { ValueOfFields } from '@chainsafe/ssz/lib/view/container';
import { IBeaconApi } from '../abstraction/beacon-api-interface';
import {
  BeaconBlockHeader,
  ExecutionPayloadHeader,
  SyncAggregate,
  SyncCommittee,
} from '../types/types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { bytesToHex } from '../../libs/typescript/ts-utils/bls';
import { computeSyncCommitteePeriodAt } from '../../libs/typescript/ts-utils/ssz-utils';
import path from 'path';

export class BeaconApi implements IBeaconApi {
  private beaconRestApis: string[];
  private currentApiIndex: number;

  constructor(beaconRestApis: string[]) {
    this.beaconRestApis = beaconRestApis;
    this.currentApiIndex = 0;
  }

  getBeaconRestApis(): string[] {
    return this.beaconRestApis;
  }

  async getHashiAdapterInfo(slot: number): Promise<{
    blockNumber: number;
    blockHash: string;
    blockNumberProof: string[];
    blockHashProof: string[];
  }> {
    const currentBlock = await (
      await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
    ).json();

    const { ssz } = await import('@lodestar/types');

    const beaconBlock = ssz.capella.BeaconBlockBody.fromJson(
      currentBlock.data.message.body,
    );

    const beaconBlockView = ssz.capella.BeaconBlockBody.toViewDU(beaconBlock);
    let beaconBlockTree = new Tree(beaconBlockView.node);

    const beaconBlockHeader = await this.getExistingBlockHeader(slot);

    const beaconBlockHeaderView =
      ssz.phase0.BeaconBlockHeader.toViewDU(beaconBlockHeader);
    const beaconBlockHeaderTree = new Tree(beaconBlockHeaderView.node);

    const bodyRootProof = beaconBlockHeaderTree
      .getSingleProof(
        ssz.phase0.BeaconBlockHeader.getPathInfo(['body_root']).gindex,
      )
      .map(bytesToHex);

    const blockNumberProof = beaconBlockTree
      .getSingleProof(
        ssz.capella.BeaconBlockBody.getPathInfo([
          'executionPayload',
          'blockNumber',
        ]).gindex,
      )
      .map(bytesToHex);

    const blockHashProof = beaconBlockTree
      .getSingleProof(
        ssz.capella.BeaconBlockBody.getPathInfo([
          'executionPayload',
          'blockHash',
        ]).gindex,
      )
      .map(bytesToHex);

    return {
      blockNumber: beaconBlock.executionPayload.blockNumber,
      blockHash: bytesToHex(beaconBlock.executionPayload.blockHash),
      blockNumberProof: [...blockNumberProof, ...bodyRootProof],
      blockHashProof: [...blockHashProof, ...bodyRootProof],
    };
  }

  async getCurrentHeadSlot(): Promise<number> {
    const currentHead = await (
      await this.fetchWithFallback('/eth/v1/beacon/headers/head')
    ).json();

    return Number(currentHead.data.header.message.slot);
  }

  async getBlockSlot(blockHash: string): Promise<number> {
    const headResult = await (
      await this.fetchWithFallback(`/eth/v1/beacon/headers/${blockHash}`)
    ).json();

    return Number(headResult.data.header.message.slot);
  }

  async getExistingBlockHeader(slot: number): Promise<
    ValueOfFields<{
      slot: UintNumberType;
      proposerIndex: UintNumberType;
      parentRoot: ByteVectorType;
      stateRoot: ByteVectorType;
      bodyRoot: ByteVectorType;
    }>
  > {
    const { ssz } = await import('@lodestar/types');

    const headResult = await (
      await this.fetchWithFallback(`/eth/v1/beacon/headers/${slot}`)
    ).json();

    return ssz.phase0.BeaconBlockHeader.fromJson(
      headResult.data.header.message,
    );
  }

  async getBlockHeaderOrClosestExisting(
    slot: number,
    limitSlot: number,
  ): Promise<
    ValueOfFields<{
      slot: UintNumberType;
      proposerIndex: UintNumberType;
      parentRoot: ByteVectorType;
      stateRoot: ByteVectorType;
      bodyRoot: ByteVectorType;
    }>
  > {
    const { ssz } = await import('@lodestar/types');

    let blockHeaderResult;

    while (slot <= limitSlot) {
      blockHeaderResult = await (
        await this.fetchWithFallback(`/eth/v1/beacon/headers/${slot}`)
      ).json();

      if (blockHeaderResult.code !== 404) {
        return ssz.phase0.BeaconBlockHeader.fromJson(
          blockHeaderResult.data.header.message,
        );
      }

      slot++;
    }

    throw new Error(
      `Closest existing block is beyond the limit of ${limitSlot}`,
    );
  }

  async getBlockSyncAggregateOrClosestExisting(
    slot: number,
    limitSlot: number,
  ): Promise<{ sync_aggregate: SyncAggregate; slot: number }> {
    let blockHeaderBodyResult;

    while (slot <= limitSlot) {
      blockHeaderBodyResult = await (
        await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
      ).json();

      if (blockHeaderBodyResult.code !== 404) {
        return {
          sync_aggregate:
            blockHeaderBodyResult.data.message.body.sync_aggregate,
          slot: slot,
        };
      }

      slot++;
    }

    throw new Error(
      `Closest existing block is beyond the limit of ${limitSlot}`,
    );
  }

  async getPrevBlockHeaderStateInfo(
    prevSlot: number,
    nextSlot: number,
  ): Promise<{
    finalityHeader: ValueOfFields<{
      slot: UintNumberType;
      proposerIndex: UintNumberType;
      parentRoot: ByteVectorType;
      stateRoot: ByteVectorType;
      bodyRoot: ByteVectorType;
    }>;
    finalityHeaderBranch: string[];
    syncCommittee: SyncCommittee;
    syncCommitteeBranch: string[];
  }> {
    const { ssz } = await import('@lodestar/types');

    const { beaconState: prevBeaconSate, stateTree: prevStateTree } =
      await this.getBeaconState(prevSlot);

    const prevFinalizedHeaderResult = await (
      await this.fetchWithFallback(
        `/eth/v1/beacon/headers/${
          '0x' + bytesToHex(prevBeaconSate.finalizedCheckpoint.root)
        }`,
      )
    ).json();

    const finalityHeader = ssz.phase0.BeaconBlockHeader.fromJson(
      prevFinalizedHeaderResult.data.header.message,
    );

    const finalityHeaderBranch = prevStateTree
      .getSingleProof(
        ssz.capella.BeaconState.getPathInfo(['finalized_checkpoint', 'root'])
          .gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    const {
      beaconState: prevFinalizedBeaconState,
      stateTree: prevFinalizedBeaconStateTree,
    } = await this.getBeaconState(finalityHeader.slot);

    const prevUpdateFinalizedSyncCommmitteePeriod =
      computeSyncCommitteePeriodAt(finalityHeader.slot);
    const currentSyncCommitteePeriod = computeSyncCommitteePeriodAt(nextSlot);

    const syncCommitteeBranch = prevFinalizedBeaconStateTree
      .getSingleProof(
        ssz.capella.BeaconState.getPathInfo([
          prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
            ? 'current_sync_committee'
            : 'next_sync_committee',
        ]).gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    const syncCommittee = {
      pubkeys: prevFinalizedBeaconState[
        prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
          ? 'currentSyncCommittee'
          : 'nextSyncCommittee'
      ].pubkeys.map(x => '0x' + bytesToHex(x)),
      aggregate_pubkey:
        '0x' +
        bytesToHex(
          prevFinalizedBeaconState[
            prevUpdateFinalizedSyncCommmitteePeriod ===
            currentSyncCommitteePeriod
              ? 'currentSyncCommittee'
              : 'nextSyncCommittee'
          ].aggregatePubkey,
        ),
    };

    return {
      finalityHeader,
      finalityHeaderBranch,
      syncCommittee,
      syncCommitteeBranch,
    };
  }

  async getFinalityBlockAndProof(slot: number): Promise<{
    finalityHeader: ValueOfFields<{
      slot: UintNumberType;
      proposerIndex: UintNumberType;
      parentRoot: ByteVectorType;
      stateRoot: ByteVectorType;
      bodyRoot: ByteVectorType;
    }>;
    finalityHeaderBranch: string[];
  }> {
    const { ssz } = await import('@lodestar/types');

    const { beaconState, stateTree } = await this.getBeaconState(slot);

    const finalizedHeaderResult = await (
      await this.fetchWithFallback(
        `/eth/v1/beacon/headers/${
          '0x' + bytesToHex(beaconState.finalizedCheckpoint.root)
        }`,
      )
    ).json();

    const finalityHeader = ssz.phase0.BeaconBlockHeader.fromJson(
      finalizedHeaderResult.data.header.message,
    );

    const finalityHeaderBranch = stateTree
      .getSingleProof(
        ssz.capella.BeaconState.getPathInfo(['finalized_checkpoint', 'root'])
          .gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    return { finalityHeader, finalityHeaderBranch };
  }

  async getBlockExecutionPayloadAndProof(slot: number): Promise<{
    executionPayloadHeader: ExecutionPayloadHeader;
    executionPayloadBranch: string[];
  }> {
    const { ssz } = await import('@lodestar/types');

    const finalizedBlockBodyResult = await (
      await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
    ).json();

    const finalizedBlockBody = ssz.capella.BeaconBlockBody.fromJson(
      finalizedBlockBodyResult.data.message.body,
    );

    const finalizedBlockBodyView =
      ssz.capella.BeaconBlockBody.toViewDU(finalizedBlockBody);
    const finalizedBlockBodyTree = new Tree(finalizedBlockBodyView.node);

    const executionPayloadBranch = finalizedBlockBodyTree
      .getSingleProof(
        ssz.capella.BeaconBlockBody.getPathInfo(['execution_payload']).gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    const executionPayloadHeader = finalizedBlockBody.executionPayload;

    (executionPayloadHeader as any as ExecutionPayloadHeader).withdrawalsRoot =
      ssz.capella.ExecutionPayload.fields.withdrawals.hashTreeRoot(
        executionPayloadHeader.withdrawals,
      );

    (executionPayloadHeader as any as ExecutionPayloadHeader).transactionsRoot =
      ssz.capella.ExecutionPayload.fields.transactions.hashTreeRoot(
        executionPayloadHeader.transactions,
      );

    return {
      executionPayloadBranch,
      executionPayloadHeader:
        finalizedBlockBody.executionPayload as any as ExecutionPayloadHeader,
    };
  }

  async getFinalizedBlockHeader(slot: number): Promise<BeaconBlockHeader> {
    const { ssz } = await import('@lodestar/types');

    const finality_checkpoints = await (
      await this.fetchWithFallback(
        `/eth/v1/beacon/states/${slot}/finality_checkpoints`,
      )
    ).json();

    const finalizedHeadResult = await (
      await this.fetchWithFallback(
        `/eth/v1/beacon/headers/${finality_checkpoints.data.finalized.root}`,
      )
    ).json();

    return ssz.phase0.BeaconBlockHeader.fromJson(
      finalizedHeadResult.data.header.message,
    );
  }

  async getExecutionStateRoot(slot: number): Promise<string> {
    const block = await (
      await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
    ).json();

    return block.data.message.body.execution_payload.state_root;
  }

  private async getBeaconState(slot: number) {
    const { ssz } = await import('@lodestar/types');

    const beaconStateSZZ = await this.fetchWithFallback(
      `/eth/v2/debug/beacon/states/${slot}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => response.arrayBuffer())
      .then(buffer => new Uint8Array(buffer));

    const beaconState = ssz.capella.BeaconState.deserialize(beaconStateSZZ);
    const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
    const stateTree = new Tree(beaconStateView.node);

    return { beaconState, stateTree };
  }

  private nextApi(): void {
    this.currentApiIndex =
      (this.currentApiIndex + 1) % this.beaconRestApis.length;
  }

  private getCurrentApi(): string {
    return this.beaconRestApis[this.currentApiIndex];
  }

  private async fetchWithFallback(
    subUrl: string,
    init?: RequestInit,
  ): Promise<Response> {
    let retries = 0;
    while (true) {
      try {
        return await fetch(this.concatUrl(subUrl), init);
      } catch (error) {
        retries++;
        if (retries >= this.beaconRestApis.length) {
          console.log('All beacon rest apis failed');
          throw error;
        }

        console.log('Beacon rest api failed:', error);

        console.log('Retrying with the next one');

        this.nextApi();
      }
    }
  }

  private concatUrl(urlPath: string): string {
    const url = new URL(this.getCurrentApi());
    url.pathname = path.join(url.pathname, urlPath);

    return url.href;
  }
}
