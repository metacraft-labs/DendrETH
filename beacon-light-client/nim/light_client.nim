import light_client_utils

# https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#initialize_light_client_store
func initialize_light_client_store*(
    trusted_block_root: Eth2Digest,
    bootstrap: LightClientBootstrap
  ): LightClientStore =
  assertLC(hash_tree_root(bootstrap.header) == trusted_block_root, BlockError.Invalid)

  assertLC(
    is_valid_merkle_branch(
      hash_tree_root(bootstrap.current_sync_committee),
      bootstrap.current_sync_committee_branch,
      log2trunc(CURRENT_SYNC_COMMITTEE_INDEX),
      get_subtree_index(CURRENT_SYNC_COMMITTEE_INDEX),
      bootstrap.header.state_root), BlockError.Invalid
  )

  return LightClientStore(
    finalized_header: bootstrap.header,
    current_sync_committee: bootstrap.current_sync_committee,
    optimistic_header: bootstrap.header)

# https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#validate_light_client_update
proc validate_light_client_update*(
    store: LightClientStore,
    update: LightClientUpdate,
    current_slot: Slot,
    genesis_validators_root: Eth2Digest
    ): void =
  # Verify sync committee has sufficient participants
  template sync_aggregate(): auto = update.sync_aggregate
  template sync_committee_bits(): auto = sync_aggregate.sync_committee_bits
  let num_active_participants = countOnes(sync_committee_bits).uint64
  assertLC(
    num_active_participants >= MIN_SYNC_COMMITTEE_PARTICIPANTS,
    BlockError.Invalid)

  # Verify update does not skip a sync committee period
  when update is SomeLightClientUpdateWithFinality:
    assertLC(
      update.attested_header.slot >= update.finalized_header.slot,
      BlockError.Invalid)
  assertLC(
    update.signature_slot > update.attested_header.slot,
    BlockError.Invalid)
  assertLC(
    current_slot >= update.signature_slot,
    BlockError.UnviableFork)
  let
    store_period = store.finalized_header.slot.sync_committee_period
    signature_period = update.signature_slot.sync_committee_period
    is_next_sync_committee_known = store.is_next_sync_committee_known
  if is_next_sync_committee_known:
    assertLC(
      signature_period in [store_period, store_period + 1],
      BlockError.MissingParent)
  else:
    assertLC(
      signature_period == store_period,
      BlockError.MissingParent)

  # Verify update is relevant
  let attested_period = update.attested_header.slot.sync_committee_period
  when update is SomeLightClientUpdateWithSyncCommittee:
    let is_sync_committee_update = update.is_sync_committee_update
  let update_has_next_sync_committee = not is_next_sync_committee_known and
    (is_sync_committee_update and attested_period == store_period)

  assertLC(
    update.attested_header.slot > store.finalized_header.slot or
      update_has_next_sync_committee, BlockError.Duplicate)

  # Verify that the `finalized_header`, if present, actually is the
  # finalized header saved in the state of the `attested_header`
  when update is SomeLightClientUpdateWithFinality:
    if not update.is_finality_update:
     assertLC(update.finalized_header.isZeroMemory, BlockError.Invalid)
    else:
      var finalized_root {.noinit.}: Eth2Digest
      if update.finalized_header.slot == GENESIS_SLOT:
        assertLC(update.finalized_header.isZeroMemory, BlockError.Invalid)
        finalized_root.reset()
      else:
        finalized_root = hash_tree_root(update.finalized_header)
      assertLC(
        is_valid_merkle_branch(
          finalized_root,
          update.finality_branch,
          log2trunc(FINALIZED_ROOT_INDEX),
          get_subtree_index(FINALIZED_ROOT_INDEX),
          update.attested_header.state_root),
        BlockError.Invalid
      )

  # Verify that the `next_sync_committee`, if present, actually is the
  # next sync committee saved in the state of the `attested_header`
  when update is SomeLightClientUpdateWithSyncCommittee:
    if not is_sync_committee_update:
      assertLC(update.next_sync_committee.isZeroMemory, BlockError.Invalid)
    else:
      if attested_period == store_period and is_next_sync_committee_known:
        assertLC(
          update.next_sync_committee == store.next_sync_committee,
          BlockError.UnviableFork)
      assertLC(
        is_valid_merkle_branch(
          hash_tree_root(update.next_sync_committee),
          update.next_sync_committee_branch,
          log2trunc(NEXT_SYNC_COMMITTEE_INDEX),
          get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX),
          update.attested_header.state_root),
        BlockError.Invalid
      )

  # # Verify sync committee aggregate signature
  let sync_committee =
    if signature_period == store_period:
      unsafeAddr store.current_sync_committee
    else:
      unsafeAddr store.next_sync_committee
  var participant_pubkeys: array[512, ValidatorPubKey]
  var i = 0
  for idx, bit in sync_aggregate.sync_committee_bits:
    if bit:
      participant_pubkeys[i] = sync_committee.pubkeys[idx]
      inc i
  let
    fork_version = forkVersionAtEpoch(update.signature_slot.epoch)
    domain = compute_domain(
      DOMAIN_SYNC_COMMITTEE, fork_version, genesis_validators_root)
    signing_root = compute_signing_root(update.attested_header, domain)
  assertLC(
    blsFastAggregateVerify(
      participant_pubkeys[0 .. num_active_participants-1], signing_root.data,
      sync_aggregate.sync_committee_signature),
    BlockError.UnviableFork
  )

# https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#apply_light_client_update
func apply_light_client_update(
    store: var LightClientStore,
    update: LightClientUpdate): void =
  let
    store_period = store.finalized_header.slot.sync_committee_period
    finalized_period = update.finalized_header.slot.sync_committee_period
  if not store.is_next_sync_committee_known:
    assert finalized_period == store_period
    when update is SomeLightClientUpdateWithSyncCommittee:
      store.next_sync_committee = update.next_sync_committee
  elif finalized_period == store_period + 1:
    store.current_sync_committee = store.next_sync_committee
    when update is SomeLightClientUpdateWithSyncCommittee:
      store.next_sync_committee = update.next_sync_committee
    else:
      store.next_sync_committee.reset()
    store.previous_max_active_participants =
      store.current_max_active_participants
    store.current_max_active_participants = 0
  if update.finalized_header.slot > store.finalized_header.slot:
    store.finalized_header = update.finalized_header
    if store.finalized_header.slot > store.optimistic_header.slot:
      store.optimistic_header = store.finalized_header

# https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#process_light_client_update
proc process_light_client_update* (
    store: var LightClientStore,
    update: LightClientUpdate,
    current_slot: Slot,
    genesis_validators_root: Eth2Digest): void =
  validate_light_client_update(
    store, update, current_slot, genesis_validators_root)

  # Track the maximum number of active participants in the committee signatures
  template sync_aggregate(): auto = update.sync_aggregate
  template sync_committee_bits(): auto = sync_aggregate.sync_committee_bits
  let num_active_participants = countOnes(sync_committee_bits).uint64
  if num_active_participants > store.current_max_active_participants:
    store.current_max_active_participants = num_active_participants

  # Update the optimistic header
  if num_active_participants > get_safety_threshold(store) and
      update.attested_header.slot > store.optimistic_header.slot:
    store.optimistic_header = update.attested_header

  # Update finalized header
  let update_has_finalized_next_sync_committee =
    not store.is_next_sync_committee_known and
    update.is_sync_committee_update and update.is_finality_update and
    update.finalized_header.slot.sync_committee_period ==
    update.attested_header.slot.sync_committee_period

  if num_active_participants * 3 >= static(sync_committee_bits.len * 2) and
      (update.finalized_header.slot > store.finalized_header.slot or
       update_has_finalized_next_sync_committee):
    apply_light_client_update(store, update)

# https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#process_light_client_finality_update
proc process_light_client_finality_update* (
    store: var LightClientStore,
    finality_update: LightClientFinalityUpdate,
    current_slot: Slot,
    genesis_validators_root: Eth2Digest): void =
  let update = LightClientUpdate(
    attested_header: finality_update.attested_header,
    next_sync_committee: SyncCommittee(),
    next_sync_committee_branch: initNextSyncCommitteeBranch() ,
    finalized_header: finality_update.finalized_header,
    finality_branch: finality_update.finality_branch,
    sync_aggregate: finality_update.sync_aggregate,
    signature_slot: finality_update.signature_slot
  )
  process_light_client_update(store, update, current_slot,
                              genesis_validators_root)

# https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#process_light_client_optimistic_update
proc process_light_client_optimistic_update* (
    store: var LightClientStore,
    optimistic_update: LightClientOptimisticUpdate,
    current_slot: Slot,
    genesis_validators_root: Eth2Digest): void =
  let update = LightClientUpdate(
    attested_header: optimistic_update.attested_header,
    next_sync_committee: SyncCommittee(),
    next_sync_committee_branch: initNextSyncCommitteeBranch(),
    finalized_header: BeaconBlockHeader(),
    finality_branch: initFinalityBranch(),
    sync_aggregate: optimistic_update.sync_aggregate,
    signature_slot: optimistic_update.signature_slot,
    )
  process_light_client_update(store, update, current_slot,
                              genesis_validators_root)
