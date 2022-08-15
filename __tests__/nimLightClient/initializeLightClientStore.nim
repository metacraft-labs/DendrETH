from nimcrypto/hash import MDigest, fromHex

import light_client_utils
import ./helpers/helpers
from ../../src/nim-light-client/light_client import initialize_light_client_store

proc initializeLightClientStoreTest(
    dataRoot: pointer,
    dataBootstrap: pointer,
    ): bool {.cdecl, exportc, dynlib} =
  var beaconBlockHeader: BeaconBlockHeader
  beaconBlockHeader.deserializeSSZType(dataRoot, sizeof(BeaconBlockHeader))

  var bootstrap: LightClientBootstrap
  bootstrap.deserializeSSZType(dataBootstrap, sizeof(LightClientBootstrap))

  let lightClientStore =
   initialize_light_client_store(hash_tree_root(beaconBlockHeader), bootstrap)

  LightClientStore(
    finalized_header: bootstrap.header,
    current_sync_committee: bootstrap.current_sync_committee,
    optimistic_header: bootstrap.header) == lightClientStore
