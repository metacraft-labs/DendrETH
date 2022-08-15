import {
  ByteVectorType,
  ContainerType,
  VectorCompositeType,
} from '@chainsafe/ssz';
import {
  SYNC_COMMITTEE_SIZE,
} from "@lodestar/params";
import { ssz } from '@lodestar/types';

export const SyncCommittee = new ContainerType(
  {
    pubkeys: new VectorCompositeType(ssz.BLSPubkey, SYNC_COMMITTEE_SIZE),
    aggregatePubkey: ssz.BLSPubkey,
  },
  {typeName: "SyncCommittee", jsonCase: "eth2"}
);

export const CurrentSyncCommittee = ContainerType<{
  pubkeys: VectorCompositeType<ByteVectorType>;
  aggregatePubkey: ByteVectorType;
}>

export type LightClientBootstrap = {
  header: typeof ssz.phase0.BeaconBlockHeader;
  current_sync_committee: ContainerType<{
    pubkeys: VectorCompositeType<ByteVectorType>;
    aggregatePubkey: ByteVectorType;
  }>;
  current_sync_committee_branch: VectorCompositeType<ByteVectorType>;

};

export class SSZSpecTypes {
  static LightClientBootstrap = new ContainerType<LightClientBootstrap>({
    header: ssz.phase0.BeaconBlockHeader,
    current_sync_committee: SyncCommittee,
    current_sync_committee_branch: new VectorCompositeType(ssz.Bytes32, 5),
  });
}
