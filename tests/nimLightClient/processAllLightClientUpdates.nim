when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import marshal
from nimcrypto/hash import MDigest, fromHex

import light_client_utils
import ./helpers/helpers
from ../../beacon-light-client/nim/light-client/light_client import
  initialize_light_client_store, process_light_client_update

proc initializeLightClientStore(
    dataRoot: pointer,
    dataBootstrap: pointer,
    ): ref LightClientStore {.wasmPragma.} =
  var beaconBlockHeader: BeaconBlockHeader
  beaconBlockHeader.deserializeSSZType(dataRoot, sizeof(BeaconBlockHeader))

  var bootstrap: LightClientBootstrap
  bootstrap.deserializeSSZType(dataBootstrap, sizeof(LightClientBootstrap))

  let lightClientStore =
   initialize_light_client_store(hash_tree_root(beaconBlockHeader), bootstrap)

  var storeRef : ref LightClientStore = new (LightClientStore)
  storeRef.finalized_header = lightClientStore.finalized_header
  storeRef.current_sync_committee = lightClientStore.current_sync_committee
  storeRef.optimistic_header = lightClientStore.optimistic_header

  return storeRef

proc processLightClientUpdate(
    dataUpdate: pointer,
    storeRef: ref LightClientStore,
    ):bool {.wasmPragma.} =
  var update: LightClientUpdate
  update.deserializeSSZType(dataUpdate, sizeof(LightClientUpdate))

  let genesis_validators_root = MDigest[256].fromHex(GENESIS_VALIDATORS_ROOT)

  process_light_client_update(storeRef[],
                              update,
                              update.signature_slot,
                              genesis_validators_root)
  return true
