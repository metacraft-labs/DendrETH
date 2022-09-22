import
  std/[typetraits, options, tables],
  ssz_serialization/[merkleization, types, proofs],
  nimcrypto/hash,
  blscurve

export options, merkleization, types, proofs

type
  Eth2Digest* = MDigest[32 * 8] ## `hash32` from spec

# Callables from stew/options
func isZeroMemory*[T](x: T): bool =
  # TODO: iterate over words here
  for b in cast[ptr array[sizeof(T), byte]](unsafeAddr x)[]:
    if b != 0:
      return false
  return true

# Callables from stew/bitops2
func log2truncNim(x: uint8|uint16|uint32): int =
  ## Quickly find the log base 2 of a 32-bit or less integer.
  # https://graphics.stanford.edu/%7Eseander/bithacks.html#IntegerLogDeBruijn
  # https://stackoverflow.com/questions/11376288/fast-computing-of-log2-for-64-bit-integers
  const lookup: array[32, uint8] = [0'u8, 9, 1, 10, 13, 21, 2, 29, 11, 14, 16, 18,
    22, 25, 3, 30, 8, 12, 20, 28, 15, 17, 24, 7, 19, 27, 23, 6, 26, 5, 4, 31]
  var v = x.uint32
  v = v or v shr 1 # first round down to one less than a power of 2
  v = v or v shr 2
  v = v or v shr 4
  v = v or v shr 8
  v = v or v shr 16
  cast[int](lookup[uint32(v * 0x07C4ACDD'u32) shr 27])

func log2truncNim(x: uint64): int =
  ## Quickly find the log base 2 of a 64-bit integer.
  # https://graphics.stanford.edu/%7Eseander/bithacks.html#IntegerLogDeBruijn
  # https://stackoverflow.com/questions/11376288/fast-computing-of-log2-for-64-bit-integers
  const lookup: array[64, uint8] = [0'u8, 58, 1, 59, 47, 53, 2, 60, 39, 48, 27, 54,
    33, 42, 3, 61, 51, 37, 40, 49, 18, 28, 20, 55, 30, 34, 11, 43, 14, 22, 4, 62,
    57, 46, 52, 38, 26, 32, 41, 50, 36, 17, 19, 29, 10, 13, 21, 56, 45, 25, 31,
    35, 16, 9, 12, 44, 24, 15, 8, 23, 7, 6, 5, 63]
  var v = x
  v = v or v shr 1 # first round down to one less than a power of 2
  v = v or v shr 2
  v = v or v shr 4
  v = v or v shr 8
  v = v or v shr 16
  v = v or v shr 32
  cast[int](lookup[(v * 0x03F6EAF2CD271461'u64) shr 58])

func log2trunc*(x: SomeUnsignedInt): int {.inline.} =
  ## Return the truncated base 2 logarithm of `x` - this is the zero-based
  ## index of the last set bit.
  ##
  ## If `x` is zero result is -1
  ##
  ## log2trunc(x) == bitsof(x) - leadingZeros(x) - 1.
  ##
  ## Example:
  ## doAssert log2trunc(0b01001000'u8) == 6
  if x == 0: -1
  else:
    when nimvm:
      log2truncNim(x)
    else:
      when declared(log2truncBuiltin):
        log2truncBuiltin(x)
      else:
        log2truncNim(x)

type
  Slot* = distinct uint64
  Epoch* = distinct uint64
  SyncCommitteePeriod* = distinct uint64
  Version* = distinct array[4, byte]

const
# Constants from base.nim
  ZERO_HASH* = Eth2Digest()

# Constants from altair.nim
# https://github.com/ethereum/consensus-specs/blob/vFuture/specs/altair/sync-protocol.md#constants
  # All of these indices are rooted in `BeaconState`.
  # The first member (`genesis_time`) is 32, subsequent members +1 each.
  # If there are ever more than 32 members in `BeaconState`, indices change!
  # `FINALIZED_ROOT_INDEX` is one layer deeper, i.e., `52 * 2 + 1`.
  # https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/ssz/merkle-proofs.md
  FINALIZED_ROOT_INDEX* = 105.GeneralizedIndex # `finalized_checkpoint` > `root`
  CURRENT_SYNC_COMMITTEE_INDEX* = 54.GeneralizedIndex # `current_sync_committee`
  NEXT_SYNC_COMMITTEE_INDEX* = 55.GeneralizedIndex # `next_sync_committee`

# Constants from crypto.nim
  RawSigSize* = 96
  RawPubKeySize* = 48
  UncompressedPubKeySize* = 96

# Constants from presets
  # Genesis
  GENESIS_FORK_VERSION = Version [byte 0x00, 0x00, 0x00, 0x00]
  GENESIS_VALIDATORS_ROOT* = "4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"
  #Phase0
  SLOTS_PER_EPOCH* {.intdefine.}: uint64 = 32
  SYNC_COMMITTEE_SIZE* = 512
  # Altair
  ALTAIR_FORK_VERSION = Version [byte 0x01, 0x00, 0x00, 0x00]
  ALTAIR_FORK_EPOCH = Epoch(74240)
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD* {.intdefine.}: uint64 = 256
  MIN_SYNC_COMMITTEE_PARTICIPANTS* = 1
  UPDATE_TIMEOUT*: uint64 = 8192

  # Bellatrix
  BELLATRIX_FORK_VERSION = Version [byte 0x02, 0x00, 0x00, 0x00]
  BELLATRIX_FORK_EPOCH = Epoch(uint64.high)

  # Sharding
  SLOTS_PER_SYNC_COMMITTEE_PERIOD* =
   SLOTS_PER_EPOCH * EPOCHS_PER_SYNC_COMMITTEE_PERIOD

# from beacon_time.nim
  # Earlier spec versions had these at a different slot
  GENESIS_SLOT* = Slot(0)
  GENESIS_EPOCH* = Epoch(0) # compute_epoch_at_slot(GENESIS_SLOT)
  FAR_FUTURE_SLOT* = Slot(not 0'u64)
  FAR_FUTURE_EPOCH* = (not 0'u64).Epoch # 2^64 - 1 in spec

  FAR_FUTURE_PERIOD* = SyncCommitteePeriod(not 0'u64)

type
# Types from base.nim
  DomainType* = distinct array[4, byte]
  Eth2Domain* = array[32, byte]

  ForkDigest* = distinct array[4, byte]

  # https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#beaconblockheader
  BeaconBlockHeader* = object
    slot*: Slot
    proposer_index*: uint64 # `ValidatorIndex` after validation
    parent_root*: Eth2Digest
    state_root*: Eth2Digest
    body_root*: Eth2Digest

  # https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#signingdata
  SigningData* = object
    object_root*: Eth2Digest
    domain*: Eth2Domain

  # https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#forkdata
  ForkData* = object
    current_version*: Version
    genesis_validators_root*: Eth2Digest

# Types from crypto.nim
  ValidatorPubKey* = object ##\
    ## Compressed raw serialized key bytes - this type is used in so as to not
    ## eagerly load keys - deserialization is slow, as are equality checks -
    ## however, it is not guaranteed that the key is valid (except in some
    ## cases, like the database state)
    blob*: array[RawPubKeySize, byte]

  CookedPubKey* = distinct blscurve.PublicKey ## Valid deserialized key
  CookedSig* = distinct blscurve.Signature  ## \
  ## Cooked signatures are those that have been loaded successfully from a
  ## ValidatorSig and are used to avoid expensive reloading as well as error
  ## checking
  ValidatorSig* = object
    blob*: array[RawSigSize, byte]

# Types from forks.nim
  BeaconStateFork* {.pure.} = enum
    Phase0,
    Altair,
    Bellatrix

# Types from altair.nim
  FinalityBranch* =
    array[log2trunc(FINALIZED_ROOT_INDEX), Eth2Digest]

  CurrentSyncCommitteeBranch* =
    array[log2trunc(CURRENT_SYNC_COMMITTEE_INDEX), Eth2Digest]

  NextSyncCommitteeBranch* =
    array[log2trunc(NEXT_SYNC_COMMITTEE_INDEX), Eth2Digest]

  # https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap
  LightClientBootstrap* = object
    header*: BeaconBlockHeader
      ## The requested beacon block header

    current_sync_committee*: SyncCommittee
      ## Current sync committee corresponding to `header`
    current_sync_committee_branch*: CurrentSyncCommitteeBranch

  # https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientupdate  LightClientUpdate* = object
  LightClientUpdate* = object
    attested_header*: BeaconBlockHeader
      ## The beacon block header that is attested to by the sync committee

    next_sync_committee*: SyncCommittee
      ## Next sync committee corresponding to `attested_header`,
      ## if signature is from current sync committee
    next_sync_committee_branch*: NextSyncCommitteeBranch

    # The finalized beacon block header attested to by Merkle branch
    finalized_header*: BeaconBlockHeader
    finality_branch*: FinalityBranch

    sync_aggregate*: SyncAggregate
    signature_slot*: Slot
      ## Slot at which the aggregate signature was created (untrusted)

  # https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientfinalityupdate
  LightClientFinalityUpdate* = object
    # The beacon block header that is attested to by the sync committee
    attested_header*: BeaconBlockHeader

    # The finalized beacon block header attested to by Merkle branch
    finalized_header*: BeaconBlockHeader
    finality_branch*: FinalityBranch

    # Sync committee aggregate signature
    sync_aggregate*: SyncAggregate
    # Slot at which the aggregate signature was created (untrusted)
    signature_slot*: Slot

  # https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientoptimisticupdate
  LightClientOptimisticUpdate* = object
    # The beacon block header that is attested to by the sync committee
    attested_header*: BeaconBlockHeader

    # Sync committee aggregate signature
    sync_aggregate*: SyncAggregate
    # Slot at which the aggregate signature was created (untrusted)
    signature_slot*: Slot

  SomeLightClientUpdateWithSyncCommittee* =
    LightClientUpdate

  SomeLightClientUpdateWithFinality* =
    LightClientUpdate |
    LightClientFinalityUpdate

  SomeLightClientUpdate* =
    LightClientUpdate |
    LightClientFinalityUpdate |
    LightClientOptimisticUpdate

  SomeLightClientObject* =
    LightClientBootstrap |
    SomeLightClientUpdate

  # https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientstore
  LightClientStore* = object
    finalized_header*: BeaconBlockHeader
      ## Beacon block header that is finalized

    current_sync_committee*: SyncCommittee
      ## Sync committees corresponding to the header
    next_sync_committee*: SyncCommittee

    best_valid_update*: Option[LightClientUpdate]
      ## Best available header to switch finalized head to if we see nothing else

    optimistic_header*: BeaconBlockHeader
      ## Most recent available reasonably-safe header

    previous_max_active_participants*: uint64
      ## Max number of active participants in a sync committee (used to compute
      ## safety threshold)
    current_max_active_participants*: uint64

  # https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/altair/beacon-chain.md#synccommittee
  SyncCommittee* = object
    pubkeys*: HashArray[Limit SYNC_COMMITTEE_SIZE, ValidatorPubKey]
    aggregate_pubkey*: ValidatorPubKey

  # https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/altair/beacon-chain.md#syncaggregate
  SyncAggregate* = object
    sync_committee_bits*: BitArray[SYNC_COMMITTEE_SIZE]
    sync_committee_signature*: ValidatorSig

# Types from helpers.nim
  LightClientUpdateMetadata* = object
    attested_slot*, finalized_slot*, signature_slot*: Slot
    has_sync_committee*, has_finality*: bool
    num_active_participants*: uint64

const DOMAIN_SYNC_COMMITTEE* = DomainType([byte 0x07, 0x00, 0x00, 0x00])

# Callables from beacon_time.nim
template ethTimeUnit*(typ: type) {.dirty.} =
  proc `+`*(x: typ, y: uint64): typ {.borrow, noSideEffect.}
  proc `-`*(x: typ, y: uint64): typ {.borrow, noSideEffect.}
  proc `-`*(x: uint64, y: typ): typ {.borrow, noSideEffect.}

  # Not closed over type in question (Slot or Epoch)
  proc `mod`*(x: typ, y: uint64): uint64 {.borrow, noSideEffect.}
  proc `div`*(x: typ, y: uint64): uint64 {.borrow, noSideEffect.}
  proc `div`*(x: uint64, y: typ): uint64 {.borrow, noSideEffect.}
  proc `-`*(x: typ, y: typ): uint64 {.borrow, noSideEffect.}

  proc `*`*(x: typ, y: uint64): uint64 {.borrow, noSideEffect.}

  proc `+=`*(x: var typ, y: typ) {.borrow, noSideEffect.}
  proc `+=`*(x: var typ, y: uint64) {.borrow, noSideEffect.}
  proc `-=`*(x: var typ, y: typ) {.borrow, noSideEffect.}
  proc `-=`*(x: var typ, y: uint64) {.borrow, noSideEffect.}

  # Comparison operators
  proc `<`*(x: typ, y: typ): bool {.borrow, noSideEffect.}
  proc `<`*(x: typ, y: uint64): bool {.borrow, noSideEffect.}
  proc `<`*(x: uint64, y: typ): bool {.borrow, noSideEffect.}
  proc `<=`*(x: typ, y: typ): bool {.borrow, noSideEffect.}
  proc `<=`*(x: typ, y: uint64): bool {.borrow, noSideEffect.}
  proc `<=`*(x: uint64, y: typ): bool {.borrow, noSideEffect.}

  proc `==`*(x: typ, y: typ): bool {.borrow, noSideEffect.}
  proc `==`*(x: typ, y: uint64): bool {.borrow, noSideEffect.}
  proc `==`*(x: uint64, y: typ): bool {.borrow, noSideEffect.}

ethTimeUnit Slot
ethTimeUnit Epoch
ethTimeUnit SyncCommitteePeriod

# https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#compute_epoch_at_slot
func epoch*(slot: Slot): Epoch = # aka compute_epoch_at_slot
  ## Return the epoch number at ``slot``.
  if slot == FAR_FUTURE_SLOT: FAR_FUTURE_EPOCH
  else: Epoch(slot div SLOTS_PER_EPOCH)

# https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/altair/validator.md#sync-committee
template sync_committee_period*(epoch: Epoch): SyncCommitteePeriod =
  if epoch == FAR_FUTURE_EPOCH: FAR_FUTURE_PERIOD
  else: SyncCommitteePeriod(epoch div EPOCHS_PER_SYNC_COMMITTEE_PERIOD)

template sync_committee_period*(slot: Slot): SyncCommitteePeriod =
  if slot == FAR_FUTURE_SLOT: FAR_FUTURE_PERIOD
  else: SyncCommitteePeriod(slot div SLOTS_PER_SYNC_COMMITTEE_PERIOD)

# Callables from altair.nim
template toFull*(
    update: SomeLightClientUpdate): LightClientUpdate =
  when update is LightClientUpdate:
    update
  elif update is SomeLightClientUpdateWithFinality:
    LightClientUpdate(
      attested_header: update.attested_header,
      finalized_header: update.finalized_header,
      finality_branch: update.finality_branch,
      sync_aggregate: update.sync_aggregate,
      signature_slot: update.signature_slot)
  else:
    LightClientUpdate(
      attested_header: update.attested_header,
      sync_aggregate: update.sync_aggregate,
      signature_slot: update.signature_slot)

# Callables from crypto.nim
func load*(v: ValidatorPubKey): Option[CookedPubKey] =
  ## Parse signature blob - this may fail
  var val: blscurve.PublicKey
  if fromBytes(val, v.blob):
    some CookedPubKey(val)
  else:
    none CookedPubKey

func load*(v: ValidatorSig): Option[CookedSig] =
  ## Parse signature blob - this may fail
  var parsed: blscurve.Signature
  if fromBytes(parsed, v.blob):
    some(CookedSig(parsed))
  else:
    none(CookedSig)

proc loadWithCache*(v: ValidatorPubKey): Option[CookedPubKey] =
  ## Parse public key blob - this may fail - this function uses a cache to
  ## avoid the expensive deserialization - for now, external public keys only
  ## come from deposits in blocks - when more sources are added, the memory
  ## usage of the cache should be considered
  var cache {.threadvar.}: Table[typeof(v.blob), CookedPubKey]

  # Try to get parse value from cache - if it's not in there, try to parse it -
  # if that's not possible, it's broken
  cache.withValue(v.blob, key) do:
    return some key[]
  do:
    # Only valid keys are cached
    let cooked = v.load()
    if cooked.isSome():
      cache[v.blob] = cooked.get()
    return cooked

proc blsFastAggregateVerify*(
       publicKeys: openArray[ValidatorPubKey],
       message: openArray[byte],
       signature: CookedSig
     ): bool =
  var unwrapped: seq[PublicKey]
  for pubkey in publicKeys:
    let realkey = pubkey.loadWithCache()
    if realkey.isNone:
      return false
    unwrapped.add PublicKey(realkey.get)

  fastAggregateVerify(unwrapped, message, blscurve.Signature(signature))

proc blsFastAggregateVerify*(
       publicKeys: openArray[ValidatorPubKey],
       message: openArray[byte],
       signature: ValidatorSig
     ): bool =
  let parsedSig = signature.load()
  parsedSig.isSome and blsFastAggregateVerify(publicKeys,
                                              message,
                                              parsedSig.get())

# Callables from base.nim
template data*(v: ForkDigest | Version | DomainType): array[4, byte] =
  distinctBase(v)

# Callables from ssz_codec.nim
template toSszType*(v: Slot|Epoch|SyncCommitteePeriod): auto = uint64(v)
template toSszType*(v: ForkDigest): auto = distinctBase(v)
template toSszType*(v: Version): auto = distinctBase(v)

# Callables from forks.nim
# https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#compute_fork_data_root
func compute_fork_data_root*(current_version: Version,
    genesis_validators_root: Eth2Digest): Eth2Digest =
  ## Return the 32-byte fork data root for the ``current_version`` and
  ## ``genesis_validators_root``.
  ## This is used primarily in signature domains to avoid collisions across
  ## forks/chains.
  hash_tree_root(ForkData(
    current_version: current_version,
    genesis_validators_root: genesis_validators_root
  ))

func stateForkAtEpoch*(epoch: Epoch): BeaconStateFork =
  ## Return the current fork for the given epoch.
  static:
    doAssert BeaconStateFork.Bellatrix > BeaconStateFork.Altair
    doAssert BeaconStateFork.Altair    > BeaconStateFork.Phase0
    doAssert GENESIS_EPOCH == 0

  if   epoch >= BELLATRIX_FORK_EPOCH: BeaconStateFork.Bellatrix
  elif epoch >= ALTAIR_FORK_EPOCH:    BeaconStateFork.Altair
  else:                               BeaconStateFork.Phase0

func forkVersionAtEpoch*(epoch: Epoch): Version =
  case stateForkAtEpoch(epoch)
  of BeaconStateFork.Bellatrix: BELLATRIX_FORK_VERSION
  of BeaconStateFork.Altair:    ALTAIR_FORK_VERSION
  of BeaconStateFork.Phase0:    GENESIS_FORK_VERSION

# Callables from helpers.nim
# https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#compute_domain
func compute_domain*(
    domain_type: DomainType,
    fork_version: Version,
    genesis_validators_root: Eth2Digest = ZERO_HASH): Eth2Domain =
  ## Return the domain for the ``domain_type`` and ``fork_version``.
  #
  # TODO Can't be used as part of a const/static expression:
  # https://github.com/nim-lang/Nim/issues/15952
  # https://github.com/nim-lang/Nim/issues/19969
  let fork_data_root =
    compute_fork_data_root(fork_version, genesis_validators_root)
  result[0..3] = domain_type.data
  result[4..31] = fork_data_root.data.toOpenArray(0, 27)

# https://github.com/ethereum/consensus-specs/blob/v1.1.10/specs/altair/sync-protocol.md#get_active_header
func is_finality_update*(update: LightClientUpdate): bool =
  not update.finalized_header.isZeroMemory

# https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/phase0/beacon-chain.md#compute_signing_root
func compute_signing_root*(ssz_object: auto, domain: Eth2Domain): Eth2Digest =
  ## Return the signing root of an object by calculating the root of the
  ## object-domain tree.
  let domain_wrapped_object = SigningData(
    object_root: hash_tree_root(ssz_object),
    domain: domain
  )
  hash_tree_root(domain_wrapped_object)

# https://github.com/ethereum/consensus-specs/blob/vFuture/specs/altair/sync-protocol.md#is_next_sync_committee_known
template is_next_sync_committee_known*(store: LightClientStore): bool =
  not isZeroMemory(store.next_sync_committee)

# https://github.com/ethereum/consensus-specs/blob/vFuture/specs/altair/sync-protocol.md#is_sync_committee_update
template is_sync_committee_update*(update: SomeLightClientUpdate): bool =
  when update is SomeLightClientUpdateWithSyncCommittee:
    not isZeroMemory(update.next_sync_committee_branch)
  else:
    false
# https://github.com/ethereum/consensus-specs/blob/v1.2.0-rc.1/specs/altair/sync-protocol.md#get_safety_threshold
func get_safety_threshold*(store: LightClientStore): uint64 =
  max(
    store.previous_max_active_participants,
    store.current_max_active_participants
  ) div 2

func toMeta*(update: SomeLightClientUpdate): LightClientUpdateMetadata =
  var meta {.noinit.}: LightClientUpdateMetadata
  meta.attested_slot =
    update.attested_header.slot
  meta.finalized_slot =
    when update is SomeLightClientUpdateWithFinality:
      update.finalized_header.slot
    else:
      GENESIS_SLOT
  meta.signature_slot =
    update.signature_slot
  meta.has_sync_committee =
    when update is SomeLightClientUpdateWithSyncCommittee:
      not update.next_sync_committee_branch.isZeroMemory
    else:
      false
  meta.has_finality =
    when update is SomeLightClientUpdateWithFinality:
      not update.finality_branch.isZeroMemory
    else:
      false
  meta.num_active_participants =
    countOnes(update.sync_aggregate.sync_committee_bits).uint64
  meta

func is_better_data*(new_meta, old_meta: LightClientUpdateMetadata): bool =
  # Compare supermajority (> 2/3) sync committee participation
  const max_active_participants = SYNC_COMMITTEE_SIZE.uint64
  let
    new_has_supermajority =
      new_meta.num_active_participants * 3 >= max_active_participants * 2
    old_has_supermajority =
      old_meta.num_active_participants * 3 >= max_active_participants * 2
  if new_has_supermajority != old_has_supermajority:
    return new_has_supermajority > old_has_supermajority
  if not new_has_supermajority:
    if new_meta.num_active_participants != old_meta.num_active_participants:
      return new_meta.num_active_participants > old_meta.num_active_participants

  # Compare presence of relevant sync committee
  let
    new_has_relevant_sync_committee = new_meta.has_sync_committee and
      new_meta.attested_slot.sync_committee_period() ==
      new_meta.signature_slot.sync_committee_period
    old_has_relevant_sync_committee = old_meta.has_sync_committee and
      old_meta.attested_slot.sync_committee_period ==
      old_meta.signature_slot.sync_committee_period
  if new_has_relevant_sync_committee != old_has_relevant_sync_committee:
    return new_has_relevant_sync_committee > old_has_relevant_sync_committee

  # Compare indication of any finality
  if new_meta.has_finality != old_meta.has_finality:
    return new_meta.has_finality > old_meta.has_finality

  # Compare sync committee finality
  if new_meta.has_finality:
    let
      new_has_sync_committee_finality =
        new_meta.finalized_slot.sync_committee_period ==
        new_meta.attested_slot.sync_committee_period
      old_has_sync_committee_finality =
        old_meta.finalized_slot.sync_committee_period ==
        old_meta.attested_slot.sync_committee_period
    if new_has_sync_committee_finality != old_has_sync_committee_finality:
      return new_has_sync_committee_finality > old_has_sync_committee_finality

  # Tiebreaker 1: Sync committee participation beyond supermajority
  if new_meta.num_active_participants != old_meta.num_active_participants:
    return new_meta.num_active_participants > old_meta.num_active_participants

  # Tiebreaker 2: Prefer older data (fewer changes to best data)
  new_meta.attested_slot < old_meta.attested_slot

template is_better_update*[A, B: SomeLightClientUpdate](
    new_update: A, old_update: B): bool =
  is_better_data(toMeta(new_update), toMeta(old_update))

template initNextSyncCommitteeBranch*(): NextSyncCommitteeBranch =
  var res: NextSyncCommitteeBranch
  for el in 0 ..< log2trunc(NEXT_SYNC_COMMITTEE_INDEX):
    res[el] = Eth2Digest()
  res

template initFinalityBranch*(): FinalityBranch =
  var res: FinalityBranch
  for el in 0 ..< log2trunc(FINALIZED_ROOT_INDEX):
    res[el] = Eth2Digest()
  res

# Other helpers
type
  BlockError* {.pure.} = enum
    Invalid = "Invalid Block"
      ## Block is broken / doesn't apply cleanly - whoever sent it is fishy (or
      ## we're buggy)

    MissingParent = "Missing Parent"
      ## We don't know the parent of this block so we can't tell if it's valid
      ## or not - it'll go into the quarantine and be reexamined when the parent
      ## appears or be discarded if finality obsoletes it

    UnviableFork = "Unviable Fork"
      ## Block is from a history / fork that does not include our most current
      ## finalized checkpoint

    Duplicate = "Duplicate"
      ## We've seen this block already, can't add again

when defined(lightClientWASM):
  proc wasmQuit(errMsg: cstring, errLength: int):int {. importc, cdecl, exportc, dynlib} =
    discard

template assertLC*(cond: untyped, msg: BlockError) =
  if not cond:
    when defined(lightClientWASM):
      discard wasmQuit($msg, ($msg).len)
    else:
      quit 1
  else: discard
