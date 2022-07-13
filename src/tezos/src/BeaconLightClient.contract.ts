// ===========
//  / TYPES \
// ===========

type Uint64 = TNat;

type Bytes98 = TBytes;
type Bytes46 = TBytes;
type Bytes32 = TBytes;
type Bytes8 = TBytes;
type Bytes4 = TBytes;
type Bytes = TBytes;

type Bit = TNat; // 0 | 1

type Slot = Uint64;
type Epoch = Uint64;
type Gwei = Uint64;

type ValidatorIndex = Uint64;
type CommitteeIndex = Uint64;

type Root = Bytes32;
type Domain = Bytes32;
type DomainType = Bytes4;
type Version = Bytes4;

type BLSPubkey = Bytes98; // TKey
type BLSSignature = Bytes46; // TSignature

// ================
//  / INTERFACES \
// ================

// PHASE 0
interface ForkData {
    current_version: Version;
    genesis_validators_root: Root;
}

interface SigningData {
    object_root: Root;
    domain: Domain;
}

interface BeaconBlockHeader {
    slot: Slot;
    proposer_index: ValidatorIndex;
    parent_root: Root;
    state_root: Root;
    body_root: Root;
}

// ALTAIR
interface SyncCommittee {
    pubkeys: TList<BLSPubkey>;
    aggregate_pubkey: BLSPubkey;
}

interface LightClientSnapshot {
    // Beacon block header
    header: BeaconBlockHeader;
    // Sync committees corresponding to the header
    current_sync_committee: SyncCommittee;
    next_sync_committee: SyncCommittee;
}

interface LightClientUpdate {
    // Update beacon block header
    header: BeaconBlockHeader;
    // Next sync committee corresponding to the header
    next_sync_committee: SyncCommittee;
    next_sync_committee_branch: TList<Bytes32>;
    // Finality proof for the update header
    finality_header: BeaconBlockHeader;
    finality_branch: TList<TBytes>;
    // Sync committee aggregate signature
    sync_committee_bits: TList<Bit>;
    sync_committee_signature: BLSSignature;
    // Fork version for the aggregate signature
    fork_version: Version;
}

interface LightClientStore {
    snapshot: LightClientSnapshot;
    valid_updates: TList<LightClientUpdate>;
}

// ===============
//  / CONSTANTS \
// ===============

@Contract
class Constants {
    // PHASE 0
    DOMAIN_SYNC_COMMITTEE: DomainType = '0x07000000' as DomainType;
    GENESIS_FORK_VERSION: Version = '0x0' as Version;

    EPOCHS_PER_SYNC_COMMITTEE_PERIOD: Uint64 = 256;

    SLOTS_PER_EPOCH: Uint64 = 32;

    BLSPUBLICKEY_LENGTH: Uint64 = 96;

    SYNC_COMMITTEE_SIZE: Uint64 = 512;

    // ALTAIR
    FINALIZED_ROOT_INDEX: Uint64 = 105;
    FINALIZED_ROOT_DEPTH: Uint64 = 6;

    MIN_SYNC_COMMITTEE_PARTICIPANTS: Uint64 = 1;

    EMPTY_BEACON_HEADER_HASH: Bytes = '0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c' as Bytes;

    EMPTY_LIGHT_CLIENT_UPDATE: LightClientUpdate = {
        header: {
            slot: 0,
            proposer_index: 0,
            parent_root: '0x0',
            state_root: '0x0',
            body_root: '0x0',
        },
        next_sync_committee: {
            pubkeys: [],
            aggregate_pubkey: '0x0',
        },
        next_sync_committee_branch: [],
        finality_header: {
            slot: 0,
            proposer_index: 0,
            parent_root: '0x0',
            state_root: '0x0',
            body_root: '0x0',
        },
        finality_branch: [],
        sync_committee_bits: [],
        sync_committee_signature: '0x0',
        fork_version: '0x0',
    };
}

// =============
//  / HELPERS \
// =============

@Contract
class Helpers extends Constants {
    // Utils
    pow = (base: TNat, exponent: TNat): TNat => {
        if (base == 1 || exponent == 1) {
            return base;
        }
        if (exponent == 0) {
            return 1;
        }

        let result: TNat = 1;
        for (let i = 0; i < exponent; i += 1) {
            result = result * base;
        }

        return result;
    };

    getElementInBytesArrayAt = (index: Uint64, arr: TList<Bytes32>): Bytes32 => {
        if ((arr as TList<Bytes32>).size() == 0 || (arr as TList<Bytes32>).size() < index || index < 0) {
            Sp.failWith("Helpers: Invalid params!")
        }

        let i = 0;
        for (const ele of arr as TList<Bytes32>) {
            if (i == index) {
                return ele;
            }
            i += 1;
        }

        return '0x0' as Bytes32;
    };

    setElementInBytesArrayAt = (index: Uint64, element: Bytes, arr: TList<Bytes32>): TList<Bytes32> => {
        if ((arr as TList<Bytes32>).size() == 0 || (arr as TList<Bytes32>).size() < index || index < 0) {
            Sp.failWith("Helpers: Invalid params!")
        }

        let i = 0;
        const result_array: TList<Bytes32> = [];
        for (const e of arr as TList<Bytes32>) {
            if (i != index) {
                result_array.push(e);
                i += 1;
            } else {
                result_array.push(element);
            }
        }

        return result_array;
    };

    getElementInUintArrayAt = (index: Uint64, arr: TList<Uint64>): Uint64 => {
        if ((arr as TList<Uint64>).size() == 0 || (arr as TList<Uint64>).size() < index || index < 0) {
            Sp.failWith("Helpers: Invalid params!")
        }

        let i = 0;
        for (const ele of arr as TList<Uint64>) {
            if (i == index) {
                return ele;
            }
            i += 1;
        }

        return 0 as Uint64;
    };

    reverse64 = (value: Uint64): Uint64 => {
        let result_bit_list: TList<Bit> = [];
        let result: Uint64 = 0;

        let byte_1: TList<Bit> = [];
        let byte_2: TList<Bit> = [];
        let byte_3: TList<Bit> = [];
        let byte_4: TList<Bit> = [];
        let byte_5: TList<Bit> = [];
        let byte_6: TList<Bit> = [];
        let byte_7: TList<Bit> = [];
        let byte_8: TList<Bit> = [];

        for (let i = 0; i < 64; i += 1) {
            let bit: Bit = 0;
            if (value != 0) {
                bit = value % 2;
                value = Sp.ediv(value, 2).openSome().fst();
            }
            if (i < 8) {
                byte_1.push(bit);
            } else if (i < 16) {
                byte_2.push(bit);
            } else if (i < 24) {
                byte_3.push(bit);
            } else if (i < 32) {
                byte_4.push(bit);
            } else if (i < 40) {
                byte_5.push(bit);
            } else if (i < 48) {
                byte_6.push(bit);
            } else if (i < 56) {
                byte_7.push(bit);
            } else {
                byte_8.push(bit);
            }
        }

        byte_1 = byte_1.reverse();
        byte_2 = byte_2.reverse();
        byte_3 = byte_3.reverse();
        byte_4 = byte_4.reverse();
        byte_5 = byte_5.reverse();
        byte_6 = byte_6.reverse();
        byte_7 = byte_7.reverse();
        byte_8 = byte_8.reverse();

        for (let bit of byte_1) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_2) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_3) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_4) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_5) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_6) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_7) {
            result_bit_list.push(bit);
        }
        for (let bit of byte_8) {
            result_bit_list.push(bit);
        }

        let counter: TNat = 0;
        for (let bit of result_bit_list) {
            if (bit == 1) {
                result += this.pow(2, 64 - counter - 1);
            }   
            counter += 1;
        }

        return result;
    };

    to_little_endian_64 = (value: Uint64): Bytes8 => {
        return Sp.pack(this.reverse64(value));
    };

    // Main helpers
    compute_epoch_at_slot = (slot: Slot): Epoch => {
        return Sp.ediv(slot, this.SLOTS_PER_EPOCH).openSome().fst();
    };

    is_valid_merkle_branch = (
        leaf: Bytes32,
        branch: TList<Bytes32>,
        depth: Uint64,
        index: Uint64,
        root: Root,
    ): TBool => {
        let value = leaf;
        let i: TNat = 0;
        for (let n of branch) {
            if ((this.pow(Sp.ediv(index, 2).openSome().fst(), i) % 2) + depth == 0) {
                value = Sp.sha256((n as Bytes32).concat(value));
            } else {
                value = Sp.sha256(value.concat(n as Bytes32));
            }

            i += 1;
            if (i == depth) {
                // exit loop
                return value == root;
            }
        }
        return false;
    };

    get_power_of_two_ceil = (n: Uint64): Uint64 => {
        if (n <= 1) {
            return 1;
        } else if (n == 2) {
            return 2;
        } else {
            return (
                n *
                this.get_power_of_two_ceil(
                    Sp.ediv(n + 1, 2)
                        .openSome()
                        .fst(),
                )
            );
        }
    };

    merkle_root = (leaves: TList<Bytes32>): Bytes32 => {
        if ((leaves as TList<TBytes>).size() == 0) {
            return '0x0';
        } else if ((leaves as TList<TBytes>).size() == 1) {
            return Sp.sha256(this.getElementInBytesArrayAt(0, leaves) as TBytes);
        } else if ((leaves as TList<TBytes>).size() == 1) {
            return Sp.sha256(
                (this.getElementInBytesArrayAt(0, leaves) as TBytes).concat(
                    this.getElementInBytesArrayAt(1, leaves) as TBytes,
                ),
            );
        }

        const bottom_length: Uint64 = this.get_power_of_two_ceil((leaves as TList<TBytes>).size());
        const tree: TList<Bytes32> = [];
        for (let i = 0; i < (leaves as TList<TBytes>).size(); i += 1) {
            this.setElementInBytesArrayAt(bottom_length + i, this.getElementInBytesArrayAt(i, leaves), tree);
        }
        for (let i = bottom_length - 1; i > 0; i -= 1) {
            this.setElementInBytesArrayAt(
                i,
                Sp.sha256(
                    (this.getElementInBytesArrayAt(i * 2, tree) as TBytes).concat(
                        this.getElementInBytesArrayAt(i * 2 + 1, tree) as TBytes,
                    ),
                ),
                tree,
            );
        }

        return this.getElementInBytesArrayAt(1, tree);
    };

    hash_tree_root__fork_data = (fork_data: ForkData): Bytes32 => {
        return Sp.sha256((fork_data.current_version as TBytes).concat(fork_data.genesis_validators_root));
    };

    hash_tree_root__signing_data = (signing_data: SigningData): Bytes32 => {
        return Sp.sha256((signing_data.object_root as TBytes).concat(signing_data.domain));
    };

    hash_tree_root__block_header = (block_header: BeaconBlockHeader): Bytes32 => {
        const leaves: TList<Bytes32> = [];
        leaves.push(this.to_little_endian_64(block_header.slot));
        leaves.push(this.to_little_endian_64(block_header.proposer_index));
        leaves.push(block_header.parent_root);
        leaves.push(block_header.state_root);
        leaves.push(block_header.body_root);
        return this.merkle_root(leaves);
    };

    hash_tree_root__sync_committee = (sync_committee: SyncCommittee): Bytes32 => {
        if ((sync_committee.pubkeys as TList<TBytes>).size() != this.SYNC_COMMITTEE_SIZE) {
            Sp.failWith(
                'Invalid sync_committee size: Committee participant should be less than' +
                    this.SYNC_COMMITTEE_SIZE +
                    1 +
                    '!',
            );
        }

        const leaves: TList<Bytes32> = [];
        for (let key of sync_committee.pubkeys) {
            if ((key as TBytes).size() != this.BLSPUBLICKEY_LENGTH) {
                Sp.failWith('Invalid pubkey: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
            }
            leaves.push(Sp.sha256((key as TBytes).concat('0x00000000000000000000000000000000')));
        }
        const pubkeys_root = this.merkle_root(leaves);

        if ((sync_committee.aggregate_pubkey as TBytes).size() != this.BLSPUBLICKEY_LENGTH) {
            Sp.failWith('Invalid aggregate pubkey: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
        }
        const aggregate_pubkeys_root: Bytes32 = Sp.sha256(
            (sync_committee.aggregate_pubkey as TBytes).concat('0x00000000000000000000000000000000'),
        );

        return Sp.sha256(pubkeys_root.concat(aggregate_pubkeys_root));
    };

    compute_fork_data_root = (current_version: Version, genesis_validators_root: Root) => {
        return this.hash_tree_root__fork_data({
            current_version,
            genesis_validators_root,
        });
    };

    compute_signing_root = (block_header: BeaconBlockHeader, domain: Domain): Root => {
        return this.hash_tree_root__signing_data({
            object_root: this.hash_tree_root__block_header(block_header),
            domain,
        });
    };

    compute_domain = (
        domain_type: DomainType,
        fork_version: Version = this.GENESIS_FORK_VERSION,
        genesis_validators_root: Root = '0x0',
    ): Domain => {
        const fork_data_root: Root = this.compute_fork_data_root(fork_version, genesis_validators_root);
        return (domain_type as TBytes).concat(fork_data_root.slice(0, 28).openSome());
    };
}

// ===================
//  / Main Contract \
// ===================

@Contract
class BeaconLightClient extends Helpers {
    blsFastAggregateVerify = (pubkeys: TList<TBytes>, root: TBytes, signature: TBytes): TBool => {
        pubkeys;
        root;
        signature;
        return false;
    };

    validate_light_client_update = (
        snapshot: LightClientSnapshot,
        update: LightClientUpdate,
        genesis_validators_root: Root,
    ) => {
        // Verify update slot is larger than snapshot slot
        if (!(update.header.slot > snapshot.header.slot)) {
            Sp.failWith('Update validation failed: Update slot is before the current snapshot slot!');
        }

        // Verify update does not skip a sync committee period
        const snapshot_period: Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(snapshot.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();
        const update_period: Uint64 = Sp.ediv(
            this.compute_epoch_at_slot(update.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();

        if (update_period != snapshot_period && update_period != snapshot_period + 1) {
            Sp.failWith('Update validation failed: Update skips a sync comittee period!');
        }

        // Verify update header root is the finalized root of the finality header, if specified
        let signed_header: BeaconBlockHeader = {
            slot: 0,
            proposer_index: 0,
            parent_root: '0x0',
            state_root: '0x0',
            body_root: '0x0',
        };
        if (this.hash_tree_root__block_header(update.finality_header) == this.EMPTY_BEACON_HEADER_HASH) {
            signed_header = update.header;
            if ((update.finality_branch as TList<TBytes>).size() != 0) {
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
        let sync_committee: SyncCommittee = {
            pubkeys: [],
            aggregate_pubkey: '0x0',
        };
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
        const participant_pubkeys: TList<Bytes> = [];
        for (let i = 0; i < (update.sync_committee_bits as TList<TNat>).size(); i += 1) {
            if (this.getElementInUintArrayAt(i, update.sync_committee_bits) == 1) {
                participant_pubkeys.push(this.getElementInBytesArrayAt(i, sync_committee.pubkeys));
            }
        }

        const domain: Domain = this.compute_domain(
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

    apply_light_client_update = (snapshot: LightClientSnapshot, update: LightClientUpdate) => {
        const snapshot_period: TNat = Sp.ediv(
            this.compute_epoch_at_slot(snapshot.header.slot),
            this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
            .openSome()
            .fst();
        const update_period: TNat = Sp.ediv(
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
        store: LightClientStore,
        update: LightClientUpdate,
        current_slot: Slot,
        genesis_validators_root: Root,
    ) => {
        this.validate_light_client_update(store.snapshot, update, genesis_validators_root);
        (store.valid_updates as TList<LightClientUpdate>).push(update);

        const update_timeout: TNat = this.SLOTS_PER_EPOCH * this.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

        let current_sync_committee_participants = 0;
        for (let bit of update.sync_committee_bits) {
            if (bit == 1) {
                current_sync_committee_participants = current_sync_committee_participants + 1;
            }
        }
        if (
            current_sync_committee_participants * 3 >= (update.sync_committee_bits as TList<TNat>).size() * 2 &&
            this.hash_tree_root__block_header(update.finality_header) != this.EMPTY_BEACON_HEADER_HASH
        ) {
            // Apply update if (1) 2/3 quorum is reached and (2) we have a finality proof.
            // Note that (2) means that the current light client design needs finality.
            // It may be changed to re-organizable light client design. See the on-going issue eth2.0-specs#2182.
            this.apply_light_client_update(store.snapshot, update);
            store.valid_updates = [];
        } else if (current_slot > store.snapshot.header.slot + update_timeout) {
            let best_valid_update: LightClientUpdate = this.EMPTY_LIGHT_CLIENT_UPDATE;
            let most_active_participants: TNat = 0;
            for (let update of store.valid_updates) {
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
