when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import serialization/object_serialization
from stew/ranges/ptr_arith import makeOpenArray
from ssz_serialization/codec import readSszValue

export object_serialization
from nimcrypto/hash import MDigest, fromHex

import light_client_utils
from light_client import
  initialize_light_client_store, process_light_client_update

proc deserializeSSZType*[T](t: var T, memory: pointer, length: Natural) =
  readSszValue(makeOpenArray(memory, byte, length), t)

template fromSszBytes*(T: type Slot, bytes: openArray[byte]): T =
  T fromSszBytes(uint64, bytes)

proc getLightClientStoreSize(): uint32 {.wasmPragma.} =
  sizeof(LightClientStore).uint32

proc initializeLightClientStoreCosmos(
    dataBootstrap: pointer,
    dataBootstrapLen: uint32
    ): ref LightClientStore {.wasmPragma.} =
  var bootstrap: LightClientBootstrap
  bootstrap.deserializeSSZType(dataBootstrap, dataBootstrapLen)

  let lightClientStore =
   initialize_light_client_store(hash_tree_root(bootstrap.header), bootstrap)

  var storeRef : ref LightClientStore = new (LightClientStore)
  storeRef.finalized_header = lightClientStore.finalized_header
  storeRef.current_sync_committee = lightClientStore.current_sync_committee
  storeRef.optimistic_header = lightClientStore.optimistic_header
  return storeRef

proc processLightClientUpdate(
    dataUpdate: pointer,
    dataUpdateLen: uint32,
    storeRef: ptr LightClientStore,
    ) {.wasmPragma.} =
  var update: LightClientUpdate
  update.deserializeSSZType(dataUpdate, dataUpdateLen)

  let genesis_validators_root = MDigest[256].fromHex(GENESIS_VALIDATORS_ROOT)

  process_light_client_update(storeRef[],
                              update,
                              update.signature_slot,
                              genesis_validators_root)
