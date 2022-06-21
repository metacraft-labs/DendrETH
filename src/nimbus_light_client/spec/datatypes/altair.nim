import ../../../../nimbus-eth2/vendor/nim-ssz-serialization/ssz_serialization/types

import ./base

type
  # https://github.com/ethereum/consensus-specs/blob/v1.1.10/specs/altair/beacon-chain.md#synccommittee
  SyncCommittee* = object
    pubkeys*: HashArray[Limit SYNC_COMMITTEE_SIZE, ValidatorPubKey]
    aggregate_pubkey*: ValidatorPubKey

  # https://github.com/ethereum/consensus-specs/blob/vFuture/specs/altair/sync-protocol.md#lightclientupdate
  LightClientUpdate* = object
    attested_header*: BeaconBlockHeader ##\
    ## The beacon block header that is attested to by the sync committee

    # Next sync committee corresponding to the active header,
    # if signature is from current sync committee
    next_sync_committee*: SyncCommittee
    next_sync_committee_branch*:
      array[log2trunc(NEXT_SYNC_COMMITTEE_INDEX), Eth2Digest]

    # The finalized beacon block header attested to by Merkle branch
    finalized_header*: BeaconBlockHeader
    finality_branch*: array[log2trunc(FINALIZED_ROOT_INDEX), Eth2Digest]

    # Sync committee aggregate signature
    sync_aggregate*: SyncAggregate

    fork_version*: Version ##\
    ## Fork version for the aggregate signature
