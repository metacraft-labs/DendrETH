// ================
//  / INTERFACES \
// ================

import type * as T from "../types/basic-types";

// PHASE 0
export interface ForkData {
    current_version: T.Version;
    genesis_validators_root: T.Root;
}

export interface SigningData {
    object_root: T.Root;
    domain: T.Domain;
}

export interface BeaconBlockHeader {
    slot: T.Slot;
    proposer_index: T.ValidatorIndex;
    parent_root: T.Root;
    state_root: T.Root;
    body_root: T.Root;
}

// ALTAIR
export interface SyncCommittee {
    pubkeys: TList<T.BLSPubkey>;
    aggregate_pubkey: T.BLSPubkey;
}

export interface LightClientSnapshot {
    // Beacon block header
    header: BeaconBlockHeader;
    // Sync committees corresponding to the header
    current_sync_committee: SyncCommittee;
    next_sync_committee: SyncCommittee;
}

export interface LightClientUpdate {
    // Update beacon block header
    header: BeaconBlockHeader;
    // Next sync committee corresponding to the header
    next_sync_committee: SyncCommittee;
    next_sync_committee_branch: TList<T.Bytes32>;
    // Finality proof for the update header
    finality_header: BeaconBlockHeader;
    finality_branch: TList<T.Bytes32>;
    // Sync committee aggregate signature
    sync_committee_bits: T.Bitvector;
    sync_committee_signature: T.BLSSignature;
    // Fork version for the aggregate signature
    fork_version: T.Version;
}

export interface LightClientStore {
    snapshot: LightClientSnapshot;
    valid_updates: TSet<LightClientUpdate>;
}