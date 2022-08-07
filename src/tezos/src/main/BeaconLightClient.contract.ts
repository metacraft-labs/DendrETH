import type * as T from "../types/basic-types";
import type * as I from "../types/interfaces";

import * as U from "../utils/Utils.contract";

// ===================
//  / Main Contract \
// ===================

@Contract
class BeaconLightClient extends U.Utils {
    constructor(public store: I.LightClientStore, public genesis_validators_root: T.Bytes32) {
        super();
    }

    blsFastAggregateVerify = (pubkeys: TList<T.BLSPubkey>, root: T.Root, signature: T.BLSSignature): TBool => {
        pubkeys;
        root;
        signature;
        return true;
    };

    validate_light_client_update = (
        update: I.LightClientUpdate,
    ) => {
        // Verify update slot is larger than snapshot slot
        if (!(update.attested_header.slot > this.store.snapshot.header.slot)) {
            Sp.failWith('Update validation failed: Update slot is before the current snapshot slot!');
        }

        // Verify update does not skip a sync committee period
        const snapshot_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(this.store.snapshot.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();
        const update_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(update.attested_header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();

        if (update_period != snapshot_period && update_period != snapshot_period + 1) {
            Sp.failWith('Update validation failed: Update skips a sync comittee period!');
        }

        // Verify update header root is the finalized root of the finality header, if specified
        let signed_header: I.BeaconBlockHeader = this.EMPTY_BEACON_HEADER;
        if (this.hash_tree_root__block_header(update.finalized_header) == this.hash_tree_root__block_header(this.EMPTY_BEACON_HEADER)) {
            signed_header = update.attested_header;
            if ((update.finality_branch as TList<T.Bytes32>).size() != 0) {
                Sp.failWith(
                    'Update validation failed: There is no finality header, but the finality branch is not empty!',
                );
            }
        } else {
            signed_header = update.finalized_header;
            if (
                !this.is_valid_merkle_branch(
                    this.hash_tree_root__block_header(update.finalized_header),
                    update.finality_branch,
                    this.FINALIZED_ROOT_DEPTH,
                    this.FINALIZED_ROOT_INDEX,
                    update.attested_header.state_root,
                )
            ) {
                Sp.failWith('Update validation failed: Merkle branch not valid!');
            }
        }

        // Verify update next sync committee if the update period incremented
        let sync_committee: I.SyncCommittee = this.EMPTY_SYNC_COMMITTEE;
        if (update_period == snapshot_period) {
            sync_committee = this.store.snapshot.current_sync_committee;
            if ((update.next_sync_committee_branch as TList<TBytes>).size() != 0) {
                Sp.failWith('Update validation failed: Next sync committee branch not empty!');
            }
        } else {
            sync_committee = this.store.snapshot.next_sync_committee;
            if (
                !this.is_valid_merkle_branch(
                    this.hash_tree_root__sync_committee(update.next_sync_committee),
                    update.next_sync_committee_branch,
                    this.FINALIZED_ROOT_DEPTH,
                    this.FINALIZED_ROOT_INDEX,
                    update.attested_header.state_root,
                )
            ) {
                Sp.failWith('Update validation failed: Merkle branch not valid!');
            }
        }

        // Verify sync committee aggregate signature
        let participants_pubkeys: TList<T.Bytes> = [];
        for (let i = 0; i < update.sync_aggregate.sync_committee_bits.length; i += 1) {
            if (this.getElementInUintArrayAt(i, update.sync_aggregate.sync_committee_bits) == 1) {
                participants_pubkeys.splice(0, 0, this.getElementInBytesArrayAt(i, sync_committee.pubkeys));
            }
        }

        // Verify sync committee has sufficient participants
        if (!(participants_pubkeys.length >= this.MIN_SYNC_COMMITTEE_PARTICIPANTS)) {
            Sp.failWith('Update validation failed: Sync committee does not have sufficient participants');
        }
        participants_pubkeys = participants_pubkeys.reverse();

        const domain: T.Domain = this.compute_domain(
            this.DOMAIN_SYNC_COMMITTEE,
            this.GENESIS_FORK_VERSION,
            this.genesis_validators_root,
        );
        const signing_root = this.compute_signing_root(signed_header, domain);

        const valid: TBool = this.blsFastAggregateVerify(
            participant_pubkeys,
            signing_root,
            update.sync_committee_signature,
        );

        if (!valid) {
            Sp.failWith('Update validation failed: Fast aggregate verification failed!');
        }
    };

    apply_light_client_update = (update: I.LightClientUpdate) => {
        const snapshot_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(this.store.snapshot.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();

        const update_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(update.attested_header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();

        if (update_period == snapshot_period + 1) {
            this.store.snapshot.current_sync_committee = update.next_sync_committee;
        }
        this.store.snapshot.header = update.attested_header;
    };

    process_light_client_update = (
        update: I.LightClientUpdate,
        current_slot: T.Slot
    ) => {
        this.validate_light_client_update(update);
        (this.store.valid_updates as TSet<I.LightClientUpdate>).add(update);

        const update_timeout: T.Uint64 = this.SLOTS_PER_EPOCH * this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

        let committee_participants = 0;
        for (let bit of update.sync_aggregate.sync_committee_bits) {
            if (bit == 1) {
                committee_participants = committee_participants + 1;
            }
        }
        if (
            committee_participants * 3 >= (update.sync_aggregate.sync_committee_bits as T.Bitvector).size() * 2 &&
            this.hash_tree_root__block_header(update.finalized_header) != this.hash_tree_root__block_header(this.EMPTY_BEACON_HEADER)
        ) {
            // Apply update if (1) 2/3 quorum is reached and (2) we have a finality proof.
            // Note that (2) means that the current light client design needs finality.
            // It may be changed to re-organizable light client design. See the on-going issue eth2.0-specs#2182.
            this.apply_light_client_update(update);
            this.store.valid_updates = [];
        } else if (current_slot > this.store.snapshot.header.slot + update_timeout) {
            let best_valid_update: I.LightClientUpdate = this.EMPTY_LIGHT_CLIENT_UPDATE;
            let most_active_participants: T.Uint64 = 0;
            for (let update of (this.store.valid_updates as TSet<I.LightClientUpdate>).elements()) {
                let active_participants = 0;
                for (let bit of update.sync_committee_bits) {
                    if (bit == 1) {
                        active_participants = active_participants + 1;
                    }
                }
                if (active_participants > most_active_participants) {
                    most_active_participants = active_participants;
                    best_valid_update = update;
                }
            }
            this.apply_light_client_update(best_valid_update);
            this.store.valid_updates = [];
        }
    };
}

Dev.compileContract('compilation', new BeaconLightClient({
    snapshot: {
        header: {
            slot: 0,
            proposer_index: 0,
            parent_root: "0x0" as T.Root,
            state_root: "0x0" as T.Root,
            body_root: "0x0" as T.Root
        },
        current_sync_committee: {
            pubkeys: [],
            aggregate_pubkey: "0x0" as T.BLSPubkey
        },
        next_sync_committee: {
            pubkeys: [],
            aggregate_pubkey: "0x0" as T.BLSPubkey
        }
    },
    valid_updates: [] as TSet<I.LightClientUpdate>
} as I.LightClientStore,
    "0x32251a5a748672e3acb1e574ec27caf3b6be68d581c44c402eb166d71a89808e"
));
