import { sha256 } from 'ethers/lib/utils';
import { IBeaconApi } from '@dendreth/relay/abstraction/beacon-api-interface';
import { Config } from '@dendreth/relay/constants/constants';
import { BeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { getPoseidonInputs } from '@dendreth/relay/utils/telepathy_utils';
import { ValueOf } from '@chainsafe/ssz';
import { reverseEndianness } from '@dendreth/utils/ts-utils/hex-utils';

export const getConstructorArgs = async (
  beaconApi: IBeaconApi,
  slot: number,
  config: Config,
) => {
  const { ssz } = await import('@lodestar/types');

  const finalizedBlockHeader = await beaconApi.getFinalizedBlockHeader(slot);
  const finalizedHeaderRoot =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(finalizedBlockHeader);

  const executioStateRoot = await beaconApi.getExecutionStateRoot(slot);

  let result = sha256(
    config.FORK_VERSION.padEnd(66, '0') +
      config.GENESIS_VALIDATORS_ROOT.slice(2),
  );

  return [
    finalizedHeaderRoot,
    finalizedBlockHeader.slot,
    finalizedHeaderRoot,
    executioStateRoot,
    config.DOMAIN_SYNC_COMMITTEE + result.slice(2, 58),
  ];
};

export const getTelepathyConstructorArgs = async (
  beaconApi: BeaconApi,
  slot: number,
) => {
  const { ssz } = await import('@lodestar/types');
  const config = await beaconApi.getSpecConfig();
  const genesisConfig = await beaconApi.getGenesisData();

  const slotsPerPeriod =
    config.SLOTS_PER_EPOCH * config.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

  const state: ValueOf<typeof ssz.deneb.BeaconState> = (
    await beaconApi.getBeaconState(BigInt(slot))
  ).beaconState;

  const syncCommitteePoseidon = await getPoseidonInputs(
    state.currentSyncCommittee.pubkeys.map(p => bytesToHex(p)),
  );

  let syncCommitteePoseidonBytes =
    '0x' +
    reverseEndianness(
      BigInt(syncCommitteePoseidon).toString(16).padStart(64, '0'),
    );

  return [
    '0x' + bytesToHex(genesisConfig.genesisValidatorsRoot),
    genesisConfig.genesisTime,
    config.SECONDS_PER_SLOT,
    slotsPerPeriod,
    Math.floor(slot / slotsPerPeriod),
    syncCommitteePoseidonBytes,
    config.DEPOSIT_CHAIN_ID,
    337,
  ];
};
