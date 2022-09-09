import { PointG1 } from "@noble/bls12-381";
import { BitVectorType } from "@chainsafe/ssz";
import { ssz } from "../../node_modules/@chainsafe/lodestar-types/lib";
import { hexToBytes, formatHex } from "./bls";


interface JSONHeader {
    slot: string;
    proposer_index: string;
    parent_root: string;
    state_root: string;
    body_root: string;
}

interface JSONUpdate {
    attested_header: JSONHeader;
    next_sync_committee: {
        pubkeys: string[];
        aggregate_pubkey: string;
    };
    next_sync_committee_branch: string[];
    finalized_header: JSONHeader;
    finality_branch: string[];
    sync_aggregate: {
        sync_committee_bits: string[];
        sync_committee_signature: string;
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

export function formatJSONUpdate(update, FORK_VERSION: string) {
    update.sync_aggregate.sync_committee_bits =
        update.sync_aggregate.sync_committee_bits.replace("0x", "");

    update.sync_aggregate.sync_committee_bits = [
        "0x".concat(
            update.sync_aggregate.sync_committee_bits.slice(
                0,
                update.sync_aggregate.sync_committee_bits.length / 2
            )
        ),
        "0x".concat(
            update.sync_aggregate.sync_committee_bits.slice(
                update.sync_aggregate.sync_committee_bits.length / 2
            )
        ),
    ];

    update.fork_version = FORK_VERSION;
    return update;
};

export function formatPubkeysToPoints(update: JSONUpdate): PointG1[] {
    const points: PointG1[] = update.next_sync_committee.pubkeys.map(x => PointG1.fromHex(formatHex(x)));
    return points;
}

export function formatBitmask(update: JSONUpdate): BitVectorType {
    const bitmask: BitVectorType = new BitVectorType(512).fromJson(update.sync_aggregate.sync_committee_bits);
    return bitmask;
}