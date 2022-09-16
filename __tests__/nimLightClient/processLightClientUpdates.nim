when defined(emcc):
  {.emit: "#include <emscripten.h>".}
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "EMSCRIPTEN_KEEPALIVE $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

from nimcrypto/hash import MDigest, fromHex

import light_client_utils
import ./helpers/helpers
from ../../src/nim-light-client/light_client
  import initialize_light_client_store, process_light_client_update
from stew/ranges/ptr_arith import makeOpenArray

proc processLightClientUpdatesTest(
    dataRoot: pointer,
    dataBootstrap: pointer,
    dataUpdates: pointer,
    updatesDataLength: int
    ): bool {.wasmPragma.} =

  var beaconBlockHeader: BeaconBlockHeader
  beaconBlockHeader.deserializeSSZType(dataRoot, sizeof(BeaconBlockHeader))

  var bootstrap: LightClientBootstrap
  bootstrap.deserializeSSZType(dataBootstrap, sizeof(LightClientBootstrap))

  let genesis_validators_root = MDigest[256].fromHex(GENESIS_VALIDATORS_ROOT)
  var lightClientStore =
   initialize_light_client_store(hash_tree_root(beaconBlockHeader), bootstrap)

  var updateOffsets: array[30, uint32]
  updateOffsets.deserializeSSZType(dataUpdates, updatesDataLength)

  var update: LightClientUpdate
  for dataUpdate in updateOffsets:
    update.deserializeSSZType(cast[pointer](dataUpdate), sizeof(LightClientUpdate))
    process_light_client_update(lightClientStore,
                                update,
                                update.signature_slot,
                                genesis_validators_root)
  return true
