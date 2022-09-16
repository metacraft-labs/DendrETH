when defined(emcc):
  {.emit: "#include <emscripten.h>".}
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "EMSCRIPTEN_KEEPALIVE $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

from nimcrypto/hash import MDigest, fromHex

import light_client_utils
import ./helpers/helpers
from ../../src/nim-light-client/light_client
  import initialize_light_client_store, validate_light_client_update

proc validateLightClientUpdateTest(
    dataRoot: pointer,
    dataBootstrap: pointer,
    dataUpdate: pointer,
    ): bool {.wasmPragma.} =
  var beaconBlockHeader: BeaconBlockHeader
  beaconBlockHeader.deserializeSSZType(dataRoot, sizeof(BeaconBlockHeader))

  var bootstrap: LightClientBootstrap
  bootstrap.deserializeSSZType(dataBootstrap, sizeof(LightClientBootstrap))

  var update: LightClientUpdate
  update.deserializeSSZType(dataUpdate, sizeof(LightClientUpdate))

  let genesis_validators_root = MDigest[256].fromHex(GENESIS_VALIDATORS_ROOT)
  let lightClientStore =
   initialize_light_client_store(hash_tree_root(beaconBlockHeader), bootstrap)

  validate_light_client_update(lightClientStore,
                               update,
                               update.signature_slot,
                               genesis_validators_root)
  return true
