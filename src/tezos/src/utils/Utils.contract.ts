import type * as T from "../types/basic-types";
import type * as I from "../types/interfaces";

import * as H from "../utils/Helpers.contract";

// =============
//  / HELPERS \
// =============

@Contract
export class Utils extends H.Helpers {
    reverse64 = (value: T.Uint64): T.Uint64 => {
        let result: T.Uint64 = 0;

        let byte_1: T.Bitvector = []; let byte_2: T.Bitvector = [];
        let byte_3: T.Bitvector = []; let byte_4: T.Bitvector = [];
        let byte_5: T.Bitvector = []; let byte_6: T.Bitvector = [];
        let byte_7: T.Bitvector = []; let byte_8: T.Bitvector = [];

        for (let i = 0; i < 64; i += 1) {
            let bit: T.Bit = 0;
            if (value != 0) {
                bit = value % 2;
                value = Sp.ediv(value, 2).openSome().fst();
            }
            if (i < 8) { byte_1.push(bit); }
            else if (i < 16) { byte_2.push(bit); }
            else if (i < 24) { byte_3.push(bit); }
            else if (i < 32) { byte_4.push(bit); }
            else if (i < 40) { byte_5.push(bit); }
            else if (i < 48) { byte_6.push(bit); }
            else if (i < 56) { byte_7.push(bit); }
            else { byte_8.push(bit); }
        }
        
        let bytes: TList<T.Bitvector> = [byte_1, byte_2, byte_3, byte_4, byte_5, byte_6, byte_7, byte_8];
        let counter: T.Uint64 = 0;
        for (let byte of bytes) {
            for (let bit of byte) {
                if (bit == 1) {
                    result += this.pow(2, 64 - counter - 1);
                }   
                counter += 1;
            }
        }

        return result;
    };

    to_little_endian_64 = (value: T.Uint64): T.Bytes8 => {
        return Sp.pack(this.reverse64(value));
    };

    compute_epoch_at_slot = (slot: T.Slot): T.Epoch => {
        return Sp.ediv(slot, this.SLOTS_PER_EPOCH).openSome().fst();
    };

    is_valid_merkle_branch = (
        leaf: T.Bytes32,
        branch: TList<T.Bytes32>,
        depth: T.Uint64,
        index: T.Uint64,
        root: T.Root,
    ): TBool => {
        let value = leaf;
        let i: T.Uint64 = 0;
        for (let n of branch) {
            if ((this.pow(Sp.ediv(index, 2).openSome().fst(), i) % 2) + depth == 0) {
                value = Sp.sha256((n as T.Bytes32).concat(value));
            } else {
                value = Sp.sha256(value.concat(n as T.Bytes32));
            }

            i += 1;
            if (i == depth) {
                // exit loop
                return value == root;
            }
        }
        return false;
    };

    get_power_of_two_ceil = (n: T.Uint64): T.Uint64 => {
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

    merkle_root = (leaves: TList<T.Bytes32>): T.Bytes32 => {
        if ((leaves as TList<T.Bytes32>).size() == 0) {
            return '0x0';
        } else if ((leaves as TList<T.Bytes32>).size() == 1) {
            return Sp.sha256(this.getElementInBytesArrayAt(0, leaves) as T.Bytes32);
        } else if ((leaves as TList<T.Bytes32>).size() == 1) {
            return Sp.sha256(
                (this.getElementInBytesArrayAt(0, leaves) as TBytes).concat(
                    this.getElementInBytesArrayAt(1, leaves) as TBytes,
                ),
            );
        }

        const bottom_length: T.Uint64 = this.get_power_of_two_ceil((leaves as TList<T.Bytes32>).size());
        const tree: TList<T.Bytes32> = [];
        for (let i = 0; i < (leaves as TList<T.Bytes32>).size(); i += 1) {
            this.setElementInBytesArrayAt(bottom_length + i, this.getElementInBytesArrayAt(i, leaves), tree);
        }
        for (let i = bottom_length - 1; i > 0; i -= 1) {
            this.setElementInBytesArrayAt(
                i,
                Sp.sha256(
                    (this.getElementInBytesArrayAt(i * 2, tree) as T.Bytes32).concat(
                        this.getElementInBytesArrayAt(i * 2 + 1, tree) as T.Bytes32,
                    ),
                ),
                tree,
            );
        }

        return this.getElementInBytesArrayAt(1, tree);
    };

    hash_tree_root__fork_data = (fork_data: I.ForkData): T.Bytes32 => {
        return Sp.sha256((fork_data.current_version as T.Version).concat(fork_data.genesis_validators_root));
    };

    hash_tree_root__signing_data = (signing_data: I.SigningData): T.Bytes32 => {
        return Sp.sha256((signing_data.object_root as T.Root).concat(signing_data.domain));
    };

    hash_tree_root__block_header = (block_header: I.BeaconBlockHeader): T.Bytes32 => {
        const leaves: TList<T.Bytes32> = [];
        leaves.push(block_header.body_root);
        leaves.push(block_header.state_root);
        leaves.push(block_header.parent_root);
        leaves.push(this.to_little_endian_64(block_header.proposer_index));
        leaves.push(this.to_little_endian_64(block_header.slot));
        return this.merkle_root(leaves);
    };

    hash_tree_root__sync_committee = (sync_committee: I.SyncCommittee): T.Bytes32 => {
        if ((sync_committee.pubkeys as TList<T.BLSPubkey>).size() != this.SYNC_COMMITTEE_SIZE) {
            Sp.failWith(
                'Invalid sync_committee size: Committee participant should be less than' +
                    this.SYNC_COMMITTEE_SIZE +
                    1 +
                    '!',
            );
        }

        let leaves: TList<T.Bytes32> = [];
        for (let key of sync_committee.pubkeys) {
            if ((key as T.BLSPubkey).size() != this.BLSPUBLICKEY_LENGTH) {
                Sp.failWith('Invalid pubkey: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
            }
            leaves.push(Sp.sha256((key as T.BLSPubkey).concat('0x00000000000000000000000000000000')));
        }
        leaves = leaves.reverse();
        const pubkeys_root = this.merkle_root(leaves);

        if ((sync_committee.aggregate_pubkey as T.BLSPubkey).size() != this.BLSPUBLICKEY_LENGTH) {
            Sp.failWith('Invalid aggregate pubkey: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
        }
        const aggregate_pubkeys_root: T.Bytes32 = Sp.sha256(
            (sync_committee.aggregate_pubkey as T.BLSPubkey).concat('0x00000000000000000000000000000000'),
        );

        return Sp.sha256(pubkeys_root.concat(aggregate_pubkeys_root));
    };

    compute_fork_data_root = (current_version: T.Version, genesis_validators_root: T.Root) => {
        return this.hash_tree_root__fork_data({
            current_version,
            genesis_validators_root,
        });
    };

    compute_signing_root = (block_header: I.BeaconBlockHeader, domain: T.Domain): T.Root => {
        return this.hash_tree_root__signing_data({
            object_root: this.hash_tree_root__block_header(block_header),
            domain,
        });
    };

    compute_domain = (
        domain_type: T.DomainType,
        fork_version: T.Version = this.GENESIS_FORK_VERSION,
        genesis_validators_root: T.Root = '0x0',
    ): T.Domain => {
        const fork_data_root: T.Root = this.compute_fork_data_root(fork_version, genesis_validators_root);
        return (domain_type as T.DomainType).concat(fork_data_root.slice(0, 28).openSome());
    };
}

Dev.compileContract('compilation', new Utils());