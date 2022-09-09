// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import '../../utils/Bitfield.sol';
import '../../spec/BeaconChain.sol';
import 'hardhat/console.sol';

/** Etherum beacon light client.
 *  Current arthitecture diverges from spec's proposed updated splitting them into:
 *  - Finalized header updates: To import a recent finalized header signed by a known sync committee by `import_finalized_header`.
 *  - Sync period updates: To advance to the next committee by `import_next_sync_committee`.
 *
 *  To stay synced to the current sync period it needs:
 *  - Get finalized_header_update and sync_period_update at least once per period.
 *
 *  To get light-client best finalized update at period N:
 *  - Fetch best finalized block's sync_aggregate_header in period N
 *  - Fetch parent_block/attested_block by sync_aggregate_header's parent_root
 *  - Fetch finalized_checkpoint_root and finalized_checkpoint_root_witness in attested_block
 *  - Fetch finalized_header by finalized_checkpoint_root
 *
 *  - sync_aggregate -> parent_block/attested_block -> finalized_checkpoint -> finalized_header
 *
 *  To get light-client sync period update at period N:
 *  - Fetch the finalized_header in light-client
 *  - Fetch the finalized_block by finalized_header.slot
 *  - Fetch next_sync_committee and next_sync_committee_witness in finalized_block
 *
 *  - finalized_header -> next_sync_committee
 *
 *  ```
 *                        Finalized               Block   Sync
 *                        Checkpoint              Header  Aggreate
 *  ----------------------|-----------------------|-------|---------> time
 *                         <---------------------   <----
 *                          finalizes               signs
 *  ```
 *
 *  To initialize, it needs:
 *  - BLS verify contract
 *  - Trust finalized_header
 *  - current_sync_committee of the trust finalized_header
 *  - genesis_validators_root of genesis state
 *
 *  When to trigger a committee update sync:
 *
 *   period 0         period 1         period 2
 *  -|----------------|----------------|----------------|-> time
 *               | now
 *                - active current_sync_committee
 *                - known next_sync_committee, signed by current_sync_committee
 *
 *
 *  next_sync_committee can be imported at any time of the period, not strictly at the period borders.
 *  - No need to query for period 0 next_sync_committee until the end of period 0
 *  - After the import next_sync_committee of period 0, populate period 1's committee
 */

interface IBLS {
  function fast_aggregate_verify(
    bytes[] calldata pubkeys,
    bytes calldata message,
    bytes calldata signature
  ) external pure returns (bool);
}

contract BeaconLightClient is BeaconChain, Bitfield {
  event FinalizedHeaderImported(BeaconBlockHeader finalized_header);
  event NextSyncCommitteeImported(
    uint64 indexed period,
    bytes32 indexed next_sync_committee_root
  );

  // address(0x0800)
  address private immutable BLS_PRECOMPILE;

  bytes32 public immutable GENESIS_VALIDATORS_ROOT;

  /**
   * A bellatrix beacon state has 25 fields, with a depth of 5.
   * | field                               | gindex | depth |
   * | ----------------------------------- | ------ | ----- |
   * | next_sync_committee                 | 55     | 5     |
   * | finalized_checkpoint_root           | 105    | 6     |
   */
  uint64 private constant NEXT_SYNC_COMMITTEE_INDEX = 55;
  uint64 private constant NEXT_SYNC_COMMITTEE_DEPTH = 5;

  uint64 private constant FINALIZED_CHECKPOINT_ROOT_INDEX = 105;
  uint64 private constant FINALIZED_CHECKPOINT_ROOT_DEPTH = 6;

  uint64 private constant EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
  uint64 private constant SLOTS_PER_EPOCH = 32;

  bytes4 private constant DOMAIN_SYNC_COMMITTEE = 0x07000000;

  uint8 constant MOD_EXP_PRECOMPILE_ADDRESS = 0x5;

  struct SyncAggregate {
    bytes32[2] sync_committee_bits;
    bytes sync_committee_signature;
  }

  struct FinalizedHeaderUpdate {
    // The beacon block header that is attested to by the sync committee
    BeaconBlockHeader attested_header;
    // Sync committee
    SyncCommittee signature_sync_committee;
    // The finalized beacon block header attested to by Merkle branch
    BeaconBlockHeader finalized_header;
    bytes32[] finality_branch;
    // Sync committee aggregate signature
    SyncAggregate sync_aggregate;
    // Slot at which the aggregate signature was created (untrusted)
    uint64 signature_slot;
    // Fork version for the aggregate signature
    bytes4 fork_version;
  }

  struct SyncCommitteePeriodUpdate {
    // Next sync committee corresponding to the finalized header
    bytes32 next_sync_committee_root;
    bytes32[] next_sync_committee_branch;
  }

  // Beacon block header that is finalized
  BeaconBlockHeader public finalized_header;

  // Attested_state_root
  bytes32 public attested_state_root;

  // Sync committees corresponding to the header
  // sync_committee_period => sync_committee_root
  bytes32 public prev_sync_committee_root;

  constructor(
    address _bls,
    uint64 _slot,
    uint64 _proposer_index,
    bytes32 _parent_root,
    bytes32 _state_root,
    bytes32 _body_root,
    bytes32 _current_sync_committee_hash,
    bytes32 _genesis_validators_root
  ) {
    BLS_PRECOMPILE = _bls;
    finalized_header = BeaconBlockHeader(
      _slot,
      _proposer_index,
      _parent_root,
      _state_root,
      _body_root
    );
    prev_sync_committee_root = _current_sync_committee_hash;
    GENESIS_VALIDATORS_ROOT = _genesis_validators_root;
  }

  function state_root() public view returns (bytes32) {
    return finalized_header.state_root;
  }

  function import_next_sync_committee(SyncCommitteePeriodUpdate calldata update)
    external
    payable
  {
    require(verify_next_sync_committee(update), '!next_sync_committee');

    prev_sync_committee_root = update.next_sync_committee_root;

    emit NextSyncCommitteeImported(
      compute_sync_committee_period(finalized_header.slot) + 1,
      update.next_sync_committee_root
    );
  }

  function import_finalized_header(FinalizedHeaderUpdate calldata update)
    external
    payable
  {
    require(
      is_supermajority(update.sync_aggregate.sync_committee_bits),
      '!supermajor'
    );

    require(
      verify_finalized_header(
        update.finalized_header,
        update.finality_branch,
        update.attested_header.state_root
      ),
      '!finalized_header'
    );

    uint64 finalized_period = compute_sync_committee_period(
      finalized_header.slot
    );
    uint64 signature_period = compute_sync_committee_period(
      update.signature_slot
    );

    require(
      signature_period == finalized_period ||
        signature_period == finalized_period + 1,
      '!signature_period'
    );

    require(
      prev_sync_committee_root ==
        hash_tree_root(update.signature_sync_committee),
      '!sync_committee'
    );

    require(
      verify_signed_header(
        update.sync_aggregate,
        update.signature_sync_committee,
        update.fork_version,
        update.attested_header
      ),
      '!sign'
    );

    require(update.finalized_header.slot > finalized_header.slot, '!new');
    finalized_header = update.finalized_header;
    attested_state_root = update.attested_header.state_root;
    emit FinalizedHeaderImported(update.finalized_header);
  }

  function verify_signed_header(
    SyncAggregate calldata sync_aggregate,
    SyncCommittee calldata sync_committee,
    bytes4 fork_version,
    BeaconBlockHeader calldata header
  ) internal view returns (bool) {
    // Verify sync committee aggregate signature
    uint256 participants = sum(sync_aggregate.sync_committee_bits);
    bytes[] memory participant_pubkeys = new bytes[](participants);
    uint64 n;
    for (uint64 i; i < SYNC_COMMITTEE_SIZE; ) {
      uint256 index = i >> 8;
      uint256 sindex = (i >> 3) % 32;
      uint256 offset = i % 8;
      if (
        (uint8(sync_aggregate.sync_committee_bits[index][sindex]) >> offset) &
          1 ==
        1
      ) {
        participant_pubkeys[n++] = sync_committee.pubkeys[i];
      }

      unchecked {
        ++i;
      }
    }

    bytes32 domain = compute_domain(
      DOMAIN_SYNC_COMMITTEE,
      fork_version,
      GENESIS_VALIDATORS_ROOT
    );

    bytes32 signing_root = compute_signing_root(header, domain);
    bytes memory message = abi.encodePacked(signing_root);
    bytes memory signature = sync_aggregate.sync_committee_signature;
    require(signature.length == BLSSIGNATURE_LENGTH, '!signature');
    return fast_aggregate_verify(participant_pubkeys, message, signature);
  }

  function verify_finalized_header(
    BeaconBlockHeader calldata header,
    bytes32[] calldata finality_branch,
    bytes32 attested_header_root
  ) internal pure returns (bool) {
    require(
      finality_branch.length == FINALIZED_CHECKPOINT_ROOT_DEPTH,
      '!finality_branch'
    );

    return
      is_valid_merkle_branch(
        hash_tree_root(header),
        finality_branch,
        FINALIZED_CHECKPOINT_ROOT_DEPTH,
        FINALIZED_CHECKPOINT_ROOT_INDEX,
        attested_header_root
      );
  }

  function verify_next_sync_committee(SyncCommitteePeriodUpdate calldata update)
    internal
    view
    returns (bool)
  {
    require(
      update.next_sync_committee_branch.length == NEXT_SYNC_COMMITTEE_DEPTH,
      '!next_sync_committee_branch'
    );

    return
      is_valid_merkle_branch(
        update.next_sync_committee_root,
        update.next_sync_committee_branch,
        NEXT_SYNC_COMMITTEE_DEPTH,
        NEXT_SYNC_COMMITTEE_INDEX,
        attested_state_root
      );
  }

  function is_supermajority(bytes32[2] calldata sync_committee_bits)
    internal
    pure
    returns (bool)
  {
    return sum(sync_committee_bits) * 3 >= SYNC_COMMITTEE_SIZE * 2;
  }

  function fast_aggregate_verify(
    bytes[] memory pubkeys,
    bytes memory message,
    bytes memory signature
  ) internal view returns (bool valid) {
    bytes memory input = abi.encodeWithSelector(
      IBLS.fast_aggregate_verify.selector,
      pubkeys,
      message,
      signature
    );
    (bool ok, bytes memory out) = BLS_PRECOMPILE.staticcall(input);
    if (ok) {
      if (out.length == 32) {
        valid = abi.decode(out, (bool));
      }
    } else {
      if (out.length > 0) {
        assembly {
          let returndata_size := mload(out)
          revert(add(32, out), returndata_size)
        }
      } else {
        revert('!verify');
      }
    }
  }

  function compute_sync_committee_period(uint64 slot)
    internal
    pure
    returns (uint64)
  {
    return slot / SLOTS_PER_EPOCH / EPOCHS_PER_SYNC_COMMITTEE_PERIOD;
  }

  function sum(bytes32[2] memory x) internal pure returns (uint256) {
    return countSetBits(uint256(x[0])) + countSetBits(uint256(x[1]));
  }
}
