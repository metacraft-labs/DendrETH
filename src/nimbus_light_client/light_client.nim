import
  spec/presets,
  spec/beacon_time,
  spec/preset/mainnet/altair_preset,
  spec/datatypes/altair

func period_contains_fork_version(
    period: SyncCommitteePeriod,
    fork_version: Version): bool {.cdecl, exportc, dynlib} =
  ## Determine whether a given `fork_version` is used during a given `period`.
  let
    periodStartEpoch = period.start_epoch
    periodEndEpoch = periodStartEpoch + EPOCHS_PER_SYNC_COMMITTEE_PERIOD - 1
  return
    if fork_version == defaultRuntimeConfig.SHARDING_FORK_VERSION:
      periodEndEpoch >= defaultRuntimeConfig.SHARDING_FORK_EPOCH
    elif fork_version == defaultRuntimeConfig.BELLATRIX_FORK_VERSION:
      periodStartEpoch < defaultRuntimeConfig.SHARDING_FORK_EPOCH and
      defaultRuntimeConfig.SHARDING_FORK_EPOCH != defaultRuntimeConfig.BELLATRIX_FORK_EPOCH and
      periodEndEpoch >= defaultRuntimeConfig.BELLATRIX_FORK_EPOCH
    elif fork_version == defaultRuntimeConfig.ALTAIR_FORK_VERSION:
      periodStartEpoch < defaultRuntimeConfig.BELLATRIX_FORK_EPOCH and
      defaultRuntimeConfig.BELLATRIX_FORK_EPOCH != defaultRuntimeConfig.ALTAIR_FORK_EPOCH and
      periodEndEpoch >= defaultRuntimeConfig.ALTAIR_FORK_EPOCH
    elif fork_version == defaultRuntimeConfig.GENESIS_FORK_VERSION:
      # Light client sync protocol requires Altair
      false
    else:
      # Unviable fork
      false

# https://github.com/ethereum/consensus-specs/blob/v1.1.10/specs/altair/sync-protocol.md#get_active_header
func is_finality_update*(update: altair.LightClientUpdate): bool
  {.cdecl, exportc, dynlib} =
  not update.finalized_header.isZeroMemory


proc start*() {.exportc: "_start".} =
  discard
