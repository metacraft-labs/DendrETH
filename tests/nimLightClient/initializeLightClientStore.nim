when defined(emcc):
  {.emit: "#include <emscripten.h>".}
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "EMSCRIPTEN_KEEPALIVE $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import marshal
from nimcrypto/hash import MDigest, fromHex

import light_client_utils
import ./helpers/helpers
from ../../beacon-light-client/nim/light_client import initialize_light_client_store

proc initializeLightClientStoreTest(
    dataRoot: pointer,
    dataBootstrap: pointer,
    ): bool {.wasmPragma.} =
  var beaconBlockHeader: BeaconBlockHeader
  beaconBlockHeader.deserializeSSZType(dataRoot, sizeof(BeaconBlockHeader))

  var bootstrap: LightClientBootstrap
  bootstrap.deserializeSSZType(dataBootstrap, sizeof(LightClientBootstrap))

  let lightClientStore =
   initialize_light_client_store(hash_tree_root(beaconBlockHeader), bootstrap)

  var expectedLightClientStore = LightClientStore(
    finalized_header: bootstrap.header,
    current_sync_committee: bootstrap.current_sync_committee,
    optimistic_header: bootstrap.header)

  return expectedLightClientStore == lightClientStore

