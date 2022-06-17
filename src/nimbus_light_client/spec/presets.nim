type
  Slot* = distinct uint64
  Epoch* = distinct uint64
  SyncCommitteePeriod* = distinct uint64
  Version* = distinct array[4, byte]

  # # Forking
  # ALTAIR_FORK_VERSION*: Version
  # ALTAIR_FORK_EPOCH*: Epoch
  # BELLATRIX_FORK_VERSION*: Version
  # BELLATRIX_FORK_EPOCH*: Epoch
  # SHARDING_FORK_VERSION*: Version
  # SHARDING_FORK_EPOCH*: Epoch

const
  FAR_FUTURE_EPOCH* = (not 0'u64).Epoch # 2^64 - 1 in spec