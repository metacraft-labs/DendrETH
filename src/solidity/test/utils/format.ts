import { PointG1, PointG2 } from "@noble/bls12-381";
import { BitArray, BitVectorType } from "@chainsafe/ssz";
import { ssz } from "@chainsafe/lodestar-types";
import { hexToBytes, formatHex, bigint_to_array, bytesToHex } from "./bls";
import { SyncCommittee } from "@chainsafe/lodestar-types/lib/altair/sszTypes";
import { Proof } from "./index";
import * as constants from "./constants";

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

export function formatJSONUpdate(update, FORK_VERSION: string): FormatedJsonUpdate {
  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(update.sync_aggregate.sync_committee_bits);
  update.sync_aggregate.sync_committee_bits = [
    BigInt(
      '0b' +
      bitmask
        .toBoolArray()
        .map(x => (x ? '1' : '0'))
        .slice(0, 6)
        .reverse()
        .join(''),
    ).toString(),
    BigInt(
      '0b' +
      bitmask
        .toBoolArray()
        .map(x => (x ? '1' : '0'))
        .slice(6, 259)
        .reverse()
        .join(''),
    ).toString(),
    BigInt(
      '0b' +
      bitmask
        .toBoolArray()
        .map(x => (x ? '1' : '0'))
        .slice(259, 512)
        .reverse()
        .join(''),
    ).toString()
  ];
  let signature: PointG2 = PointG2.fromSignature(formatHex(update.sync_aggregate.sync_committee_signature));

  update.sync_aggregate.sync_committee_signature = [
    [
      bigint_to_array(55, 7, signature.toAffine()[0].c0.value),
      bigint_to_array(55, 7, signature.toAffine()[0].c1.value)
    ],
    [
      bigint_to_array(55, 7, signature.toAffine()[1].c0.value),
      bigint_to_array(55, 7, signature.toAffine()[1].c1.value)
    ]
  ];

  update.fork_version = FORK_VERSION;
  return update;
};

export function formatLightClientUpdate(update: FormatedJsonUpdate, proof: Proof) {
  return {
    attested_header: update.attested_header,
    finalized_header: update.finalized_header,
    finality_branch: update.finality_branch,
    sync_aggregate: update.sync_aggregate,
    signature_slot: update.signature_slot,
    fork_version: constants.ALTAIR_FORK_VERSION,
    next_sync_committee_root: hashTreeRootSyncCommitee(update.next_sync_committee),
    next_sync_committee_branch: update.next_sync_committee_branch,
    a: proof.a,
    b: proof.b,
    c: proof.c
  };
}

export function formatPubkeysToPoints(sync_commitee: SyncCommitee): PointG1[] {
  const points: PointG1[] = sync_commitee.pubkeys.map(x => PointG1.fromHex(formatHex(x)));
  return points;
}

export function hashTreeRootSyncCommitee(sync_commitee: SyncCommitee): string {
  let wrapper = SyncCommittee.defaultValue();
  wrapper.pubkeys = sync_commitee.pubkeys.map(hexToBytes);
  wrapper.aggregatePubkey = hexToBytes(sync_commitee.aggregate_pubkey);

  return "0x" + bytesToHex(SyncCommittee.hashTreeRoot(wrapper));
}

export function formatBitmask(sync_committee_bits: string): BitArray {
  const bitmask = new BitVectorType(512).fromJson(sync_committee_bits);
  return bitmask;
}
