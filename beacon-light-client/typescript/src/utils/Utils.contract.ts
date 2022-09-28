import * as T from '../types/basic-types';
import type * as I from '../types/interfaces';

import * as H from '../utils/Helpers.contract';

// =============
//  / HELPERS \
// =============

@Contract
export class Utils extends H.Helpers {
    reverse64 = (v: T.Uint64): T.Uint64 => {
        let r: T.Uint64 = 0;
        let c: T.Uint64 = 63;
        for (let i = 63; i >= 0; i -= 1) {
            let p: T.Uint64 = 0;
            let m: T.Uint64 = Sp.ediv(i, 8).openSome().snd();

            p = i - m + (c - i);

            if (m == 0) {
                c -= 8;
            }

            if (Sp.ediv(v, 2).openSome().snd() == 1) {
                r += this.pow(2, p);
            }
            v = Sp.ediv(v, 2).openSome().fst();
        }
        return r as T.Uint64;
    };

    to_little_endian_64 = (value: T.Uint64): T.Bytes8 => {
        return Sp.pack(this.reverse64(value));
    };

    compute_epoch_at_slot = (slot: T.Slot): T.Epoch => {
        return Sp.ediv(slot, this.SLOTS_PER_EPOCH).openSome().fst();
    };

    get_power_of_two_ceil = (n: T.Uint64): T.Uint64 => {
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

    merkle_root = (leaves: TList<T.Bytes32>): T.Bytes32 => {
        if ((leaves as TList<T.Bytes32>).size() == 0) {
            return '0x0000000000000000000000000000000000000000000000000000000000000000';
        } else if ((leaves as TList<T.Bytes32>).size() == 1) {
            return Sp.sha256(this.getElementInBytesArrayAt(0, leaves) as T.Bytes32);
        } else if ((leaves as TList<T.Bytes32>).size() == 2) {
            return Sp.sha256(
                (this.getElementInBytesArrayAt(0, leaves) as TBytes).concat(
                    this.getElementInBytesArrayAt(1, leaves) as TBytes,
                ),
            );
        }

        const bottom_length: T.Uint64 = this.get_power_of_two_ceil((leaves as TList<T.Bytes32>).size());
        const tree: TList<T.Bytes32> = [];
        for (let i = 0; i < bottom_length * 2; i += 1) {
            tree.push('0x0000000000000000000000000000000000000000000000000000000000000000' as T.Bytes32);
        }
        for (let i = 0; i < (leaves as TList<T.Bytes32>).size(); i += 1) {
            tree = this.setElementInBytesArrayAt(bottom_length + i, this.getElementInBytesArrayAt(i, leaves), tree);
        }
        for (let i = bottom_length - 1; i > 0; i -= 1) {
            tree = this.setElementInBytesArrayAt(
                i,
                tree,
                Sp.sha256(
                    (this.getElementInBytesArrayAt(i * 2, tree) as T.Bytes32).concat(
                        this.getElementInBytesArrayAt(i * 2 + 1, tree) as T.Bytes32,
                    ),
                ),
            );
        }

        return this.getElementInBytesArrayAt(1, tree);
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
            if (Sp.ediv(index, this.pow(2, i)).openSome().fst() % 2 == 1) {
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

    hash_tree_root__fork_data = (fork_data: I.ForkData): T.Bytes32 => {
        return Sp.sha256('0x00000000000000000000000000000000000000000000000000000000' as T.Bytes)
            .concat(fork_data.current_version as T.Version)
            .concat(fork_data.genesis_validators_root);
    };

    hash_tree_root__signing_data = (signing_data: I.SigningData): T.Bytes32 => {
        return Sp.sha256((signing_data.object_root as T.Root).concat(signing_data.domain));
    };

    hash_tree_root__block_header = (block_header: I.BeaconBlockHeader): T.Bytes32 => {
        const leaves: TList<T.Bytes32> = [];
        leaves.push(block_header.body_root);
        leaves.push(block_header.state_root);
        leaves.push(block_header.parent_root);
        leaves.push(
            this.to_little_endian_64(block_header.proposer_index).concat(
                '0x000000000000000000000000000000000000000000000000' as T.Bytes,
            ),
        );
        leaves.push(
            this.to_little_endian_64(block_header.slot).concat(
                '0x000000000000000000000000000000000000000000000000' as T.Bytes,
            ),
        );
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

        if ((sync_committee.aggregate_pubkey as T.BLSPubkey).size() != this.BLSPUBLICKEY_LENGTH) {
            Sp.failWith('Invalid aggregate pubkey: Length should be equal to ' + this.BLSPUBLICKEY_LENGTH + '!');
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
        fork_version: T.Version,
        genesis_validators_root: T.Root,
    ): T.Domain => {
        const fork_data_root: T.Root = this.compute_fork_data_root(fork_version, genesis_validators_root);
        return (domain_type as T.DomainType).concat(fork_data_root.slice(0, 28).openSome());
    };
}

Dev.compileContract('compilation', new Utils());
