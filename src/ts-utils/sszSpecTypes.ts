import {
  UintBigintType,
  BitVectorType,
  ByteVectorType,
  ContainerType,
  VectorCompositeType,
  ListBasicType
} from '@chainsafe/ssz';
import {
  SYNC_COMMITTEE_SIZE,
  NEXT_SYNC_COMMITTEE_DEPTH,
  FINALIZED_ROOT_DEPTH,
} from "@lodestar/params";
import { ssz } from '@lodestar/types';

export const SyncCommittee = new ContainerType(
  {
    pubkeys: new VectorCompositeType(ssz.BLSPubkey, SYNC_COMMITTEE_SIZE),
    aggregatePubkey: ssz.BLSPubkey,
  },
  {typeName: "SyncCommittee", jsonCase: "eth2"}
);

export const SyncCommitteeBits = new BitVectorType(SYNC_COMMITTEE_SIZE);

export const SyncAggregate = new ContainerType(
  {
    sync_committee_bits: SyncCommitteeBits,
    sync_committee_signature: ssz.BLSSignature,
  },
);

export type LightClientBootstrap = {
  header: typeof ssz.phase0.BeaconBlockHeader;
  current_sync_committee: ContainerType<{
    pubkeys: VectorCompositeType<ByteVectorType>;
    aggregatePubkey: ByteVectorType;
  }>;
  current_sync_committee_branch: VectorCompositeType<ByteVectorType>;

};

export class SSZSpecTypes {
  static updatesArray = new ListBasicType(new UintBigintType(4), 100);

  static LightClientBootstrap = new ContainerType<LightClientBootstrap>({
    header: ssz.phase0.BeaconBlockHeader,
    current_sync_committee: SyncCommittee,
    current_sync_committee_branch: new VectorCompositeType(ssz.Bytes32, 5),
  });
  static LightClientUpdate = new ContainerType(
    {
      attested_header: ssz.phase0.BeaconBlockHeader,
      next_sync_committee: SyncCommittee,
      next_sync_committee_branch: new VectorCompositeType(ssz.Bytes32, NEXT_SYNC_COMMITTEE_DEPTH),
      finalized_header: ssz.phase0.BeaconBlockHeader,
      finality_branch: new VectorCompositeType(ssz.Bytes32, FINALIZED_ROOT_DEPTH),
      sync_aggregate: SyncAggregate,
      signature_slot: ssz.Slot,
    },
  );
}
