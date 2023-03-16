import { UintNumberType, ByteVectorType } from '@chainsafe/ssz';
import { ValueOfFields } from '@chainsafe/ssz/lib/view/container';
import { IBeaconApi } from '../abstraction/beacon-api-interface';
import {
  ExecutionPayloadHeader,
  SyncAggregate,
  SyncCommittee,
} from '../types/types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { bytesToHex } from '../../libs/typescript/ts-utils/bls';
import { computeSyncCommitteePeriodAt } from '../../libs/typescript/ts-utils/ssz-utils';

export class BeaconApi implements IBeaconApi {
  private beaconRestApi: string;

  constructor(beaconRestApi: string) {
    this.beaconRestApi = beaconRestApi;
  }

  async getCurrentHeadSlot(): Promise<number> {
    const currentHead = await (
      await fetch(`${this.beaconRestApi}/eth/v1/beacon/headers/head`)
    ).json();

    return Number(currentHead.data.header.message.slot);
  }

  async getBlockSlot(blockHash: string): Promise<number> {
    const headResult = await (
      await fetch(`${this.beaconRestApi}/eth/v1/beacon/headers/${blockHash}`)
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
      await fetch(`${this.beaconRestApi}/eth/v1/beacon/headers/${slot}`)
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
        await fetch(`${this.beaconRestApi}/eth/v1/beacon/headers/${slot}`)
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
        await fetch(`${this.beaconRestApi}/eth/v2/beacon/blocks/${slot}`)
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

    const prevBeaconStateSZZ = await fetch(
      `${this.beaconRestApi}/eth/v2/debug/beacon/states/${prevSlot}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => response.arrayBuffer())
      .then(buffer => new Uint8Array(buffer));

    const prevBeaconSate =
      ssz.capella.BeaconState.deserialize(prevBeaconStateSZZ);
    const prevBeaconStateView =
      ssz.capella.BeaconState.toViewDU(prevBeaconSate);
    const prevStateTree = new Tree(prevBeaconStateView.node);

    const prevFinalizedHeaderResult = await (
      await fetch(
        `${this.beaconRestApi}/eth/v1/beacon/headers/${
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

    const prevFinalizedHeaderBeaconStateSZZ = await fetch(
      `${this.beaconRestApi}/eth/v2/debug/beacon/states/${finalityHeader.slot}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => response.arrayBuffer())
      .then(buffer => new Uint8Array(buffer));

    const prevFinalizedBeaconState = ssz.capella.BeaconState.deserialize(
      prevFinalizedHeaderBeaconStateSZZ,
    );
    const prevFinalizedBeaconStateView = ssz.capella.BeaconState.toViewDU(
      prevFinalizedBeaconState,
    );
    const prevFinalizedBeaconStateTree = new Tree(
      prevFinalizedBeaconStateView.node,
    );

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

    const beaconStateResult = await fetch(
      `${this.beaconRestApi}/eth/v2/debug/beacon/states/${slot}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => response.arrayBuffer())
      .then(buffer => new Uint8Array(buffer));

    const beaconState = ssz.capella.BeaconState.deserialize(beaconStateResult);
    const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
    const stateTree = new Tree(beaconStateView.node);

    const finalizedHeaderResult = await (
      await fetch(
        `${this.beaconRestApi}/eth/v1/beacon/headers/${
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

  async getBlockExecutionPayload(slot: number): Promise<{
    executionPayloadHeader: ExecutionPayloadHeader;
    executionPayloadBranch: string[];
  }> {
    const { ssz } = await import('@lodestar/types');

    const finalizedBlockBodyResult = await (
      await fetch(`${this.beaconRestApi}/eth/v2/beacon/blocks/${slot}`)
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
}
