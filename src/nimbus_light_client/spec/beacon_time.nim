import presets

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

  # Nim integration
  proc `$`*(x: typ): string {.borrow, noSideEffect.}
#   proc hash*(x: typ): Hash {.borrow, noSideEffect.}

  template asUInt64*(v: typ): uint64 = distinctBase(v)
  template shortLog*(v: typ): auto = distinctBase(v)

#   # Serialization
#   proc writeValue*(writer: var JsonWriter, value: typ)
#                   {.raises: [IOError, Defect].}=
#     writeValue(writer, uint64 value)

#   proc readValue*(reader: var JsonReader, value: var typ)
#                  {.raises: [IOError, SerializationError, Defect].} =
#     value = typ reader.readValue(uint64)
template ethVersionUnit*(typ: type) {.dirty.} =
  proc `==`*(x: typ, y: typ): bool {.borrow, noSideEffect.}


ethTimeUnit Slot
ethTimeUnit Epoch
ethTimeUnit SyncCommitteePeriod
ethVersionUnit Version


template start_epoch*(period: SyncCommitteePeriod): Epoch =
  ## Return the start epoch of ``period``.
  const maxPeriod = SyncCommitteePeriod(
    FAR_FUTURE_EPOCH div EPOCHS_PER_SYNC_COMMITTEE_PERIOD)
  if period >= maxPeriod: FAR_FUTURE_EPOCH
  else: Epoch(period * EPOCHS_PER_SYNC_COMMITTEE_PERIOD)