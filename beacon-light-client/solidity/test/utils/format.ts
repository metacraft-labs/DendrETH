import { PointG2 } from '@noble/bls12-381';
import { BitVectorType } from '@chainsafe/ssz';
import { ssz } from '@lodestar/types';
import { hexToBytes, formatHex, bigint_to_array } from './bls';
import { Proof } from './index';

export interface JSONHeader {
  slot: string;
  proposer_index: string;
  parent_root: string;
  state_root: string;
  body_root: string;
}

interface SyncCommitee {
  pubkeys: string[];
  aggregate_pubkey: string;
}

export interface FormatedJsonUpdate {
  attested_header: JSONHeader;
  next_sync_committee: SyncCommitee;
  next_sync_committee_branch: string[];
  finalized_header: JSONHeader;
  finality_branch: string[];
  sync_aggregate: {
    sync_committee_bits: string[];
    sync_committee_signature: string[7][2][2];
  };
  signature_slot: string;
}

export function formatJSONBlockHeader(header: JSONHeader) {
  const block_header = ssz.phase0.BeaconBlockHeader.defaultValue();
  block_header.slot = parseInt(header.slot);
  block_header.proposerIndex = parseInt(header.proposer_index);
  block_header.parentRoot = hexToBytes(header.parent_root);
  block_header.stateRoot = hexToBytes(header.state_root);
  block_header.bodyRoot = hexToBytes(header.body_root);
  return block_header;
}

export function formatJSONUpdate(
  update,
  FORK_VERSION: string,
): FormatedJsonUpdate {
  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(
    update.sync_aggregate.sync_committee_bits,
  );
  update.sync_aggregate.sync_committee_bits = bitmask
    .toBoolArray()
    .map(x => (x ? '1' : '0'));
  let signature: PointG2 = PointG2.fromSignature(
    formatHex(update.sync_aggregate.sync_committee_signature),
  );

  update.sync_aggregate.sync_committee_signature = [
    [
      bigint_to_array(55, 7, signature.toAffine()[0].c0.value),
      bigint_to_array(55, 7, signature.toAffine()[0].c1.value),
    ],
    [
      bigint_to_array(55, 7, signature.toAffine()[1].c0.value),
      bigint_to_array(55, 7, signature.toAffine()[1].c1.value),
    ],
  ];

  update.fork_version = FORK_VERSION;
  return update;
}

export function formatLightClientUpdate(
  update: FormatedJsonUpdate,
  proof: Proof,
) {
  return {
    attested_header: update.attested_header,
    finalized_header: update.finalized_header,
    finality_branch: update.finality_branch,
    a: proof.a,
    b: proof.b,
    c: proof.c,
  };
}
