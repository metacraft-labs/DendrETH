import { UintNumberType, ByteVectorType } from '@chainsafe/ssz';
import { ValueOfFields } from '@chainsafe/ssz/lib/view/container';
import { IBeaconApi } from '@/abstraction/beacon-api-interface';
import {
  BeaconBlockHeader,
  ExecutionPayloadHeader,
  SyncAggregate,
  SyncCommittee,
  Validator,
} from '@/types/types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import {
  SSZ,
  CapellaOrDeneb,
  computeSyncCommitteePeriodAt,
} from '@dendreth/utils/ts-utils/ssz-utils';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { prometheusTiming } from '@dendreth/utils/ts-utils/prometheus-utils';
import { panic, sleep } from '@dendreth/utils/ts-utils/common-utils';
import EventSource from 'eventsource';
// @ts-ignore
import { StateId } from '@lodestar/api/beacon/routes/beacon';

const logger = getGenericLogger();

export async function getBeaconApi(
  beaconRestApis: string[],
): Promise<BeaconApi> {
  const { ssz } = await import('@lodestar/types');
  return new BeaconApi(beaconRestApis, ssz);
}

export class BeaconApi implements IBeaconApi {
  currentApiIndex = 0;

  constructor(
    public readonly beaconRestApis: string[],
    public readonly ssz: SSZ,
  ) {}

  async getCurrentSSZ(slot: bigint): Promise<CapellaOrDeneb> {
    const forkSchedule = await (
      await this.fetchWithFallback('/eth/v1/config/fork_schedule')
    ).json();
    const forkEpoch = BigInt(
      forkSchedule.data[forkSchedule.data.length - 1].epoch,
    );
    const SLOTS_PER_EPOCH = 32n;
    return (
      slot >= forkEpoch * SLOTS_PER_EPOCH ? this.ssz.deneb : this.ssz.capella
    ) as CapellaOrDeneb;
  }

  getBeaconRestApis(): string[] {
    return this.beaconRestApis;
  }

  async getHashiAdapterInfo(slot: number): Promise<{
    slotProof: string[];
    blockNumber: number;
    blockHash: string;
    blockNumberProof: string[];
    blockHashProof: string[];
  }> {
    const currentBlock = await (
      await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
    ).json();

    const currentSszFork = await this.getCurrentSSZ(BigInt(slot));
    const beaconBlock = currentSszFork.BeaconBlockBody.fromJson(
      currentBlock.data.message.body,
    );

    const beaconBlockView =
      currentSszFork.BeaconBlockBody.toViewDU(beaconBlock);
    let beaconBlockTree = new Tree(beaconBlockView.node);

    const beaconBlockHeader = await this.getExistingBlockHeader(slot);

    const beaconBlockHeaderView =
      this.ssz.phase0.BeaconBlockHeader.toViewDU(beaconBlockHeader);
    const beaconBlockHeaderTree = new Tree(beaconBlockHeaderView.node);

    const bodyRootProof = beaconBlockHeaderTree
      .getSingleProof(
        this.ssz.phase0.BeaconBlockHeader.getPathInfo(['body_root']).gindex,
      )
      .map(bytesToHex);

    const blockNumberProof = beaconBlockTree
      .getSingleProof(
        currentSszFork.BeaconBlockBody.getPathInfo([
          'executionPayload',
          'blockNumber',
        ]).gindex,
      )
      .map(bytesToHex);

    const blockHashProof = beaconBlockTree
      .getSingleProof(
        currentSszFork.BeaconBlockBody.getPathInfo([
          'executionPayload',
          'blockHash',
        ]).gindex,
      )
      .map(bytesToHex);

    const slotProof = beaconBlockHeaderTree
      .getSingleProof(
        this.ssz.phase0.BeaconBlockHeader.getPathInfo(['slot']).gindex,
      )
      .map(bytesToHex);

    return {
      slotProof: slotProof,
      blockNumber: beaconBlock.executionPayload.blockNumber,
      blockHash: bytesToHex(beaconBlock.executionPayload.blockHash),
      blockNumberProof: [...blockNumberProof, ...bodyRootProof],
      blockHashProof: [...blockHashProof, ...bodyRootProof],
    };
  }

  subscribeForEvents(events: string[]): EventSource {
    return new EventSource(
      this.concatUrl(`/eth/v1/events?topics=${events.join(',')}`),
    );
  }

  async getCurrentHeadSlot(): Promise<number> {
    logger.info('Getting CurrentHeadSlot..');

    const currentHead = await prometheusTiming(
      async () =>
        (await this.fetchWithFallback('/eth/v1/beacon/headers/head')).json(),
      'getCurrentHeadSlot',
    );

    return Number(currentHead.data.header.message.slot);
  }

  async getGenesisData(): Promise<
    ValueOfFields<{
      genesisTime: UintNumberType;
      genesisValidatorsRoot: ByteVectorType;
      genesisForkVersion: ByteVectorType;
    }>
  > {
    const genesisData = await (
      await this.fetchWithFallback('/eth/v1/beacon/genesis')
    ).json();

    return this.ssz.phase0.Genesis.fromJson(genesisData.data);
  }

  async getBlockSlot(blockHash: string): Promise<number> {
    const headResult = await (
      await this.fetchWithFallback(`/eth/v1/beacon/headers/${blockHash}`)
    ).json();

    logger.info('Got CurrentHeadSlot..');
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
    const headResult = await (
      await this.fetchWithFallback(`/eth/v1/beacon/headers/${slot}`)
    ).json();

    return this.ssz.phase0.BeaconBlockHeader.fromJson(
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
    while (slot <= limitSlot) {
      const blockHeaderResult = await (
        await this.fetchWithFallback(`/eth/v1/beacon/headers/${slot}`)
      ).json();

      if (blockHeaderResult.code !== 404) {
        return this.ssz.phase0.BeaconBlockHeader.fromJson(
          blockHeaderResult.data.header.message,
        );
      }

      slot++;
    }

    throw new Error(
      `Closest existing block is beyond the limit of ${limitSlot}`,
    );
  }

  async getBlockHeader(slot: StateId) {
    const { ssz } = await import('@lodestar/types');

    let blockHeaderResult = await (
      await this.fetchWithFallback(`/eth/v1/beacon/headers/${slot}`)
    ).json();

    if (blockHeaderResult.code === 404) {
      throw new Error('block header is not present');
    }

    return ssz.phase0.BeaconBlockHeader.fromJson(
      blockHeaderResult.data.header.message,
    );
  }

  async getBlockSyncAggregateOrClosestExisting(
    slot: number,
    limitSlot: number,
  ): Promise<{ sync_aggregate: SyncAggregate; slot: number }> {
    while (slot <= limitSlot) {
      const blockHeaderBodyResult = await (
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
    const { beaconState: prevBeaconSate, stateTree: prevStateTree } =
      await prometheusTiming(
        async () =>
          (await this.getBeaconState(BigInt(prevSlot))) ||
          panic('Could not fetch beacon state'),
        'getPrevBeaconState',
      );

    const prevFinalizedHeaderResult = await (
      await this.fetchWithFallback(
        `/eth/v1/beacon/headers/${
          '0x' + bytesToHex(prevBeaconSate.finalizedCheckpoint.root)
        }`,
      )
    ).json();

    const finalityHeader = this.ssz.phase0.BeaconBlockHeader.fromJson(
      prevFinalizedHeaderResult.data.header.message,
    );

    const currentSszFork = await this.getCurrentSSZ(BigInt(nextSlot));
    const finalityHeaderBranch = prevStateTree
      .getSingleProof(
        currentSszFork.BeaconState.getPathInfo(['finalized_checkpoint', 'root'])
          .gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    const {
      beaconState: prevFinalizedBeaconState,
      stateTree: prevFinalizedBeaconStateTree,
    } = await prometheusTiming(
      async () =>
        (await this.getBeaconState(BigInt(finalityHeader.slot))) ||
        panic('Could not fetch beacon state'),
      'getPrevFinalizedBeaconState',
    );

    const prevUpdateFinalizedSyncCommmitteePeriod =
      computeSyncCommitteePeriodAt(finalityHeader.slot);
    const currentSyncCommitteePeriod = computeSyncCommitteePeriodAt(nextSlot);

    const syncCommitteeBranch = prevFinalizedBeaconStateTree
      .getSingleProof(
        currentSszFork.BeaconState.getPathInfo([
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
    const { beaconState, stateTree } = await prometheusTiming(
      async () =>
        (await this.getBeaconState(BigInt(slot))) ||
        panic('Could not fetch beacon state'),
      'getBeaconState',
    );

    const finalizedHeaderResult = await (
      await this.fetchWithFallback(
        `/eth/v1/beacon/headers/${
          '0x' + bytesToHex(beaconState.finalizedCheckpoint.root)
        }`,
      )
    ).json();

    const currentSszFork = await this.getCurrentSSZ(BigInt(slot));
    const finalityHeader = this.ssz.phase0.BeaconBlockHeader.fromJson(
      finalizedHeaderResult.data.header.message,
    );
    const finalityHeaderBranch = stateTree
      .getSingleProof(
        currentSszFork.BeaconState.getPathInfo(['finalized_checkpoint', 'root'])
          .gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    return { finalityHeader, finalityHeaderBranch };
  }

  async getBlockExecutionPayloadAndProof(slot: number): Promise<{
    executionPayloadHeader: ExecutionPayloadHeader;
    executionPayloadBranch: string[];
  }> {
    const currentSszFork = await this.getCurrentSSZ(BigInt(slot));
    const finalizedBlockBodyResult = await (
      await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
    ).json();

    const finalizedBlockBody = currentSszFork.BeaconBlockBody.fromJson(
      finalizedBlockBodyResult.data.message.body,
    );

    const finalizedBlockBodyView =
      currentSszFork.BeaconBlockBody.toViewDU(finalizedBlockBody);
    const finalizedBlockBodyTree = new Tree(finalizedBlockBodyView.node);

    const executionPayloadBranch = finalizedBlockBodyTree
      .getSingleProof(
        currentSszFork.BeaconBlockBody.getPathInfo(['execution_payload'])
          .gindex,
      )
      .map(x => '0x' + bytesToHex(x));

    const executionPayloadHeader = finalizedBlockBody.executionPayload;

    (executionPayloadHeader as any as ExecutionPayloadHeader).withdrawalsRoot =
      currentSszFork.ExecutionPayload.fields.withdrawals.hashTreeRoot(
        executionPayloadHeader.withdrawals,
      );

    (executionPayloadHeader as any as ExecutionPayloadHeader).transactionsRoot =
      currentSszFork.ExecutionPayload.fields.transactions.hashTreeRoot(
        executionPayloadHeader.transactions,
      );

    return {
      executionPayloadBranch,
      executionPayloadHeader:
        finalizedBlockBody.executionPayload as any as ExecutionPayloadHeader,
    };
  }

  async getFinalizedBlockHeader(slot: number): Promise<BeaconBlockHeader> {
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

    return this.ssz.phase0.BeaconBlockHeader.fromJson(
      finalizedHeadResult.data.header.message,
    );
  }

  async getExecutionStateRoot(slot: number): Promise<string> {
    const block = await (
      await this.fetchWithFallback(`/eth/v2/beacon/blocks/${slot}`)
    ).json();

    return block.data.message.body.execution_payload.state_root;
  }

  async getHeadSlot(): Promise<bigint> {
    const res = await (
      await this.fetchWithFallback('/eth/v1/beacon/headers/head')
    ).json();
    return BigInt(res.data.header.message.slot);
  }

  async getValidators(
    slot: bigint,
    validatorsCount: number | undefined = undefined,
    offset: number | undefined = undefined,
  ): Promise<Validator[]> {
    const { ssz } = await import('@lodestar/types');

    if (validatorsCount !== undefined && validatorsCount < 10000) {
      // use the validators endpoint
      let url = `/eth/v1/beacon/states/${slot}/validators`;
      let range = [...Array(validatorsCount).keys()];
      if (offset !== undefined) {
        range = range.map(index => index + offset);
      }
      url = url + `?id=${range.join(',')}`;

      const response = await this.fetchWithFallback(url);
      if (response.status === 404) {
        throw new Error('status 404 not found');
      }

      const validators = await response.json();
      validators.data.sort((v1: any, v2: any) => +v1.index - +v2.index);
      return ssz.phase0.Validators.fromJson(
        validators.data.map((x: any) => x.validator),
      );
    } else {
      // fetch an ssz beacon state to extract the validators from it
      const { beaconState } =
        (await this.getBeaconState(slot)) ||
        panic('Could not fetch beacon state');
      return beaconState.validators.slice(offset || 0, validatorsCount);
    }
  }

  async getFirstNonMissingSlotInEpoch(epoch: bigint): Promise<bigint> {
    for (let relativeSlot = 0n; relativeSlot < 31n; ++relativeSlot) {
      const slot = epoch * 32n + relativeSlot;
      try {
        const status = await this.pingEndpoint(
          `/eth/v1/beacon/blocks/${slot}/root`,
        );
        if (status === 200) {
          return slot;
        }
      } catch (error) {
        console.error(error);
      }
    }
    throw new Error('Did not find non-empty slot in epoch');
  }

  async getBlockRootBySlot(stateId: StateId) {
    let url = `/eth/v1/beacon/blocks/${stateId}/root`;
    const json = await (await this.fetchWithFallback(url)).json();
    return json.data.root;
  }

  async getBeaconStateSSZBytes(stateId: StateId) {
    const beaconStateSZZ = await this.fetchWithFallback(
      `/eth/v2/debug/beacon/states/${stateId}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => response.arrayBuffer())
      .then(buffer => new Uint8Array(buffer));
    return beaconStateSZZ;
  }

  async getLastFinalizedCheckpoint(): Promise<bigint> {
    const reponse = await this.fetchWithFallback(
      '/eth/v1/beacon/states/head/finality_checkpoints',
    );
    const json = await reponse.json();
    return BigInt(json.data.finalized.epoch);
  }

  async getBeaconBlock(slot: bigint) {
    logger.info('Getting Beacon block..');

    const beaconBlockSSZ = await this.fetchWithFallback(
      `/eth/v2/debug/beacon/blocks/${slot}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => {
        if (response.status === 404) {
          throw 'Could not fetch beacon state (404 not found)';
        }
        return response.arrayBuffer();
      })
      .then(buffer => new Uint8Array(buffer))
      .catch(console.error);

    if (!beaconBlockSSZ) {
      return null;
    }

    const currentSszFork = await this.getCurrentSSZ(slot);
    const beaconBlock = currentSszFork.BeaconBlock.deserialize(beaconBlockSSZ);

    logger.info('Got Beacon block');
    return beaconBlock;
  }

  async getBeaconState(slot: bigint) {
    logger.info('Getting Beacon State..');

    const beaconStateSZZ = await this.fetchWithFallback(
      `/eth/v2/debug/beacon/states/${slot}`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => {
        if (response.status === 404) {
          throw 'Could not fetch beacon state (404 not found)';
        }
        return response.arrayBuffer();
      })
      .then(buffer => new Uint8Array(buffer))
      .catch(console.error);

    if (!beaconStateSZZ) {
      throw new Error('Could not fetch beacon state');
    }

    const currentSszFork = await this.getCurrentSSZ(slot);
    const beaconState = currentSszFork.BeaconState.deserialize(beaconStateSZZ);
    const beaconStateView = currentSszFork.BeaconState.toViewDU(beaconState);
    const stateTree = new Tree(beaconStateView.node);

    logger.info('Got Beacon State');
    return { beaconState, stateTree };
  }

  private nextApi(): void {
    this.currentApiIndex =
      (this.currentApiIndex + 1) % this.beaconRestApis.length;
  }

  private getCurrentApi(): string {
    return this.beaconRestApis[this.currentApiIndex];
  }

  private async fetchWithFallbackNoRetry(
    subUrl: string,
    init?: RequestInit,
  ): Promise<Response> {
    return fetch(this.concatUrl(subUrl), init);
  }

  private async fetchWithFallback(
    subUrl: string,
    init?: RequestInit,
  ): Promise<Response> {
    let retries = 0;
    const maxApiRetries = 5;

    while (true) {
      console.log(this.getCurrentApi());
      try {
        const result = await this.fetchWithFallbackNoRetry(subUrl, init);

        if (result.status == 404) {
          retries++;
          if (retries >= this.beaconRestApis.length * maxApiRetries) {
            throw new Error('Could not find the requested resource');
          }
          logger.warn('404 not found');
          logger.warn('Retrying as sometimes info appears with lag');
          await sleep(1000);
          this.nextApi();
          continue;
        }

        if (result.status === 429) {
          logger.warn('Rate limit exceeded');

          logger.warn('Retrying with the next one');
          this.nextApi();
          continue;
        }

        return result;
      } catch (error) {
        retries++;
        if (retries >= this.beaconRestApis.length * maxApiRetries) {
          logger.error('All beacon rest apis failed');
          throw error;
        }

        logger.error(`Beacon rest api failed: ${error}`);

        logger.error('Retrying with the next one');

        this.nextApi();
      }
    }
  }

  async pingEndpoint(endpoint: string, init?: RequestInit): Promise<number> {
    const response = await this.fetchWithFallbackNoRetry(endpoint, init);
    return response.status;
  }

  private concatUrl(urlPath: string): string {
    const baseUrl = this.getCurrentApi();
    const finalUrl = `${
      baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl
    }/${urlPath.startsWith('/') ? urlPath.slice(1) : urlPath}`;

    console.log('url href', finalUrl);
    return finalUrl;
  }
}
