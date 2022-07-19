import type * as T from "../types/basic-types";
import type * as I from "../types/interfaces";

// ===============
//  / CONSTANTS \
// ===============

@Contract
export class Constants {
    // PHASE 0
    DOMAIN_SYNC_COMMITTEE: T.DomainType = '0x07000000' as T.DomainType;
    GENESIS_FORK_VERSION: T.Version = '0x0' as T.Version;

    EPOCHS_PER_SYNC_COMMITTEE_PERIOD: T.Uint64 = 256;

    SLOTS_PER_EPOCH: T.Uint64 = 32;

    BLSPUBLICKEY_LENGTH: T.Uint64 = 96;

    SYNC_COMMITTEE_SIZE: T.Uint64 = 512;

    // ALTAIR
    FINALIZED_ROOT_INDEX: T.Uint64 = 105;
    FINALIZED_ROOT_DEPTH: T.Uint64 = 6;

    MIN_SYNC_COMMITTEE_PARTICIPANTS: T.Uint64 = 1;

    EMPTY_BEACON_HEADER: I.BeaconBlockHeader = {
        slot: 0 as T.Slot,
        proposer_index: 0 as T.ValidatorIndex,
        parent_root: '0x0' as T.Root,
        state_root: '0x0' as T.Root,
        body_root: '0x0' as T.Root,
    }

    EMPTY_SYNC_COMMITTEE: I.SyncCommittee = {
        pubkeys: [] as TList<T.BLSPubkey>,
        aggregate_pubkey: '0x0' as T.BLSPubkey,
    }

    EMPTY_LIGHT_CLIENT_UPDATE: I.LightClientUpdate = {
        header: this.EMPTY_BEACON_HEADER,
        next_sync_committee: this.EMPTY_SYNC_COMMITTEE,
        next_sync_committee_branch: [] as TList<T.Bytes32>,
        finality_header: this.EMPTY_BEACON_HEADER,
        finality_branch: [] as TList<TBytes>,
        sync_committee_bits: [] as T.Bitvector,
        sync_committee_signature: '0x0' as T.BLSSignature,
        fork_version: '0x0'as T.Version, 
    };
}

Dev.compileContract('compilation', new Constants());