import type * as T from "../types/basic-types";
import type * as I from "../types/interfaces";

import * as U from "../utils/Utils.contract";

// ===================
//  / Main Contract \
// ===================

@Contract
class BeaconLightClient extends U.Utils {
    
    blsFastAggregateVerify = (pubkeys: TList<T.BLSPubkey>, root: T.Root, signature: T.BLSSignature): TBool => {
        pubkeys;
        root;
        signature;
        return false;
    };

    validate_light_client_update = (
        snapshot: I.LightClientSnapshot,
        update: I.LightClientUpdate,
        genesis_validators_root: T.Root,
    ) => {
        // Verify update slot is larger than snapshot slot
        if (!(update.header.slot > snapshot.header.slot)) {
            Sp.failWith('Update validation failed: Update slot is before the current snapshot slot!');
        }

        // Verify update does not skip a sync committee period
        const snapshot_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(snapshot.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();
        const update_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(update.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();

        if (update_period != snapshot_period && update_period != snapshot_period + 1) {
            Sp.failWith('Update validation failed: Update skips a sync comittee period!');
        }

        // Verify update header root is the finalized root of the finality header, if specified
        let signed_header: I.BeaconBlockHeader = this.EMPTY_BEACON_HEADER;
        if (this.hash_tree_root__block_header(update.finality_header) == this.hash_tree_root__block_header(this.EMPTY_BEACON_HEADER)) {
            signed_header = update.header;
            if ((update.finality_branch as TList<T.Bytes32>).size() != 0) {
                Sp.failWith(
                    'Update validation failed: There is no finality header, but the finality branch is not empty!',
                );
            }
        } else {
            signed_header = update.finality_header;
            if (
                !this.is_valid_merkle_branch(
                    this.hash_tree_root__block_header(update.header),
                    update.next_sync_committee_branch,
                    this.FINALIZED_ROOT_INDEX,
                    this.FINALIZED_ROOT_DEPTH,
                    update.finality_header.state_root,
                )
            ) {
                Sp.failWith('Update validation failed: Merkle branch not valid!');
            }
        }

        // Verify update next sync committee if the update period incremented
        let sync_committee: I.SyncCommittee = this.EMPTY_SYNC_COMMITTEE;
        if (update_period == snapshot_period) {
            sync_committee = snapshot.current_sync_committee;
            if ((update.next_sync_committee_branch as TList<TBytes>).size() != 0) {
                Sp.failWith('Update validation failed: Next sync committee branch not empty!');
            }
        } else {
            sync_committee = snapshot.next_sync_committee;
            if (
                !this.is_valid_merkle_branch(
                    this.hash_tree_root__sync_committee(update.next_sync_committee),
                    update.next_sync_committee_branch,
                    this.FINALIZED_ROOT_INDEX,
                    this.FINALIZED_ROOT_DEPTH,
                    update.header.state_root,
                )
            ) {
                Sp.failWith('Update validation failed: Merkle branch not valid!');
            }
        }

        // Verify sync committee has sufficient participants
        let current_sync_committee_participants = 0;
        for (let bit of update.sync_committee_bits) {
            if (bit == 1) {
                current_sync_committee_participants = current_sync_committee_participants + 1;
            }
        }
        if (!(current_sync_committee_participants >= this.MIN_SYNC_COMMITTEE_PARTICIPANTS)) {
            Sp.failWith('Update validation failed: Sync committee does not have sufficient participants');
        }

        // Verify sync committee aggregate signature
        let participant_pubkeys: TList<T.BLSPubkey> = [];
        for (let i = 0; i < (update.sync_committee_bits as T.Bitvector).size(); i += 1) {
            if (this.getElementInUintArrayAt(i, update.sync_committee_bits) == 1) {
                participant_pubkeys.push(this.getElementInBytesArrayAt(i, sync_committee.pubkeys));
            }
        }
        participant_pubkeys = participant_pubkeys.reverse();

        const domain: T.Domain = this.compute_domain(
            this.DOMAIN_SYNC_COMMITTEE,
            update.fork_version,
            genesis_validators_root,
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

    apply_light_client_update = (snapshot: I.LightClientSnapshot, update: I.LightClientUpdate) => {
        const snapshot_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(snapshot.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();
        const update_period: T.Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(update.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();

        if (update_period == snapshot_period + 1) {
            snapshot.current_sync_committee = snapshot.next_sync_committee;
            snapshot.next_sync_committee = update.next_sync_committee;
        }
        snapshot.header = update.header;
    };

    process_light_client_update = (
        store: I.LightClientStore,
        update: I.LightClientUpdate,
        current_slot: T.Slot,
        genesis_validators_root: T.Root,
    ) => {
        this.validate_light_client_update(store.snapshot, update, genesis_validators_root);
        (store.valid_updates as TSet<I.LightClientUpdate>).add(update);

        const update_timeout: T.Uint64 = this.SLOTS_PER_EPOCH * this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

        let current_sync_committee_participants = 0;
        for (let bit of update.sync_committee_bits) {
            if (bit == 1) {
                current_sync_committee_participants = current_sync_committee_participants + 1;
            }
        }
        if (
            current_sync_committee_participants * 3 >= (update.sync_committee_bits as T.Bitvector).size() * 2 &&
            this.hash_tree_root__block_header(update.finality_header) != this.hash_tree_root__block_header(this.EMPTY_BEACON_HEADER)
        ) {
            // Apply update if (1) 2/3 quorum is reached and (2) we have a finality proof.
            // Note that (2) means that the current light client design needs finality.
            // It may be changed to re-organizable light client design. See the on-going issue eth2.0-specs#2182.
            this.apply_light_client_update(store.snapshot, update);
            store.valid_updates = [];
        } else if (current_slot > store.snapshot.header.slot + update_timeout) {
            let best_valid_update: I.LightClientUpdate = this.EMPTY_LIGHT_CLIENT_UPDATE;
            let most_active_participants: T.Uint64 = 0;
            for (let update of (store.valid_updates as TSet<I.LightClientUpdate>).elements()) {
                let current_update_active_participants = 0;
                for (let bit of update.sync_committee_bits) {
                    if (bit == 1) {
                        current_update_active_participants = current_update_active_participants + 1;
                    }
                }
                if (current_update_active_participants > most_active_participants) {
                    most_active_participants = current_update_active_participants;
                    best_valid_update = update;
                }
            }
            this.apply_light_client_update(store.snapshot, best_valid_update);
            store.valid_updates = [];
        }
    };
}

Dev.compileContract('compilation', new BeaconLightClient());
