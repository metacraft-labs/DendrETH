const {
    Sp,
    empty_bytes,
    Uint8ArrayToHexString
} = require("./environment");


// ===============
//  / CONSTANTS \
// ===============

// @Contract
class Constants {
    // PHASE 0
    DOMAIN_SYNC_COMMITTEE = [7, 0, 0, 0];
    GENESIS_FORK_VERSION = empty_bytes(4);

    EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;

    SLOTS_PER_EPOCH = 32;

    BLSPUBLICKEY_LENGTH = 48;

    SYNC_COMMITTEE_SIZE = 512;

    // ALTAIR
    FINALIZED_ROOT_INDEX = 105;
    FINALIZED_ROOT_DEPTH = 6;

    NEXT_SYNC_COMMITTEE_INDEX = 55;
    NEXT_SYNC_COMMITTEE_DEPTH = 5;

    MIN_SYNC_COMMITTEE_PARTICIPANTS = 1;

    EMPTY_BEACON_HEADER = {
        slot: 0,
        proposer_index: 0,
        parent_root: empty_bytes(32),
        state_root: empty_bytes(32),
        body_root: empty_bytes(32),
    };

    EMPTY_SYNC_COMMITTEE = {
        pubkeys: [],
        aggregate_pubkey: empty_bytes(98),
    };

    EMPTY_LIGHT_CLIENT_UPDATE = {
        header: this.EMPTY_BEACON_HEADER,
        next_sync_committee: this.EMPTY_SYNC_COMMITTEE,
        next_sync_committee_branch: [],
        finality_header: this.EMPTY_BEACON_HEADER,
        finality_branch: [],
        sync_committee_bits: [],
        sync_committee_signature: empty_bytes(46),
        fork_version: empty_bytes(4),
    };
};

// =============
//  / HELPERS \
// =============

// @Contract
class Helpers extends Constants {
    pow = (base, exponent) => {
        if (base == 1) {
            return base;
        }

        let result = 1;
        for (let i = 0; i < exponent; i += 1) {
            result = result * base;
        }

        return result;
    };

    getElementInUintArrayAt = (index, arr) => {
        if (index >= arr.length) {
            return 0;
        }

        let i = 0;
        for (const ele of arr) {
            if (i == index) {
                return ele;
            }
            i += 1;
        }

        Sp.failWith("Helpers: Invalid params!");
        return 0;
    };

    getElementInBytesArrayAt = (index, arr) => {
        if (index >= arr.length) {
            return empty_bytes(32);
        }

        let i = 0;
        for (const ele of arr) {
            if (i == index) {
                return ele;
            }
            i += 1;
        }

        Sp.failWith("Helpers: Invalid params!");
        return empty_bytes(32);
    };

    setElementInBytesArrayAt = (index, arr, element) => {
        if (index >= arr.length) {
            Sp.failWith("Helpers: Invalid params!");
        }

        let i = 0;
        const result_array = [];
        for (const e of arr) {
            if (i != index) {
                result_array.splice(0, 0, e);
            } else {
                result_array.splice(0, 0, element);
            }
            i += 1;
        }

        return result_array.reverse();
    };
};


// ===========
//  / UTILS \
// ===========

// @Contract
class Utils extends Helpers {
    reverse64 = (v) => {
        let r = 0;
        let c = 63;
        for (let i = 63; i >= 0; i -= 1) {
            let p = 0;
            let m = Sp.ediv(i, 8).openSome().snd();

            p = (i - m + (c - i));

            if (m == 0) {
                c -= 8;
            }

            if (Sp.ediv(v, 2).openSome().snd() == 1) {
                r += this.pow(2, p);
            }
            v = Sp.ediv(v, 2).openSome().fst();
        }
        return r;
    };

    to_little_endian_64 = (value) => {
        const bytesReverseValue = Sp.pack(this.reverse64(value));
        return [...Array(8 - bytesReverseValue.length).fill(0), ...bytesReverseValue];
    };

    compute_epoch_at_slot = (slot) => {
        return Sp.ediv(slot, this.SLOTS_PER_EPOCH).openSome().fst();
    };

    get_power_of_two_ceil = (n) => {
        if (n <= 1) {
            return 1;
        } else if (n == 2) {
            return 2;
        } else {
            return (
                2 *
                this.get_power_of_two_ceil(
                    Sp.ediv(n + 1, 2)
                        .openSome()
                        .fst(),
                )
            );
        }
    };

    merkle_root = (leaves) => {
        if (leaves.length == 0) {
            return empty_bytes(32);
        } else if (leaves.length == 1) {
            return Sp.sha256(this.getElementInBytesArrayAt(0, leaves));
        } else if (leaves.length == 2) {
            return Sp.sha256([
                ...this.getElementInBytesArrayAt(0, leaves),
                ...this.getElementInBytesArrayAt(1, leaves)
            ]);
        }

        const bottom_length = this.get_power_of_two_ceil(leaves.length);
        let tree = Array(bottom_length * 2).fill(empty_bytes(32));
        for (let i = 0; i < leaves.length; i += 1) {
            tree = this.setElementInBytesArrayAt(bottom_length + i, tree, this.getElementInBytesArrayAt(i, leaves));
        }
        for (let i = bottom_length - 1; i > 0; i -= 1) {
            tree = this.setElementInBytesArrayAt(
                i,
                tree,
                Sp.sha256([
                    ...this.getElementInBytesArrayAt(i * 2, tree),
                    ...this.getElementInBytesArrayAt(i * 2 + 1, tree)
                ])
            );
        }

        return this.getElementInBytesArrayAt(1, tree);
    };

    is_valid_merkle_branch = (
        leaf,
        branch,
        depth,
        index,
        root,
    ) => {
        let value = leaf;
        let i = 0;
        for (let n of branch) {
            // if ((this.pow(Sp.ediv(index, 2).openSome().fst(), i) % 2) + depth == 0) {
            if (Sp.ediv(index, this.pow(2, i)).openSome().fst() % 2 == 1) {
                value = Sp.sha256([...n, ...value]);
            } else {
                value = Sp.sha256([...value, ...n]);
            }

            i += 1;
            if (i == depth) {
                // exit loop
                return Uint8ArrayToHexString(value) == Uint8ArrayToHexString(root);
            }
        }
        return false;
    };

    hash_tree_root__fork_data = (fork_data) => {
        return Sp.sha256([...Array(28).fill(0), ...fork_data.current_version, ...fork_data.genesis_validators_root]);
    };

    hash_tree_root__signing_data = (signing_data) => {
        return Sp.sha256([...signing_data.object_root, ...signing_data.domain]);
    };

    hash_tree_root__block_header = (block_header) => {
        const leaves = [];
        leaves.splice(0, 0, block_header.body_root);
        leaves.splice(0, 0, block_header.state_root);
        leaves.splice(0, 0, block_header.parent_root);
        leaves.splice(0, 0, [...this.to_little_endian_64(block_header.proposer_index), ...Array(24).fill(0)]);
        leaves.splice(0, 0, [...this.to_little_endian_64(block_header.slot), ...Array(24).fill(0)]);
        return this.merkle_root(leaves);
    };

    hash_tree_root__sync_committee = (sync_committee) => {
        if (sync_committee.pubkeys.length != this.SYNC_COMMITTEE_SIZE) {
            Sp.failWith(
                'Invalid sync_committee size: Committee participant should be equal to' +
                this.SYNC_COMMITTEE_SIZE +
                '!',
            );
        }

        if (sync_committee.aggregate_pubkey.length != this.BLSPUBLICKEY_LENGTH) {
            Sp.failWith('Invalid aggregate pubkey: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
        }

        let leaves = [];
        for (let key of sync_committee.pubkeys) {
            if (key.length != this.BLSPUBLICKEY_LENGTH) {
                Sp.failWith('Invalid pubkey length: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
            }
            leaves.splice(0, 0, Sp.sha256([...key, ...Array(16).fill(0)]));
        }
        leaves = leaves.reverse();
        const pubkeys_root = this.merkle_root(leaves);

        const aggregate_pubkeys_root = Sp.sha256(
            [...sync_committee.aggregate_pubkey, ...Array(16).fill(0)]
        );

        return Sp.sha256([...pubkeys_root, ...aggregate_pubkeys_root]);
    };

    compute_fork_data_root = (current_version, genesis_validators_root) => {
        return this.hash_tree_root__fork_data({
            current_version,
            genesis_validators_root,
        });
    };

    compute_signing_root = (block_header, domain) => {
        return this.hash_tree_root__signing_data({
            object_root: this.hash_tree_root__block_header(block_header),
            domain,
        });
    };

    compute_domain = (
        domain_type,
        fork_version,
        genesis_validators_root,
    ) => {
        const fork_data_root = this.compute_fork_data_root(fork_version, genesis_validators_root);
        return [...domain_type, ...fork_data_root.slice(0, 28)];
    };
};


// ===================
//  / Main Contract \
// ===================

class BeaconLightClient extends Utils {
    constructor(_store, _genesis_validators_root) {
        super();
        this.store = _store;
        this.genesis_validators_root = _genesis_validators_root;
    }

    blsFastAggregateVerify = (pubkeys, root, signature) => {
        pubkeys;
        root;
        signature;
        return true;
    };

    compute_sync_committee_period = (slot) => {
        return Sp.ediv(
            this.compute_epoch_at_slot(slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();
    };

    validate_light_client_update = (
        update
    ) => {
        // Verify update slot is larger than snapshot slot
        if (!(update.attested_header.slot > this.store.snapshot.header.slot)) {
            Sp.failWith('Update validation failed: Update slot is before the current snapshot slot!');
        }

        // Verify update does not skip a sync committee period
        const update_period = this.compute_sync_committee_period(update.attested_header.slot);
        const snapshot_period = this.compute_sync_committee_period(this.store.snapshot.header.slot);

        if (update_period != snapshot_period && update_period != snapshot_period + 1) {
            console.log(update_period, snapshot_period);
            Sp.failWith('Update validation failed: Update skips a sync comittee period!');
        }

        // Verify update header root is the finalized root of the finality header, if specified
        let signed_header = this.EMPTY_BEACON_HEADER;
        if (this.hash_tree_root__block_header(update.finalized_header) == this.hash_tree_root__block_header(this.EMPTY_BEACON_HEADER)) {
            signed_header = update.attested_header;
            if (update.finality_branch.length != 0) {
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
        let sync_committee = this.EMPTY_SYNC_COMMITTEE;
        if (update_period == snapshot_period) {
            sync_committee = this.store.snapshot.current_sync_committee;
            if (update.next_sync_committee_branch.length != 0) {
                Sp.failWith('Update validation failed: Next sync committee branch not empty!');
            }
        } else {
            sync_committee = update.next_sync_committee;
            if (
                !this.is_valid_merkle_branch(
                    this.hash_tree_root__sync_committee(update.next_sync_committee),
                    update.next_sync_committee_branch,
                    this.NEXT_SYNC_COMMITTEE_DEPTH,
                    this.NEXT_SYNC_COMMITTEE_INDEX,
                    update.attested_header.state_root,
                )
            ) {
                Sp.failWith('Update validation failed: Merkle branch not valid!');
            }
        }

        // Verify sync committee aggregate signature
        let participants_pubkeys = [];
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

        const domain = this.compute_domain(
            this.DOMAIN_SYNC_COMMITTEE,
            this.GENESIS_FORK_VERSION,
            this.genesis_validators_root,
        );
        const signing_root = this.compute_signing_root(signed_header, domain);

        const valid = this.blsFastAggregateVerify(
            participants_pubkeys,
            signing_root,
            update.sync_committee_signature,
        );

        if (!valid) {
            Sp.failWith('Update validation failed: Fast aggregate verification failed!');
        }
    };

    apply_light_client_update = (update) => {
        const update_period = this.compute_sync_committee_period(update.attested_header.slot);
        const snapshot_period = this.compute_sync_committee_period(this.store.snapshot.header.slot);

        if (update_period == snapshot_period + 1) {
            this.store.snapshot.current_sync_committee = update.next_sync_committee;
        }
        this.store.snapshot.header = update.attested_header;
    };

    process_light_client_update = (
        update,
        current_slot
    ) => {
        this.validate_light_client_update(update);
        this.store.valid_updates.splice(0, 0, update);

        const update_timeout = this.SLOTS_PER_EPOCH * this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

        let committee_participants = 0;
        for (let bit of update.sync_aggregate.sync_committee_bits) {
            if (bit == 1) {
                committee_participants = committee_participants + 1;
            }
        }
        if (
            committee_participants * 3 >= update.sync_aggregate.sync_committee_bits.length * 2 &&
            this.hash_tree_root__block_header(update.finalized_header) != this.hash_tree_root__block_header(this.EMPTY_BEACON_HEADER)
        ) {
            // Apply update if (1) 2/3 quorum is reached and (2) we have a finality proof.
            // Note that (2) means that the current light client design needs finality.
            // It may be changed to re-organizable light client design. See the on-going issue eth2.0-specs#2182.
            this.apply_light_client_update(update);
            this.store.valid_updates = [];
        } else if (current_slot > this.store.snapshot.header.slot + update_timeout) {
            let best_valid_update = this.EMPTY_LIGHT_CLIENT_UPDATE;
            let most_active_participants = 0;
            for (let update of this.store.valid_updates) {
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
};

module.exports = {
    Sp,
    BeaconLightClient,
    Utils,
    Helpers,
    Constants
};