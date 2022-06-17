import
  spec/presets,
  spec/beacon_time,
  spec/preset/mainnet/altair_preset,
  spec/configs/mainnet
# import
#   ../../nimbus-eth2/vendor/nim-stew/stew/bitops2,
#   ../../nimbus-eth2/vendor/nim-stew/stew/objects,
#   ../../nimbus-eth2/beacon_chain/spec/datatypes/altair,
#   ../../nimbus-eth2/beacon_chain/spec/helpers

# from ../../nimbus-eth2/beacon_chain/consensus_object_pools/block_pools_types import BlockError

func period_contains_fork_version(
    period: SyncCommitteePeriod,
    fork_version: Version): bool =
  ## Determine whether a given `fork_version` is used during a given `period`.
  let
    periodStartEpoch = period.start_epoch
    periodEndEpoch = periodStartEpoch + EPOCHS_PER_SYNC_COMMITTEE_PERIOD - 1
  return
    if fork_version == SHARDING_FORK_VERSION:
      periodEndEpoch >= SHARDING_FORK_EPOCH
    elif fork_version == BELLATRIX_FORK_VERSION:
      periodStartEpoch < SHARDING_FORK_EPOCH and
      SHARDING_FORK_EPOCH != BELLATRIX_FORK_EPOCH and
      periodEndEpoch >= BELLATRIX_FORK_EPOCH
    elif fork_version == ALTAIR_FORK_VERSION:
      periodStartEpoch < BELLATRIX_FORK_EPOCH and
      BELLATRIX_FORK_EPOCH != ALTAIR_FORK_EPOCH and
      periodEndEpoch >= ALTAIR_FORK_EPOCH
    elif fork_version == GENESIS_FORK_VERSION:
      # Light client sync protocol requires Altair
      false
    else:
      # Unviable fork
      false
